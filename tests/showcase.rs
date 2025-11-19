mod helpers;

use crate::helpers::{api_path, setup};
use boosty_api::{api_client::ApiClient, error::ApiError};
use reqwest::{Client, header::CONTENT_TYPE};
use std::fs;

#[tokio::test]
async fn test_get_showcase_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";

    let path = api_path(&format!("blog/{blog}/showcase/?limit=10&only_visible=true"));
    let raw = fs::read_to_string("tests/fixtures/api_response_showcase.json").unwrap();

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let resp = client
        .get_showcase(blog, Some(10), Some(true), None)
        .await
        .unwrap();

    assert!(!resp.data.showcase_items.is_empty());
}

#[tokio::test]
async fn test_get_showcase_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";

    let path = api_path(&format!("blog/{blog}/showcase/"));

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("invalid json")
        .create_async()
        .await;

    let res = client.get_showcase(blog, None, None, None).await;
    assert!(matches!(res, Err(ApiError::JsonParseDetailed { error: _ })));
}

#[tokio::test]
async fn test_get_showcase_http_error() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";

    let path = api_path(&format!("blog/{blog}/showcase/"));

    server
        .mock("GET", path.as_str())
        .with_status(500)
        .create_async()
        .await;

    let res = client.get_showcase(blog, None, None, None).await;
    assert!(matches!(res, Err(ApiError::HttpStatus { .. })));
}

#[tokio::test]
async fn test_change_showcase_status_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";

    let path = api_path(&format!("blog/{blog}/showcase/status/"));

    server
        .mock("PUT", path.as_str())
        .match_header("content-type", "application/x-www-form-urlencoded")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("{is_enabled: true}")
        .create_async()
        .await;

    let res = client.change_showcase_status(blog, true).await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_change_showcase_status_unauthorized() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";

    let path = api_path(&format!("blog/{blog}/showcase/status/"));

    server
        .mock("PUT", path.as_str())
        .with_status(401)
        .create_async()
        .await;

    let res = client.change_showcase_status(blog, true).await;
    assert!(matches!(res, Err(ApiError::Unauthorized)));
}

#[tokio::test]
async fn test_change_showcase_status_http_error() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";

    let path = api_path(&format!("blog/{blog}/showcase/status/"));

    server
        .mock("PUT", path.as_str())
        .with_status(500)
        .create_async()
        .await;

    let res = client.change_showcase_status(blog, false).await;
    assert!(matches!(res, Err(ApiError::HttpStatus { .. })));
}
