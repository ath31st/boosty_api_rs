use crate::{
    api_client::ApiClient,
    error::ResultApi,
    model::{BundleItemsResponse, BundlesResponse},
};

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

    /// Get posts within a specific bundle.
    ///
    /// # Arguments
    /// * `blog_name` - Blog name
    /// * `bundle_id` - Bundle UUID
    /// * `full_data` - Whether to fetch full data
    /// * `limit` - Number of posts to fetch
    /// * `for_owner` - Whether to fetch as owner
    /// * `comments_limit` - Comments limit
    /// * `reply_limit` - Reply limit
    ///
    /// # Returns
    /// * On success, returns a `BundleItemsResponse` containing the `bundleItems` field with `BundleItem` items.
    ///
    /// # Errors
    /// * `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// * `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// * `ApiError::HttpRequest` if the HTTP request fails.
    /// * `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `BundleItemsResponse`.
    pub async fn get_bundle(
        &self,
        blog_name: &str,
        bundle_id: &str,
        full_data: bool,
        limit: u32,
        for_owner: bool,
        comments_limit: u32,
        reply_limit: u32,
    ) -> ResultApi<BundleItemsResponse> {
        let path = format!(
            "blog/{blog_name}/bundle/{bundle_id}/post/?full_data={full_data}&limit={limit}&for_owner={for_owner}&comments_limit={comments_limit}&reply_limit={reply_limit}"
        );

        let response = self.get_request(&path).await?;
        let response = self.handle_response(&path, response).await?;

        self.parse_json(response).await
    }
}
