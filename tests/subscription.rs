use std::fs;

use boosty_api::{api_client::ApiClient, error::ApiError};
use reqwest::{Client, header::CONTENT_TYPE};

use crate::helpers::{api_path, setup};

mod helpers;

#[tokio::test]
async fn test_headers_as_map_after_set_bearer_token() {
    let (mut _server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);
    client.set_bearer_token("tok123").await.unwrap();

    let map = client.headers_as_map();
    assert_eq!(
        map.get("accept").map(|s| s.as_str()),
        Some("application/json")
    );
}

#[tokio::test]
async fn test_get_subscription_levels_default() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";
    let api_path = api_path(&format!("blog/{blog}/subscription_level/"));

    let raw = fs::read_to_string("tests/fixtures/api_response_subscription_levels.json").unwrap();

    server
        .mock("GET", api_path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw.clone())
        .create_async()
        .await;

    let levels = client
        .get_blog_subscription_levels(blog, None)
        .await
        .unwrap();
    assert_eq!(levels.data.len(), 2);
    assert_eq!(levels.data[0].id, 1);
    assert_eq!(levels.data[0].name, "Basic");
}

#[tokio::test]
async fn test_get_subscription_levels_show_free() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";
    let api_path = api_path(&format!(
        "blog/{blog}/subscription_level/?show_free_level=true"
    ));

    let raw = fs::read_to_string("tests/fixtures/api_response_subscription_levels.json").unwrap();

    server
        .mock("GET", api_path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let levels = client
        .get_blog_subscription_levels(blog, Some(true))
        .await
        .unwrap();
    assert_eq!(levels.data.len(), 2);
    assert_eq!(levels.data[1].id, 2);
    assert!(levels.data[1].is_limited);
}

#[tokio::test]
async fn test_get_subscriptions_unauthorized() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let api_path = api_path("user/subscriptions?limit=30&with_follow=true");

    server
        .mock("GET", api_path.as_str())
        .with_status(401)
        .create_async()
        .await;

    let res = client.get_user_subscriptions(Some(30), Some(true)).await;
    assert!(matches!(res, Err(ApiError::Unauthorized)));
}

#[tokio::test]
async fn test_get_subscriptions_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let api_path = api_path("user/subscriptions?limit=30&with_follow=true");
    let raw = fs::read_to_string("tests/fixtures/api_response_subscriptions.json").unwrap();

    server
        .mock("GET", api_path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let resp = client
        .get_user_subscriptions(Some(30), Some(true))
        .await
        .unwrap();
    assert_eq!(resp.data.len(), 1);
    let sub = &resp.data[0];
    assert_eq!(sub.id, 39989023);
    assert_eq!(resp.limit, 30);
}
