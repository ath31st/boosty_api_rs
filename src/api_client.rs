use crate::api_response::{Post, SubscriptionLevel, SubscriptionsResponse, Target};
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
///     let targets = api_client.get_blog_targets("some-blog-name").await?;
///     println!("{:#?}", targets);
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

    /// Fetch a single post once, without automatic retry on "not available" or HTTP 401.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or name of the blog.
    /// - `post_id`: identifier of the post.
    ///
    /// # Returns
    ///
    /// On success, returns the `Post` object.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpStatus` for other non-success HTTP statuses, with status and endpoint info.
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParseDetailed` if the response body cannot be parsed into a `Post`.
    async fn fetch_post_once(&self, blog_name: &str, post_id: &str) -> ResultApi<Post> {
        let path = format!("blog/{blog_name}/post/{post_id}");
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

    /// Fetch a single post, with automatic retry on "not available" if refresh-token flow is configured.
    ///
    /// If the first attempt yields a `Post` where `post.not_available()` is true, and refresh-token
    /// flow is enabled, forces a token refresh and retries once.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: blog identifier/name.
    /// - `post_id`: post identifier.
    ///
    /// # Returns
    ///
    /// On success, returns the fetched `Post`.
    ///
    /// # Errors
    ///
    ///  - `ApiError::Unauthorized`
    ///  - `ApiError::HttpStatus`
    ///  - `ApiError::JsonParseDetailed`
    ///  - Errors from token refresh (`ApiError::Auth`)
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
    /// # Returns
    ///
    /// On success, returns a vector of `Post` objects.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` field cannot be deserialized into a vector of `Post`.
    async fn fetch_posts_once(&self, blog_name: &str, limit: i32) -> ResultApi<Vec<Post>> {
        let path = format!("blog/{blog_name}/post/?limit={limit}");
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }

    /// Fetch multiple posts, with automatic retry if any post is marked "not available" and refresh-token flow is set.
    ///
    /// Checks if any fetched post returns `not_available() == true`. If so, and if refresh-token authentication is configured,
    /// forces a token refresh and retries the request once.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or name of the blog.
    /// - `limit`: maximum number of posts to fetch.
    ///
    /// # Returns
    ///
    /// On success, returns a vector of `Post` objects.
    /// On failure, returns an `ApiError`.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the HTTP request fails.
    /// - `ApiError::Auth` if refreshing the authentication token fails.
    /// - `ApiError::JsonParse` if the response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if JSON cannot be deserialized into `Post` objects.
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

    /// Fetch all targets for a blog, optionally handling HTTP errors and JSON parsing.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: identifier or slug of the blog whose targets should be retrieved.
    ///
    /// # Returns
    ///
    /// A `Vec<Target>` decoded from the `"data"` field of the JSON response.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response body cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` array cannot be deserialized into `Target` structs.
    pub async fn get_blog_targets(&self, blog_name: &str) -> ResultApi<Vec<Target>> {
        let path = format!("target/{blog_name}/");
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }

    /// Fetch subscription levels for a blog, with optional inclusion of the free level.
    ///
    /// If `show_free_level` is `Some(true)`, appends `?show_free_level=true` to the URL.
    ///
    /// # Parameters
    ///
    /// - `blog_name`: the identifier or name of the blog.
    /// - `show_free_level`: when `Some(true)`, include the free subscription level in results.
    ///
    /// # Returns
    ///
    /// On success, returns a `Vec<SubscriptionLevel>` parsed from the `"data"` array of the response.
    ///
    /// # Errors
    ///
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the `"data"` field cannot be deserialized into subscription level items.
    pub async fn get_blog_subscription_levels(
        &self,
        blog_name: &str,
        show_free_level: Option<bool>,
    ) -> ResultApi<Vec<SubscriptionLevel>> {
        let mut path = format!("blog/{blog_name}/subscription_level/");
        if let Some(flag) = show_free_level {
            path.push_str(&format!("?show_free_level={flag}"));
        }
        let response = self.get_request(&path).await?;

        let json: Value = response.json().await.map_err(ApiError::JsonParse)?;

        let parsed = from_value(json["data"].clone()).map_err(ApiError::Deserialization)?;

        Ok(parsed)
    }

    /// Fetch the current user's subscriptions, with optional pagination and follow filter.
    ///
    /// Sends a GET request to `/v1/user/subscriptions` with query parameters:
    /// - `limit`: maximum number of items to return (default server-side if omitted).
    /// - `with_follow`: when `Some(true)`, include subscriptions to followed blogs.
    ///
    /// # Parameters
    ///
    /// - `limit`: optional maximum number of subscriptions to fetch.
    /// - `with_follow`: optional flag to include subscriptions on followed blogs.
    ///
    /// # Returns
    ///
    /// On success, returns a `SubscriptionsResponse` containing the list of subscriptions and pagination info.
    ///
    /// # Errors
    ///
    /// - `ApiError::Unauthorized` if the HTTP status is 401 Unauthorized.
    /// - `ApiError::HttpRequest` if the network request fails.
    /// - `ApiError::JsonParse` if the HTTP response cannot be parsed as JSON.
    /// - `ApiError::Deserialization` if the JSON cannot be deserialized into `SubscriptionsResponse`.
    pub async fn get_user_subscriptions(
        &self,
        limit: Option<u32>,
        with_follow: Option<bool>,
    ) -> ResultApi<SubscriptionsResponse> {
        let mut path = "user/subscriptions".to_string();
        let mut params = Vec::new();
        if let Some(l) = limit {
            params.push(format!("limit={l}"));
        }
        if let Some(f) = with_follow {
            params.push(format!("with_follow={f}"));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        let response = self.get_request(&path).await?;
        let status = response.status();
        if status == StatusCode::UNAUTHORIZED {
            return Err(ApiError::Unauthorized);
        }

        let subs = response
            .json::<SubscriptionsResponse>()
            .await
            .map_err(ApiError::JsonParse)?;

        Ok(subs)
    }
}
