use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::clients::ApiError;
use crate::routes::helpers::{validate_hash, VerboseQuery};
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/getDSProof/{txid}", get(get_ds_proof))
        .route("/getdsproof/{txid}", get(get_ds_proof))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "dsproof" }))
}

async fn get_ds_proof(
    State(state): State<AppState>,
    Path(txid): Path<String>,
    Query(q): Query<VerboseQuery>,
) -> Result<Json<Value>, ApiError> {
    validate_hash(&txid, "txid")?;
    let verbosity = if q.verbose.unwrap_or(false) { 3 } else { 2 };
    let result = state
        .full_node
        .call("getdsproof", json!([txid, verbosity]))
        .await?;
    Ok(Json(result))
}
