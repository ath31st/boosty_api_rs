#[cfg(test)]
mod tests {
    use boosty_api::api_client::ApiClient;
    use reqwest::Client;
    use std::fs;
    use tokio;

    #[tokio::test]
    async fn test_fetch_post() {
        let mut server = mockito::Server::new_async().await;

        let blog_name = "testblog";
        let post_id = "123";

        let mock_response =
            fs::read_to_string("tests/fixtures/api_response_video_image.json").unwrap();

        server
            .mock(
                "GET",
                format!("/v1/blog/{}/post/{}", blog_name, post_id).as_str(),
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let req_client = Client::new();
        let client = ApiClient::new(req_client, &server.url());
        
        let result = client.fetch_post(blog_name, post_id).await.unwrap();

        assert_eq!(result.id, "post001");
        assert_eq!(result.title, "Test post");
    }

    #[tokio::test]
    async fn test_fetch_posts() {
        let mut server = mockito::Server::new_async().await;

        let blog_name = "testblog";
        let limit = 2;

        let mock_response = fs::read_to_string("tests/fixtures/api_response_posts.json").unwrap();
        let path = format!("/v1/blog/{}/post/?limit={}", blog_name, limit);

        server
            .mock("GET", path.as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(&mock_response)
            .create_async()
            .await;

        let req_client = Client::new();
        let client = ApiClient::new(req_client, &server.url());
        let result = client.fetch_posts(blog_name, limit).await.unwrap();

        assert_eq!(result.len(), 2);

        let first = &result[0];
        assert_eq!(first.id, "post001");
        assert_eq!(first.title, "Post One");

        let second = &result[1];
        assert_eq!(second.id, "post002");
        assert_eq!(second.title, "Post Two");

        assert_eq!(first.user.name, "TestUser1");
        assert_eq!(second.user.flags.show_post_donations, false);
    }
}
