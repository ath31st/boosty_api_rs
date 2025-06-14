use crate::api_response::Post;
use anyhow::{Context, Result};
use reqwest::header::{ACCEPT, AUTHORIZATION, CACHE_CONTROL, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Response};
use serde_json::{Value, from_value};

pub struct ApiClient {
    base_url: String,
    client: Client,
    headers: HeaderMap,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("roosty-downloader/0.1"),
        );
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert("DNT", HeaderValue::from_static("1"));

        Self {
            base_url: base_url.into(),
            client: Client::new(),
            headers,
        }
    }

    pub fn set_bearer_token(&mut self, access_token: &str) {
        let value = HeaderValue::from_str(&format!("Bearer {}", access_token))
            .expect("Invalid token format");
        self.headers.insert(AUTHORIZATION, value);
    }

    pub fn headers_as_map(&self) -> std::collections::HashMap<String, String> {
        self.headers
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|value| (k.to_string(), value.to_string()))
            })
            .collect()
    }

    async fn get_request(&self, path: &str) -> Result<Response> {
        let url = format!("{}/v1/{}", self.base_url, path);
        self.client
            .get(&url)
            .headers(self.headers.clone())
            .send()
            .await
            .with_context(|| format!("Failed to send GET request to '{}'", url))
    }

    pub async fn fetch_post(&self, blog_name: &str, post_id: &str) -> Result<Post> {
        let path = format!("blog/{}/post/{}", blog_name, post_id);
        let response = self
            .get_request(&path)
            .await
            .with_context(|| format!("Failed to get post from path '{}'", path))?;

        let parsed = response
            .json::<Post>()
            .await
            .with_context(|| "Failed to deserialize Post")?;
        Ok(parsed)
    }

    pub async fn fetch_posts(&self, blog_name: &str, limit: i32) -> Result<Vec<Post>> {
        let path = format!("blog/{}/post/?limit={}", blog_name, limit);
        let response = self
            .get_request(&path)
            .await
            .with_context(|| format!("Failed to get posts from path '{}'", path))?;

        let json: Value = response
            .json()
            .await
            .with_context(|| "Failed to parse response body as JSON")?;

        let parsed = from_value(json["data"].clone())
            .with_context(|| "Failed to deserialize 'data' field into Vec<Post>")?;

        Ok(parsed)
    }
}
