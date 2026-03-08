use super::HttpProxyClient;

pub type FulcrumClient = HttpProxyClient;

pub fn new(base_url: &str, timeout_ms: u64) -> FulcrumClient {
    HttpProxyClient::new(base_url, timeout_ms, "fulcrum")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clients::ApiError;
    use serde_json::json;
    use std::time::Duration;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn get_returns_json() {
        let server = MockServer::start().await;
        let client = new(&server.uri(), 5000);

        Mock::given(method("GET"))
            .and(path("/electrumx/balance"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"confirmed": 100000})))
            .mount(&server)
            .await;

        let result = client.get("/electrumx/balance").await.unwrap();
        assert_eq!(result["confirmed"], 100000);
    }

    #[tokio::test]
    async fn post_sends_body() {
        let server = MockServer::start().await;
        let client = new(&server.uri(), 5000);

        let req_body = json!({"address": "bitcoincash:qz..."});

        Mock::given(method("POST"))
            .and(path("/electrumx/utxos"))
            .and(body_json(&req_body))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"utxos": []})))
            .mount(&server)
            .await;

        let result = client.post("/electrumx/utxos", req_body).await.unwrap();
        assert_eq!(result["utxos"], json!([]));
    }

    #[tokio::test]
    async fn not_found_returns_backend_error() {
        let server = MockServer::start().await;
        let client = new(&server.uri(), 5000);

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(404).set_body_string("not found"))
            .mount(&server)
            .await;

        let err = client.get("/missing").await.unwrap_err();
        match err {
            ApiError::BackendError { status, .. } => assert_eq!(status, 404),
            other => panic!("expected BackendError, got: {other}"),
        }
    }

    #[tokio::test]
    async fn retry_on_5xx() {
        let server = MockServer::start().await;
        let client = new(&server.uri(), 5000);

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503).set_body_string("down"))
            .up_to_n_times(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
            .mount(&server)
            .await;

        let result = client.get("/test").await.unwrap();
        assert_eq!(result["ok"], true);
    }

    #[tokio::test]
    async fn timeout_returns_backend_timeout() {
        let server = MockServer::start().await;
        let client = new(&server.uri(), 50);

        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({}))
                    .set_delay(Duration::from_secs(5)),
            )
            .mount(&server)
            .await;

        let err = client.get("/slow").await.unwrap_err();
        assert!(matches!(err, ApiError::BackendTimeout(_)));
    }
}
