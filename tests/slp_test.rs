mod common;

use serde_json::json;

const BASE: &str = "/v6/slp";

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_returns_slp() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "slp");
}

#[tokio::test]
async fn slp_status_success() {
    let (app, _, _, slp) = common::setup().await;
    common::mock_fulcrum_get(&slp, "/slp/status/", json!({"status": "running"})).await;

    let (status, json) = common::get(app, &format!("{BASE}/status")).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "running");
}

// ---------------------------------------------------------------------------
// Address
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slp_address_success() {
    let (app, _, _, slp) = common::setup().await;
    common::mock_fulcrum_post(
        &slp,
        "/slp/address/",
        json!({"balance": {"confirmed": 1000}}),
    )
    .await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/address"),
        json!({"address": common::VALID_ADDRESS}),
    )
    .await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn slp_address_empty() {
    let (app, _, _, _) = common::setup().await;

    let (status, _) =
        common::post_json(app, &format!("{BASE}/address"), json!({"address": ""})).await;
    assert_eq!(status, 400);
}

#[tokio::test]
async fn slp_address_missing() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::post_json(app, &format!("{BASE}/address"), json!({})).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("address"));
}

// ---------------------------------------------------------------------------
// Txid
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slp_txid_success() {
    let (app, _, _, slp) = common::setup().await;
    common::mock_fulcrum_post(&slp, "/slp/tx/", json!({"isValid": true})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/txid"),
        json!({"txid": common::VALID_TXID}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["isValid"], true);
}

#[tokio::test]
async fn slp_txid_empty() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::post_json(app, &format!("{BASE}/txid"), json!({"txid": ""})).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("is required"));
}

#[tokio::test]
async fn slp_txid_wrong_length() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) =
        common::post_json(app, &format!("{BASE}/txid"), json!({"txid": "abcdef"})).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("64-character"));
}

// ---------------------------------------------------------------------------
// Token
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slp_token_success() {
    let (app, _, _, slp) = common::setup().await;
    common::mock_fulcrum_post(&slp, "/slp/token/", json!({"tokenId": "abc"})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/token"),
        json!({"tokenId": common::VALID_TXID}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["tokenId"], "abc");
}

#[tokio::test]
async fn slp_token_data_success() {
    let (app, _, _, slp) = common::setup().await;
    common::mock_fulcrum_post(
        &slp,
        "/slp/token/",
        json!({"tokenData": {"ticker": "TEST", "documentUri": "ipfs://abc123"}}),
    )
    .await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/token/data"),
        json!({"tokenId": common::VALID_TXID}),
    )
    .await;
    assert_eq!(status, 200);
    // Response transformed: tokenData → genesisData, with immutableData and mutableData
    assert!(json.get("genesisData").is_some());
    assert_eq!(json["genesisData"]["ticker"], "TEST");
    assert_eq!(json["immutableData"], "ipfs://abc123");
    assert_eq!(json["mutableData"], "");
}

// ---------------------------------------------------------------------------
// Backend error propagation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn slp_backend_error() {
    let (app, _, _, slp) = common::setup().await;
    common::mock_fulcrum_error(&slp, "/slp/status/", 500).await;

    let (status, json) = common::get(app, &format!("{BASE}/status")).await;
    // 500 from SLP → mapped to 502 by ApiError::BackendError
    assert_eq!(status, 502);
    assert!(json["error"].as_str().is_some());
}
