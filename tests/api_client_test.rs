#[cfg(test)]
mod tests {
    use boosty_api::api_client::ApiClient;
    use boosty_api::error::ApiError;
    use mockito::Server;
    use reqwest::Client;
    use reqwest::header::CONTENT_TYPE;
    use serde_json::{Value, json};
    use std::fs;

    fn api_path(path: &str) -> String {
        format!("/v1/{}", path)
    }

    #[tokio::test]
    async fn test_fetch_post_unauthorized() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blogx";
        let post_id = "pid";
        let path = api_path(&format!("blog/{}/post/{}", blog, post_id));

        server
            .mock("GET", path.as_str())
            .with_status(401)
            .create_async()
            .await;

        let res = client.fetch_post(blog, post_id).await;
        assert!(matches!(res, Err(ApiError::Unauthorized)));
    }

    #[tokio::test]
    async fn test_fetch_post_invalid_json() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blog";
        let post_id = "p";
        let path = api_path(&format!("blog/{}/post/{}", blog, post_id));

        server
            .mock("GET", path.as_str())
            .with_status(200)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body("not a valid json")
            .create_async()
            .await;

        let res = client.fetch_post(blog, post_id).await;
        assert!(matches!(res, Err(ApiError::JsonParseDetailed { .. })));
    }

    #[tokio::test]
    async fn test_fetch_post_not_available_but_no_refresh() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blog";
        let post_id = "99";
        let path = api_path(&format!("blog/{}/post/{}", blog, post_id));

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

        let result = client.fetch_post(blog, post_id).await.unwrap();
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
            .set_refresh_token_and_device_id("old_refresh".into(), "device123".into())
            .await
            .unwrap();

        let blog = "blog";
        let post_id = "100";
        let api_get_path = api_path(&format!("blog/{}/post/{}", blog, post_id));

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

        let result = client.fetch_post(blog, post_id).await.unwrap();
        assert_eq!(result.id, "100");
        assert_eq!(result.title, "Old Title");
    }

    #[tokio::test]
    async fn test_fetch_post_refresh_error() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        client
            .set_refresh_token_and_device_id("r".into(), "d".into())
            .await
            .unwrap();

        let blog = "b";
        let post_id = "77";
        let api_get_path = api_path(&format!("blog/{}/post/{}", blog, post_id));

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

        let res = client.fetch_post(blog, post_id).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_fetch_posts_unauthorized() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blog";
        let limit = 3;
        let api_path = api_path(&format!("blog/{}/post/?limit={}", blog, limit));

        server
            .mock("GET", api_path.as_str())
            .with_status(401)
            .create_async()
            .await;

        let res = client.fetch_posts(blog, limit).await;
        assert!(res.is_err());
    }

    #[tokio::test]
    async fn test_fetch_posts_invalid_json() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        let blog = "blog";
        let limit = 5;
        let api_path = api_path(&format!("blog/{}/post/?limit={}", blog, limit));

        server
            .mock("GET", api_path.as_str())
            .with_status(200)
            .with_header(CONTENT_TYPE, "application/json")
            .with_body("not json")
            .create_async()
            .await;

        let res = client.fetch_posts(blog, limit).await;
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
        let api_path = api_path(&format!("blog/{}/post/?limit={}", blog, limit));

        let raw = fs::read_to_string("tests/fixtures/api_response_posts.json").unwrap();
        let mut value: Value = serde_json::from_str(&raw).unwrap();
        if let Some(arr) = value.get_mut("data").and_then(|v| v.as_array_mut()) {
            if let Some(first) = arr.get_mut(0) {
                first["id"] = Value::String("p1".to_string());
                first["title"] = Value::String("Title1".to_string());
                first["hasAccess"] = Value::Bool(true);
            }
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

        let result = client.fetch_posts(blog, limit).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].id, "p1");
    }

    #[tokio::test]
    async fn test_set_refresh_and_fetch_post_header_and_flow() {
        let mut server = Server::new_async().await;
        let base = server.url();
        let client = ApiClient::new(Client::new(), &base);

        client
            .set_refresh_token_and_device_id("refA".into(), "devA".into())
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
        let api_get_path = api_path(&format!("blog/{}/post/{}", blog, post_id));

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

        let result = client.fetch_post(blog, post_id).await.unwrap();
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
}
