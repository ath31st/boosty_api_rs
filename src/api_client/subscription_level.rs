use crate::api_client::{ApiClient, QueryParams};
use crate::error::ResultApi;
use crate::model::SubscriptionLevelResponse;

impl ApiClient {
    /// Fetch subscription levels for a blog, with optional inclusion of the free level.
    pub async fn get_blog_subscription_levels(
        &self,
        blog_name: &str,
        show_free_level: Option<bool>,
    ) -> ResultApi<SubscriptionLevelResponse> {
        let path = format!("blog/{blog_name}/subscription_level/");

        let query = QueryParams::new()
            .push("show_free_level", show_free_level);

        self.get_json(&path, &query.as_slice()).await
    }
}
