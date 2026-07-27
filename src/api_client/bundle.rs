use crate::{
    api_client::ApiClient,
    error::{ApiError, ResultApi},
    model::{BundleItemsResponse, BundleQuery, BundlesResponse},
};

impl ApiClient {
    /// Get all bundles for a blog.
    pub async fn get_bundles(&self, blog_name: &str) -> ResultApi<BundlesResponse> {
        let path = format!("blog/{blog_name}/bundle/");
        self.get_json(&path, &[]).await
    }

    /// Get posts within a specific bundle.
    pub async fn get_bundle(
        &self,
        blog_name: &str,
        bundle_id: &str,
        query: &BundleQuery,
    ) -> ResultApi<BundleItemsResponse> {
        let query_string = serde_urlencoded::to_string(query).map_err(ApiError::Serialization)?;

        let path = format!("blog/{blog_name}/bundle/{bundle_id}/post/?{query_string}");
        self.get_json(&path, &[]).await
    }
}
