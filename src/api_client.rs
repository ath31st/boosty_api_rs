mod comment;
mod post;
mod showcase;
mod subscription_level;
mod target;
mod user;

use crate::auth_provider::AuthProvider;
use crate::error::{ApiError, ResultApi, ResultAuth};
use reqwest::header::{ACCEPT, CACHE_CONTROL, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Response, multipart};

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
///     // Or use refresh token + device ID:
///     // api_client.set_refresh_token_and_device_id("your-refresh-token", "your-device-id").await?;
///
///     let post = api_client.get_post("some-blog-name", "post-id").await?;
///     println!("{:#?}", post);
///
///     let targets = api_client.get_blog_targets("some-blog-name").await?;
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

    /// Internal: perform a GET request to given API path, applying auth header.
    ///
    /// # Parameters
    ///
    /// - `path`: relative path under `/v1/`, e.g. `"blog/{}/post/{}"`.
    ///
    /// # Returns
    ///
    /// On success, returns `reqwest::Response`. On network error, returns `ApiError::HttpRequest`.
    async fn get_request(&self, path: &str) -> ResultApi<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        let url = format!("{}/v1/{}", self.base_url, path);
        self.client
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(ApiError::HttpRequest)
    }

    /// Internal: perform a POST request with optional form or JSON body.
    ///
    /// Automatically applies authentication headers and prepends the base URL (`/v1/` prefix).
    ///
    /// # Parameters
    ///
    /// - `path`: relative API path under `/v1/`.
    /// - `body`: an object that can be serialized either as JSON or `application/x-www-form-urlencoded`.
    /// - `as_form`: if `true`, serialize body as `x-www-form-urlencoded`; otherwise, serialize as JSON.
    ///
    /// # Returns
    ///
    /// On success, returns a `reqwest::Response`.  
    /// On network failure, returns [`ApiError::HttpRequest`].
    async fn post_request<T: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
        as_form: bool,
    ) -> ResultApi<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        let url = format!("{}/v1/{}", self.base_url, path);

        let builder = self.client.post(&url).headers(headers);

        let request = if as_form {
            builder.form(body)
        } else {
            builder.json(body)
        };

        request.send().await.map_err(ApiError::HttpRequest)
    }

    /// Internal: perform a POST request with multipart form.
    ///
    /// Automatically applies authentication headers and prepends the base URL (`/v1/` prefix).
    ///
    /// # Parameters
    ///
    /// - `path`: relative API path under `/v1/`.
    /// - `form`: a multipart form.
    ///
    /// # Returns
    ///
    /// On success, returns a `reqwest::Response`.  
    /// On network failure, returns [`ApiError::HttpRequest`].
    async fn post_multipart(&self, path: &str, form: multipart::Form) -> ResultApi<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        headers.remove("Content-Type");

        let url = format!("{}/v1/{}", self.base_url, path);

        let request = self.client.post(&url).headers(headers).multipart(form);

        request.send().await.map_err(ApiError::HttpRequest)
    }

    /// Internal: perform a DELETE request to the given API path.
    ///
    /// Automatically applies authentication headers and prepends the base URL (`/v1/` prefix).
    ///
    /// # Parameters
    ///
    /// - `path`: relative API path under `/v1/`.
    ///
    /// # Returns
    ///
    /// On success, returns a `reqwest::Response`.  
    /// On network failure, returns [`ApiError::HttpRequest`].
    async fn delete_request(&self, path: &str) -> ResultApi<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        let url = format!("{}/v1/{}", self.base_url, path);

        self.client
            .delete(&url)
            .headers(headers)
            .send()
            .await
            .map_err(ApiError::HttpRequest)
    }

    /// Internal: perform a PUT request with optional form or JSON body.
    ///
    /// Automatically applies authentication headers and prepends the base URL (`/v1/` prefix).
    ///
    /// # Parameters
    ///
    /// - `path`: relative API path under `/v1/`.
    /// - `body`: object to serialize either as JSON or `application/x-www-form-urlencoded`.
    /// - `as_form`: if `true`, serialize body as `x-www-form-urlencoded`; otherwise, serialize as JSON.
    ///
    /// # Returns
    ///
    /// On success, returns a `reqwest::Response`.  
    /// On network failure, returns [`ApiError::HttpRequest`].
    async fn put_request<T: serde::Serialize + ?Sized>(
        &self,
        path: &str,
        body: &T,
        as_form: bool,
    ) -> ResultApi<Response> {
        let mut headers = self.headers.clone();
        self.auth_provider.apply_auth_header(&mut headers).await?;

        let url = format!("{}/v1/{}", self.base_url, path);

        let builder = self.client.put(&url).headers(headers);

        let request = if as_form {
            builder.form(body)
        } else {
            builder.json(body)
        };

        request.send().await.map_err(ApiError::HttpRequest)
    }
}
