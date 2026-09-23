use std::{collections::BTreeMap, sync::Arc};

/// What happened to a table this namespace asked the main node for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableRequestState {
    /// Asked for - neither the table nor a "not found" has arrived yet.
    Pending,
    /// The table has arrived: the main node pushes every change of it to this node.
    Replicated,
    /// The main node does not have the table. The node keeps an empty one its readers stay
    /// subscribed to, and asks for the table again from time to time, so the rows reach them
    /// once the table is created - without anybody reconnecting.
    NotFound,
}

pub(super) struct TablesToRetry<TConnection> {
    pub connection: Arc<TConnection>,
    pub tables: Vec<String>,
}

/// State of the link of one namespace to the main node.
///
/// Generic over the connection only so that the logic can be tested without a socket.
pub(super) struct MainNodeConnectionInner<TConnection> {
    connection: Option<Arc<TConnection>>,
    requested_tables: BTreeMap<String, TableRequestState>,
}

impl<TConnection> MainNodeConnectionInner<TConnection> {
    pub fn new() -> Self {
        Self {
            connection: None,
            requested_tables: BTreeMap::new(),
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connection.is_some()
    }

    pub fn get_connection(&self) -> Option<Arc<TConnection>> {
        self.connection.clone()
    }

    /// Every table asked for, with what happened to it so far.
    pub fn get_requested_tables(&self) -> Vec<(String, TableRequestState)> {
        self.requested_tables
            .iter()
            .map(|(table_name, state)| (table_name.to_string(), *state))
            .collect()
    }

    /// Registers the table as requested and returns the connection the request has to go out
    /// over right now. `None` when there is nothing to send: the table is asked for already, or
    /// there is no connection - then the request goes out as soon as there is one.
    pub fn request_table(&mut self, table_name: &str) -> Option<Arc<TConnection>> {
        if self.requested_tables.contains_key(table_name) {
            return None;
        }

        self.requested_tables
            .insert(table_name.to_string(), TableRequestState::Pending);

        self.connection.clone()
    }

    /// A fresh connection has no subscriptions on the main node - every table asked for before
    /// has to be asked for again. Returns them.
    pub fn connected(&mut self, connection: Arc<TConnection>) -> Vec<String> {
        self.connection = Some(connection);
        self.requested_tables.keys().cloned().collect()
    }

    pub fn disconnected(&mut self) {
        self.connection = None;
    }

    pub fn table_replicated(&mut self, table_name: &str) {
        self.requested_tables
            .insert(table_name.to_string(), TableRequestState::Replicated);
    }

    pub fn table_not_found(&mut self, table_name: &str) {
        self.requested_tables
            .insert(table_name.to_string(), TableRequestState::NotFound);
    }

    pub fn get_tables_to_retry(&self) -> Option<TablesToRetry<TConnection>> {
        let connection = self.connection.clone()?;

        let tables: Vec<String> = self
            .requested_tables
            .iter()
            .filter(|(_, state)| **state == TableRequestState::NotFound)
            .map(|(table_name, _)| table_name.to_string())
            .collect();

        if tables.is_empty() {
            return None;
        }

        Some(TablesToRetry { connection, tables })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{MainNodeConnectionInner, TableRequestState};

    struct TestConnection;

    #[test]
    fn test_table_requested_while_disconnected_is_requested_on_connect() {
        let mut inner = MainNodeConnectionInner::<TestConnection>::new();

        assert!(inner.request_table("table").is_none());

        let to_request = inner.connected(Arc::new(TestConnection));
        assert_eq!(vec!["table".to_string()], to_request);
    }

    #[test]
    fn test_table_is_requested_once() {
        let mut inner = MainNodeConnectionInner::<TestConnection>::new();
        inner.connected(Arc::new(TestConnection));

        assert!(inner.request_table("table").is_some());
        assert!(inner.request_table("table").is_none());
    }

    #[test]
    fn test_every_requested_table_is_requested_again_after_reconnect() {
        let mut inner = MainNodeConnectionInner::<TestConnection>::new();
        inner.connected(Arc::new(TestConnection));

        inner.request_table("pending");
        inner.request_table("replicated");
        inner.table_replicated("replicated");

        inner.disconnected();
        assert!(!inner.is_connected());

        let to_request = inner.connected(Arc::new(TestConnection));
        assert_eq!(
            vec!["pending".to_string(), "replicated".to_string()],
            to_request
        );
    }

    #[test]
    fn test_not_found_table_is_retried_until_it_arrives() {
        let mut inner = MainNodeConnectionInner::<TestConnection>::new();
        inner.connected(Arc::new(TestConnection));

        inner.request_table("table");
        inner.table_not_found("table");

        // Still requested: another reader of it does not ask the main node once more.
        assert!(inner.request_table("table").is_none());

        let to_retry = inner.get_tables_to_retry().unwrap();
        assert_eq!(vec!["table".to_string()], to_retry.tables);

        inner.table_replicated("table");
        assert!(inner.get_tables_to_retry().is_none());
    }

    #[test]
    fn test_requested_tables_report_their_state() {
        let mut inner = MainNodeConnectionInner::<TestConnection>::new();
        inner.connected(Arc::new(TestConnection));

        inner.request_table("pending");
        inner.request_table("replicated");
        inner.request_table("not-found");
        inner.table_replicated("replicated");
        inner.table_not_found("not-found");

        assert_eq!(
            vec![
                ("not-found".to_string(), TableRequestState::NotFound),
                ("pending".to_string(), TableRequestState::Pending),
                ("replicated".to_string(), TableRequestState::Replicated),
            ],
            inner.get_requested_tables()
        );
    }

    #[test]
    fn test_nothing_is_retried_while_disconnected() {
        let mut inner = MainNodeConnectionInner::<TestConnection>::new();

        inner.request_table("table");
        inner.table_not_found("table");

        assert!(inner.get_tables_to_retry().is_none());
    }
}
