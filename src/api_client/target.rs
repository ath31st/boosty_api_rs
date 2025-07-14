use crate::api_client::ApiClient;
use crate::api_response::TargetResponse;
use crate::error::{ApiError, ResultApi};

impl ApiClient {
    /// Get all targets for a blog.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or slug of the blog whose targets should be retrieved.
    ///
    /// # Returns
    ///
    /// A `TargetResponse` decoded from the full JSON body.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response body cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the body cannot be deserialized into `TargetResponse`.
    pub async fn get_blog_targets(&self, blog_name: &str) -> ResultApi<TargetResponse> {
        let path = format!("target/{blog_name}/");
        let response = self.get_request(&path).await?;

        let parsed = response
            .json::<TargetResponse>()
            .await
            .map_err(ApiError::JsonParse)?;

        Ok(parsed)
    }
}
