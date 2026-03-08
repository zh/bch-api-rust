pub mod clients;
pub mod config;
pub mod middleware;
pub mod routes;

pub use config::Config;

use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub http_client: reqwest::Client,
    pub full_node: clients::full_node::FullNodeClient,
    pub fulcrum: clients::fulcrum::FulcrumClient,
    pub slp: clients::slp::SlpClient,
}
