use crate::api_client::ApiClient;
use crate::api_response::Post;
use crate::error::{ApiError, ResultApi};
use reqwest::StatusCode;
use serde_json::{Value, from_value};

impl ApiClient {
    /// Fetch a single post once, without automatic retry on "not available" or HTTP 401.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or name of the blog.
    /// - `post_id`: identifier of the post.
    ///
    /// # Returns
    ///
    /// On success, returns the `Post` object.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `Post`.
    async fn fetch_post_once(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let path = format!("blog/{blog_name}/post/{post_id}");
        let response = self.get_request(&path).await?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized);
        }
        if !status.is_success() {
            let endpoint = path.clone();
            return Err(ApiError::HttpStatus { status, endpoint });
        }

        let body = response.text().await?;
        let parsed =
            serde_json::from_str::<Post>(&body).map_err(|e| ApiError::JsonParseDetailed {
                error: e.to_string(),
            })?;

        Ok(parsed)
    }

    /// Fetch a single post, with automatic retry on "not available" if refresh-token flow is configured.
    ///
    /// If the first attempt yields a `Post` where `post.not_available()` is true, and refresh-token
    /// flow is enabled, forces a token refresh and retries once.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `post_id`: post identifier.
    ///
    /// # Returns
    ///
    /// On success, returns the fetched `Post`.
    ///
    /// # Errors
    ///
    ///  - `ApiError::Unauthorized`
    ///  - `ApiError::HttpStatus`
    ///  - `ApiError::JsonParseDetailed`
    ///  - Errors from token refresh (`ApiError::Auth`)
    pub async fn fetch_post(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let mut post = self.fetch_post_once(blog_name, post_id).await?;
        if post.not_available() && self.auth_provider.has_refresh_and_device_id().await {
            self.auth_provider.force_refresh().await?;
            post = self.fetch_post_once(blog_name, post_id).await?;
        }
        Ok(post)
    }

    /// Fetch multiple posts once, parsing JSON array under `"data"` key.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `limit`: maximum number of posts to fetch.
    ///
    /// # Returns
    ///
    /// On success, returns a vector of `Post` objects.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` field cannot be deserialized into a vector of `Post`.
    async fn fetch_posts_once(&self, blog_name: &str, limit: i32) -> ResultApi<Vec<Post>> {
        let path = format!("blog/{blog_name}/post/?limit={limit}");
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }

    /// Fetch multiple posts, with automatic retry if any post is marked "not available" and refresh-token flow is set.
    ///
    /// Checks if any fetched post returns `not_available() == true`. If so, and if refresh-token authentication is configured,
    /// forces a token refresh and retries the request once.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or name of the blog.
    /// - `limit`: maximum number of posts to fetch.
    ///
    /// # Returns
    ///
    /// On success, returns a vector of `Post` objects.
    /// On failure, returns an `ApiError`.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::Auth` if refreshing the authentication token fails.
    /// - `ApiError::JsonParse` if the response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if JSON cannot be deserialized into `Post` objects.
    pub async fn fetch_posts(&self, blog_name: &str, limit: i32) -> ResultApi<Vec<Post>> {
        let mut posts = self.fetch_posts_once(blog_name, limit).await?;
        if posts.iter().any(|p| p.not_available())
            && self.auth_provider.has_refresh_and_device_id().await
        {
            self.auth_provider
                .force_refresh()
                .await
                .map_err(ApiError::Auth)?;

            posts = self.fetch_posts_once(blog_name, limit).await?;
        }
        Ok(posts)
    }
}
