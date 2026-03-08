mod common;

use serde_json::json;

const BASE: &str = "/v6/full-node/dsproof";

#[tokio::test]
async fn status_returns_dsproof() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "dsproof");
}

#[tokio::test]
async fn get_ds_proof_success() {
    let (app, rpc, _, _) = common::setup().await;
    // Default verbose → verbosity 2
    common::mock_rpc(&rpc, "getdsproof", json!({"dspid": "abc123"})).await;

    let (status, json) =
        common::get(app, &format!("{BASE}/getDSProof/{}", common::VALID_TXID)).await;
    assert_eq!(status, 200);
    assert_eq!(json["dspid"], "abc123");
}

#[tokio::test]
async fn get_ds_proof_verbose_true() {
    let (app, rpc, _, _) = common::setup().await;
    // ?verbose=true → verbosity 3
    common::mock_rpc(&rpc, "getdsproof", json!({"dspid": "abc123", "hex": "ff"})).await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/getDSProof/{}?verbose=true", common::VALID_TXID),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["dspid"], "abc123");
}

#[tokio::test]
async fn get_ds_proof_invalid_txid() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::get(app, &format!("{BASE}/getDSProof/badtxid")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("64-character"));
}
