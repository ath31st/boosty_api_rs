use crate::{api_client::ApiClient, error::ResultApi, model::BundlesResponse};

impl ApiClient {
    /// Get all bundles for a blog.
    ///
    /// # Arguments
    /// * `blog_name` - Blog name
    ///
    /// # Returns
    /// * On success, returns a `BundlesResponse` containing the `bundles` field with `Bundle` items.
    ///
    /// # Errors
    /// * `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// * `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// * `ApiError::HttpRequest` if the HTTP request fails.
    /// * `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `BundlesResponse`.
    pub async fn get_bundles(&self, blog_name: &str) -> ResultApi<BundlesResponse> {
        let path = format!("blog/{blog_name}/bundle/");

        let response = self.get_request(&path).await?;
        let response = self.handle_response(&path, response).await?;

        self.parse_json(response).await
    }
}
