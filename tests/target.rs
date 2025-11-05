mod helpers;

use std::fs;

use boosty_api::{api_client::ApiClient, error::ApiError, model::TargetType};
use reqwest::{Client, header::CONTENT_TYPE};
use serde_json::json;

use crate::helpers::{api_path, setup};

#[tokio::test]
async fn test_get_targets_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";
    let api_path = api_path(&format!("target/{blog}/"));

    let raw = fs::read_to_string("tests/fixtures/api_response_targets.json").unwrap();

    server
        .mock("GET", api_path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(raw)
        .create_async()
        .await;

    let targets = client.get_blog_targets(blog).await.unwrap();
    assert!(!targets.data.is_empty());
    let first = &targets.data[0];
    assert_eq!(first.id, 600101);
    assert_eq!(first.description, "🏠 Saving for a new family home");
    assert_eq!(first.target_sum, 1200000.5);
}

#[tokio::test]
async fn test_get_targets_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let blog = "blogx";
    let api_path = api_path(&format!("target/{blog}/"));

    server
        .mock("GET", api_path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("not a valid json")
        .create_async()
        .await;

    let res = client.get_blog_targets(blog).await;
    assert!(matches!(res, Err(ApiError::JsonParse(_))));
}

#[tokio::test]
async fn test_create_target_money_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let path = api_path("target/money");
    let blog_url = "blogx";
    let description = "New target";
    let target_sum = 1000.4;
    let id = 111;

    let response_body = json!({
        "id": id,
        "bloggerUrl": blog_url,
        "description": description,
        "bloggerId": 1,
        "priority": 1,
        "createdAt": 1_697_000_000,
        "targetSum": target_sum,
        "currentSum": 0,
        "finishTime": null,
        "type": "money"
    })
    .to_string();

    server
        .mock("POST", path.as_str())
        .match_header("content-type", "application/x-www-form-urlencoded")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(response_body)
        .create_async()
        .await;

    let result = client
        .create_blog_target(blog_url, description, target_sum, TargetType::Money)
        .await
        .unwrap();

    assert_eq!(result.id, id);
    assert_eq!(result.blogger_url, blog_url);
    assert_eq!(result.description, description);
    assert_eq!(result.target_sum, target_sum);
    assert_eq!(result.current_sum as u32, 0);
}

#[tokio::test]
async fn test_create_target_subscribers_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let path = api_path("target/subscribers");
    let blog_url = "blogx";
    let description = "New target";
    let target_sum = 1000.4;
    let id = 111;

    let response_body = json!({
        "id": id,
        "bloggerUrl": blog_url,
        "description": description,
        "bloggerId": 1,
        "priority": 1,
        "createdAt": 1_697_000_000,
        "targetSum": target_sum,
        "currentSum": 0,
        "finishTime": null,
        "type": "money"
    })
    .to_string();

    server
        .mock("POST", path.as_str())
        .match_header("content-type", "application/x-www-form-urlencoded")
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(response_body)
        .create_async()
        .await;

    let result = client
        .create_blog_target(blog_url, description, target_sum, TargetType::Subscribers)
        .await
        .unwrap();

    assert_eq!(result.id, id);
    assert_eq!(result.blogger_url, blog_url);
    assert_eq!(result.description, description);
    assert_eq!(result.target_sum, target_sum);
    assert_eq!(result.current_sum as u32, 0);
}

#[tokio::test]
async fn test_delete_target_success() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let target_id = 456;
    let path = api_path(format!("target/{}", target_id).as_str());

    server
        .mock("DELETE", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body(r#"{}"#)
        .create_async()
        .await;

    let result = client.delete_blog_target(target_id).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_target_invalid_json() {
    let (mut server, base) = setup().await;
    let client = ApiClient::new(Client::new(), &base);

    let target_id = 789;
    let path = api_path(format!("target/{}", target_id).as_str());

    server
        .mock("DELETE", path.as_str())
        .with_status(200)
        .with_header(CONTENT_TYPE, "application/json")
        .with_body("invalid json")
        .create_async()
        .await;

    let result = client.delete_blog_target(target_id).await;
    assert!(matches!(result, Err(ApiError::JsonParse(_))));
}
