pub mod fulcrum;
pub mod full_node;
pub mod slp;

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use reqwest::Client;
use serde_json::Value;
use std::fmt;
use std::time::Duration;

/// Build a shared reqwest HTTP client with the given timeout.
pub fn build_http_client(timeout_ms: u64) -> Client {
    Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .expect("failed to build reqwest client")
}

// ---------------------------------------------------------------------------
// ApiError
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub enum ApiError {
    /// Backend did not respond in time → 504
    BackendTimeout(String),
    /// Backend returned an error → status depends on upstream code
    BackendError {
        status: u16,
        message: String,
        service: String,
    },
    /// Caller sent bad input → 400
    InvalidInput(String),
    /// Bitcoin RPC returned an error object → 400
    RpcError { code: i64, message: String },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BackendTimeout(svc) => write!(f, "{svc} backend timed out"),
            Self::BackendError {
                status,
                message,
                service,
            } => write!(f, "{service} backend error ({status}): {message}"),
            Self::InvalidInput(msg) => write!(f, "invalid input: {msg}"),
            Self::RpcError { code, message } => write!(f, "RPC error {code}: {message}"),
        }
    }
}

impl std::error::Error for ApiError {}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_msg) = match &self {
            ApiError::BackendTimeout(_) => (StatusCode::GATEWAY_TIMEOUT, self.to_string()),
            ApiError::RpcError { .. } => (StatusCode::BAD_REQUEST, self.to_string()),
            ApiError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            ApiError::BackendError {
                status,
                message,
                service,
            } => {
                let sc = match *status {
                    429 => StatusCode::TOO_MANY_REQUESTS,
                    501 => StatusCode::NOT_IMPLEMENTED,
                    500 | 502..=599 => StatusCode::BAD_GATEWAY,
                    s @ 400..=499 => StatusCode::from_u16(s).unwrap_or(StatusCode::BAD_REQUEST),
                    _ => StatusCode::INTERNAL_SERVER_ERROR,
                };
                (sc, format!("{service}: {message}"))
            }
        };
        (
            status,
            axum::Json(serde_json::json!({ "error": error_msg })),
        )
            .into_response()
    }
}

// ---------------------------------------------------------------------------
// HttpProxyClient — shared by Fulcrum and SLP
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct HttpProxyClient {
    pub(crate) http: Client,
    pub(crate) base_url: String,
    pub(crate) service: String,
}

impl HttpProxyClient {
    pub fn new(base_url: &str, timeout_ms: u64, service: &str) -> Self {
        Self {
            http: build_http_client(timeout_ms),
            base_url: base_url.trim_end_matches('/').to_string(),
            service: service.to_string(),
        }
    }

    pub async fn get(&self, path: &str) -> Result<Value, ApiError> {
        with_retry(|| self.do_get(path)).await
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value, ApiError> {
        with_retry(|| self.do_post(path, body.clone())).await
    }

    async fn do_get(&self, path: &str) -> Result<Value, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| map_reqwest_error(e, &self.service))?;

        self.handle_response(resp).await
    }

    async fn do_post(&self, path: &str, body: Value) -> Result<Value, ApiError> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| map_reqwest_error(e, &self.service))?;

        self.handle_response(resp).await
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value, ApiError> {
        let status = resp.status();
        if status.is_success() {
            resp.json::<Value>()
                .await
                .map_err(|e| ApiError::BackendError {
                    status: 500,
                    message: format!("invalid JSON from backend: {e}"),
                    service: self.service.clone(),
                })
        } else {
            let message = resp.text().await.unwrap_or_default();
            Err(ApiError::BackendError {
                status: status.as_u16(),
                message,
                service: self.service.clone(),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

pub fn map_reqwest_error(e: reqwest::Error, service: &str) -> ApiError {
    if e.is_timeout() {
        ApiError::BackendTimeout(service.to_string())
    } else if e.is_connect() {
        ApiError::BackendError {
            status: 503,
            message: format!("connection refused: {e}"),
            service: service.to_string(),
        }
    } else {
        ApiError::BackendError {
            status: 500,
            message: e.to_string(),
            service: service.to_string(),
        }
    }
}

/// Retry once on 5xx or timeout.
pub async fn with_retry<F, Fut>(f: F) -> Result<Value, ApiError>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<Value, ApiError>>,
{
    let result = f().await;
    match &result {
        Err(ApiError::BackendTimeout(_))
        | Err(ApiError::BackendError {
            status: 500..=599, ..
        }) => {
            // One retry
            f().await
        }
        _ => result,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: convert an ApiError into a response and extract status code.
    async fn error_status(err: ApiError) -> u16 {
        let resp = err.into_response();
        resp.status().as_u16()
    }

    #[tokio::test]
    async fn backend_timeout_returns_504() {
        assert_eq!(
            error_status(ApiError::BackendTimeout("test".into())).await,
            504
        );
    }

    #[tokio::test]
    async fn rpc_error_returns_400() {
        assert_eq!(
            error_status(ApiError::RpcError {
                code: -1,
                message: "bad".into()
            })
            .await,
            400
        );
    }

    #[tokio::test]
    async fn invalid_input_returns_400() {
        assert_eq!(
            error_status(ApiError::InvalidInput("nope".into())).await,
            400
        );
    }

    #[tokio::test]
    async fn backend_error_429_returns_429() {
        assert_eq!(
            error_status(ApiError::BackendError {
                status: 429,
                message: "rate limit".into(),
                service: "test".into(),
            })
            .await,
            429
        );
    }

    #[tokio::test]
    async fn backend_error_500_returns_502() {
        assert_eq!(
            error_status(ApiError::BackendError {
                status: 500,
                message: "fail".into(),
                service: "test".into(),
            })
            .await,
            502
        );
    }

    #[tokio::test]
    async fn backend_error_502_returns_502() {
        assert_eq!(
            error_status(ApiError::BackendError {
                status: 502,
                message: "fail".into(),
                service: "test".into(),
            })
            .await,
            502
        );
    }

    #[tokio::test]
    async fn backend_error_403_returns_403() {
        assert_eq!(
            error_status(ApiError::BackendError {
                status: 403,
                message: "forbidden".into(),
                service: "test".into(),
            })
            .await,
            403
        );
    }
}
