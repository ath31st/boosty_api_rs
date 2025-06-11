use crate::api_response::ApiResponse;
use crate::headers::Headers;
use anyhow::Result;
use reqwest::{Client, Response};
use serde_json::{Value, from_value};

pub struct ApiClient {
    base_url: String,
    client: Client,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: Client::new(),
        }
    }

    async fn get_request(&self, path: &str, headers: Option<&Headers>) -> Result<Response> {
        let url = format!("{}/v1/{}", self.base_url, path);
        let mut builder = self.client.get(&url);

        if let Some(h) = headers {
            builder = builder.headers(h.map.clone());
        }

        let response = builder.send().await?;
        Ok(response)
    }

    pub async fn fetch_post(
        &self,
        blog_name: &str,
        post_id: &str,
        headers: Option<&Headers>,
    ) -> Result<ApiResponse> {
        let path = format!("blog/{}/post/{}", blog_name, post_id);
        let response = self.get_request(&path, headers).await?;
        let parsed = response.json::<ApiResponse>().await?;
        Ok(parsed)
    }

    pub async fn fetch_posts(
        &self,
        blog_name: &str,
        limit: i32,
        headers: Option<&Headers>,
    ) -> Result<Vec<ApiResponse>> {
        let path = format!("blog/{}/post/?limit={}", blog_name, limit);
        let response = self.get_request(&path, headers).await?;
        let json: Value = response.json().await?;
        let parsed = from_value(json["data"].clone())?;
        Ok(parsed)
    }
}
