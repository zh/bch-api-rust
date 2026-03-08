mod common;

const BASE: &str = "/v6/price";

#[tokio::test]
async fn status_returns_price() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, BASE).await;
    assert_eq!(status, 200);
    assert_eq!(json["status"], "price");
}

#[tokio::test]
async fn get_psffpp_not_configured() {
    // With empty psffpp_proxy_url (test default), returns 501
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, &format!("{BASE}/psffpp")).await;
    assert_eq!(status, 501);
    assert!(json["error"]
        .as_str()
        .unwrap()
        .contains("PSFFPP proxy not configured"));
}

/// This test hits the real CoinEx API — run with `cargo test -- --ignored`
#[tokio::test]
#[ignore]
async fn get_bch_usd_success() {
    let (app, _, _, _) = common::setup().await;
    let (status, json) = common::get(app, &format!("{BASE}/bchusd")).await;
    assert_eq!(status, 200);
    assert!(json["usd"].as_str().is_some());
}
