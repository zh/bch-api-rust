use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::clients::ApiError;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/getNetworkInfo", get(get_network_info))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "control" }))
}

async fn get_network_info(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let result = state.full_node.call("getnetworkinfo", json!([])).await?;
    Ok(Json(result))
}
