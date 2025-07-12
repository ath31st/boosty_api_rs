use crate::api_client::ApiClient;
use crate::api_response::SubscriptionLevel;
use crate::error::{ApiError, ResultApi};
use serde_json::{Value, from_value};

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
    /// On success, returns a `Vec<SubscriptionLevel>` parsed from the `"data"` array of the response.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` field cannot be deserialized into subscription level items.
    pub async fn get_blog_subscription_levels(
        &self,
        blog_name: &str,
        show_free_level: Option<bool>,
    ) -> ResultApi<Vec<SubscriptionLevel>> {
        let mut path = format!("blog/{blog_name}/subscription_level/");
        if let Some(flag) = show_free_level {
            path.push_str(&format!("?show_free_level={flag}"));
        }
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }
}
