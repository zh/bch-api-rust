use super::HttpProxyClient;

pub type SlpClient = HttpProxyClient;

pub fn new(base_url: &str, timeout_ms: u64) -> SlpClient {
    HttpProxyClient::new(base_url, timeout_ms, "slp_indexer")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_name_is_slp_indexer() {
        let client = new("http://127.0.0.1:0", 5000);
        assert_eq!(client.service, "slp_indexer");
    }
}
