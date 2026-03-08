use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::AppState;

/// Auth middleware: checks Bearer token and/or x402 payment header.
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: Request<Body>,
    next: Next,
) -> Response {
    let cfg = &state.config;

    // Try Bearer token first
    if cfg.use_basic_auth {
        if let Some(auth_header) = req.headers().get("authorization") {
            if let Ok(val) = auth_header.to_str() {
                if let Some(token) = val.strip_prefix("Bearer ") {
                    if token == cfg.basic_auth_token {
                        return next.run(req).await;
                    }
                }
            }
        }

        // If only basic auth is enabled (no x402), reject
        if !cfg.x402_enabled {
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "error": "Unauthorized",
                    "message": "Valid Bearer token required in Authorization header"
                })),
            )
                .into_response();
        }
    }

    // Try x402 payment header
    if cfg.x402_enabled {
        if let Some(payment_header) = req.headers().get("x-402-payment") {
            if let Ok(payment_val) = payment_header.to_str() {
                // Verify payment with facilitator
                match verify_x402_payment(&state, payment_val, req.uri().path()).await {
                    Ok(true) => return next.run(req).await,
                    Ok(false) => {
                        return (
                            StatusCode::PAYMENT_REQUIRED,
                            Json(json!({ "error": "Payment verification failed." })),
                        )
                            .into_response();
                    }
                    Err(e) => {
                        tracing::error!("x402 verification error: {e}");
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": "Payment verification service error." })),
                        )
                            .into_response();
                    }
                }
            }
        }

        // No payment header present
        return (
            StatusCode::PAYMENT_REQUIRED,
            Json(json!({
                "error": "Payment required. Include x-402-payment header.",
                "bchAddress": cfg.server_bch_address,
                "priceSat": cfg.x402_price_sat,
            })),
        )
            .into_response();
    }

    // Neither auth method enabled — should not reach here if middleware is conditionally applied
    next.run(req).await
}

async fn verify_x402_payment(
    state: &AppState,
    payment_token: &str,
    endpoint: &str,
) -> Result<bool, String> {
    let cfg = &state.config;

    let body = json!({
        "payment": payment_token,
        "endpoint": endpoint,
        "bchAddress": cfg.server_bch_address,
        "priceSat": cfg.x402_price_sat,
    });

    let resp = state
        .http_client
        .post(&cfg.facilitator_url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("facilitator request failed: {e}"))?;

    if !resp.status().is_success() {
        return Ok(false);
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("invalid facilitator response: {e}"))?;

    Ok(json.get("valid").and_then(|v| v.as_bool()).unwrap_or(false))
}
