mod bundle;
mod comment;
mod post;
mod showcase;
mod subscription_level;
mod target;
mod user;

use std::fmt::Display;

use crate::auth_provider::{AuthProvider, TokenPair};
use crate::error::{ApiError, ResultApi, ResultAuth};
use reqwest::header::{ACCEPT, CACHE_CONTROL, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Response, multipart};

/// Builder for optional query parameters that owns its string values.
#[derive(Default)]
pub(crate) struct QueryParams(Vec<(String, String)>);

impl QueryParams {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a parameter. If `value` is `None`, the parameter is skipped.
    pub fn push(mut self, key: &str, value: Option<impl Display>) -> Self {
        if let Some(v) = value {
            self.0.push((key.to_string(), v.to_string()));
        }
        self
    }

    pub fn as_slice(&self) -> Vec<(&str, &str)> {
        self.0
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }
}

/// Default number of posts to fetch per page.
const DEFAULT_PAGE_SIZE: usize = 20;

/// Client for interacting with Boosty API.
///
/// Handles base URL, common headers, and delegates authentication to `AuthProvider`.
/// Provides methods to get a single post or multiple posts.
///
/// # Examples
///
/// ```rust,no_run
/// use boosty_api::api_client::ApiClient;
/// use reqwest::Client;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = Client::new();
///     let base_url = "https://api.example.com";
///     let api_client = ApiClient::new(client, base_url);
///
///     // Use static bearer token:
///     api_client.set_bearer_token("your-access-token").await?;
///
///     // Or use refresh token + device ID, then explicitly refresh:
///     // api_client.set_refresh_token_and_device_id("your-refresh-token", "your-device-id").await?;
///     // let tokens = api_client.refresh_tokens().await?;
///
///     let post = api_client.get_post("blog_name", "post_id").await?;
///     println!("{:#?}", post);
///
///     let targets = api_client.get_blog_targets("blog_name").await?;
///     println!("{:#?}", targets);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ApiClient {
    base_url: String,
    client: Client,
    headers: HeaderMap,
    auth_provider: AuthProvider,
}

impl ApiClient {
    /// Creates a new `ApiClient`.
    ///
    /// # Parameters
    ///
    /// - `client`: a configured `reqwest::Client` for HTTP requests.
    /// - `base_url`: base URL of the Boosty API (e.g., `"https://api.example.com"`).
    ///
    /// # Returns
    ///
    /// A new `ApiClient` with default headers prepared and an internal `AuthProvider`.
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

    /// Prepare default headers for all requests:
    /// - `Accept: application/json`
    /// - `User-Agent: ...`
    /// - `Cache-Control: no-cache`
    /// - `DNT: 1`
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

    /// Set a static bearer token for authentication.
    ///
    /// This disables any previously configured refresh-token flow.
    ///
    /// # Parameters
    ///
    /// - `access_token`: the bearer token string; must be non-empty.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::EmptyAccessToken` if `access_token` is empty.
    pub async fn set_bearer_token(&self, access_token: &str) -> ResultAuth<()> {
        self.auth_provider
            .set_access_token_only(access_token.to_string())
            .await
    }

    /// Set refresh token and device ID for OAuth-like refresh flow.
    ///
    /// Disables any previously set static bearer token.
    ///
    /// # Parameters
    ///
    /// - refresh_token: non-empty refresh token string.
    /// - device_id: non-empty device identifier.
    ///
    /// # Errors
    ///
    /// Returns AuthError::EmptyRefreshToken if refresh_token is empty,
    /// or AuthError::EmptyDeviceId if device_id is empty.
    pub async fn set_refresh_token_and_device_id(
        &self,
        refresh_token: &str,
        device_id: &str,
    ) -> ResultAuth<()> {
        self.auth_provider
            .set_refresh_token_and_device_id(refresh_token.to_string(), device_id.to_string())
            .await
    }

    /// Clear refresh token and device ID (disables refresh flow).
    pub async fn clear_refresh_and_device_id(&self) {
        self.auth_provider.clear_refresh_and_device_id().await
    }

    /// Clear access token (disables static token).
    pub async fn clear_access_token(&self) {
        self.auth_provider.clear_access_token().await
    }

