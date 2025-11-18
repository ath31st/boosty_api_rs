use reqwest::StatusCode;

use crate::{
    api_client::ApiClient,
    error::{ApiError, ResultApi},
    model::ShowcaseResponse,
};

impl ApiClient {
    /// Get blog showcase
    ///
    /// # Arguments
    /// * `blog_name` - Blog name
    /// * `limit` - Limit
    /// * `only_visible` - Only visible
    /// * `offset` - Offset
    ///
    /// # Returns
    /// * On success, returns a `ShowcaseResponse` containing the `data` field with `showcase_items`.
    ///
    /// # Errors
    /// * `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// * `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// * `ApiError::HttpRequest` if the HTTP request fails.
    /// * `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `ShowcaseResponse`.
    pub async fn get_showcase(
        &self,
        blog_name: &str,
        limit: Option<u32>,
        only_visible: Option<bool>,
        offset: Option<u32>,
    ) -> ResultApi<ShowcaseResponse> {
        let mut path = format!("blog/{blog_name}/showcase/");

        let mut params = Vec::new();
        if let Some(o) = offset {
            params.push(format!("offset={o}"));
        }
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if let Some(ov) = only_visible {
            params.push(format!("only_visible={ov}"));
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
        let parsed = serde_json::from_str::<ShowcaseResponse>(&body).map_err(|e| {
            ApiError::JsonParseDetailed {
                error: e.to_string(),
            }
        })?;

        Ok(parsed)
    }

    /// Change blog showcase status
    ///
    /// # Arguments
    /// * `blog_name` - Blog name
    /// * `status` - Status (true to enable, false to disable)
    ///
    /// # Returns
    /// * On success, returns `()`.
    ///
    /// # Errors
    /// * `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// * `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// * `ApiError::HttpRequest` if the HTTP request fails.
    pub async fn change_showcase_status(&self, blog_name: &str, status: bool) -> ResultApi<()> {
        let path = format!("blog/{blog_name}/showcase/status/");

        let response = self
            .put_request(&path, &serde_json::json!({"is_enabled": status}), true)
            .await?;

        let status = response.status();

        if !status.is_success() {
            let endpoint = path.clone();
            return Err(ApiError::HttpStatus { status, endpoint });
        }

        Ok(())
    }
}
