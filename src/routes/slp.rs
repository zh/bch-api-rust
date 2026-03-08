use axum::{
    extract::State,
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
        .route("/status", get(slp_status))
        .route("/address", post(slp_address))
        .route("/txid", post(slp_txid))
        .route("/token", post(slp_token))
        .route("/token/data", post(slp_token_data))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "slp" }))
}

async fn slp_status(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.slp.get("/slp/status/").await?;
    Ok(Json(result))
}

async fn slp_address(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let address = body
        .get("address")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::InvalidInput("address is required".into()))?;
    validate_address(address)?;

    let result = state.slp.post("/slp/address/", body).await?;
    Ok(Json(result))
}

async fn slp_txid(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let txid = body
        .get("txid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ApiError::InvalidInput("txid is required".into()))?;
    validate_hash(txid, "txid")?;

    let result = state.slp.post("/slp/tx/", body).await?;
    Ok(Json(result))
}

async fn slp_token(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let result = state.slp.post("/slp/token/", body).await?;
    Ok(Json(result))
}

async fn slp_token_data(
    State(state): State<AppState>,
    Json(body): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    let result = state.slp.post("/slp/token/", body).await?;

    // Transform response: { tokenData: {...} } → { genesisData: {...}, immutableData, mutableData }
    let genesis_data = result.get("tokenData").cloned().unwrap_or(json!(null));

    let immutable_data = genesis_data
        .get("documentUri")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok(Json(json!({
        "genesisData": genesis_data,
        "immutableData": immutable_data,
        "mutableData": ""
    })))
}
