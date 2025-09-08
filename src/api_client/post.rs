use crate::api_client::ApiClient;
use crate::api_response::{Post, PostsResponse};
use crate::error::{ApiError, ResultApi};
use reqwest::StatusCode;

impl ApiClient {
    /// Get a single post once, without automatic retry on "not available" or HTTP 401.
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
    pub async fn get_post(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
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

    /// Get multiple posts for a blog.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `limit`: maximum number of posts to fetch.
    ///
    /// # Returns
    ///
    /// On success, returns a `PostsResponse` containing the `data` field with `Post` items.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` field cannot be deserialized into a vector of `Post`.
    pub async fn get_posts(&self, blog_name: &str, limit: usize) -> ResultApi<PostsResponse> {
        let path = format!("blog/{blog_name}/post/?limit={limit}");
        let response = self.get_request(&path).await?;
        let status = response.status();

        if status == 401 {
            return Err(ApiError::Unauthorized);
        }

        let posts_response = response
            .json::<PostsResponse>()
            .await
            .map_err(ApiError::JsonParse)?;
        Ok(posts_response)
    }
}
