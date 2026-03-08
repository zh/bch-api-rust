#![allow(dead_code)]

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::middleware as axum_mw;
use axum::Router;
use bch_api_rust::{clients, middleware, routes, AppState, Config};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, MockServer, ResponseTemplate};

pub const VALID_HASH: &str = "000000000000000002e63058c9bda37ad72fc98e3154cce2de15e76a33f9e71e";
pub const VALID_TXID: &str = "a1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";
/// Known valid mainnet cashaddr (P2PKH for hash160 of the first ever BCH tx).
pub const VALID_ADDRESS: &str = "bitcoincash:qr6m7j9njldwwzlg9v7v53unlr4jkmx6eylep8ekg2";

// ---------------------------------------------------------------------------
// App builders
// ---------------------------------------------------------------------------

pub async fn setup() -> (Router, MockServer, MockServer, MockServer) {
    let rpc = MockServer::start().await;
    let fulcrum = MockServer::start().await;
    let slp = MockServer::start().await;

    let state = build_state(&rpc, &fulcrum, &slp, false, "", false, "");

    let app = Router::new()
        .nest("/v6", routes::api_routes())
        .with_state(state);

    (app, rpc, fulcrum, slp)
}

pub async fn setup_with_auth(
    bearer_token: &str,
    x402_enabled: bool,
    facilitator_url: &str,
) -> (Router, MockServer, MockServer, MockServer) {
    let rpc = MockServer::start().await;
    let fulcrum = MockServer::start().await;
    let slp = MockServer::start().await;

    let use_basic_auth = !bearer_token.is_empty();
    let state = build_state(
        &rpc,
        &fulcrum,
        &slp,
        use_basic_auth,
        bearer_token,
        x402_enabled,
        facilitator_url,
    );

    let mut api_router = routes::api_routes();
    if use_basic_auth || x402_enabled {
        api_router = api_router.layer(axum_mw::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));
    }

    let app = Router::new().nest("/v6", api_router).with_state(state);

    (app, rpc, fulcrum, slp)
}

fn build_state(
    rpc: &MockServer,
    fulcrum: &MockServer,
    slp: &MockServer,
    use_basic_auth: bool,
    basic_auth_token: &str,
    x402_enabled: bool,
    facilitator_url: &str,
) -> AppState {
    let cfg = Config {
        port: 0,
        api_prefix: "/v6".into(),
        network: "mainnet".into(),
        rpc_baseurl: rpc.uri(),
        rpc_username: "user".into(),
        rpc_password: "pass".into(),
        rpc_timeout_ms: 5000,
        fulcrum_api: fulcrum.uri(),
        fulcrum_timeout_ms: 5000,
        slp_indexer_api: slp.uri(),
        slp_indexer_timeout_ms: 5000,
        x402_enabled,
        server_bch_address: "bitcoincash:testaddr".into(),
        facilitator_url: facilitator_url.into(),
        x402_price_sat: 200,
        use_basic_auth,
        basic_auth_token: basic_auth_token.into(),
        coinex_api_url: "https://api.coinex.com/v1/market/ticker?market=bchusdt".into(),
        psffpp_proxy_url: String::new(),
    };

    AppState {
        config: Arc::new(cfg),
        http_client: reqwest::Client::new(),
        full_node: clients::full_node::FullNodeClient::new(&rpc.uri(), "user", "pass", 5000),
        fulcrum: clients::fulcrum::new(&fulcrum.uri(), 5000),
        slp: clients::slp::new(&slp.uri(), 5000),
    }
}

// ---------------------------------------------------------------------------
// Request helpers
// ---------------------------------------------------------------------------

pub async fn get(app: Router, uri: &str) -> (StatusCode, Value) {
    let resp = app
        .oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    extract(resp).await
}

pub async fn get_with_header(
    app: Router,
    uri: &str,
    header_name: &str,
    header_value: &str,
) -> (StatusCode, Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .header(header_name, header_value)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    extract(resp).await
}

pub async fn post_json(app: Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    extract(resp).await
}

async fn extract(resp: axum::http::Response<Body>) -> (StatusCode, Value) {
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: Value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, json)
}

// ---------------------------------------------------------------------------
// Mock helpers
// ---------------------------------------------------------------------------

pub async fn mock_rpc(server: &MockServer, rpc_method: &str, result: Value) {
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": rpc_method})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": result,
            "error": null,
            "id": format!("bch-api-rust-{rpc_method}")
        })))
        .mount(server)
        .await;
}

pub async fn mock_rpc_error(server: &MockServer, rpc_method: &str, code: i64, msg: &str) {
    Mock::given(method("POST"))
        .and(body_partial_json(json!({"method": rpc_method})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": null,
            "error": {"code": code, "message": msg},
            "id": format!("bch-api-rust-{rpc_method}")
        })))
        .mount(server)
        .await;
}

pub async fn mock_fulcrum_get(server: &MockServer, path: &str, result: Value) {
    Mock::given(method("GET"))
        .and(wiremock::matchers::path(path))
        .respond_with(ResponseTemplate::new(200).set_body_json(result))
        .mount(server)
        .await;
}

pub async fn mock_fulcrum_post(server: &MockServer, path: &str, result: Value) {
    Mock::given(method("POST"))
        .and(wiremock::matchers::path(path))
        .respond_with(ResponseTemplate::new(200).set_body_json(result))
        .mount(server)
        .await;
}

pub async fn mock_fulcrum_error(server: &MockServer, path: &str, status: u16) {
    Mock::given(wiremock::matchers::path(path))
        .respond_with(
            ResponseTemplate::new(status).set_body_json(json!({"error": "backend error"})),
        )
        .mount(server)
        .await;
}
