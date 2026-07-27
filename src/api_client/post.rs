use crate::api_client::{ApiClient, DEFAULT_PAGE_SIZE, QueryParams};
use crate::error::ResultApi;
use crate::model::{Post, PostsResponse};

impl ApiClient {
    /// Get a single post.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401.
    /// - `ApiError::HttpStatus` for other non-success statuses.
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParseDetailed` if JSON deserialization fails.
    pub async fn get_post(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let path = format!("blog/{blog_name}/post/{post_id}");
        self.get_json(&path, &[]).await
    }

    /// Get multiple posts for a blog with client-side pagination.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `limit`: total number of posts to fetch.
    /// - `page_size`: posts per page (defaults to 20).
    /// - `start_offset`: optional offset to start from.
    pub async fn get_posts(
        &self,
        blog_name: &str,
        limit: usize,
        page_size: Option<usize>,
        start_offset: Option<String>,
    ) -> ResultApi<Vec<Post>> {
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);

        let mut all_posts = Vec::new();
        let mut offset = start_offset;

        loop {
            let current_limit = page_size.min(limit - all_posts.len());
            let path = format!("blog/{blog_name}/post/");

            let query = QueryParams::new()
                .push("limit", Some(current_limit))
                .push("offset", offset.as_deref());

            let posts_response: PostsResponse = self.get_json(&path, &query.as_slice()).await?;

            let data_len = posts_response.data.len();
            all_posts.extend(posts_response.data);

            if posts_response.extra.is_last || all_posts.len() >= limit || data_len == 0 {
                break;
            }

            offset = Some(posts_response.extra.offset);
        }

        Ok(all_posts)
    }
}
