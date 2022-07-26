use my_http_server_controllers::controllers::documentation::{
    data_types::HttpDataType, out_results::HttpResult,
};

pub fn empty(description: &str) -> HttpResult {
    HttpResult {
        http_code: 202,
        nullable: true,
        description: description.to_string(),
        data_type: HttpDataType::None,
    }
}
