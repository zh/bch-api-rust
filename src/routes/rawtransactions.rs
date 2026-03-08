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
        .route("/decodeRawTransaction/{hex}", get(decode_raw_transaction))
        .route("/decodeRawTransaction", post(decode_raw_transaction_bulk))
        .route("/decodeScript/{hex}", get(decode_script))
        .route("/decodeScript", post(decode_script_bulk))
        .route("/getRawTransaction/{txid}", get(get_raw_transaction))
        .route("/getRawTransaction", post(get_raw_transaction_bulk))
        .route("/sendRawTransaction/{hex}", get(send_raw_transaction))
        .route("/sendRawTransaction", post(send_raw_transaction_bulk))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "rawtransactions" }))
}

async fn decode_raw_transaction(
    State(state): State<AppState>,
    Path(hex): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if hex.is_empty() {
        return Err(ApiError::InvalidInput("hex is required".into()));
    }
    let result = state
        .full_node
        .call("decoderawtransaction", json!([hex]))
        .await?;
    Ok(Json(result))
}

async fn decode_raw_transaction_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let hexes = extract_string_array(&body, "hexes")?;

    let mut results = Vec::with_capacity(hexes.len());
    for hex in &hexes {
        let result = state
            .full_node
            .call("decoderawtransaction", json!([hex]))
            .await?;
        results.push(result);
    }
    Ok(Json(json!(results)))
}

async fn decode_script(
    State(state): State<AppState>,
    Path(hex): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if hex.is_empty() {
        return Err(ApiError::InvalidInput("hex is required".into()));
    }
    let result = state.full_node.call("decodescript", json!([hex])).await?;
    Ok(Json(result))
}

async fn decode_script_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let hexes = extract_string_array(&body, "hexes")?;

    let mut results = Vec::with_capacity(hexes.len());
    for hex in &hexes {
        let result = state.full_node.call("decodescript", json!([hex])).await?;
        results.push(result);
    }
    Ok(Json(json!(results)))
}

/// If the transaction has a `blockhash`, fetch the block header and insert `height`.
async fn enrich_with_block_height(state: &AppState, result: &mut Value) {
    if let Some(blockhash) = result.get("blockhash").and_then(|b| b.as_str()) {
        let blockhash = blockhash.to_string();
        if let Ok(header) = state
            .full_node
            .call("getblockheader", json!([blockhash, true]))
            .await
        {
            if let Some(height) = header.get("height") {
                if let Some(obj) = result.as_object_mut() {
                    obj.insert("height".to_string(), height.clone());
                }
            }
        }
    }
}

async fn get_raw_transaction(
    State(state): State<AppState>,
    Path(txid): Path<String>,
    Query(q): Query<VerboseQuery>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let verbose = q.verbose.unwrap_or(false);
    let verbose_int = if verbose { 1 } else { 0 };

    let mut result = state
        .full_node
        .call("getrawtransaction", json!([txid, verbose_int]))
        .await?;

    if verbose {
        enrich_with_block_height(&state, &mut result).await;
    }

    Ok(Json(result))
}

async fn get_raw_transaction_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let txids = extract_hash_array(&body, "txids")?;
    let verbose = body
        .get("verbose")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let verbose_int = if verbose { 1 } else { 0 };

    let mut results = Vec::with_capacity(txids.len());
    for txid in &txids {
        let mut result = state
            .full_node
            .call("getrawtransaction", json!([txid, verbose_int]))
            .await?;

        if verbose {
            enrich_with_block_height(&state, &mut result).await;
        }

        results.push(result);
    }
    Ok(Json(json!(results)))
}

async fn send_raw_transaction(
    State(state): State<AppState>,
    Path(hex): Path<String>,
) -> Result<Json<Value>, ApiError> {
    if hex.is_empty() {
        return Err(ApiError::InvalidInput("hex is required".into()));
    }
    let result = state
        .full_node
        .call("sendrawtransaction", json!([hex]))
        .await?;
    Ok(Json(result))
}

/// Bulk send is SERIAL to prevent double-spend race conditions.
async fn send_raw_transaction_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let hexes = extract_string_array(&body, "hexes")?;

    let mut results = Vec::with_capacity(hexes.len());
    for hex in &hexes {
        let result = state
            .full_node
            .call("sendrawtransaction", json!([hex]))
            .await?;
        results.push(result);
    }
    Ok(Json(json!(results)))
}
