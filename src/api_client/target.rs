use crate::api_client::ApiClient;
use crate::error::{ApiError, ResultApi};
use crate::model::{NewTarget, Target, TargetResponse, TargetType, UpdateTarget};

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
        let response = self.handle_response(&path, response).await?;

        self.parse_json(response).await
    }

    /// Create a new target for a blog.
    ///
    /// # Parameters
    ///
    /// - `blog_url`: identifier or slug of the blog for which the target is created.
    /// - `description`: textual description of the target (e.g., purpose of the fundraising).
    /// - `target_sum`: numerical value of the target amount.
    /// - `target_type`: one of [`TargetType::Money`] or [`TargetType::Subscribers`].
    ///
    /// # Returns
    ///
    /// A [`Target`] object deserialized from the JSON response body.
    ///
    /// # Errors
    ///
    /// - [`ApiError::HttpRequest`] — if the network request fails.
    /// - [`ApiError::JsonParse`] — if the response body cannot be parsed as valid JSON.
    /// - [`ApiError::Deserialization`] — if the JSON does not match the [`Target`] structure.
    pub async fn create_blog_target(
        &self,
        blog_name: &str,
        description: &str,
        target_sum: f64,
        target_type: TargetType,
    ) -> ResultApi<Target> {
        let path = match target_type {
            TargetType::Money => "target/money",
            TargetType::Subscribers => "target/subscribers",
        };

        let form = NewTarget {
            blog_url: blog_name.into(),
            description: description.into(),
            target_sum,
        };

        let response = self.post_request(path, &form, true).await?;
        let response = self.handle_response(path, response).await?;

        self.parse_json(response).await
    }

    /// Delete a target by its ID.
    ///
    /// # Parameters
    ///
    /// - `target_id`: numerical ID of the target to delete.
    ///
    /// # Returns
    ///
    /// `()` on success. The API returns 200 OK with an empty JSON body.
    ///
    /// # Errors
    ///
    /// - [`ApiError::HttpRequest`] — if the network request fails.
    /// - [`ApiError::JsonParse`] — if the response body cannot be parsed as JSON (rare for DELETE).
    pub async fn delete_blog_target(&self, target_id: u64) -> ResultApi<()> {
        let path = format!("target/{}", target_id);

        let response = self.delete_request(&path).await?;

        let _ = response
            .json::<serde_json::Value>()
            .await
            .map_err(ApiError::JsonParse)?;

        Ok(())
    }

    /// Update an existing target by its ID.
    ///
    /// # Parameters
    ///
    /// - `target_id`: numerical ID of the target.
    /// - `description`: new textual description of the target.
    /// - `target_sum`: new target amount.
    ///
    /// # Returns
    ///
    /// Updated [`Target`] object.
    ///
    /// # Errors
    ///
    /// - [`ApiError::HttpRequest`] — if the network request fails.
    /// - [`ApiError::JsonParse`] — if JSON parsing fails.
    pub async fn update_blog_target(
        &self,
        target_id: u64,
        description: &str,
        target_sum: f64,
    ) -> ResultApi<Target> {
        let form = UpdateTarget {
            target_id,
            description: description.into(),
            target_sum,
        };

        let path = format!("target/{}", target_id);

        let response = self.put_request(&path, &form, true).await?;
        let response = self.handle_response(&path, response).await?;

        self.parse_json(response).await
    }
}
