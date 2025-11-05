use crate::api_client::ApiClient;
use crate::error::{ApiError, ResultApi};
use crate::model::SubscriptionsResponse;
use reqwest::StatusCode;

impl ApiClient {
    /// Fetch the current user's subscriptions, with optional pagination and follow filter.
    ///
    /// Sends a GET request with query parameters:
    /// - `limit`: maximum number of items to return (default server-side if omitted).
    /// - `with_follow`: when `Some(true)`, include subscriptions to followed blogs.
    ///
    /// # Parameters
    ///
    /// - `limit`: optional maximum number of subscriptions to fetch.
    /// - `with_follow`: optional flag to include subscriptions on followed blogs.
    ///
    /// # Returns
    ///
    /// On success, returns a `SubscriptionsResponse` containing the list of subscriptions and pagination info.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the JSON cannot be deserialized into `SubscriptionsResponse`.
    pub async fn get_user_subscriptions(
        &self,
        limit: Option<u32>,
        with_follow: Option<bool>,
    ) -> ResultApi<SubscriptionsResponse> {
        let mut path = "user/subscriptions".to_string();
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if let Some(f) = with_follow {
            params.push(format!("with_follow={f}"));
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

        let subs = response
            .json::<SubscriptionsResponse>()
            .await
            .map_err(ApiError::JsonParse)?;

        Ok(subs)
    }
}
