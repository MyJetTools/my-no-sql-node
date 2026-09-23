use my_no_sql_sdk::{
    core::rust_extensions::date_time::DateTimeAsMicroseconds,
    tcp_contracts::sync_to_main::UpdateEntityStatisticsData,
};

/// The read statistics a request asks to update, as the headers every read accepts carry them.
/// The node does not keep them itself - they are forwarded to the main node.
pub struct UpdateStatisticsHeaders<'s> {
    pub update_partition_last_read_time: Option<bool>,
    pub set_partition_expiration_time: Option<&'s str>,
    pub update_rows_last_read_time: Option<bool>,
    pub set_rows_expiration_time: Option<&'s str>,
}

impl<'s> From<UpdateStatisticsHeaders<'s>> for UpdateEntityStatisticsData {
    fn from(src: UpdateStatisticsHeaders<'s>) -> Self {
        Self {
            partition_last_read_moment: src.update_partition_last_read_time.unwrap_or(false),
            row_last_read_moment: src.update_rows_last_read_time.unwrap_or(false),
            partition_expiration_moment: parse_expiration_time(src.set_partition_expiration_time),
            row_expiration_moment: parse_expiration_time(src.set_rows_expiration_time),
        }
    }
}

/// `None` - the header is absent, the expiration time stays as it is. `Some(None)` - the value
/// is not a date, which resets the expiration time: the entity never expires.
fn parse_expiration_time(src: Option<&str>) -> Option<Option<DateTimeAsMicroseconds>> {
    Some(DateTimeAsMicroseconds::from_str(src?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_headers_update_nothing() {
        let result: UpdateEntityStatisticsData = UpdateStatisticsHeaders {
            update_partition_last_read_time: None,
            set_partition_expiration_time: None,
            update_rows_last_read_time: None,
            set_rows_expiration_time: None,
        }
        .into();

        assert!(!result.has_data_to_update());
    }

    #[test]
    fn test_expiration_time_is_set_and_reset() {
        let result: UpdateEntityStatisticsData = UpdateStatisticsHeaders {
            update_partition_last_read_time: Some(true),
            set_partition_expiration_time: Some("2026-01-01T00:00:00"),
            update_rows_last_read_time: None,
            set_rows_expiration_time: Some(""),
        }
        .into();

        assert!(result.partition_last_read_moment);
        assert!(!result.row_last_read_moment);
        assert!(result.partition_expiration_moment.unwrap().is_some());
        assert!(result.row_expiration_moment.unwrap().is_none());
    }
}
