use crate::api_response::Post;
use crate::auth_provider::AuthProvider;
use crate::error::{ApiError, ResultApi, ResultAuth};
use reqwest::header::{ACCEPT, CACHE_CONTROL, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::{Client, Response, StatusCode};
use serde_json::{Value, from_value};

/// Client for interacting with Boosty API.
///
/// Handles base URL, common headers, and delegates authentication to `AuthProvider`.
/// Provides methods to fetch a single post or multiple posts.
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
///     let post = api_client.fetch_post("some-blog-name", "post-id").await?;
///     println!("{:#?}", post);
///
///     Ok(())
/// }
/// ```
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
    /// This disables any previously set static bearer token.
    ///
    /// # Parameters
    ///
    /// - `refresh_token`: refresh token string; must be non-empty.
    /// - `device_id`: device identifier; must be non-empty.
    ///
    /// # Errors
    ///
    /// Returns `AuthError::EmptyRefreshToken` or `AuthError::EmptyDeviceId` if any argument is empty.
    pub async fn set_refresh_token_and_device_id(
        &self,
        refresh_token: &str,
        device_id: &str,
    ) -> ResultAuth<()> {
        self.auth_provider
            .set_refresh_token_and_device_id(refresh_token.to_string(), device_id.to_string())
            .await
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
            .map_err(|err| ApiError::HttpRequest(err))
    }

    /// Fetch a single post once, without automatic retry on "not available" or 401.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `post_id`: post identifier.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if HTTP 401 is returned.
    /// - `ApiError::HttpStatus` for other non-success statuses.
    /// - `ApiError::JsonParseDetailed` if response body cannot be parsed into `Post`.
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

        let body = response.text().await?;
        let parsed =
            serde_json::from_str::<Post>(&body).map_err(|e| ApiError::JsonParseDetailed {
                error: e.to_string(),
            })?;

        Ok(parsed)
    }

    /// Fetch a single post, with automatic retry on "not available" if refresh flow is configured.
    ///
    /// If the first attempt yields a `Post` where `post.not_available()` is true, and refresh-token flow is set,
    /// forces a token refresh and retries once.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `post_id`: post identifier.
    ///
    /// # Returns
    ///
    /// On success, returns `Post`. On failure, returns an `ApiError`.
    pub async fn fetch_post(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let mut post = self.fetch_post_once(blog_name, post_id).await?;
        if post.not_available() && self.auth_provider.has_refresh_and_device_id().await {
            self.auth_provider.force_refresh().await?;
            post = self.fetch_post_once(blog_name, post_id).await?;
        }
        Ok(post)
    }

    /// Fetch multiple posts once, parsing JSON array under `"data"` key.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `limit`: maximum number of posts to fetch.
    ///
    /// # Errors
    ///
    /// Returns `ApiError::JsonParse` on failure to parse intermediate JSON, or
    /// `ApiError::Deserialization` if converting `"data"` array to `Vec<Post>` fails.
    async fn fetch_posts_once(&self, blog_name: &str, limit: i32) -> ResultApi<Vec<Post>> {
        let path = format!("blog/{}/post/?limit={}", blog_name, limit);
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }

    /// Fetch multiple posts, with automatic retry on "not available" if any post has `not_available() == true`
    /// and refresh-token flow is configured.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `limit`: maximum number of posts to fetch.
    ///
    /// # Returns
    ///
    /// On success, returns `Vec<Post>`. On failure, returns an `ApiError`.
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
