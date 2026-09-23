use std::collections::BTreeSet;

use my_no_sql_sdk::core::db::DbNamespaceName;

/// Everything about a reader which changes while it is connected. It lives behind a single
/// lock: the namespace may only change while the reader neither holds nor waits for a table,
/// and that check has to see the very tables it guards.
pub(super) struct DataReaderUpdatableData {
    namespace: DbNamespaceName,
    tables: BTreeSet<String>,
    /// Tables the reader subscribed to which the node does not hold yet - they are requested
    /// from the main node, and the reader is served once the answer arrives.
    awaiting_tables: BTreeSet<String>,
}

impl DataReaderUpdatableData {
    pub fn new() -> Self {
        Self {
            namespace: DbNamespaceName::default(),
            tables: BTreeSet::new(),
            awaiting_tables: BTreeSet::new(),
        }
    }

    pub fn get_namespace(&self) -> &DbNamespaceName {
        &self.namespace
    }

    /// Refused once the reader holds or waits for a table: it has been given that namespace's
    /// data, and swapping the namespace underneath would leave it with the rows of one namespace
    /// while being told about the updates of another. Re-stating the current namespace is not a
    /// change - that is what lets an HTTP reader subscribe to a second table.
    pub fn set_namespace(&mut self, namespace: DbNamespaceName) -> Result<(), String> {
        if self.namespace == namespace {
            return Ok(());
        }

        if !self.tables.is_empty() || !self.awaiting_tables.is_empty() {
            return Err(format!(
                "Namespace can not be changed to '{}' after the connection is subscribed to a table",
                namespace
            ));
        }

        self.namespace = namespace;

        Ok(())
    }

    /// `false` - the reader works in another namespace by now: a concurrent HTTP subscribe of
    /// the same session switched it. Nothing is changed then.
    pub fn subscribe(&mut self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        if self.namespace != *namespace {
            return false;
        }

        self.tables.insert(table_name.to_string());
        true
    }

    pub fn unsubscribe(&mut self, table_name: &str) {
        self.tables.remove(table_name);
        self.awaiting_tables.remove(table_name);
    }

    pub fn is_subscribed_to(&self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        self.namespace == *namespace && self.tables.contains(table_name)
    }

    /// `false` - the reader works in another namespace by now, see `subscribe`.
    pub fn add_awaiting_table(&mut self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        if self.namespace != *namespace {
            return false;
        }

        self.awaiting_tables.insert(table_name.to_string());
        true
    }

    /// Removes the table from the awaited ones. `true` means the caller is the one to serve the
    /// reader - whoever takes it first does, and nobody else.
    pub fn take_awaiting_table(&mut self, namespace: &DbNamespaceName, table_name: &str) -> bool {
        if self.namespace != *namespace {
            return false;
        }

        self.awaiting_tables.remove(table_name)
    }

    pub fn get_table_names(&self) -> Vec<String> {
        self.tables.iter().cloned().collect()
    }

    pub fn get_awaiting_table_names(&self) -> Vec<String> {
        self.awaiting_tables.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use my_no_sql_sdk::core::db::DbNamespaceName;

    use super::DataReaderUpdatableData;

    #[test]
    fn test_namespace_is_set_before_subscription() {
        let mut data = DataReaderUpdatableData::new();

        data.set_namespace("alpha".into()).unwrap();
        assert!(data.subscribe(&"alpha".into(), "table"));

        assert!(data.is_subscribed_to(&"alpha".into(), "table"));
        assert!(!data.is_subscribed_to(&DbNamespaceName::default(), "table"));
    }

    #[test]
    fn test_namespace_can_not_be_changed_after_subscription() {
        let mut data = DataReaderUpdatableData::new();

        assert!(data.subscribe(&DbNamespaceName::default(), "table"));

        assert!(data.set_namespace("alpha".into()).is_err());
        assert!(data.set_namespace(DbNamespaceName::default()).is_ok());
    }

    #[test]
    fn test_namespace_can_not_be_changed_while_waiting_for_a_table() {
        let mut data = DataReaderUpdatableData::new();

        assert!(data.add_awaiting_table(&DbNamespaceName::default(), "table"));

        assert!(data.set_namespace("alpha".into()).is_err());
    }

    #[test]
    fn test_awaiting_table_is_taken_once_and_in_its_namespace_only() {
        let mut data = DataReaderUpdatableData::new();
        data.set_namespace("alpha".into()).unwrap();

        assert!(data.add_awaiting_table(&"alpha".into(), "table"));

        assert!(!data.take_awaiting_table(&DbNamespaceName::default(), "table"));
        assert!(data.take_awaiting_table(&"alpha".into(), "table"));
        assert!(!data.take_awaiting_table(&"alpha".into(), "table"));
    }

    #[test]
    fn test_mutations_of_another_namespace_are_refused() {
        let mut data = DataReaderUpdatableData::new();
        data.set_namespace("beta".into()).unwrap();

        assert!(!data.subscribe(&"alpha".into(), "table"));
        assert!(!data.add_awaiting_table(&"alpha".into(), "table"));
        assert!(!data.is_subscribed_to(&"beta".into(), "table"));
        assert!(data.set_namespace("alpha".into()).is_ok());
    }

    #[test]
    fn test_unsubscribe_forgets_awaiting_table() {
        let mut data = DataReaderUpdatableData::new();

        assert!(data.add_awaiting_table(&DbNamespaceName::default(), "table"));
        data.unsubscribe("table");

        assert!(!data.take_awaiting_table(&DbNamespaceName::default(), "table"));
    }
}
