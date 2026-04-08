mod helpers;

use std::fs;

use boosty_api::{api_client::ApiClient, error::ApiError, model::BundleQuery};
use mockito::Matcher;
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::json;

use crate::helpers::{api_path, setup};

#[tokio::test]
async fn test_get_bundles_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "testblog";
    let path = api_path(&format!("blog/{blog}/bundle/"));
    let raw = fs::read_to_string("tests/fixtures/api_response_bundles.json").unwrap();

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let result = client.get_bundles(blog).await.unwrap();
    assert_eq!(result.data.bundles.len(), 2);
    assert_eq!(result.data.bundles[0].id, "bundle-001");
    assert_eq!(result.data.bundles[0].title, "Test Bundle One");
    assert_eq!(result.data.bundles[0].price, 500);
    assert!(result.data.bundles[0].has_access);
    assert!(!result.data.bundles[0].hidden);
    assert_eq!(result.data.bundles[0].published_posts_counter, 10);
    assert_eq!(result.data.bundles[0].accessible_posts_counter, 8);
    assert_eq!(result.data.bundles[1].id, "bundle-002");
    assert!(result.data.bundles[1].hidden);
    assert!(!result.data.bundles[1].has_access);
}

#[tokio::test]
async fn test_get_bundles_unauthorized() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";
    let path = api_path(&format!("blog/{blog}/bundle/"));

    server
        .mock("GET", path.as_str())
        .with_status(401)
        .create_async()
        .await;

    let res = client.get_bundles(blog).await;
    assert!(matches!(res, Err(ApiError::Unauthorized)));
}

#[tokio::test]
async fn test_get_bundles_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let path = api_path(&format!("blog/{blog}/bundle/"));

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("not valid json {{{")
        .create_async()
        .await;

    let res = client.get_bundles(blog).await;
    assert!(matches!(res, Err(ApiError::JsonParseDetailed { .. })));
}

#[tokio::test]
async fn test_get_bundles_http_error() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let path = api_path(&format!("blog/{blog}/bundle/"));

    server
        .mock("GET", path.as_str())
        .with_status(500)
        .create_async()
        .await;

    let res = client.get_bundles(blog).await;
    assert!(matches!(res, Err(ApiError::HttpStatus { .. })));
}

#[tokio::test]
async fn test_get_bundles_empty_list() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "emptyblog";
    let path = api_path(&format!("blog/{blog}/bundle/"));
    let empty_response = json!({
        "data": {
            "bundles": []
        }
    })
    .to_string();

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(empty_response)
        .create_async()
        .await;

    let result = client.get_bundles(blog).await.unwrap();
    assert!(result.data.bundles.is_empty());
}

#[tokio::test]
async fn test_get_bundle_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "testblog";
    let bundle_id = "bundle-001";
    let query = BundleQuery::default();

    let raw = fs::read_to_string("tests/fixtures/api_response_bundle_items.json").unwrap();

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let result = client.get_bundle(blog, bundle_id, &query).await.unwrap();
    assert_eq!(result.data.bundle_items.len(), 2);
    assert_eq!(result.data.bundle_items[0].post.id, "post-bundle-001");
    assert_eq!(result.data.bundle_items[0].post.title, "Bundle Post One");
    assert_eq!(result.data.bundle_items[0].position, 1);
    assert_eq!(result.data.bundle_items[0].bundle_id, "bundle-001");
    assert!(!result.data.bundle_items[0].is_draft);
    assert!(result.extra.is_last);
    assert_eq!(result.extra.offset, 0);
}

#[tokio::test]
async fn test_get_bundle_unauthorized() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let bundle_id = "bundle-xyz";
    let query = BundleQuery::default();

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .with_status(401)
        .create_async()
        .await;

    let res = client.get_bundle(blog, bundle_id, &query).await;
    assert!(matches!(res, Err(ApiError::Unauthorized)));
}

#[tokio::test]
async fn test_get_bundle_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let bundle_id = "bundle-xyz";
    let query = BundleQuery::default();

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("{invalid json")
        .create_async()
        .await;

    let res = client.get_bundle(blog, bundle_id, &query).await;
    assert!(matches!(res, Err(ApiError::JsonParseDetailed { .. })));
}

#[tokio::test]
async fn test_get_bundle_http_error() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let bundle_id = "bundle-xyz";
    let query = BundleQuery::default();

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .with_status(500)
        .create_async()
        .await;

    let res = client.get_bundle(blog, bundle_id, &query).await;
    assert!(matches!(res, Err(ApiError::HttpStatus { .. })));
}

#[tokio::test]
async fn test_get_bundle_with_custom_query() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "testblog";
    let bundle_id = "bundle-001";
    let query = BundleQuery {
        full_data: Some(false),
        limit: Some(5),
        for_owner: Some(false),
        comments_limit: Some(1),
        reply_limit: Some(1),
    };

    let raw = fs::read_to_string("tests/fixtures/api_response_bundle_items.json").unwrap();

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let result = client.get_bundle(blog, bundle_id, &query).await.unwrap();
    assert!(!result.data.bundle_items.is_empty());
}

#[tokio::test]
async fn test_get_bundle_items_empty() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "emptyblog";
    let bundle_id = "bundle-empty";
    let query = BundleQuery::default();

    let empty_response = json!({
        "data": {
            "bundleItems": []
        },
        "extra": {
            "isLast": true,
            "offset": 0
        }
    })
    .to_string();

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(empty_response)
        .create_async()
        .await;

    let result = client.get_bundle(blog, bundle_id, &query).await.unwrap();
    assert!(result.data.bundle_items.is_empty());
    assert!(result.extra.is_last);
}

#[tokio::test]
async fn test_set_bearer_token_in_get_bundles() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    client.set_bearer_token("bundleToken123").await.unwrap();

    let blog = "testblog";

    let raw = fs::read_to_string("tests/fixtures/api_response_bundles.json").unwrap();

    server
        .mock("GET", &*api_path(&format!("blog/{blog}/bundle/")))
        .match_header("authorization", "Bearer bundleToken123")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let result = client.get_bundles(blog).await.unwrap();
    assert_eq!(result.data.bundles.len(), 2);
}

#[tokio::test]
async fn test_get_bundle_with_refresh_token_flow() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    client
        .set_refresh_token_and_device_id("refresh_bundle", "device_bundle_123")
        .await
        .unwrap();

    let blog = "testblog";
    let bundle_id = "bundle-001";
    let query = BundleQuery::default();

    let raw = fs::read_to_string("tests/fixtures/api_response_bundle_items.json").unwrap();

    let oauth_resp = json!({
        "access_token": "bundle_access_token",
        "refresh_token": "bundle_refresh_token",
        "expires_in": 3600
    })
    .to_string();

    server
        .mock("POST", "/oauth/token/")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(oauth_resp)
        .expect(1)
        .create_async()
        .await;

    server
        .mock(
            "GET",
            Matcher::Regex(format!("^/v1/blog/{blog}/bundle/{bundle_id}/post/\\?.*$")),
        )
        .match_header("authorization", "Bearer bundle_access_token")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .expect(1)
        .create_async()
        .await;

    let result = client.get_bundle(blog, bundle_id, &query).await.unwrap();
    assert_eq!(result.data.bundle_items.len(), 2);
}
