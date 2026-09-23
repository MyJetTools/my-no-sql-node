use my_http_server::macros::*;
use my_no_sql_sdk::tcp_contracts::sync_to_main::UpdateEntityStatisticsData;
use rest_api_shared::GetRowInputModel;

use crate::http::controllers::UpdateStatisticsHeaders;

#[derive(MyHttpInput)]
pub struct RowsCountInputContract {
    #[http_header(name = "ns"; description = "Namespace to work in. Empty or absent means the default namespace")]
    pub namespace: Option<String>,

    #[http_query(name = "tableName"; description = "Name of a table")]
    pub table_name: String,

    #[http_query(name = "partitionKey"; description = "Partition Key")]
    pub partition_key: Option<String>,
}

/// The read statistics a `/api/Row` request asks to update. The shared wire model carries no
/// behaviour, so this lives on the node side.
pub fn get_update_statistics(input_data: &GetRowInputModel) -> UpdateEntityStatisticsData {
    UpdateStatisticsHeaders {
        update_partition_last_read_time: input_data.update_partition_last_read_access_time,
        set_partition_expiration_time: input_data.set_partition_expiration_time.as_deref(),
        update_rows_last_read_time: input_data.update_db_rows_last_read_access_time,
        set_rows_expiration_time: input_data.set_db_rows_expiration_time.as_deref(),
    }
    .into()
}
