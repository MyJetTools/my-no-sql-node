use std::sync::Arc;

use my_http_server::{HttpContext, HttpRequestHeaders};

use crate::{app::AppContext, db_operations::DbOperationError, namespaces::NodeNamespace};

/// Header naming the namespace a request works in - the one the main node reads. No header, or
/// an empty one, means the default namespace.
///
/// It is read from the request here, for every action, so the query parameter fallback applies
/// everywhere. The input contracts declare the header too, but only so it shows up in swagger.
pub const NAMESPACE_HEADER: &str = "ns";

/// Namespace the request names, if it names one at all. The header wins; a query parameter of
/// the same name is the fallback for requests which can not carry a header.
pub fn get_request_namespace_name(ctx: &HttpContext) -> Option<&str> {
    let header = ctx
        .request
        .get_headers()
        .try_get_case_insensitive_as_str(NAMESPACE_HEADER)
        .ok()
        .flatten()
        .filter(|value| !value.is_empty());

    if header.is_some() {
        return header;
    }

    find_query_param(ctx.request.get_uri().query()?, NAMESPACE_HEADER)
}

/// Validated namespace name. A missing or empty one is the default namespace.
pub fn parse_namespace_name(name: Option<&str>) -> Result<&str, DbOperationError> {
    let name = match name.map(|name| name.trim()) {
        Some(name) if !name.is_empty() => name,
        _ => return Ok(my_no_sql_sdk::DEFAULT_NAMESPACE),
    };

    if let Err(err) = my_no_sql_sdk::validate_namespace_name(name) {
        return Err(DbOperationError::NamespaceNameValidationError(
            err.to_string(),
        ));
    }

    Ok(name)
}

/// Namespace an HTTP read works in. The node holds only the namespaces its readers subscribed
/// in, so one it has never heard of is answered "namespace not found".
pub fn get_request_namespace(
    app: &AppContext,
    ctx: &HttpContext,
) -> Result<Arc<NodeNamespace>, DbOperationError> {
    let name = parse_namespace_name(get_request_namespace_name(ctx))?;

    match app.namespaces.get(name) {
        Some(namespace) => Ok(namespace),
        None => Err(DbOperationError::NamespaceNotFound(name.to_string())),
    }
}

pub fn find_query_param<'s>(query: &'s str, key: &str) -> Option<&'s str> {
    for pair in query.split('&') {
        // A valueless element (`?flag`) is somebody else's parameter, not the end of the query.
        let Some((pair_key, value)) = pair.split_once('=') else {
            continue;
        };

        if pair_key.eq_ignore_ascii_case(key) {
            if value.is_empty() {
                return None;
            }

            return Some(value);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_query_param() {
        assert_eq!(
            Some("alpha"),
            find_query_param("tableName=t&ns=alpha", NAMESPACE_HEADER)
        );
        assert_eq!(
            Some("alpha"),
            find_query_param("flag&NS=alpha", NAMESPACE_HEADER)
        );
        assert_eq!(None, find_query_param("ns=&tableName=t", NAMESPACE_HEADER));
        assert_eq!(None, find_query_param("tableName=t", NAMESPACE_HEADER));
    }

    #[test]
    fn test_parse_namespace_name() {
        assert_eq!(
            my_no_sql_sdk::DEFAULT_NAMESPACE,
            parse_namespace_name(None).unwrap()
        );
        assert_eq!(
            my_no_sql_sdk::DEFAULT_NAMESPACE,
            parse_namespace_name(Some(" ")).unwrap()
        );
        assert_eq!("alpha", parse_namespace_name(Some("alpha")).unwrap());
        assert!(parse_namespace_name(Some("Alpha")).is_err());
    }
}
