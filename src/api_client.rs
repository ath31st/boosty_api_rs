use crate::api_response::Post;
use crate::auth_provider::AuthProvider;
use crate::error::{ApiError, ResultApi, ResultAuth};
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, CACHE_CONTROL, USER_AGENT};
use reqwest::{Client, Response, StatusCode};
use serde_json::{from_value, Value};

pub struct ApiClient {
    base_url: String,
    client: Client,
    headers: HeaderMap,
    auth_provider: AuthProvider,
}

impl ApiClient {
    pub fn new(client: Client, base_url: impl Into<String> + Clone) -> Self {
        let base_url = base_url.into();
        let headers = Self::prepare_headers();

        let auth_provider = AuthProvider::new(client.clone(), base_url.clone());

        Self {
            base_url,
            client,
            headers,
            auth_provider,
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

    pub async fn set_bearer_token(&self, access_token: &str) -> ResultAuth<()> {
        self.auth_provider
            .set_access_token_only(access_token.to_string())
            .await
    }

    pub async fn set_refresh_token_and_device_id(
        &self,
        refresh_token: &str,
        device_id: &str,
    ) -> ResultAuth<()> {
        self.auth_provider
            .set_refresh_token_and_device_id(refresh_token.to_string(), device_id.to_string())
            .await
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

    async fn get_request(&self, path: &str) -> ResultApi<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        let url = format!("{}/v1/{}", self.base_url, path);
        self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|err| ApiError::HttpRequest(err))
    }

    async fn fetch_post_once(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let path = format!("blog/{}/post/{}", blog_name, post_id);
        let response = self.get_request(&path).await?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized);
        }
        if !status.is_success() {
            let endpoint = path.clone();
            return Err(ApiError::HttpStatus { status, endpoint });
        }
        let parsed = response.json::<Post>().await.map_err(ApiError::JsonParse)?;
        Ok(parsed)
    }

    pub async fn fetch_post(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let mut post = self.fetch_post_once(blog_name, post_id).await?;
        if post.not_available() && self.auth_provider.has_refresh_and_device_id().await {
            self.auth_provider.force_refresh().await?;
            post = self.fetch_post_once(blog_name, post_id).await?;
        }
        Ok(post)
    }

    async fn fetch_posts_once(&self, blog_name: &str, limit: i32) -> ResultApi<Vec<Post>> {
        let path = format!("blog/{}/post/?limit={}", blog_name, limit);
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }

    pub async fn fetch_posts(&self, blog_name: &str, limit: i32) -> ResultApi<Vec<Post>> {
        let mut posts = self.fetch_posts_once(blog_name, limit).await?;
        if posts.iter().any(|p| p.not_available())
            && self.auth_provider.has_refresh_and_device_id().await
        {
            self.auth_provider
                .force_refresh()
                .await
                .map_err(ApiError::Auth)?;

            posts = self.fetch_posts_once(blog_name, limit).await?;
        }
        Ok(posts)
    }
}
