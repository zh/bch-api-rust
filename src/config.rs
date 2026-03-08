use std::env;
use std::fmt;

pub struct Config {
    // Server
    pub port: u16,
    pub api_prefix: String,
    pub network: String,

    // BCH Full Node RPC
    pub rpc_baseurl: String,
    pub rpc_username: String,
    pub rpc_password: String,
    pub rpc_timeout_ms: u64,

    // Fulcrum Indexer
    pub fulcrum_api: String,
    pub fulcrum_timeout_ms: u64,

    // SLP Token Indexer
    pub slp_indexer_api: String,
    pub slp_indexer_timeout_ms: u64,

    // x402 Payment
    pub x402_enabled: bool,
    pub server_bch_address: String,
    pub facilitator_url: String,
    pub x402_price_sat: u64,

    // Basic Auth
    pub use_basic_auth: bool,
    pub basic_auth_token: String,

    // Price
    pub coinex_api_url: String,

    // PSFFPP Proxy (JS API)
    pub psffpp_proxy_url: String,
}

/// Parse a boolean string matching JS `normalizeBoolean` behavior.
/// Accepts true/1/yes/on (case-insensitive) as true; false/0/no/off as false.
fn parse_bool(val: &str, default: bool) -> bool {
    match val.trim().to_lowercase().as_str() {
        "true" | "1" | "yes" | "on" => true,
        "false" | "0" | "no" | "off" => false,
        _ => default,
    }
}

impl Config {
    pub fn from_env() -> Self {
        Self::parse(|key| env::var(key).ok())
    }

