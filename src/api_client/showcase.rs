use crate::{api_client::{ApiClient, QueryParams}, error::ResultApi, model::ShowcaseResponse};

impl ApiClient {
    /// Get blog showcase.
    pub async fn get_showcase(
        &self,
        blog_name: &str,
        limit: Option<u32>,
        only_visible: Option<bool>,
        offset: Option<u32>,
    ) -> ResultApi<ShowcaseResponse> {
        let path = format!("blog/{blog_name}/showcase/");

        let query = QueryParams::new()
            .push("offset", offset)
            .push("limit", limit)
            .push("only_visible", only_visible);

        self.get_json(&path, &query.as_slice()).await
    }

    /// Change blog showcase status.
    pub async fn change_showcase_status(&self, blog_name: &str, status: bool) -> ResultApi<()> {
        let path = format!("blog/{blog_name}/showcase/status/");
        self.put_form_ok(&path, &serde_json::json!({"is_enabled": status})).await
    }
}
