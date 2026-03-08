use axum::{extract::State, routing::get, Json, Router};
use serde_json::{json, Value};

use crate::clients::ApiError;
use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(status))
        .route("/bchusd", get(get_bch_usd))
        .route("/psffpp", get(get_psffpp))
}

async fn status() -> Json<Value> {
    Json(json!({ "status": "price" }))
}

async fn get_bch_usd(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let resp = state
        .http_client
        .get(&state.config.coinex_api_url)
        .send()
        .await
        .map_err(|e| ApiError::BackendError {
            status: 502,
            message: format!("CoinEx API error: {e}"),
            service: "price".into(),
        })?;

    if !resp.status().is_success() {
        return Err(ApiError::BackendError {
            status: resp.status().as_u16(),
            message: "CoinEx API returned error".into(),
            service: "price".into(),
        });
    }

    let body: Value = resp.json().await.map_err(|e| ApiError::BackendError {
        status: 502,
        message: format!("Invalid JSON from CoinEx: {e}"),
        service: "price".into(),
    })?;

    let price: f64 = body
        .get("data")
        .and_then(|d| d.get("ticker"))
        .and_then(|t| t.get("last"))
        .and_then(|l| l.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    Ok(Json(json!({ "usd": price })))
}

async fn get_psffpp(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let proxy_url = &state.config.psffpp_proxy_url;
    if proxy_url.is_empty() {
        return Err(ApiError::BackendError {
            status: 501,
            message: "PSFFPP proxy not configured (set PSFFPP_PROXY_URL)".into(),
            service: "price".into(),
        });
    }

    let resp = state
        .http_client
        .get(format!("{proxy_url}/price/psffpp"))
        .send()
        .await
        .map_err(|e| ApiError::BackendError {
            status: 502,
            message: format!("PSFFPP proxy error: {e}"),
            service: "price".into(),
        })?;

    let body: Value = resp.json().await.map_err(|e| ApiError::BackendError {
        status: 502,
        message: format!("Invalid JSON from PSFFPP proxy: {e}"),
        service: "price".into(),
    })?;

    Ok(Json(body))
}
