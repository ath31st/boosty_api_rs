use crate::api_client::ApiClient;
use crate::error::ResultApi;
use crate::model::{NewTarget, Target, TargetResponse, TargetType, UpdateTarget};

impl ApiClient {
    /// Get all targets for a blog.
    pub async fn get_blog_targets(&self, blog_name: &str) -> ResultApi<TargetResponse> {
        let path = format!("target/{blog_name}/");
        self.get_json(&path, &[]).await
    }

    /// Create a new target for a blog.
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

        self.post_form_json(path, &form).await
    }

    /// Delete a target by its ID.
    pub async fn delete_blog_target(&self, target_id: u64) -> ResultApi<()> {
        let path = format!("target/{}", target_id);
        self.delete_ok(&path).await
    }

    /// Update an existing target by its ID.
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
        self.put_form_json(&path, &form).await
    }
}
