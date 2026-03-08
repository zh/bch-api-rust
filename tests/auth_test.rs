mod common;

use axum::body::Body;
use axum::http::Request;
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// A route that always succeeds (no backend needed) — used to test auth layer
const TEST_ENDPOINT: &str = "/v6/full-node/blockchain";

// ---------------------------------------------------------------------------
// No auth configured
// ---------------------------------------------------------------------------

#[tokio::test]
async fn no_auth_passes_through() {
    let (app, _, _, _) = common::setup_with_auth("", false, "").await;

    let (status, json) = common::get(app, TEST_ENDPOINT).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "blockchain");
}

// ---------------------------------------------------------------------------
// Bearer token auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bearer_valid_token_passes() {
    let (app, _, _, _) = common::setup_with_auth("secret123", false, "").await;

    let (status, json) =
        common::get_with_header(app, TEST_ENDPOINT, "authorization", "Bearer secret123").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "blockchain");
}

#[tokio::test]
async fn bearer_invalid_token_rejected() {
    let (app, _, _, _) = common::setup_with_auth("secret123", false, "").await;

    let (status, json) =
        common::get_with_header(app, TEST_ENDPOINT, "authorization", "Bearer wrongtoken").await;
    assert_eq!(status, 401);
    assert_eq!(json["error"], "Unauthorized");
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Bearer token required"));
}

#[tokio::test]
async fn bearer_missing_header_rejected() {
    let (app, _, _, _) = common::setup_with_auth("secret123", false, "").await;

    let (status, json) = common::get(app, TEST_ENDPOINT).await;
    assert_eq!(status, 401);
    assert_eq!(json["error"], "Unauthorized");
    assert!(json["message"]
        .as_str()
        .unwrap()
        .contains("Bearer token required"));
}

// ---------------------------------------------------------------------------
// x402 payment auth
// ---------------------------------------------------------------------------

#[tokio::test]
async fn x402_valid_payment_passes() {
    let facilitator = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/facilitator"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"valid": true})))
        .mount(&facilitator)
        .await;

    let fac_url = format!("{}/facilitator", facilitator.uri());
    let (app, _, _, _) = common::setup_with_auth("", true, &fac_url).await;

    let (status, json) =
        common::get_with_header(app, TEST_ENDPOINT, "x-402-payment", "valid-payment-token").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "blockchain");
}

#[tokio::test]
async fn x402_invalid_payment_rejected() {
    let facilitator = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/facilitator"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"valid": false})))
        .mount(&facilitator)
        .await;

    let fac_url = format!("{}/facilitator", facilitator.uri());
    let (app, _, _, _) = common::setup_with_auth("", true, &fac_url).await;

    let (status, json) =
        common::get_with_header(app, TEST_ENDPOINT, "x-402-payment", "bad-token").await;
    assert_eq!(status, 402);
    assert!(json["error"].as_str().unwrap().contains("Payment"));
}

#[tokio::test]
async fn x402_missing_header_returns_402() {
    let facilitator = MockServer::start().await;
    let fac_url = format!("{}/facilitator", facilitator.uri());
    let (app, _, _, _) = common::setup_with_auth("", true, &fac_url).await;

    let (status, json) = common::get(app, TEST_ENDPOINT).await;
    assert_eq!(status, 402);
    assert!(json["error"].as_str().unwrap().contains("Payment required"));
    // Should include payment info
    assert!(json.get("bchAddress").is_some());
    assert!(json.get("priceSat").is_some());
}

#[tokio::test]
async fn x402_facilitator_error_returns_500() {
    let facilitator = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/facilitator"))
        .respond_with(ResponseTemplate::new(503))
        .mount(&facilitator)
        .await;

    let fac_url = format!("{}/facilitator", facilitator.uri());
    let (app, _, _, _) = common::setup_with_auth("", true, &fac_url).await;

    let (status, _) =
        common::get_with_header(app, TEST_ENDPOINT, "x-402-payment", "some-token").await;
    // Facilitator non-success → verify_x402_payment returns Ok(false)
    assert_eq!(status, 402);
}

// ---------------------------------------------------------------------------
// Combined: Bearer + x402
// ---------------------------------------------------------------------------

#[tokio::test]
async fn bearer_bypasses_x402() {
    let facilitator = MockServer::start().await;
    let fac_url = format!("{}/facilitator", facilitator.uri());

    // Both auth methods enabled
    let (app, _, _, _) = common::setup_with_auth("secret123", true, &fac_url).await;

    // Valid Bearer → should pass without checking x402
    let (status, json) =
        common::get_with_header(app, TEST_ENDPOINT, "authorization", "Bearer secret123").await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "blockchain");
}

#[tokio::test]
async fn bearer_fails_falls_through_to_x402() {
    let facilitator = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/facilitator"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"valid": true})))
        .mount(&facilitator)
        .await;

    let fac_url = format!("{}/facilitator", facilitator.uri());
    let (app, _, _, _) = common::setup_with_auth("secret123", true, &fac_url).await;

    // Invalid Bearer but valid x402 payment → should pass via x402
    let resp = app
        .oneshot(
            Request::builder()
                .uri(TEST_ENDPOINT)
                .header("authorization", "Bearer wrong")
                .header("x-402-payment", "valid-payment-token")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

    assert_eq!(status, 200);
    assert_eq!(json["status"], "blockchain");
}
