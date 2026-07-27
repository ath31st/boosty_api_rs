use crate::api_client::{ApiClient, QueryParams};
use crate::error::ResultApi;
use crate::model::SubscriptionsResponse;

impl ApiClient {
    /// Fetch the current user's subscriptions.
    pub async fn get_user_subscriptions(
        &self,
        limit: Option<u32>,
        with_follow: Option<bool>,
    ) -> ResultApi<SubscriptionsResponse> {
        let query = QueryParams::new()
            .push("limit", limit)
            .push("with_follow", with_follow);

        self.get_json("user/subscriptions", &query.as_slice()).await
    }
}
