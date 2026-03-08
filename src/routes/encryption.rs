use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde_json::{json, Value};

use crate::clients::ApiError;
use crate::routes::helpers::validate_address;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/publickey/{address}", get(get_public_key))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "encryption" }))
}

/// Get public key for an address by scanning transaction history.
/// 1. Validate address
/// 2. Get transaction history from fulcrum
/// 3. For each txid, get verbose raw transaction from full node
/// 4. Scan vin[].scriptSig.asm for 66-char (compressed) or 130-char (uncompressed) pubkey
async fn get_public_key(
    State(state): State<AppState>,
    Path(address): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let addr = validate_address(&address)?;

    // Get transaction history from fulcrum
    let tx_data = state
        .fulcrum
        .get(&format!("/electrumx/transactions/{addr}"))
        .await?;

    let transactions = tx_data
        .get("transactions")
        .and_then(|t| t.as_array())
        .cloned()
        .unwrap_or_default();

    if transactions.is_empty() {
        return Ok(Json(json!({
            "success": true,
            "publicKey": "not found"
        })));
    }

    // Scan transactions for public key
    for tx in &transactions {
        let txid = match tx.get("tx_hash").and_then(|h| h.as_str()) {
            Some(id) => id,
            None => continue,
        };

        // Get verbose raw transaction
        let raw_tx = match state
            .full_node
            .call("getrawtransaction", json!([txid, 1]))
            .await
        {
            Ok(tx) => tx,
            Err(_) => continue,
        };

        // Scan vin for public key in scriptSig
        if let Some(vins) = raw_tx.get("vin").and_then(|v| v.as_array()) {
            for vin in vins {
                if let Some(asm) = vin
                    .get("scriptSig")
                    .and_then(|s| s.get("asm"))
                    .and_then(|a| a.as_str())
                {
                    // Split asm into parts, look for pubkey-sized hex strings
                    for part in asm.split_whitespace() {
                        let len = part.len();
                        // 66 chars = compressed pubkey, 130 chars = uncompressed pubkey
                        if (len == 66 || len == 130) && part.chars().all(|c| c.is_ascii_hexdigit())
                        {
                            return Ok(Json(json!({
                                "success": true,
                                "publicKey": part
                            })));
                        }
                    }
                }
            }
        }
    }

    Ok(Json(json!({
        "success": true,
        "publicKey": "not found"
    })))
}
