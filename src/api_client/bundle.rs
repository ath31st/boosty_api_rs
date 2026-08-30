use crate::{
    api_client::{ApiClient, DEFAULT_PAGE_SIZE},
    error::{ApiError, ResultApi},
    model::{BundleItemsResponse, BundleQuery, BundlesResponse, Post},
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

    /// Get posts from a bundle with client-side pagination.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `bundle_id`: bundle identifier.
    /// - `limit`: total number of posts to fetch.
    /// - `page_size`: posts per page (defaults to 20).
    pub async fn get_bundle_posts(
        &self,
        blog_name: &str,
        bundle_id: &str,
        limit: usize,
        page_size: Option<usize>,
    ) -> ResultApi<Vec<Post>> {
        let page_size = page_size.unwrap_or(DEFAULT_PAGE_SIZE);

        let mut all_posts = Vec::new();
        let mut offset = None;

        loop {
            let current_limit = page_size.min(limit.saturating_sub(all_posts.len()));
            if current_limit == 0 {
                break;
            }

            let query = BundleQuery {
                full_data: Some(true),
                limit: Some(current_limit as u32),
                for_owner: Some(false),
                comments_limit: Some(0),
                reply_limit: Some(0),
                offset,
            };

            let response = self.get_bundle(blog_name, bundle_id, &query).await?;
            let page_len = response.data.bundle_items.len();
            let next_offset = response.extra.offset;

            all_posts.extend(response.data.bundle_items.into_iter().map(|item| item.post));

            if response.extra.is_last || all_posts.len() >= limit || page_len == 0 {
                break;
            }

            if offset == Some(next_offset) {
                break;
            }
            offset = Some(next_offset);
        }

        Ok(all_posts)
    }
}
