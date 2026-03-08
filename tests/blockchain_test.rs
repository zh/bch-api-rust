mod common;

use serde_json::json;

const BASE: &str = "/v6/full-node/blockchain";

// ---------------------------------------------------------------------------
// Status
// ---------------------------------------------------------------------------

#[tokio::test]
async fn status_returns_blockchain() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "blockchain");
}

// ---------------------------------------------------------------------------
// getBestBlockHash
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_best_block_hash_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getbestblockhash", json!(common::VALID_HASH)).await;

    let (status, json) = common::get(app, &format!("{BASE}/getBestBlockHash")).await;
    assert_eq!(status, 200);
    assert_eq!(json, common::VALID_HASH);
}

#[tokio::test]
async fn get_best_block_hash_rpc_error() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc_error(&rpc, "getbestblockhash", -28, "Loading block index").await;

    let (status, json) = common::get(app, &format!("{BASE}/getBestBlockHash")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("RPC error"));
}

// ---------------------------------------------------------------------------
// getBlockchainInfo
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_blockchain_info_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getblockchaininfo", json!({"blocks": 800000})).await;

    let (status, json) = common::get(app, &format!("{BASE}/getBlockchainInfo")).await;
    assert_eq!(status, 200);
    assert_eq!(json["blocks"], 800000);
}

// ---------------------------------------------------------------------------
// getBlockCount
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_block_count_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getblockcount", json!(800123)).await;

    let (status, json) = common::get(app, &format!("{BASE}/getBlockCount")).await;
    assert_eq!(status, 200);
    assert_eq!(json, 800123);
}

// ---------------------------------------------------------------------------
// getBlockHeader
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_block_header_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(
        &rpc,
        "getblockheader",
        json!({"hash": common::VALID_HASH, "height": 700000}),
    )
    .await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/getBlockHeader/{}", common::VALID_HASH),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["height"], 700000);
}

#[tokio::test]
async fn get_block_header_verbose_false() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getblockheader", json!("0100000000000000...hex...")).await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/getBlockHeader/{}?verbose=false", common::VALID_HASH),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_string());
}

#[tokio::test]
async fn get_block_header_invalid_hash() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::get(app, &format!("{BASE}/getBlockHeader/badhash")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("64-character"));
}

// ---------------------------------------------------------------------------
// getBlockHeader bulk
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_block_header_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getblockheader", json!({"height": 700000})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getBlockHeader"),
        json!({"hashes": [common::VALID_HASH]}),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_array());
    assert_eq!(json[0]["height"], 700000);
}

#[tokio::test]
async fn get_block_header_bulk_not_array() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getBlockHeader"),
        json!({"hashes": "not-array"}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("needs to be an array"));
}

#[tokio::test]
async fn get_block_header_bulk_too_large() {
    let (app, _, _, _) = common::setup().await;
    let hashes: Vec<&str> = vec![common::VALID_HASH; 25];

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getBlockHeader"),
        json!({"hashes": hashes}),
    )
    .await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("too large"));
}

// ---------------------------------------------------------------------------
// getChainTips
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_chain_tips_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getchaintips", json!([{"height": 800000}])).await;

    let (status, json) = common::get(app, &format!("{BASE}/getChainTips")).await;
    assert_eq!(status, 200);
    assert!(json.is_array());
}

// ---------------------------------------------------------------------------
// getDifficulty
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_difficulty_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getdifficulty", json!(1.23e12)).await;

    let (status, json) = common::get(app, &format!("{BASE}/getDifficulty")).await;
    assert_eq!(status, 200);
    assert!(json.is_number());
}

// ---------------------------------------------------------------------------
// getMempoolEntry
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_mempool_entry_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getmempoolentry", json!({"size": 225})).await;

    let (status, json) = common::get(
        app,
        &format!("{BASE}/getMempoolEntry/{}", common::VALID_TXID),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["size"], 225);
}

#[tokio::test]
async fn get_mempool_entry_invalid_txid() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) = common::get(app, &format!("{BASE}/getMempoolEntry/short")).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("64-character"));
}

#[tokio::test]
async fn get_mempool_entry_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getmempoolentry", json!({"size": 225})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getMempoolEntry"),
        json!({"txids": [common::VALID_TXID]}),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_array());
    assert_eq!(json[0]["size"], 225);
}

// ---------------------------------------------------------------------------
// getMempoolAncestors
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_mempool_ancestors_verbose() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getmempoolancestors", json!({"txid1": {"size": 100}})).await;

    let (status, json) = common::get(
        app,
        &format!(
            "{BASE}/getMempoolAncestors/{}?verbose=true",
            common::VALID_TXID
        ),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_object());
}

