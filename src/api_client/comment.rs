use reqwest::StatusCode;

use crate::{
    api_client::ApiClient,
    api_response::{Comment, CommentsResponse},
    error::{ApiError, ResultApi},
};

impl ApiClient {
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
        let status = response.status();

        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized);
        }

        if !status.is_success() {
            let endpoint = path.clone();
            return Err(ApiError::HttpStatus { status, endpoint });
        }

        let body = response.text().await?;
        let parsed = serde_json::from_str::<CommentsResponse>(&body).map_err(|e| {
            ApiError::JsonParseDetailed {
                error: e.to_string(),
            }
        })?;

        Ok(parsed)
    }

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
}
