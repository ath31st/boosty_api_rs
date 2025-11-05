#[cfg(test)]
mod tests {
    use boosty_api::error::ApiError;
    use boosty_api::{api_client::ApiClient, api_response::TargetType};
    use mockito::Server;
    use reqwest::Client;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::{Value, json};
    use std::fs;

    fn api_path(path: &str) -> String {
        format!("/v1/{path}")
    }

    #[tokio::test]
    async fn test_fetch_post_unauthorized() {
        let mut server = Server::new_async().await;
        let base = server.url();
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
    async fn test_fetch_post_invalid_json() {
        let mut server = Server::new_async().await;
        let base = server.url();
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
    async fn test_fetch_post_not_available_but_no_refresh() {
        let mut server = Server::new_async().await;
        let base = server.url();
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
        assert_eq!(result.title, "Unavailable but returned");
    }

    #[tokio::test]
    async fn test_fetch_post_with_refresh() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let req_client = Client::new();
        let client = ApiClient::new(req_client.clone(), &base);

        client
            .set_refresh_token_and_device_id("old_refresh", "device123")
            .await
            .unwrap();

        let blog = "blog";
        let post_id = "100";
        let api_get_path = api_path(&format!("blog/{blog}/post/{post_id}"));

        let raw = fs::read_to_string("tests/fixtures/api_response_video_image.json").unwrap();
        let mut first_value: Value = serde_json::from_str(&raw).unwrap();
        first_value["id"] = Value::String(post_id.to_string());
        first_value["title"] = Value::String("Old Title".to_string());
        let first_body = first_value.to_string();

        server
            .mock("GET", api_get_path.as_str())
            .with_status(200)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body(first_body)
            .expect(1)
            .create_async()
            .await;

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

        let mut second_value: Value = serde_json::from_str(&raw).unwrap();
        second_value["id"] = Value::String(post_id.to_string());
        second_value["title"] = Value::String("New Title".to_string());
        let second_body = second_value.to_string();

        server
            .mock("GET", api_get_path.as_str())
            .with_status(200)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body(second_body)
            .expect(1)
            .create_async()
            .await;

        let result = client.get_post(blog, post_id).await.unwrap();
        assert_eq!(result.id, "100");
        assert_eq!(result.title, "Old Title");
    }

    #[tokio::test]
    async fn test_fetch_post_refresh_error() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        client
            .set_refresh_token_and_device_id("r", "d")
            .await
            .unwrap();

        let blog = "b";
        let post_id = "77";
        let api_get_path = api_path(&format!("blog/{blog}/post/{post_id}"));

        let raw = fs::read_to_string("tests/fixtures/api_response_video_image.json").unwrap();
        let mut value: Value = serde_json::from_str(&raw).unwrap();
        value["id"] = Value::String(post_id.to_string());
        value["title"] = Value::String("Will fail refresh".to_string());
        let body = value.to_string();

        server
            .mock("GET", api_get_path.as_str())
            .with_status(200)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body(body)
            .expect(1)
            .create_async()
            .await;

        server
            .mock("POST", "/oauth/token/")
            .with_status(500)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body(r#"{"error":"server"}"#)
            .expect(1)
            .create_async()
            .await;

        let res = client.get_post(blog, post_id).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_fetch_posts_unauthorized() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blog";
        let limit = 3;
        let api_path = api_path(&format!("blog/{blog}/post/?limit={limit}"));

        server
            .mock("GET", api_path.as_str())
            .with_status(401)
            .create_async()
            .await;

        let res = client.get_posts(blog, limit).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_fetch_posts_invalid_json() {
        let mut server = Server::new_async().await;
        let base = server.url();
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

        let res = client.get_posts(blog, limit).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_set_bearer_token_in_fetch_posts() {
        let mut server = Server::new_async().await;
        let base = server.url();
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

        let result = client.get_posts(blog, limit).await.unwrap();
        assert_eq!(result.data.len(), 2);
        assert_eq!(result.data[0].id, "p1");
    }

    #[tokio::test]
    async fn test_set_refresh_and_fetch_post_header_and_flow() {
        let mut server = Server::new_async().await;
        let base = server.url();
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

    #[tokio::test]
    async fn test_headers_as_map_after_set_bearer_token() {
        let _server = Server::new_async().await;
        let base = _server.url();
        let client = ApiClient::new(Client::new(), &base);
        client.set_bearer_token("tok123").await.unwrap();

        let map = client.headers_as_map();
        assert_eq!(
            map.get("accept").map(|s| s.as_str()),
            Some("application/json")
        );
    }

    #[tokio::test]
    async fn test_get_targets_success() {
        let mut server = Server::new_async().await;
        let base = server.url();
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
        let mut server = Server::new_async().await;
        let base = server.url();
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
    async fn test_get_subscription_levels_default() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blogx";
        let api_path = api_path(&format!("blog/{blog}/subscription_level/"));

        let raw =
            fs::read_to_string("tests/fixtures/api_response_subscription_levels.json").unwrap();

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
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blogx";
        let api_path = api_path(&format!(
            "blog/{blog}/subscription_level/?show_free_level=true"
        ));

        let raw =
            fs::read_to_string("tests/fixtures/api_response_subscription_levels.json").unwrap();

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
        let mut server = Server::new_async().await;
        let base = server.url();
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
        let mut server = Server::new_async().await;
        let base = server.url();
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

    #[tokio::test]
    async fn test_create_target_money_success() {
        let mut server = Server::new_async().await;
        let base = server.url();
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
            .create_target(blog_url, description, target_sum, TargetType::Money)
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
        let mut server = Server::new_async().await;
        let base = server.url();
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
            .create_target(blog_url, description, target_sum, TargetType::Subscribers)
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
        let mut server = Server::new_async().await;
        let base = server.url();
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

        let result = client.delete_target(target_id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_delete_target_invalid_json() {
        let mut server = Server::new_async().await;
        let base = server.url();
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

        let result = client.delete_target(target_id).await;
        assert!(matches!(result, Err(ApiError::JsonParse(_))));
    }
}
