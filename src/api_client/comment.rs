use reqwest::multipart::{Form, Part};

use crate::{
    api_client::ApiClient,
    error::{ApiError, ResultApi},
    model::{Comment, CommentBlock, CommentsResponse},
};

impl ApiClient {
    /// Get comments response
    ///
    /// # Arguments
    ///
    /// * `blog_name` - Blog name (blog url)
    /// * `post_id` - Post id (optional)
    /// * `limit` - Limit comments per request (optional)
    /// * `reply_limit` - Reply levels (optional)
    /// * `order` - Top or bottom (optional)
    /// * `offset` - Offset (intId comment) (optional)
    ///
    /// # Returns
    ///
    /// On success, returns a `CommentsResponse` containing the `data` field with `Comment` items.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `CommentsResponse`.
    pub async fn get_comments_response(
        &self,
        blog_name: &str,
        post_id: &str,
        limit: Option<u32>,
        reply_limit: Option<u32>,
        order: Option<&str>,
        offset: Option<u64>,
    ) -> ResultApi<CommentsResponse> {
        let mut path = format!("blog/{blog_name}/post/{post_id}/comment/");

        let mut params = Vec::new();
        if let Some(o) = offset {
            params.push(format!("offset={o}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if let Some(rl) = reply_limit {
            params.push(format!("reply_limit={rl}"));
        }
        if let Some(ord) = order {
            params.push(format!("order={ord}"));
        }

        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        let response = self.get_request(&path).await?;
        let response = self.handle_response(&path, response).await?;

        self.parse_json(response).await
    }

    /// Get all comments for a post.
    ///
    /// # Arguments
    ///
    /// * `blog_name` - Blog name (blog url)
    /// * `post_id` - Post id (optional)
    /// * `limit` - Limit comments per request (optional)
    /// * `reply_limit` - Reply levels (optional)
    /// * `order` - Top or bottom (optional)
    ///
    /// # Returns
    ///
    /// On success, returns a vector of `Comment` items.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `Comment`.
    pub async fn get_all_comments(
        &self,
        blog_name: &str,
        post_id: &str,
        limit: Option<u32>,
        reply_limit: Option<u32>,
        order: Option<&str>,
    ) -> ResultApi<Vec<Comment>> {
        let mut all_comments = Vec::new();
        let mut offset: Option<u64> = None;

        loop {
            let resp = self
                .get_comments_response(blog_name, post_id, limit, reply_limit, order, offset)
                .await?;

            if resp.data.is_empty() {
                break;
            }

            let last_id = resp.data.last().map(|c| c.int_id);

            all_comments.extend(resp.data);

            if resp.extra.is_last && resp.extra.is_first {
                break;
            }

            if let Some(id) = last_id {
                offset = Some(id);
            } else {
                break;
            }
        }

        Ok(all_comments)
    }

    /// Create a new comment.
    ///
    /// # Arguments
    ///
    /// * `blog_name` - Blog name (blog url)
    /// * `post_id` - Post id
    /// * `blocks` - Vector of `CommentBlock` items with the comment content
    /// * `reply_id` - Reply id (optional)
    ///
    /// # Returns
    ///
    /// On success, returns a `Comment` item.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `Comment`.
    /// - `ApiError::Other` if form creation fails.
    pub async fn create_comment(
        &self,
        blog_name: &str,
        post_id: &str,
        blocks: &[CommentBlock],
        reply_id: Option<u64>,
    ) -> ResultApi<Comment> {
        let path = format!("blog/{blog_name}/post/{post_id}/comment/");

        let mut form = Form::new().text("from_page", "blog");

        for block in blocks {
            form = form.part(
                "data[]",
                Part::text(serde_json::to_string(block).map_err(|e| {
                    ApiError::JsonParseDetailed {
                        error: e.to_string(),
                    }
                })?)
                .mime_str("application/json")
                .map_err(|e| ApiError::Other(e.to_string()))?,
            );
        }

        if let Some(id) = reply_id {
            form = form.text("reply_id", id.to_string());
        }

        let response = self.post_multipart(&path, form).await?;
        let response = self.handle_response(&path, response).await?;

        self.parse_json(response).await
    }
}
