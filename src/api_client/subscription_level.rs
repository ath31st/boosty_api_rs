use crate::api_client::ApiClient;
use crate::error::{ApiError, ResultApi};
use crate::model::SubscriptionLevelResponse;

impl ApiClient {
    /// Fetch subscription levels for a blog, with optional inclusion of the free level.
    ///
    /// If `show_free_level` is `Some(true)`, appends `?show_free_level=true` to the URL.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: the identifier or name of the blog.
    /// - `show_free_level`: when `Some(true)`, include the free subscription level in results.
    ///
    /// # Returns
    ///
    /// On success, returns a `SubscriptionLevelResponse` containing the `"data"` array with levels.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the body cannot be deserialized into `SubscriptionLevelResponse`.
    pub async fn get_blog_subscription_levels(
        &self,
        blog_name: &str,
        show_free_level: Option<bool>,
    ) -> ResultApi<SubscriptionLevelResponse> {
        let mut path = format!("blog/{blog_name}/subscription_level/");
        if let Some(flag) = show_free_level {
            path.push_str(&format!("?show_free_level={flag}"));
        }

        let response = self.get_request(&path).await?;

        let parsed = response
            .json::<SubscriptionLevelResponse>()
            .await
            .map_err(ApiError::JsonParse)?;

        Ok(parsed)
    }
}
