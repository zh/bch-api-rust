mod common;

use serde_json::json;

const BASE: &str = "/v6/full-node/control";

#[tokio::test]
async fn status_returns_control() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "control");
}

#[tokio::test]
async fn get_network_info_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(
        &rpc,
        "getnetworkinfo",
        json!({"version": 250100, "subversion": "/Bitcoin Cash Node:25.1.0/"}),
    )
    .await;

    let (status, json) = common::get(app, &format!("{BASE}/getNetworkInfo")).await;
    assert_eq!(status, 200);
    assert_eq!(json["version"], 250100);
}

#[tokio::test]
async fn get_network_info_rpc_error() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc_error(&rpc, "getnetworkinfo", -1, "not available").await;

    let (status, json) = common::get(app, &format!("{BASE}/getNetworkInfo")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("RPC error"));
}
