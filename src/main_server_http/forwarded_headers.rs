/// Headers of the incoming request which do not travel on: hop-by-hop ones, the ones fl-url and
/// hyper set themselves for the new request (`host` would also become the TLS server name and
/// part of the connection pool key), and `x-forwarded-for`, which is rebuilt with the client.
///
/// `content-encoding` does not travel either: my-http-server decodes a compressed body as it reads
/// it, so what the node sends on is always the decoded body - encoded again only by the node
/// itself (`CompressWrites`).
const NOT_FORWARDED_REQUEST_HEADERS: [&str; 12] = [
    "host",
    "connection",
    "keep-alive",
    "proxy-connection",
    "transfer-encoding",
    "te",
    "trailer",
    "upgrade",
    "content-length",
    "expect",
    "x-forwarded-for",
    "content-encoding",
];

/// Hop-by-hop headers of the main node's answer - they describe the connection it came over,
/// not the answer.
pub const NOT_FORWARDED_RESPONSE_HEADERS: [&str; 7] = [
    "connection",
    "keep-alive",
    "proxy-connection",
    "transfer-encoding",
    "te",
    "trailer",
    "upgrade",
];

pub fn is_forwarded_request_header(name: &str) -> bool {
    !NOT_FORWARDED_REQUEST_HEADERS
        .iter()
        .any(|itm| name.eq_ignore_ascii_case(itm))
}

/// `x-forwarded-for` with the client appended - the chain a proxy in front of the node started
/// is kept.
pub fn build_forwarded_for(incoming: Option<&str>, client_ip: &str) -> String {
    match incoming.map(|itm| itm.trim()).filter(|itm| !itm.is_empty()) {
        Some(incoming) => format!("{}, {}", incoming, client_ip),
        None => client_ip.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hop_by_hop_and_rebuilt_headers_do_not_travel() {
        for name in [
            "Host",
            "content-length",
            "Connection",
            "Transfer-Encoding",
            "X-Forwarded-For",
            "Content-Encoding",
        ] {
            assert!(!is_forwarded_request_header(name), "{name}");
        }

        for name in ["ns", "session", "content-type", "apikey", "x-compress"] {
            assert!(is_forwarded_request_header(name), "{name}");
        }
    }

    #[test]
    fn test_forwarded_for_chain() {
        assert_eq!("10.0.0.5", build_forwarded_for(None, "10.0.0.5"));
        assert_eq!("10.0.0.5", build_forwarded_for(Some(" "), "10.0.0.5"));
        assert_eq!(
            "1.2.3.4, 10.0.0.5",
            build_forwarded_for(Some("1.2.3.4"), "10.0.0.5")
        );
    }
}
