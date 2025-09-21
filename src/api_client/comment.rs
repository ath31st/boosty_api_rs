use reqwest::StatusCode;

use crate::{
    api_client::ApiClient,
    api_response::CommentsResponse,
    error::{ApiError, ResultApi},
};

impl ApiClient {
    pub async fn get_comments(
        &self,
        blog_name: &str,
        post_id: &str,
        limit: Option<u32>,
        reply_limit: Option<u32>,
        order: Option<&str>,
    ) -> ResultApi<CommentsResponse> {
        let mut path = format!("blog/{blog_name}/post/{post_id}/comment/");

        let mut params = Vec::new();
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
}
