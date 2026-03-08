use axum::{
    http::{header, StatusCode},
    middleware as axum_mw,
    routing::get,
    Json, Router, ServiceExt,
};
use bch_api_rust::{clients, middleware, routes, AppState, Config};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower::Layer;
use tower_http::cors::CorsLayer;
use tower_http::normalize_path::NormalizePathLayer;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

#[tokio::main]
async fn main() {
    let _ = dotenvy::dotenv(); // silently ignore if .env missing

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bch_api_rust=info,tower_http=debug".into()),
        )
        .init();

    let cfg = Config::from_env();
    info!("config: {cfg}");

    let port = cfg.port;
    let api_prefix = cfg.api_prefix.clone();

    let full_node = clients::full_node::FullNodeClient::new(
        &cfg.rpc_baseurl,
        &cfg.rpc_username,
        &cfg.rpc_password,
        cfg.rpc_timeout_ms,
    );
    let fulcrum = clients::fulcrum::new(&cfg.fulcrum_api, cfg.fulcrum_timeout_ms);
    let slp = clients::slp::new(&cfg.slp_indexer_api, cfg.slp_indexer_timeout_ms);

    let state = AppState {
        config: Arc::new(cfg),
        http_client: reqwest::Client::new(),
        full_node,
        fulcrum,
        slp,
    };

    // CORS: allow all origins (match Express cors() defaults)
    let cors = CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods(tower_http::cors::Any)
        .allow_headers(vec![
            header::CONTENT_TYPE,
            header::ACCEPT,
            header::AUTHORIZATION,
        ]);

    // API routes under the configured prefix (default: /v6)
    let mut api_router = routes::api_routes();

    // Conditionally apply auth middleware
    if state.config.use_basic_auth || state.config.x402_enabled {
        api_router = api_router.layer(axum_mw::from_fn_with_state(
            state.clone(),
            middleware::auth_middleware,
        ));
    }

    let router = Router::new()
        .route("/health", get(health_check))
        .nest(&api_prefix, api_router)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(300),
        ))
        .with_state(state);

    // Strip trailing slashes before route matching
    let app = NormalizePathLayer::trim_trailing_slash().layer(router);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("bch-api-rust listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    axum::serve(
        listener,
        ServiceExt::<axum::http::Request<axum::body::Body>>::into_make_service(app),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}

async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "bch-api-rust",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => info!("received Ctrl+C, shutting down"),
        _ = terminate => info!("received SIGTERM, shutting down"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app() -> axum::Router {
        let dummy = "http://127.0.0.1:0";
        let state = AppState {
            config: Arc::new(Config::from_env()),
            http_client: reqwest::Client::new(),
            full_node: clients::full_node::FullNodeClient::new(dummy, "", "", 1000),
            fulcrum: clients::fulcrum::new(dummy, 1000),
            slp: clients::slp::new(dummy, 1000),
        };
        axum::Router::new()
            .route("/health", get(health_check))
            .nest("/v6", routes::api_routes())
            .with_state(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = test_app();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "bch-api-rust");
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_root_status() {
        let app = test_app();

        let response = app
            .oneshot(Request::builder().uri("/v6").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), 200);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "bch-api-rust");
    }
}
