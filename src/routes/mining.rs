use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::clients::ApiError;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/getMiningInfo", get(get_mining_info))
        .route("/getNetworkHashPS", get(get_network_hash_ps))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "mining" }))
}

async fn get_mining_info(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getmininginfo", json!([])).await?;
    Ok(Json(result))
}

#[derive(serde::Deserialize)]
struct HashPSQuery {
    nblocks: Option<i64>,
    height: Option<i64>,
}

async fn get_network_hash_ps(
    State(state): State<AppState>,
    Query(q): Query<HashPSQuery>,
) -> Result<Json<Value>, ApiError> {
    let nblocks = q.nblocks.unwrap_or(120);
    let height = q.height.unwrap_or(-1);
    let result = state
        .full_node
        .call("getnetworkhashps", json!([nblocks, height]))
        .await?;
    Ok(Json(result))
}
