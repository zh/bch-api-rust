mod common;

use serde_json::json;

const BASE: &str = "/v6/full-node/mining";

#[tokio::test]
async fn status_returns_mining() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "mining");
}

#[tokio::test]
async fn get_mining_info_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(
        &rpc,
        "getmininginfo",
        json!({"blocks": 800000, "difficulty": 1.5e11}),
    )
    .await;

    let (status, json) = common::get(app, &format!("{BASE}/getMiningInfo")).await;
    assert_eq!(status, 200);
    assert_eq!(json["blocks"], 800000);
}

#[tokio::test]
async fn get_mining_info_rpc_error() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc_error(&rpc, "getmininginfo", -1, "Internal error").await;

    let (status, json) = common::get(app, &format!("{BASE}/getMiningInfo")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("RPC error"));
}

#[tokio::test]
async fn get_network_hash_ps_defaults() {
    let (app, rpc, _, _) = common::setup().await;
    // Default: nblocks=120, height=-1
    common::mock_rpc(&rpc, "getnetworkhashps", json!(1.23e18)).await;

    let (status, json) = common::get(app, &format!("{BASE}/getNetworkHashPS")).await;
    assert_eq!(status, 200);
    assert!(json.is_number());
}

#[tokio::test]
async fn get_network_hash_ps_custom_params() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getnetworkhashps", json!(5.67e18)).await;

    let (status, _) = common::get(
        app,
        &format!("{BASE}/getNetworkHashPS?nblocks=200&height=700000"),
    )
    .await;
    assert_eq!(status, 200);
}
