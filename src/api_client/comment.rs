use reqwest::multipart::{Form, Part};

use crate::{
    api_client::{ApiClient, QueryParams},
    error::{ApiError, ResultApi},
    model::{Comment, CommentBlock, CommentsResponse},
};

impl ApiClient {
    /// Get comments response for a post.
    ///
    /// # Arguments
    ///
    /// * `blog_name` - Blog name (blog url)
    /// * `post_id` - Post id
    /// * `limit` - Limit comments per request (optional)
    /// * `reply_limit` - Reply levels (optional)
    /// * `order` - Top or bottom (optional)
    /// * `offset` - Offset (intId comment) (optional)
    pub async fn get_comments_response(
        &self,
        blog_name: &str,
        post_id: &str,
        limit: Option<u32>,
        reply_limit: Option<u32>,
        order: Option<&str>,
        offset: Option<u64>,
    ) -> ResultApi<CommentsResponse> {
        let path = format!("blog/{blog_name}/post/{post_id}/comment/");

        let query = QueryParams::new()
            .push("offset", offset)
            .push("limit", limit)
            .push("reply_limit", reply_limit)
            .push("order", order);

        self.get_json(&path, &query.as_slice()).await
    }

    /// Get all comments for a post (paginated automatically).
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

        self.post_multipart_json(&path, form).await
    }
}
