pub mod blockchain;
pub mod control;
pub mod dsproof;
pub mod encryption;
pub mod fulcrum;
pub mod helpers;
pub mod mining;
pub mod price;
pub mod rawtransactions;
pub mod slp;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};

use crate::AppState;

/// Build the full API route tree under the configured prefix (e.g. /v6).
pub fn api_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(api_status))
        .nest("/full-node/blockchain", blockchain::routes())
        .nest("/full-node/rawtransactions", rawtransactions::routes())
        .nest("/full-node/mining", mining::routes())
        .nest("/full-node/control", control::routes())
        .nest("/full-node/dsproof", dsproof::routes())
        .nest("/fulcrum", fulcrum::routes())
        .nest("/slp", slp::routes())
        .nest("/encryption", encryption::routes())
        .nest("/price", price::routes())
}

async fn api_status() -> Json<Value> {
    Json(json!({ "status": "bch-api-rust" }))
}