// ---------------------------------------------------------------------------
// getMempoolInfo
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_mempool_info_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getmempoolinfo", json!({"size": 42})).await;

    let (status, json) = common::get(app, &format!("{BASE}/getMempoolInfo")).await;
    assert_eq!(status, 200);
    assert_eq!(json["size"], 42);
}

// ---------------------------------------------------------------------------
// getRawMempool
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_raw_mempool_verbose() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getrawmempool", json!({"txid1": {"size": 100}})).await;

    let (status, json) = common::get(app, &format!("{BASE}/getRawMempool?verbose=true")).await;
    assert_eq!(status, 200);
    assert!(json.is_object());
}

// ---------------------------------------------------------------------------
// getTxOut
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_tx_out_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "gettxout", json!({"value": 1.5})).await;

    let (status, json) =
        common::get(app, &format!("{BASE}/getTxOut/{}/0", common::VALID_TXID)).await;
    assert_eq!(status, 200);
    assert_eq!(json["value"], 1.5);
}

#[tokio::test]
async fn get_tx_out_include_mempool_false() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "gettxout", json!({"value": 1.5})).await;

    let (status, _) = common::get(
        app,
        &format!(
            "{BASE}/getTxOut/{}/0?includeMempool=false",
            common::VALID_TXID
        ),
    )
    .await;
    assert_eq!(status, 200);
}

#[tokio::test]
async fn get_tx_out_post_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "gettxout", json!({"value": 2.0})).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getTxOut"),
        json!({"txid": common::VALID_TXID, "vout": 1}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["value"], 2.0);
}

#[tokio::test]
async fn get_tx_out_post_missing_txid() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) =
        common::post_json(app, &format!("{BASE}/getTxOut"), json!({"vout": 0})).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("txid"));
}

// ---------------------------------------------------------------------------
// getTxOutProof
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_tx_out_proof_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "gettxoutproof", json!("proof_hex_data")).await;

    let (status, json) =
        common::get(app, &format!("{BASE}/getTxOutProof/{}", common::VALID_TXID)).await;
    assert_eq!(status, 200);
    assert_eq!(json, "proof_hex_data");
}

#[tokio::test]
async fn get_tx_out_proof_bulk_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "gettxoutproof", json!("proof_hex_data")).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getTxOutProof"),
        json!({"txids": [common::VALID_TXID]}),
    )
    .await;
    assert_eq!(status, 200);
    assert!(json.is_array());
}

// ---------------------------------------------------------------------------
// verifyTxOutProof
// ---------------------------------------------------------------------------

#[tokio::test]
async fn verify_tx_out_proof_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "verifytxoutproof", json!([common::VALID_TXID])).await;

    let (status, json) = common::get(app, &format!("{BASE}/verifyTxOutProof/someproofhex")).await;
    assert_eq!(status, 200);
    assert!(json.is_array());
}

#[tokio::test]
async fn verify_tx_out_proof_bulk_flattens() {
    let (app, rpc, _, _) = common::setup().await;
    // verifytxoutproof returns an array; bulk should flatten
    common::mock_rpc(&rpc, "verifytxoutproof", json!(["txid_a", "txid_b"])).await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/verifyTxOutProof"),
        json!({"proofs": ["proof1"]}),
    )
    .await;
    assert_eq!(status, 200);
    let arr = json.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0], "txid_a");
    assert_eq!(arr[1], "txid_b");
}

// ---------------------------------------------------------------------------
// getBlock
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_block_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(
        &rpc,
        "getblock",
        json!({"hash": common::VALID_HASH, "tx": []}),
    )
    .await;

    let (status, json) = common::post_json(
        app,
        &format!("{BASE}/getBlock"),
        json!({"blockhash": common::VALID_HASH, "verbosity": 2}),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(json["hash"], common::VALID_HASH);
}

#[tokio::test]
async fn get_block_missing_hash() {
    let (app, _, _, _) = common::setup().await;

    let (status, json) =
        common::post_json(app, &format!("{BASE}/getBlock"), json!({"verbosity": 1})).await;
    assert_eq!(status, 400);
    assert!(json["error"].as_str().unwrap().contains("blockhash"));
}

// ---------------------------------------------------------------------------
// getBlockHash
// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_block_hash_success() {
    let (app, rpc, _, _) = common::setup().await;
    common::mock_rpc(&rpc, "getblockhash", json!(common::VALID_HASH)).await;

    let (status, json) = common::get(app, &format!("{BASE}/getBlockHash/700000")).await;
    assert_eq!(status, 200);
    assert_eq!(json, common::VALID_HASH);
}
