mod common;

use serde_json::json;
use wiremock::matchers::{body_partial_json, method};
use wiremock::{Mock, ResponseTemplate};

const BASE: &str = "/v6/encryption";

// A valid 66-char compressed public key (hex)
const VALID_PUBKEY: &str = "02b4632d08485ff1df2db55b9dafd23347d1c47a457072a1e87be26896549a8737";

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_returns_encryption() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "encryption");
}

// ---------------------------------------------------------------------------
// getPublicKey
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_pubkey_found() {
    let (app, rpc, fulcrum, _) = common::setup().await;

    // Fulcrum returns transaction history
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": [{"tx_hash": common::VALID_TXID, "height": 700000}]}),
    )
    .await;

    // Full node returns verbose raw tx with scriptSig containing pubkey
    common::mock_rpc(
        &rpc,
        "getrawtransaction",
        json!({
            "txid": common::VALID_TXID,
            "vin": [{
                "scriptSig": {
                    "asm": format!("3045022100abcd {VALID_PUBKEY}")
                }
            }]
        }),
    )
    .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/publickey/{}", common::VALID_ADDRESS)).await;
    assert_eq!(status, 200);
    assert_eq!(json["success"], true);
    assert_eq!(json["publicKey"], VALID_PUBKEY);
}

#[tokio::test]
async fn get_pubkey_not_found_no_history() {
    let (app, _, fulcrum, _) = common::setup().await;

    // Empty transaction history
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": []}),
    )
    .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/publickey/{}", common::VALID_ADDRESS)).await;
    assert_eq!(status, 200);
    assert_eq!(json["success"], true);
    assert_eq!(json["publicKey"], "not found");
}

#[tokio::test]
async fn get_pubkey_not_found_no_scriptsig() {
    let (app, rpc, fulcrum, _) = common::setup().await;

    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": [{"tx_hash": common::VALID_TXID, "height": 700000}]}),
    )
    .await;

    // Raw tx without scriptSig (e.g., coinbase or segwit)
    common::mock_rpc(
        &rpc,
        "getrawtransaction",
        json!({
            "txid": common::VALID_TXID,
            "vin": [{"coinbase": "04ffff001d0104"}]
        }),
    )
    .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/publickey/{}", common::VALID_ADDRESS)).await;
    assert_eq!(status, 200);
    assert_eq!(json["publicKey"], "not found");
}

#[tokio::test]
async fn get_pubkey_invalid_address() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::get(app, &format!("{BASE}/publickey/notanaddress")).await;
    assert_eq!(status, 400);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("Invalid BCH address"));
}

#[tokio::test]
async fn get_pubkey_multi_tx_search() {
    let (app, rpc, fulcrum, _) = common::setup().await;

    let txid2 = "b1075db55d416d3ca199f55b6084e2115b9345e16c5cf302fc80e9d5fbf5d48d";

    // Two transactions
    common::mock_fulcrum_get(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        json!({"transactions": [
            {"tx_hash": common::VALID_TXID, "height": 700000},
            {"tx_hash": txid2, "height": 700001}
        ]}),
    )
    .await;

    // First tx: no pubkey in scriptSig
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "getrawtransaction", "params": [common::VALID_TXID, 1]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": {
                "txid": common::VALID_TXID,
                "vin": [{"scriptSig": {"asm": "3045022100abcdef 304502210012"}}]
            },
            "error": null,
            "id": "bch-api-rust-getrawtransaction"
        })))
        .mount(&rpc)
        .await;

    // Second tx: has pubkey
    Mock::given(method("POST"))
        .and(body_partial_json(
            json!({"method": "getrawtransaction", "params": [txid2, 1]}),
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "result": {
                "txid": txid2,
                "vin": [{
                    "scriptSig": {
                        "asm": format!("3045022100abcd {VALID_PUBKEY}")
                    }
                }]
            },
            "error": null,
            "id": "bch-api-rust-getrawtransaction"
        })))
        .mount(&rpc)
        .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/publickey/{}", common::VALID_ADDRESS)).await;
    assert_eq!(status, 200);
    assert_eq!(json["publicKey"], VALID_PUBKEY);
}

#[tokio::test]
async fn get_pubkey_fulcrum_error() {
    let (app, _, fulcrum, _) = common::setup().await;

    common::mock_fulcrum_error(
        &fulcrum,
        &format!("/electrumx/transactions/{}", common::VALID_ADDRESS),
        500,
    )
    .await;

    let (status, json) =
        common::get(app, &format!("{BASE}/publickey/{}", common::VALID_ADDRESS)).await;
    // 500 from fulcrum → 502 Bad Gateway
    assert_eq!(status, 502);
    assert!(json["error"].as_str().is_some());
}