    /// Explicitly refresh the access token using the previously configured
    /// refresh token and device ID.
    ///
    /// Returns a [`TokenPair`] with the new access/refresh tokens so the caller
    /// can persist them. The internal state is also updated, so subsequent API
    /// requests will use the new access token.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::MissingCredentials` if refresh token or device ID are not set.
    pub async fn refresh_tokens(&self) -> ResultAuth<TokenPair> {
        self.auth_provider.refresh_tokens().await
    }

    /// Expose current default headers as a `HashMap<String, String>`.
    ///
    /// Useful for inspecting what headers will be sent without authentication.
    ///
    /// # Returns
    ///
    /// Map of header names to their string values.
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

    // ── Low-level request helpers ───────────────────────────────────

    async fn auth_headers(&self) -> ResultApi<HeaderMap> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;
        Ok(headers)
    }

    fn url(&self, path: &str) -> String {
        format!("{}/v1/{}", self.base_url, path)
    }

    async fn send_request(
        &self,
        path: &str,
        builder: reqwest::RequestBuilder,
    ) -> ResultApi<Response> {
        let response = builder.send().await.map_err(ApiError::HttpRequest)?;
        self.handle_response(path, response).await
    }

    // ── High-level pipeline helpers ──────────────────────────────

    /// GET `path` → check status → deserialize JSON.
    pub(crate) async fn get_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, &str)],
    ) -> ResultApi<T> {
        let headers = self.auth_headers().await?;
        let mut builder = self.client.get(self.url(path)).headers(headers);
        if !query.is_empty() {
            builder = builder.query(query);
        }
        let response = self.send_request(path, builder).await?;
        self.parse_json(response).await
    }

    /// POST form `path` → check status → deserialize JSON.
    pub(crate) async fn post_form_json<
        B: serde::Serialize + ?Sized,
        T: serde::de::DeserializeOwned,
    >(
        &self,
        path: &str,
        body: &B,
    ) -> ResultApi<T> {
        let headers = self.auth_headers().await?;
        let builder = self.client.post(self.url(path)).headers(headers).form(body);
        let response = self.send_request(path, builder).await?;
        self.parse_json(response).await
    }

    /// POST JSON `path` → check status → deserialize JSON.
    #[allow(dead_code)]
    pub(crate) async fn post_json_json<
        B: serde::Serialize + ?Sized,
        T: serde::de::DeserializeOwned,
    >(
        &self,
        path: &str,
        body: &B,
    ) -> ResultApi<T> {
        let headers = self.auth_headers().await?;
        let builder = self.client.post(self.url(path)).headers(headers).json(body);
        let response = self.send_request(path, builder).await?;
        self.parse_json(response).await
    }

    /// POST multipart `path` → check status → deserialize JSON.
    pub(crate) async fn post_multipart_json<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        form: multipart::Form,
    ) -> ResultApi<T> {
        let mut headers = self.auth_headers().await?;
        headers.remove("Content-Type");
        let builder = self
            .client
            .post(self.url(path))
            .headers(headers)
            .multipart(form);
        let response = self.send_request(path, builder).await?;
        self.parse_json(response).await
    }

    /// PUT form `path` → check status → deserialize JSON.
    pub(crate) async fn put_form_json<
        B: serde::Serialize + ?Sized,
        T: serde::de::DeserializeOwned,
    >(
        &self,
        path: &str,
        body: &B,
    ) -> ResultApi<T> {
        let headers = self.auth_headers().await?;
        let builder = self.client.put(self.url(path)).headers(headers).form(body);
        let response = self.send_request(path, builder).await?;
        self.parse_json(response).await
    }

    /// PUT form `path` → check status (ignore body).
    pub(crate) async fn put_form_ok<B: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        body: &B,
    ) -> ResultApi<()> {
        let headers = self.auth_headers().await?;
        let builder = self.client.put(self.url(path)).headers(headers).form(body);
        self.send_request(path, builder).await?;
        Ok(())
    }

    /// DELETE `path` → check status (ignore body).
    pub(crate) async fn delete_ok(&self, path: &str) -> ResultApi<()> {
        let headers = self.auth_headers().await?;
        let builder = self.client.delete(self.url(path)).headers(headers);
        self.send_request(path, builder).await?;
        Ok(())
    }
}
