mod common;

use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, ResponseTemplate};

const BASE: &str = "/v6/fulcrum";

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_returns_fulcrum() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "fulcrum");
}

// ---------------------------------------------------------------------------
// Balance
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_balance_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/balance/{}", common::VALID_ADDRESS),
        json!({"balance": {"confirmed": 100000, "unconfirmed": 0}}),
    )
    .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/balance/{}", common::VALID_ADDRESS)).await;
    assert_eq!(status, 200);
    assert_eq!(json["balance"]["confirmed"], 100000);
}

#[tokio::test]
async fn get_balance_invalid_address() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::get(app, &format!("{BASE}/balance/notanaddress")).await;
    assert_eq!(status, 400);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid BCH address"));
}

#[tokio::test]
async fn balance_bulk_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(
        &fulcrum,
        "/electrumx/balance/",
        json!({"balances": [{"confirmed": 100000}]}),
    )
    .await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/balance"),
        json!({"addresses": [common::VALID_ADDRESS]}),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.get("balances").is_some());
}

#[tokio::test]
async fn balance_bulk_not_array() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/balance"),
        json!({"addresses": "not-array"}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("needs to be an array"));
}

#[tokio::test]
async fn balance_bulk_too_large() {
    let (app, _, _, _) = common::setup().await;
    let addresses: Vec<&str> = vec![common::VALID_ADDRESS; 25];

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/balance"),
        json!({"addresses": addresses}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("too large"));
}

// ---------------------------------------------------------------------------
// UTXOs
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_utxos_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/utxos/{}", common::VALID_ADDRESS),
        json!({"utxos": [{"tx_hash": common::VALID_TXID, "value": 50000}]}),
    )
    .await;

    let (status, json) = common::get(app, &format!("{BASE}/utxos/{}", common::VALID_ADDRESS)).await;
    assert_eq!(status, 200);
    assert!(json.get("utxos").is_some());
}

#[tokio::test]
async fn utxos_bulk_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(&fulcrum, "/electrumx/utxos/", json!({"utxos": []})).await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/utxos"),
        json!({"addresses": [common::VALID_ADDRESS]}),
    )
    .await;
    assert_eq!(status, 200);
}

// ---------------------------------------------------------------------------
// TX Data
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_tx_data_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/tx/data/{}", common::VALID_TXID),
        json!({"txid": common::VALID_TXID}),
    )
    .await;

    let (status, json) = common::get(app, &format!("{BASE}/tx/data/{}", common::VALID_TXID)).await;
    assert_eq!(status, 200);
    assert_eq!(json["txid"], common::VALID_TXID);
}

#[tokio::test]
async fn tx_data_bulk_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(&fulcrum, "/electrumx/tx/data", json!({"txData": []})).await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/tx/data"),
        json!({"txids": [common::VALID_TXID]}),
    )
    .await;
    assert_eq!(status, 200);
}

// ---------------------------------------------------------------------------
// TX Broadcast
// ---------------------------------------------------------------------------

#[tokio::test]
async fn tx_broadcast_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(
        &fulcrum,
        "/electrumx/tx/broadcast",
        json!(common::VALID_TXID),
    )
    .await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/tx/broadcast"),
        json!({"txHex": "0200aabb"}),
    )
    .await;
    assert_eq!(status, 200);
}

// ---------------------------------------------------------------------------
// Block Headers
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_block_headers_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    // Default count=1 is forwarded as query param
    Mock::given(method("GET"))
        .and(path("/electrumx/block/headers/700000"))
        .and(query_param("count", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({"headers": "hex_data"})))
        .mount(&fulcrum)
        .await;

    let (status, json) = common::get(app, &format!("{BASE}/block/headers/700000")).await;
    assert_eq!(status, 200);
    assert_eq!(json["headers"], "hex_data");
}

