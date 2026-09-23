use std::time::Duration;

use flurl::{body::HttpRequestBody, FlUrl, FlUrlError, FlUrlMode};
use my_http_client::MyHttpClientError;
use my_http_server::{hyper::Method, HttpContext, HttpFailResult, HttpOkResult, HttpOutput};

use super::forwarded_headers::*;

/// Smaller bodies are sent as they are - gzip does not pay off on them.
const COMPRESS_MIN_SIZE: usize = 1024;

/// Bigger bodies are sent as they are too: the main node decodes a compressed body up to this
/// size and answers 413 past it, while a plain body is taken whatever its size.
const COMPRESS_MAX_SIZE: usize = my_http_server::DEFAULT_MAX_DECOMPRESSED_BODY_SIZE;

/// What a writer gets when the node can not get its request to the main node. A 400: the SDK
/// writer treats some 5xx answers as success (`clean_and_bulk_insert`, `delete_partitions`), and a
/// write which did not happen must never look like one which did.
const MAIN_SERVER_FAILED_STATUS_CODE: u16 = 400;

/// On top of the request timeout, for connecting - fl-url's own timeout covers only the wait for
/// the answer, while the connections to a main node which is down are dialled one after another,
/// 5 s each.
const CONNECT_ALLOWANCE: Duration = Duration::from_secs(1);

/// How a forward ended - for the log and the metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForwardOutcome {
    /// The main node answered - whatever the status.
    Answered(u16),
    /// The request never reached the main node.
    NotReachable,
    /// The connection broke with the request in flight - it may have been applied.
    Broken,
    /// No answer in time - the request may have been applied.
    Timeout,
    /// The node is not told where the main node is.
    NotConfigured,
    /// The request was refused before it went anywhere.
    Refused,
}

impl ForwardOutcome {
    pub fn as_metric_label(&self) -> &'static str {
        match self {
            ForwardOutcome::Answered(status) if *status < 300 => "2xx",
            ForwardOutcome::Answered(status) if *status < 400 => "3xx",
            ForwardOutcome::Answered(status) if *status < 500 => "4xx",
            ForwardOutcome::Answered(_) => "5xx",
            ForwardOutcome::NotReachable => "not_reachable",
            ForwardOutcome::Broken => "broken",
            ForwardOutcome::Timeout => "timeout",
            ForwardOutcome::NotConfigured => "not_configured",
            ForwardOutcome::Refused => "refused",
        }
    }
}

/// The main node's HTTP api, reached the way a writer reaches it: over h2 (h2c with prior
/// knowledge for `http://`, ALPN for `https://`), one multiplexed connection.
pub struct MainServerHttp {
    url: String,
    compress: bool,
    timeout: Duration,
}

impl MainServerHttp {
    pub fn new(url: String, compress: bool, timeout: Duration) -> Self {
        Self {
            url,
            compress,
            timeout,
        }
    }

