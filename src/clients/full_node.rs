use reqwest::Client;
use serde_json::{json, Value};

use super::{build_http_client, map_reqwest_error, with_retry, ApiError};

const SERVICE: &str = "full_node";
const DEFAULT_RPC_ERROR_CODE: i64 = -1;
const DEFAULT_RPC_ERROR_MSG: &str = "unknown RPC error";

/// Try to extract an RPC error from a JSON-RPC response body.
fn extract_rpc_error(body: &Value) -> Option<ApiError> {
    let err = body.get("error").filter(|e| !e.is_null())?;
    let code = err
        .get("code")
        .and_then(|c| c.as_i64())
        .unwrap_or(DEFAULT_RPC_ERROR_CODE);
    let message = err
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or(DEFAULT_RPC_ERROR_MSG)
        .to_string();
    Some(ApiError::RpcError { code, message })
}

#[derive(Clone)]
pub struct FullNodeClient {
    http: Client,
    base_url: String,
    username: String,
    password: String,
}

impl FullNodeClient {
    pub fn new(base_url: &str, username: &str, password: &str, timeout_ms: u64) -> Self {
        Self {
            http: build_http_client(timeout_ms),
            base_url: base_url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    /// Send a JSON-RPC 1.0 call to the full node.
    pub async fn call(&self, method: &str, params: Value) -> Result<Value, ApiError> {
        let method = method.to_string();
        let params = params.clone();
        with_retry(|| self.do_call(&method, &params)).await
    }

    async fn do_call(&self, method: &str, params: &Value) -> Result<Value, ApiError> {
        let payload = json!({
            "jsonrpc": "1.0",
            "id": format!("bch-api-rust-{method}"),
            "method": method,
            "params": params,
        });

        let resp = self
            .http
            .post(&self.base_url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&payload)
            .send()
            .await
            .map_err(|e| map_reqwest_error(e, SERVICE))?;

        let status = resp.status();
        if !status.is_success() {
            // bitcoind returns HTTP 500 for RPC errors with a JSON body.
            // Try to extract the RPC error before falling back to BackendError.
            let body_text = resp.text().await.unwrap_or_default();
            if let Ok(body) = serde_json::from_str::<Value>(&body_text) {
                if let Some(err) = extract_rpc_error(&body) {
                    return Err(err);
                }
            }
            return Err(ApiError::BackendError {
                status: status.as_u16(),
                message: body_text,
                service: SERVICE.to_string(),
            });
        }

        let body: Value = resp.json().await.map_err(|e| ApiError::BackendError {
            status: 500,
            message: format!("invalid JSON from {SERVICE}: {e}"),
            service: SERVICE.to_string(),
        })?;

        // Check for RPC-level error
        if let Some(err) = extract_rpc_error(&body) {
            return Err(err);
        }

        // Return the "result" field
        Ok(body.get("result").cloned().unwrap_or(Value::Null))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use wiremock::matchers::{body_json, header, method};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup() -> (MockServer, FullNodeClient) {
        let server = MockServer::start().await;
        let client = FullNodeClient::new(&server.uri(), "user", "pass", 5000);
        (server, client)
    }

    #[tokio::test]
    async fn valid_rpc_response() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": { "blocks": 800000 },
                "error": null,
                "id": "bch-api-rust-getblockchaininfo"
            })))
            .mount(&server)
            .await;

        let result = client.call("getblockchaininfo", json!([])).await.unwrap();
        assert_eq!(result["blocks"], 800000);
    }

    #[tokio::test]
    async fn rpc_error_response() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": null,
                "error": { "code": -5, "message": "Invalid address" },
                "id": "bch-api-rust-validateaddress"
            })))
            .mount(&server)
            .await;

        let err = client
            .call("validateaddress", json!(["bad"]))
            .await
            .unwrap_err();
        match err {
            ApiError::RpcError { code, message } => {
                assert_eq!(code, -5);
                assert_eq!(message, "Invalid address");
            }
            other => panic!("expected RpcError, got: {other}"),
        }
    }

    #[tokio::test]
    async fn correct_payload_format() {
        let (server, client) = setup().await;

        Mock::given(body_json(json!({
            "jsonrpc": "1.0",
            "id": "bch-api-rust-getblock",
            "method": "getblock",
            "params": ["abc123", 1]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": "ok",
            "error": null,
            "id": "bch-api-rust-getblock"
        })))
        .mount(&server)
        .await;

        let result = client.call("getblock", json!(["abc123", 1])).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn sends_basic_auth_header() {
        let (server, client) = setup().await;

        // "user:pass" base64 → "dXNlcjpwYXNz"
        Mock::given(header("authorization", "Basic dXNlcjpwYXNz"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": true,
                "error": null,
                "id": "bch-api-rust-test"
            })))
            .mount(&server)
            .await;

        let result = client.call("test", json!([])).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn timeout_returns_backend_timeout() {
        let (server, _) = setup().await;
        // Create client with very short timeout
        let client = FullNodeClient::new(&server.uri(), "user", "pass", 50);

        Mock::given(method("POST"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"result": null, "error": null, "id": "x"}))
                    .set_delay(Duration::from_secs(5)),
            )
            .mount(&server)
            .await;

        let err = client.call("test", json!([])).await.unwrap_err();
        assert!(matches!(err, ApiError::BackendTimeout(_)));
    }

    #[tokio::test]
    async fn retry_on_500_succeeds_second_time() {
        let (server, client) = setup().await;

        // First call returns 500, second returns success
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": "retry_ok",
                "error": null,
                "id": "bch-api-rust-test"
            })))
            .mount(&server)
            .await;

        let result = client.call("test", json!([])).await.unwrap();
        assert_eq!(result, "retry_ok");
    }

    #[tokio::test]
    async fn retry_on_500_both_fail() {
        let (server, client) = setup().await;

        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_string("server down"))
            .mount(&server)
            .await;

        let err = client.call("test", json!([])).await.unwrap_err();
        match err {
            ApiError::BackendError { status, .. } => assert_eq!(status, 500),
            other => panic!("expected BackendError, got: {other}"),
        }
    }

    #[tokio::test]
    async fn rpc_error_in_500_response() {
        let (server, client) = setup().await;

        // bitcoind returns HTTP 500 with JSON RPC error body
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                "result": null,
                "error": { "code": -5, "message": "Transaction not in mempool" },
                "id": "bch-api-rust-getmempoolentry"
            })))
            .mount(&server)
            .await;

        let err = client
            .call("getmempoolentry", json!(["abc"]))
            .await
            .unwrap_err();
        match err {
            ApiError::RpcError { code, message } => {
                assert_eq!(code, -5);
                assert_eq!(message, "Transaction not in mempool");
            }
            other => panic!("expected RpcError, got: {other}"),
        }
    }

    #[tokio::test]
    async fn connection_refused() {
        // Connect to a port with no listener
        let client = FullNodeClient::new("http://127.0.0.1:1", "u", "p", 2000);
        let err = client.call("test", json!([])).await.unwrap_err();
        match err {
            ApiError::BackendError { status, .. } => assert_eq!(status, 503),
            ApiError::BackendTimeout(_) => {} // also acceptable on some platforms
            other => panic!("expected BackendError(503) or BackendTimeout, got: {other}"),
        }
    }
}