#[tokio::test]
async fn get_block_headers_with_count() {
    let (app, _, fulcrum, _) = common::setup().await;
    Mock::given(method("GET"))
        .and(path("/electrumx/block/headers/700000"))
        .and(query_param("count", "2"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({"headers": ["header1", "header2"]})),
        )
        .mount(&fulcrum)
        .await;

    let (status, json) = common::get(app, &format!("{BASE}/block/headers/700000?count=2")).await;
    assert_eq!(status, 200);
    let headers = json["headers"].as_array().unwrap();
    assert_eq!(headers.len(), 2);
}

#[tokio::test]
async fn block_headers_bulk_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(&fulcrum, "/electrumx/block/headers", json!({"headers": []})).await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/block/headers"),
        json!({"heights": [700000, 700001]}),
    )
    .await;
    assert_eq!(status, 200);
}

// ---------------------------------------------------------------------------
// Transactions
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_transactions_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": [{"tx_hash": common::VALID_TXID, "height": 700000}]}),
    )
    .await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/transactions/{}", common::VALID_ADDRESS),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.get("transactions").is_some());
    // Result is sorted and truncated to 100
    assert_eq!(json["transactions"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn get_transactions_all_txs() {
    let (app, _, fulcrum, _) = common::setup().await;
    // all_txs variant calls fulcrum without the suffix, then sorts/truncates
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": [
            {"tx_hash": "aaa", "height": 100},
            {"tx_hash": "bbb", "height": 300},
            {"tx_hash": "ccc", "height": 200}
        ]}),
    )
    .await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/transactions/{}/true", common::VALID_ADDRESS),
    )
    .await;
    assert_eq!(status, 200);
    // With all_txs=true, all transactions returned (no truncation), sorted desc
    let txs = json["transactions"].as_array().unwrap();
    assert_eq!(txs.len(), 3);
    assert_eq!(txs[0]["height"], 300);
    assert_eq!(txs[1]["height"], 200);
    assert_eq!(txs[2]["height"], 100);
}

#[tokio::test]
async fn get_transactions_all_txs_false_truncates() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": [
            {"tx_hash": "aaa", "height": 100},
            {"tx_hash": "bbb", "height": 200}
        ]}),
    )
    .await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/transactions/{}/false", common::VALID_ADDRESS),
    )
    .await;
    assert_eq!(status, 200);
    // With all_txs=false, truncated to 100 (but only 2 here), sorted desc
    let txs = json["transactions"].as_array().unwrap();
    assert_eq!(txs.len(), 2);
    assert_eq!(txs[0]["height"], 200);
    assert_eq!(txs[1]["height"], 100);
}

#[tokio::test]
async fn transactions_bulk_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(
        &fulcrum,
        "/electrumx/transactions/",
        json!({"transactions": []}),
    )
    .await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/transactions"),
        json!({"addresses": [common::VALID_ADDRESS]}),
    )
    .await;
    assert_eq!(status, 200);
}

// ---------------------------------------------------------------------------
// Unconfirmed
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_unconfirmed_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/unconfirmed/{}", common::VALID_ADDRESS),
        json!({"utxos": []}),
    )
    .await;

    let (status, _) = common::get(
        app,
        &format!("{BASE}/unconfirmed/{}", common::VALID_ADDRESS),
    )
    .await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn unconfirmed_bulk_success() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_post(&fulcrum, "/electrumx/unconfirmed/", json!({"utxos": []})).await;

    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/unconfirmed"),
        json!({"addresses": [common::VALID_ADDRESS]}),
    )
    .await;
    assert_eq!(status, 200);
}

// ---------------------------------------------------------------------------
// Backend error propagation
// ---------------------------------------------------------------------------

#[tokio::test]
async fn fulcrum_backend_error() {
    let (app, _, fulcrum, _) = common::setup().await;
    common::mock_fulcrum_error(
        &fulcrum,
        &format!("/electrumx/balance/{}", common::VALID_ADDRESS),
        503,
    )
    .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/balance/{}", common::VALID_ADDRESS)).await;
    // 503 from backend → mapped to 502 (Bad Gateway) by ApiError::BackendError
    assert_eq!(status, 502);
    assert!(json["error"].as_str().is_some());
}