    /// Sends the request on to the main node as it came - method, path and query byte for byte,
    /// headers, body - and answers with what the main node answered: status, headers, body.
    pub async fn forward(
        &self,
        ctx: &mut HttpContext,
    ) -> (Result<HttpOkResult, HttpFailResult>, ForwardOutcome) {
        let method = ctx.request.method.clone();

        let carries_body = method == Method::POST || method == Method::PUT;
        if !carries_body && method != Method::GET && method != Method::DELETE {
            return (
                Err(HttpFailResult::from((
                    405u16,
                    format!("Method {} is not forwarded to the main node", method),
                ))),
                ForwardOutcome::Refused,
            );
        }

        // Validated at start up - an error here must still never become a panic, which the
        // server would answer with a 500.
        let fl_url = match FlUrl::try_new(self.url.as_str()) {
            Ok(fl_url) => fl_url,
            Err(err) => {
                let message = format!("MainServerHttp is not a valid url: {:?}", err);
                return (
                    Err(HttpFailResult::from((MAIN_SERVER_FAILED_STATUS_CODE, message))),
                    ForwardOutcome::Refused,
                );
            }
        };

        let mut fl_url = fl_url
            .update_mode(FlUrlMode::H2)
            .set_timeout(self.timeout)
            .append_raw_ending_to_url(ctx.request.get_path_and_query());

        let mut incoming_forwarded_for = None;

        for (name, value) in ctx.request.data.headers().iter() {
            let name = name.as_str();

            // A value which is not text can not be given to fl-url - no header of the api is.
            let Ok(value) = value.to_str() else {
                continue;
            };

            if name.eq_ignore_ascii_case("x-forwarded-for") {
                incoming_forwarded_for = Some(value.to_string());
                continue;
            }

            if !is_forwarded_request_header(name) {
                continue;
            }

            fl_url = fl_url.with_header(name, value);
        }

        let client_ip = ctx.request.addr.ip_as_string();
        fl_url = fl_url.with_header(
            "x-forwarded-for",
            build_forwarded_for(incoming_forwarded_for.as_deref(), client_ip.as_str()),
        );

        let body = if carries_body {
            // Decoded already if the client compressed it - by the action parsing its input, or
            // right here.
            let body = match ctx.request.receive_body().await {
                Ok(body) => body.get_body(),
                Err(err) => return (Err(err), ForwardOutcome::Refused),
            };

            if self.compress && (COMPRESS_MIN_SIZE..=COMPRESS_MAX_SIZE).contains(&body.len()) {
                fl_url = fl_url.compress();
            }

            Some(HttpRequestBody::from_raw_data(body, None))
        } else {
            None
        };

        let send = async move {
            match body {
                Some(body) if method == Method::POST => fl_url.post(body).await,
                Some(body) => fl_url.put(body).await,
                None if method == Method::GET => fl_url.get().await,
                // No DELETE of the api carries a body - its parameters are in the query.
                None => fl_url.delete().await,
            }
        };

        let response = match tokio::time::timeout(self.timeout + CONNECT_ALLOWANCE, send).await {
            Ok(Ok(response)) => response,
            Ok(Err(err)) => return self.failed(ctx, classify_error(&err), format!("{}", err)),
            Err(_) => return self.failed(ctx, ForwardOutcome::Timeout, String::new()),
        };

        let status = response.get_status_code();
        let mut response = response.into_hyper_response();

        let headers = response.headers_mut();
        for name in NOT_FORWARDED_RESPONSE_HEADERS {
            headers.remove(name);
        }

        (
            HttpOutput::Raw(response).into_ok_result(false),
            ForwardOutcome::Answered(status),
        )
    }

    fn failed(
        &self,
        ctx: &HttpContext,
        outcome: ForwardOutcome,
        err: String,
    ) -> (Result<HttpOkResult, HttpFailResult>, ForwardOutcome) {
        let message = match outcome {
            ForwardOutcome::NotReachable => {
                format!("Main node {} is not reachable: {}", self.url, err)
            }
            ForwardOutcome::Timeout => format!(
                "Main node {} did not answer in {:?}. The request may or may not have been applied",
                self.url, self.timeout
            ),
            ForwardOutcome::Broken
            | ForwardOutcome::Answered(_)
            | ForwardOutcome::NotConfigured
            | ForwardOutcome::Refused => format!(
                "The connection to the main node {} failed with the request in flight: {}. The request may or may not have been applied",
                self.url, err
            ),
        };

        my_logger::LOGGER.write_warning(
            "MainServerHttp",
            message.as_str(),
            my_logger::LogEventCtx::new()
                .add("method", ctx.request.method.to_string())
                .add("path", ctx.request.get_path_and_query().to_string()),
        );

        (
            Err(HttpFailResult::from((MAIN_SERVER_FAILED_STATUS_CODE, message))),
            outcome,
        )
    }
}

/// Only an error of connecting says the request did not reach the main node. Any other one came
/// with the request already sent - or being sent.
fn classify_error(err: &FlUrlError) -> ForwardOutcome {
    if err.is_timeout() {
        return ForwardOutcome::Timeout;
    }

    let not_sent = matches!(
        err,
        FlUrlError::MyHttpClientError(MyHttpClientError::CanNotConnectToRemoteHost(_))
            | FlUrlError::CanNotEstablishConnection(_)
            | FlUrlError::InvalidUrl(_)
    );

    if not_sent {
        ForwardOutcome::NotReachable
    } else {
        ForwardOutcome::Broken
    }
}
