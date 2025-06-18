use crate::api_response::Post;
use crate::auth_provider::AuthProvider;
use anyhow::{bail, Context, Result};
use reqwest::header::{
    HeaderMap, HeaderValue, ACCEPT, CACHE_CONTROL, COOKIE, USER_AGENT,
};
use reqwest::{Client, Response, StatusCode};
use serde_json::{from_value, Value};

pub struct ApiClient {
    base_url: String,
    client: Client,
    headers: HeaderMap,
    auth_provider: AuthProvider,
}

impl ApiClient {
    pub fn new(base_url: impl Into<String> + Clone) -> Self {
        let base_url = base_url.into();
        let headers = Self::prepare_headers();

        Self {
            base_url: base_url.clone(),
            client: Client::new(),
            headers,
            auth_provider: AuthProvider::new(base_url),
        }
    }

    fn prepare_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/110.0.0.0 Safari/537.36"),
        );
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));
        headers.insert("DNT", HeaderValue::from_static("1"));
        headers
    }

    pub fn set_bearer_token(&mut self, access_token: &str) -> Result<()> {
        self.auth_provider
            .set_access_token_only(access_token.to_string())
    }

    pub fn set_refresh_token_and_device_id(
        &mut self,
        refresh_token: &str,
        device_id: &str,
    ) -> Result<()> {
        self.auth_provider
            .set_refresh_token_and_device_id(refresh_token.to_string(), device_id.to_string())
    }

    pub fn append_cookie(&mut self, key: &str, value: &str) {
        use std::fmt::Write;

        let mut existing = self
            .headers
            .get(COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if !existing.is_empty() {
            existing.push_str("; ");
        }

        write!(&mut existing, "{}={}", key, value).unwrap();

        self.headers
            .insert(COOKIE, HeaderValue::from_str(&existing).unwrap());
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

    async fn get_request(&mut self, path: &str) -> Result<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        let url = format!("{}/v1/{}", self.base_url, path);
        self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .with_context(|| format!("Failed to send GET request to '{}'", url))
    }

    pub async fn fetch_post(&mut self, blog_name: &str, post_id: &str) -> Result<Post> {
        let path = format!("blog/{}/post/{}", blog_name, post_id);
        let response = self
            .get_request(&path)
            .await
            .with_context(|| format!("Failed to get post from path '{}'", path))?;

        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            bail!("Unauthorized (401): Invalid or missing token");
        } else if !status.is_success() {
            bail!("HTTP error {} when fetching post", status);
        }

        let parsed = response
            .json::<Post>()
            .await
            .with_context(|| "Failed to deserialize Post")?;
        Ok(parsed)
    }

    pub async fn fetch_posts(&mut self, blog_name: &str, limit: i32) -> Result<Vec<Post>> {
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