    fn parse(get: impl Fn(&str) -> Option<String>) -> Self {
        Self {
            // Server
            port: get("PORT").and_then(|p| p.parse().ok()).unwrap_or(5943),
            api_prefix: get("API_PREFIX").unwrap_or_else(|| "/v6".into()),
            network: get("NETWORK").unwrap_or_else(|| "mainnet".into()),

            // BCH Full Node RPC
            rpc_baseurl: get("RPC_BASEURL").unwrap_or_else(|| "http://127.0.0.1:8332".into()),
            rpc_username: get("RPC_USERNAME").unwrap_or_default(),
            rpc_password: get("RPC_PASSWORD").unwrap_or_default(),
            rpc_timeout_ms: get("RPC_TIMEOUT_MS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(15000),

            // Fulcrum Indexer
            fulcrum_api: get("FULCRUM_API").unwrap_or_default(),
            fulcrum_timeout_ms: get("FULCRUM_TIMEOUT_MS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(15000),

            // SLP Token Indexer
            slp_indexer_api: get("SLP_INDEXER_API").unwrap_or_default(),
            slp_indexer_timeout_ms: get("SLP_INDEXER_TIMEOUT_MS")
                .and_then(|p| p.parse().ok())
                .unwrap_or(15000),

            // x402 Payment
            x402_enabled: get("X402_ENABLED")
                .map(|v| parse_bool(&v, true))
                .unwrap_or(true),
            server_bch_address: get("SERVER_BCH_ADDRESS")
                .unwrap_or_else(|| "bitcoincash:qqlrzp23w08434twmvr4fxw672whkjy0py26r63g3d".into()),
            facilitator_url: get("FACILITATOR_URL")
                .unwrap_or_else(|| "http://localhost:4345/facilitator".into()),
            x402_price_sat: get("X402_PRICE_SAT")
                .and_then(|p| p.parse().ok())
                .unwrap_or(200),

            // Basic Auth
            use_basic_auth: get("USE_BASIC_AUTH")
                .map(|v| parse_bool(&v, false))
                .unwrap_or(false),
            basic_auth_token: get("BASIC_AUTH_TOKEN").unwrap_or_default(),

            // Price
            coinex_api_url: get("COINEX_API_URL")
                .unwrap_or_else(|| "https://api.coinex.com/v1/market/ticker?market=bchusdt".into()),

            // PSFFPP Proxy
            psffpp_proxy_url: get("PSFFPP_PROXY_URL").unwrap_or_default(),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "port={} prefix={} network={} rpc={} fulcrum={} slp={} x402={} auth={} psffpp={}",
            self.port,
            self.api_prefix,
            self.network,
            self.rpc_baseurl,
            if self.fulcrum_api.is_empty() {
                "(none)"
            } else {
                &self.fulcrum_api
            },
            if self.slp_indexer_api.is_empty() {
                "(none)"
            } else {
                &self.slp_indexer_api
            },
            self.x402_enabled,
            self.use_basic_auth,
            if self.psffpp_proxy_url.is_empty() {
                "(none)"
            } else {
                &self.psffpp_proxy_url
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn env_from(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |key: &str| map.get(key).cloned()
    }

    #[test]
    fn defaults_when_no_env() {
        let cfg = Config::parse(|_| None);
        assert_eq!(cfg.port, 5943);
        assert_eq!(cfg.api_prefix, "/v6");
        assert_eq!(cfg.network, "mainnet");
        assert_eq!(cfg.rpc_baseurl, "http://127.0.0.1:8332");
        assert_eq!(cfg.rpc_username, "");
        assert_eq!(cfg.rpc_password, "");
        assert_eq!(cfg.rpc_timeout_ms, 15000);
        assert_eq!(cfg.fulcrum_api, "");
        assert_eq!(cfg.fulcrum_timeout_ms, 15000);
        assert_eq!(cfg.slp_indexer_api, "");
        assert_eq!(cfg.slp_indexer_timeout_ms, 15000);
        assert!(cfg.x402_enabled);
        assert_eq!(
            cfg.server_bch_address,
            "bitcoincash:qqlrzp23w08434twmvr4fxw672whkjy0py26r63g3d"
        );
        assert_eq!(cfg.facilitator_url, "http://localhost:4345/facilitator");
        assert_eq!(cfg.x402_price_sat, 200);
        assert!(!cfg.use_basic_auth);
        assert_eq!(cfg.basic_auth_token, "");
        assert_eq!(
            cfg.coinex_api_url,
            "https://api.coinex.com/v1/market/ticker?market=bchusdt"
        );
        assert_eq!(cfg.psffpp_proxy_url, "");
    }

    #[test]
    fn parses_all_vars() {
        let cfg = Config::parse(env_from(&[
            ("PORT", "8080"),
            ("API_PREFIX", "/v7"),
            ("NETWORK", "testnet3"),
            ("RPC_BASEURL", "http://10.0.0.5:8332"),
            ("RPC_USERNAME", "rpcuser"),
            ("RPC_PASSWORD", "rpcpass"),
            ("RPC_TIMEOUT_MS", "30000"),
            ("FULCRUM_API", "http://fulcrum:3000"),
            ("FULCRUM_TIMEOUT_MS", "20000"),
            ("SLP_INDEXER_API", "http://slp:5010"),
            ("SLP_INDEXER_TIMEOUT_MS", "25000"),
            ("X402_ENABLED", "true"),
            ("SERVER_BCH_ADDRESS", "bitcoincash:qz..."),
            ("FACILITATOR_URL", "http://fac:4345/facilitator"),
            ("X402_PRICE_SAT", "500"),
            ("USE_BASIC_AUTH", "true"),
            ("BASIC_AUTH_TOKEN", "mytoken123"),
            ("COINEX_API_URL", "https://custom.coinex.example/ticker"),
            ("PSFFPP_PROXY_URL", "http://localhost:5942/v6"),
        ]));
        assert_eq!(cfg.port, 8080);
        assert_eq!(cfg.api_prefix, "/v7");
        assert_eq!(cfg.network, "testnet3");
        assert_eq!(cfg.rpc_baseurl, "http://10.0.0.5:8332");
        assert_eq!(cfg.rpc_username, "rpcuser");
        assert_eq!(cfg.rpc_password, "rpcpass");
        assert_eq!(cfg.rpc_timeout_ms, 30000);
        assert_eq!(cfg.fulcrum_api, "http://fulcrum:3000");
        assert_eq!(cfg.fulcrum_timeout_ms, 20000);
        assert_eq!(cfg.slp_indexer_api, "http://slp:5010");
        assert_eq!(cfg.slp_indexer_timeout_ms, 25000);
        assert!(cfg.x402_enabled);
        assert_eq!(cfg.server_bch_address, "bitcoincash:qz...");
        assert_eq!(cfg.facilitator_url, "http://fac:4345/facilitator");
        assert_eq!(cfg.x402_price_sat, 500);
        assert!(cfg.use_basic_auth);
        assert_eq!(cfg.basic_auth_token, "mytoken123");
        assert_eq!(cfg.coinex_api_url, "https://custom.coinex.example/ticker");
        assert_eq!(cfg.psffpp_proxy_url, "http://localhost:5942/v6");
    }

    #[test]
    fn bool_flag_variations() {
        // "1" and "true"
        let cfg = Config::parse(env_from(&[("X402_ENABLED", "1"), ("USE_BASIC_AUTH", "1")]));
        assert!(cfg.x402_enabled);
        assert!(cfg.use_basic_auth);

        // "false" and "0"
        let cfg = Config::parse(env_from(&[
            ("X402_ENABLED", "false"),
            ("USE_BASIC_AUTH", "0"),
        ]));
        assert!(!cfg.x402_enabled);
        assert!(!cfg.use_basic_auth);

        // JS-compatible: "yes"/"on" (case-insensitive)
        let cfg = Config::parse(env_from(&[
            ("X402_ENABLED", "yes"),
            ("USE_BASIC_AUTH", "ON"),
        ]));
        assert!(cfg.x402_enabled);
        assert!(cfg.use_basic_auth);

        // JS-compatible: "no"/"off" (case-insensitive)
        let cfg = Config::parse(env_from(&[
            ("X402_ENABLED", "no"),
            ("USE_BASIC_AUTH", "OFF"),
        ]));
        assert!(!cfg.x402_enabled);
        assert!(!cfg.use_basic_auth);

        // Case-insensitive: "True", "FALSE"
        let cfg = Config::parse(env_from(&[
            ("X402_ENABLED", "True"),
            ("USE_BASIC_AUTH", "FALSE"),
        ]));
        assert!(cfg.x402_enabled);
        assert!(!cfg.use_basic_auth);
    }

    #[test]
    fn invalid_numbers_fall_back_to_defaults() {
        let cfg = Config::parse(env_from(&[
            ("PORT", "not_a_number"),
            ("RPC_TIMEOUT_MS", "bad"),
            ("FULCRUM_TIMEOUT_MS", "nope"),
            ("SLP_INDEXER_TIMEOUT_MS", "x"),
            ("X402_PRICE_SAT", "abc"),
        ]));
        assert_eq!(cfg.port, 5943);
        assert_eq!(cfg.rpc_timeout_ms, 15000);
        assert_eq!(cfg.fulcrum_timeout_ms, 15000);
        assert_eq!(cfg.slp_indexer_timeout_ms, 15000);
        assert_eq!(cfg.x402_price_sat, 200);
    }

    #[test]
    fn display_format() {
        let cfg = Config::parse(|_| None);
        let s = cfg.to_string();
        assert!(s.contains("port=5943"));
        assert!(s.contains("prefix=/v6"));
        assert!(s.contains("network=mainnet"));
        assert!(s.contains("rpc=http://127.0.0.1:8332"));
        assert!(s.contains("fulcrum=(none)"));
        assert!(s.contains("slp=(none)"));
        assert!(s.contains("x402=true"));
        assert!(s.contains("auth=false"));
        assert!(s.contains("psffpp=(none)"));
    }

    #[test]
    fn display_format_with_backends() {
        let cfg = Config::parse(env_from(&[
            ("FULCRUM_API", "http://fulcrum:3000"),
            ("SLP_INDEXER_API", "http://slp:5010"),
            ("X402_ENABLED", "true"),
            ("USE_BASIC_AUTH", "true"),
        ]));
        let s = cfg.to_string();
        assert!(s.contains("fulcrum=http://fulcrum:3000"));
        assert!(s.contains("slp=http://slp:5010"));
        assert!(s.contains("x402=true"));
        assert!(s.contains("auth=true"));
    }
}
