mod common;

use serde_json::json;

const BASE: &str = "/v6/full-node/rawtransactions";

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_returns_rawtransactions() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "rawtransactions");
}

// ---------------------------------------------------------------------------
// decodeRawTransaction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn decode_raw_tx_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "decoderawtransaction", json!({"txid": "abc"})).await;

    let (status, json) = common::get(app, &format!("{BASE}/decodeRawTransaction/0200aabbcc")).await;
    assert_eq!(status, 200);
    assert_eq!(json["txid"], "abc");
}

#[tokio::test]
async fn decode_raw_tx_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "decoderawtransaction", json!({"txid": "abc"})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/decodeRawTransaction"),
        json!({"hexes": ["0200aa", "0200bb"]}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn decode_raw_tx_bulk_not_array() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/decodeRawTransaction"),
        json!({"hexes": "not-array"}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("needs to be an array"));
}

#[tokio::test]
async fn decode_raw_tx_bulk_too_large() {
    let (app, _, _, _) = common::setup().await;
    let hexes: Vec<&str> = vec!["aa"; 25];
    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/decodeRawTransaction"),
        json!({"hexes": hexes}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("too large"));
}

// ---------------------------------------------------------------------------
// decodeScript
// ---------------------------------------------------------------------------

#[tokio::test]
async fn decode_script_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "decodescript", json!({"asm": "OP_DUP"})).await;

    let (status, json) = common::get(app, &format!("{BASE}/decodeScript/76a914")).await;
    assert_eq!(status, 200);
    assert_eq!(json["asm"], "OP_DUP");
}

#[tokio::test]
async fn decode_script_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "decodescript", json!({"asm": "OP_DUP"})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/decodeScript"),
        json!({"hexes": ["76a914", "a914"]}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json.as_array().unwrap().len(), 2);
}

// ---------------------------------------------------------------------------
// getRawTransaction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_raw_tx_success_non_verbose() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getrawtransaction", json!("0200aabbccdd...")).await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/getRawTransaction/{}", common::VALID_TXID),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_string()); // hex string when verbose=false (default)
}

#[tokio::test]
async fn get_raw_tx_verbose_appends_height() {
    let (app, rpc, _, _) = common::setup().await;

    // First: getrawtransaction returns verbose obj with blockhash
    common::mock_rpc(
        &rpc,
        "getrawtransaction",
        json!({"txid": common::VALID_TXID, "blockhash": common::VALID_HASH, "confirmations": 10}),
    )
    .await;

    // Second: getblockheader returns height
    common::mock_rpc(&rpc, "getblockheader", json!({"height": 700000})).await;

    let (status, json) = common::get(
        app,
        &format!(
            "{BASE}/getRawTransaction/{}?verbose=true",
            common::VALID_TXID
        ),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["height"], 700000);
    assert_eq!(json["txid"], common::VALID_TXID);
}

#[tokio::test]
async fn get_raw_tx_verbose_no_blockhash() {
    let (app, rpc, _, _) = common::setup().await;
    // Unconfirmed tx — no blockhash field
    common::mock_rpc(
        &rpc,
        "getrawtransaction",
        json!({"txid": common::VALID_TXID, "confirmations": 0}),
    )
    .await;

    let (status, json) = common::get(
        app,
        &format!(
            "{BASE}/getRawTransaction/{}?verbose=true",
            common::VALID_TXID
        ),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.get("height").is_none());
}

#[tokio::test]
async fn get_raw_tx_verbose_header_fails() {
    let (app, rpc, _, _) = common::setup().await;

    common::mock_rpc(
        &rpc,
        "getrawtransaction",
        json!({"txid": common::VALID_TXID, "blockhash": common::VALID_HASH}),
    )
    .await;

    // getblockheader fails → height should be absent
    common::mock_rpc_error(&rpc, "getblockheader", -5, "Block not found").await;

    let (status, json) = common::get(
        app,
        &format!(
            "{BASE}/getRawTransaction/{}?verbose=true",
            common::VALID_TXID
        ),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.get("height").is_none());
    assert_eq!(json["txid"], common::VALID_TXID);
}

#[tokio::test]
async fn get_raw_tx_invalid_txid() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, &format!("{BASE}/getRawTransaction/badtxid")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("64-character"));
}

#[tokio::test]
async fn get_raw_tx_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getrawtransaction", json!("0200aabb...")).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getRawTransaction"),
        json!({"txids": [common::VALID_TXID]}),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_array());
}

#[tokio::test]
async fn get_raw_tx_bulk_not_array() {
    let (app, _, _, _) = common::setup().await;
    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/getRawTransaction"),
        json!({"txids": "not-array"}),
    )
    .await;
    assert_eq!(status, 400);
}

#[tokio::test]
async fn get_raw_tx_bulk_too_large() {
    let (app, _, _, _) = common::setup().await;
    let txids: Vec<&str> = vec![common::VALID_TXID; 25];
    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getRawTransaction"),
        json!({"txids": txids}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("too large"));
}

// ---------------------------------------------------------------------------
// sendRawTransaction
// ---------------------------------------------------------------------------

#[tokio::test]
async fn send_raw_tx_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "sendrawtransaction", json!(common::VALID_TXID)).await;

    let (status, json) = common::get(app, &format!("{BASE}/sendRawTransaction/0200aabbccdd")).await;
    assert_eq!(status, 200);
    assert_eq!(json, common::VALID_TXID);
}

#[tokio::test]
async fn send_raw_tx_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "sendrawtransaction", json!(common::VALID_TXID)).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/sendRawTransaction"),
        json!({"hexes": ["0200aabb", "0200ccdd"]}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn send_raw_tx_bulk_not_array() {
    let (app, _, _, _) = common::setup().await;
    let (status, _) = common::post_json(
        app,
        &format!("{BASE}/sendRawTransaction"),
        json!({"hexes": "not-array"}),
    )
    .await;
    assert_eq!(status, 400);
}
