use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde_json::{json, Value};

use crate::clients::ApiError;
use crate::routes::helpers::*;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/getBestBlockHash", get(get_best_block_hash))
        .route("/getBlockchainInfo", get(get_blockchain_info))
        .route("/getBlockCount", get(get_block_count))
        .route("/getBlockHeader/{hash}", get(get_block_header))
        .route("/getBlockHeader", post(get_block_header_bulk))
        .route("/getChainTips", get(get_chain_tips))
        .route("/getDifficulty", get(get_difficulty))
        .route("/getMempoolEntry/{txid}", get(get_mempool_entry))
        .route("/getMempoolEntry", post(get_mempool_entry_bulk))
        .route("/getMempoolAncestors/{txid}", get(get_mempool_ancestors))
        .route("/getMempoolInfo", get(get_mempool_info))
        .route("/getRawMempool", get(get_raw_mempool))
        .route("/getTxOut/{txid}/{n}", get(get_tx_out))
        .route("/getTxOut", post(get_tx_out_post))
        .route("/getTxOutProof/{txid}", get(get_tx_out_proof))
        .route("/getTxOutProof", post(get_tx_out_proof_bulk))
        .route("/verifyTxOutProof/{proof}", get(verify_tx_out_proof))
        .route("/verifyTxOutProof", post(verify_tx_out_proof_bulk))
        .route("/getBlock", post(get_block))
        .route("/getblock", post(get_block))
        .route("/getBlockHash/{height}", get(get_block_hash))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "blockchain" }))
}

async fn get_best_block_hash(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getbestblockhash", json!([])).await?;
    Ok(Json(result))
}

async fn get_blockchain_info(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getblockchaininfo", json!([])).await?;
    Ok(Json(result))
}

async fn get_block_count(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getblockcount", json!([])).await?;
    Ok(Json(result))
}

async fn get_block_header(
    State(state): State<AppState>,
    Path(hash): Path<String>,
    Query(q): Query<VerboseQuery>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&hash, "hash")?;
    let verbose = q.verbose.unwrap_or(true);
    let result = state
        .full_node
        .call("getblockheader", json!([hash, verbose]))
        .await?;
    Ok(Json(result))
}

async fn get_block_header_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let hashes = extract_hash_array(&body, "hashes")?;
    let verbose = body
        .get("verbose")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let mut results = Vec::with_capacity(hashes.len());
    for hash in &hashes {
        let result = state
            .full_node
            .call("getblockheader", json!([hash, verbose]))
            .await?;
        results.push(result);
    }
    Ok(Json(json!(results)))
}

async fn get_chain_tips(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getchaintips", json!([])).await?;
    Ok(Json(result))
}

async fn get_difficulty(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getdifficulty", json!([])).await?;
    Ok(Json(result))
}

async fn get_mempool_entry(
    State(state): State<AppState>,
    Path(txid): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let result = state
        .full_node
        .call("getmempoolentry", json!([txid]))
        .await?;
    Ok(Json(result))
}

async fn get_mempool_entry_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let txids = extract_hash_array(&body, "txids")?;

    let mut results = Vec::with_capacity(txids.len());
    for txid in &txids {
        let result = state
            .full_node
            .call("getmempoolentry", json!([txid]))
            .await?;
        results.push(result);
    }
    Ok(Json(json!(results)))
}

async fn get_mempool_ancestors(
    State(state): State<AppState>,
    Path(txid): Path<String>,
    Query(q): Query<VerboseQuery>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let verbose = q.verbose.unwrap_or(false);
    let result = state
        .full_node
        .call("getmempoolancestors", json!([txid, verbose]))
        .await?;
    Ok(Json(result))
}

async fn get_mempool_info(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getmempoolinfo", json!([])).await?;
    Ok(Json(result))
}

async fn get_raw_mempool(
    State(state): State<AppState>,
    Query(q): Query<VerboseQuery>,
) -> Result<Json<Value>, ApiError> {
    let verbose = q.verbose.unwrap_or(false);
    let result = state
        .full_node
        .call("getrawmempool", json!([verbose]))
        .await?;
    Ok(Json(result))
}

#[derive(serde::Deserialize)]
struct IncludeMempoolQuery {
    #[serde(rename = "includeMempool")]
    include_mempool: Option<bool>,
}

async fn get_tx_out(
    State(state): State<AppState>,
    Path((txid, n)): Path<(String, u32)>,
    Query(q): Query<IncludeMempoolQuery>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let include_mempool = q.include_mempool.unwrap_or(true);
    let result = state
        .full_node
        .call("gettxout", json!([txid, n, include_mempool]))
        .await?;
    Ok(Json(result))
}

async fn get_tx_out_post(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let txid = body
        .get("txid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::InvalidInput("txid is required".into()))?;
    validate_hash(txid, "txid")?;

    let vout = body.get("vout").and_then(|v| v.as_u64()).unwrap_or(0);

    let mempool = body
        .get("mempool")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let result = state
        .full_node
        .call("gettxout", json!([txid, vout, mempool]))
        .await?;
    Ok(Json(result))
}

async fn get_tx_out_proof(
    State(state): State<AppState>,
    Path(txid): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let result = state
        .full_node
        .call("gettxoutproof", json!([[txid]]))
        .await?;
    Ok(Json(result))
}

async fn get_tx_out_proof_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let txids = extract_hash_array(&body, "txids")?;

    let mut results = Vec::with_capacity(txids.len());
    for txid in &txids {
        let result = state
            .full_node
            .call("gettxoutproof", json!([[txid]]))
            .await?;
        results.push(result);
    }
    Ok(Json(json!(results)))
}

async fn verify_tx_out_proof(
    State(state): State<AppState>,
    Path(proof): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if proof.is_empty() {
        return Err(ApiError::InvalidInput("proof is required".into()));
    }
    let result = state
        .full_node
        .call("verifytxoutproof", json!([proof]))
        .await?;
    Ok(Json(result))
}

async fn verify_tx_out_proof_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let proofs = extract_string_array(&body, "proofs")?;

    let mut results = Vec::new();
    for proof in &proofs {
        let result = state
            .full_node
            .call("verifytxoutproof", json!([proof]))
            .await?;
        // verifytxoutproof returns an array of txids; flatten into results
        if let Some(arr) = result.as_array() {
            results.extend(arr.iter().cloned());
        } else {
            results.push(result);
        }
    }
    Ok(Json(json!(results)))
}

async fn get_block(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let blockhash = body
        .get("blockhash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::InvalidInput("blockhash is required".into()))?;
    validate_hash(blockhash, "blockhash")?;

    let verbosity = body.get("verbosity").and_then(|v| v.as_i64()).unwrap_or(1);

    let result = state
        .full_node
        .call("getblock", json!([blockhash, verbosity]))
        .await?;
    Ok(Json(result))
}

async fn get_block_hash(
    State(state): State<AppState>,
    Path(height): Path<i64>,
) -> Result<Json<Value>, ApiError> {
    let result = state
        .full_node
        .call("getblockhash", json!([height]))
        .await?;
    Ok(Json(result))
}
