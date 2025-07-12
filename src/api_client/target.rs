use crate::api_client::ApiClient;
use crate::api_response::Target;
use crate::error::{ApiError, ResultApi};
use serde_json::{Value, from_value};

impl ApiClient {
    /// Fetch all targets for a blog, optionally handling HTTP errors and JSON parsing.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or slug of the blog whose targets should be retrieved.
    ///
    /// # Returns
    ///
    /// A `Vec<Target>` decoded from the `"data"` field of the JSON response.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response body cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` array cannot be deserialized into `Target` structs.
    pub async fn get_blog_targets(&self, blog_name: &str) -> ResultApi<Vec<Target>> {
        let path = format!("target/{blog_name}/");
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }
}
