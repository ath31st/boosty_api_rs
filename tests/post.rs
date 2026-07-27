use std::fs;

use boosty_api::{api_client::ApiClient, error::ApiError};
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::{Value, json};

use crate::helpers::{api_path, setup};

mod helpers;

#[tokio::test]
async fn test_get_post_unauthorized() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";
    let post_id = "pid";
    let path = api_path(&format!("blog/{blog}/post/{post_id}"));

    server
        .mock("GET", path.as_str())
        .with_status(401)
        .create_async()
        .await;

    let res = client.get_post(blog, post_id).await;
    assert!(matches!(res, Err(ApiError::Unauthorized)));
}

#[tokio::test]
async fn test_get_post_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let post_id = "p";
    let path = api_path(&format!("blog/{blog}/post/{post_id}"));

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("not a valid json")
        .create_async()
        .await;

    let res = client.get_post(blog, post_id).await;
    assert!(matches!(res, Err(ApiError::JsonParseDetailed { .. })));
}

#[tokio::test]
async fn test_get_post_not_available_but_no_refresh() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let post_id = "99";
    let path = api_path(&format!("blog/{blog}/post/{post_id}"));

    let raw = fs::read_to_string("tests/fixtures/api_response_video_image.json").unwrap();
    let mut value: Value = serde_json::from_str(&raw).unwrap();
    value["id"] = Value::String(post_id.to_string());
    value["title"] = Value::String("Unavailable but returned".to_string());
    let body = value.to_string();

    server
        .mock("GET", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(body)
        .create_async()
        .await;

    let result = client.get_post(blog, post_id).await.unwrap();
    assert_eq!(result.id, "99");
    assert_eq!(result.title, Some(String::from("Unavailable but returned")));
}

#[tokio::test]
async fn test_get_post_with_refresh() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    client
        .set_refresh_token_and_device_id("old_refresh", "device123")
        .await
        .unwrap();

    let blog = "blog";
    let post_id = "100";
    let api_get_path = api_path(&format!("blog/{blog}/post/{post_id}"));

    let oauth_resp = json!({
        "access_token": "new_access_token",
        "refresh_token": "new_refresh_token",
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

    let pair = client.refresh_tokens().await.unwrap();
    assert_eq!(pair.access_token, "new_access_token");

    let raw = fs::read_to_string("tests/fixtures/api_response_video_image.json").unwrap();
    let mut value: Value = serde_json::from_str(&raw).unwrap();
    value["id"] = Value::String(post_id.to_string());
    value["title"] = Value::String("Post Title".to_string());
    let body = value.to_string();

    server
        .mock("GET", api_get_path.as_str())
        .match_header("authorization", "Bearer new_access_token")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(body)
        .expect(1)
        .create_async()
        .await;

    let result = client.get_post(blog, post_id).await.unwrap();
    assert_eq!(result.id, "100");
    assert_eq!(result.title, Some(String::from("Post Title")));
}

#[tokio::test]
async fn test_refresh_tokens_error() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    client
        .set_refresh_token_and_device_id("r", "d")
        .await
        .unwrap();

    server
        .mock("POST", "/oauth/token/")
        .with_status(500)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(r#"{"error":"server"}"#)
        .expect(1)
        .create_async()
        .await;

    let res = client.refresh_tokens().await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_get_posts_unauthorized() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let limit = 3;
    let api_path = api_path(&format!("blog/{blog}/post/?limit={limit}"));

    server
        .mock("GET", api_path.as_str())
        .with_status(401)
        .create_async()
        .await;

    let res = client.get_posts(blog, limit, None, None).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_get_posts_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blog";
    let limit = 5;
    let api_path = api_path(&format!("blog/{blog}/post/?limit={limit}"));

    server
        .mock("GET", api_path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("not json")
        .create_async()
        .await;

    let res = client.get_posts(blog, limit, None, None).await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_set_bearer_token_in_get_posts() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    client.set_bearer_token("tokXYZ").await.unwrap();

    let blog = "testblog";
    let limit = 2;
    let api_path = api_path(&format!("blog/{blog}/post/?limit={limit}"));

    let raw = fs::read_to_string("tests/fixtures/api_response_posts.json").unwrap();
    let mut value: Value = serde_json::from_str(&raw).unwrap();
    if let Some(arr) = value.get_mut("data").and_then(|v| v.as_array_mut())
        && let Some(first) = arr.get_mut(0)
    {
        first["id"] = Value::String("p1".to_string());
        first["title"] = Value::String("Title1".to_string());
        first["hasAccess"] = Value::Bool(true);
    }
    let list_body = value.to_string();

    server
        .mock("GET", api_path.as_str())
        .match_header("authorization", "Bearer tokXYZ")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(list_body)
        .create_async()
        .await;

    let result = client.get_posts(blog, limit, None, None).await.unwrap();
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].id, "p1");
}

#[tokio::test]
async fn test_set_refresh_and_get_post_header_and_flow() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    client
        .set_refresh_token_and_device_id("refA", "devA")
        .await
        .unwrap();

    let oauth_resp = json!({
        "access_token": "fresh_token",
        "refresh_token": "fresh_ref",
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

    client.refresh_tokens().await.unwrap();

    let blog = "blog";
    let post_id = "55";
    let api_get_path = api_path(&format!("blog/{blog}/post/{post_id}"));

    let raw = fs::read_to_string("tests/fixtures/api_response_video_image.json").unwrap();
    let mut value: Value = serde_json::from_str(&raw).unwrap();
    value["id"] = Value::String(post_id.to_string());
    value["title"] = Value::String("Title55".to_string());
    value["hasAccess"] = Value::Bool(true);
    let resp_body = value.to_string();

    server
        .mock("GET", api_get_path.as_str())
        .match_header("authorization", "Bearer fresh_token")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(resp_body)
        .create_async()
        .await;

    let result = client.get_post(blog, post_id).await.unwrap();
    assert_eq!(result.id, "55");
}
