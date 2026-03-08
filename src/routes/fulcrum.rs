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
        .route("/balance/{address}", get(get_balance))
        .route("/balance", post(get_balance_bulk))
        .route("/utxos/{address}", get(get_utxos))
        .route("/utxos", post(get_utxos_bulk))
        .route("/tx/data/{txid}", get(get_tx_data))
        .route("/tx/data", post(get_tx_data_bulk))
        .route("/tx/broadcast", post(tx_broadcast))
        .route("/block/headers/{height}", get(get_block_headers))
        .route("/block/headers", post(get_block_headers_bulk))
        .route("/transactions/{address}", get(get_transactions))
        .route(
            "/transactions/{address}/{all_txs}",
            get(get_transactions_all),
        )
        .route("/transactions", post(get_transactions_bulk))
        .route("/unconfirmed/{address}", get(get_unconfirmed))
        .route("/unconfirmed", post(get_unconfirmed_bulk))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "fulcrum" }))
}

// --- Balance ---

async fn get_balance(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let addr = validate_address(&address)?;
    let result = state
        .fulcrum
        .get(&format!("/electrumx/balance/{addr}"))
        .await?;
    Ok(Json(result))
}

async fn get_balance_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let addresses = extract_address_array(&body, "addresses")?;
    let result = state
        .fulcrum
        .post("/electrumx/balance/", json!({ "addresses": addresses }))
        .await?;
    Ok(Json(result))
}

// --- UTXOs ---

async fn get_utxos(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let addr = validate_address(&address)?;
    let result = state
        .fulcrum
        .get(&format!("/electrumx/utxos/{addr}"))
        .await?;
    Ok(Json(result))
}

async fn get_utxos_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let addresses = extract_address_array(&body, "addresses")?;
    let result = state
        .fulcrum
        .post("/electrumx/utxos/", json!({ "addresses": addresses }))
        .await?;
    Ok(Json(result))
}

// --- TX Data ---

async fn get_tx_data(
    State(state): State<AppState>,
    Path(txid): Path<String>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let result = state
        .fulcrum
        .get(&format!("/electrumx/tx/data/{txid}"))
        .await?;
    Ok(Json(result))
}

async fn get_tx_data_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    // Pass through txids and verbose to fulcrum
    let result = state.fulcrum.post("/electrumx/tx/data", body).await?;
    Ok(Json(result))
}

// --- TX Broadcast ---

async fn tx_broadcast(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let result = state.fulcrum.post("/electrumx/tx/broadcast", body).await?;
    Ok(Json(result))
}

// --- Block Headers ---

#[derive(serde::Deserialize)]
struct BlockHeaderQuery {
    count: Option<u32>,
}

async fn get_block_headers(
    State(state): State<AppState>,
    Path(height): Path<String>,
    Query(q): Query<BlockHeaderQuery>,
) -> Result<Json<Value>, ApiError> {
    let count = q.count.unwrap_or(1);
    let result = state
        .fulcrum
        .get(&format!("/electrumx/block/headers/{height}?count={count}"))
        .await?;
    Ok(Json(result))
}

async fn get_block_headers_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let result = state.fulcrum.post("/electrumx/block/headers", body).await?;
    Ok(Json(result))
}

// --- Transactions ---

async fn get_transactions(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let addr = validate_address(&address)?;
    let mut result = state
        .fulcrum
        .get(&format!("/electrumx/transactions/{addr}"))
        .await?;

    // Sort descending by height and truncate to 100 (default allTxs=false behavior)
    if let Some(txs) = result
        .get_mut("transactions")
        .and_then(|t| t.as_array_mut())
    {
        txs.sort_by(|a, b| {
            let ha = a.get("height").and_then(|h| h.as_i64()).unwrap_or(0);
            let hb = b.get("height").and_then(|h| h.as_i64()).unwrap_or(0);
            hb.cmp(&ha)
        });
        txs.truncate(100);
    }

    Ok(Json(result))
}

async fn get_transactions_all(
    State(state): State<AppState>,
    Path((address, all_txs)): Path<(String, String)>,
) -> Result<Json<Value>, ApiError> {
    let addr = validate_address(&address)?;
    let mut result = state
        .fulcrum
        .get(&format!("/electrumx/transactions/{addr}"))
        .await?;

    // Sort transactions descending by height (newest first), matching JS API behavior
    if let Some(txs) = result
        .get_mut("transactions")
        .and_then(|t| t.as_array_mut())
    {
        txs.sort_by(|a, b| {
            let ha = a.get("height").and_then(|h| h.as_i64()).unwrap_or(0);
            let hb = b.get("height").and_then(|h| h.as_i64()).unwrap_or(0);
            hb.cmp(&ha)
        });

        if all_txs != "true" {
            txs.truncate(100);
        }
    }

    Ok(Json(result))
}

async fn get_transactions_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let addresses = extract_address_array(&body, "addresses")?;
    let result = state
        .fulcrum
        .post(
            "/electrumx/transactions/",
            json!({ "addresses": addresses }),
        )
        .await?;
    Ok(Json(result))
}

// --- Unconfirmed ---

async fn get_unconfirmed(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let addr = validate_address(&address)?;
    let result = state
        .fulcrum
        .get(&format!("/electrumx/unconfirmed/{addr}"))
        .await?;
    Ok(Json(result))
}

async fn get_unconfirmed_bulk(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let addresses = extract_address_array(&body, "addresses")?;
    let result = state
        .fulcrum
        .post("/electrumx/unconfirmed/", json!({ "addresses": addresses }))
        .await?;
    Ok(Json(result))
}
