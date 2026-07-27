use crate::error::{AuthError, ResultAuth};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Tokens returned after a successful refresh.
#[derive(Debug, Clone, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

/// Internal state for authentication.
#[derive(Debug)]
struct AuthState {
    /// Static access token, if set via `set_access_token_only`.
    static_access_token: Option<String>,
    /// Device ID for refresh flow.
    device_id: Option<String>,
    /// Refresh token for refresh flow.
    refresh_token: Option<String>,
    /// Current valid access token obtained from refresh flow.
    access_token: Option<String>,
}

/// Provider managing authentication: either static token or refresh-token flow.
///
/// Token refresh is **never** performed automatically. Call [`AuthProvider::refresh_tokens`]
/// explicitly when you need to obtain or renew an access token.
#[derive(Clone, Debug)]
pub struct AuthProvider {
    client: Client,
    base_url: String,
    state: Arc<Mutex<AuthState>>,
}

impl AuthProvider {
    /// Create a new AuthProvider with given reqwest `Client` and base URL.
    ///
    /// Initially no credentials are set.
    pub fn new(client: Client, base_url: impl Into<String>) -> Self {
        let state = AuthState {
            static_access_token: None,
            device_id: None,
            refresh_token: None,
            access_token: None,
        };
        Self {
            client,
            base_url: base_url.into(),
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Apply authorization header to given headers map.
    ///
    /// Uses the currently known access token (static or obtained via prior
    /// [`refresh_tokens`](Self::refresh_tokens) call). Does **not** trigger any
    /// HTTP requests — if no token is available, the header is simply not set.
    pub async fn apply_auth_header(&self, headers: &mut HeaderMap) -> ResultAuth<()> {
        let st = self.state.lock().await;

        let token = st
            .static_access_token
            .as_deref()
            .or(st.access_token.as_deref());

        if let Some(tok) = token {
            let hv = HeaderValue::from_str(&format!("Bearer {tok}"))
                .map_err(|_| AuthError::InvalidTokenFormat)?;
            headers.insert(AUTHORIZATION, hv);
        }

        Ok(())
    }

    /// Set only static access token, disabling refresh flow.
    ///
    /// If `access` is empty, returns `AuthError::EmptyAccessToken`.
    pub async fn set_access_token_only(&self, access: String) -> ResultAuth<()> {
        if access.is_empty() {
            return Err(AuthError::EmptyAccessToken);
        }
        let mut st = self.state.lock().await;
        st.static_access_token = Some(access);
        st.device_id = None;
        st.refresh_token = None;
        st.access_token = None;
        Ok(())
    }

    /// Set refresh token and device ID for refresh flow, disabling static token.
    ///
    /// This does **not** perform a refresh — call [`refresh_tokens`](Self::refresh_tokens)
    /// afterwards to obtain an access token.
    ///
    /// Returns error if either is empty.
    pub async fn set_refresh_token_and_device_id(
        &self,
        refresh: String,
        device_id: String,
    ) -> ResultAuth<()> {
        if refresh.is_empty() {
            return Err(AuthError::EmptyRefreshToken);
        }
        if device_id.is_empty() {
            return Err(AuthError::EmptyDeviceId);
        }
        let mut st = self.state.lock().await;
        st.static_access_token = None;
        st.refresh_token = Some(refresh);
        st.device_id = Some(device_id);
        st.access_token = None;
        Ok(())
    }

    /// Perform an explicit token refresh via the OAuth endpoint.
    ///
    /// Requires that refresh token and device ID were previously set via
    /// [`set_refresh_token_and_device_id`](Self::set_refresh_token_and_device_id).
    ///
    /// On success, updates the internal access and refresh tokens and returns
    /// a [`TokenPair`] so the caller can persist the new credentials.
    pub async fn refresh_tokens(&self) -> ResultAuth<TokenPair> {
        let mut st = self.state.lock().await;

        let refresh_token = st
            .refresh_token
            .clone()
            .ok_or(AuthError::MissingCredentials)?;
        let device_id = st.device_id.clone().ok_or(AuthError::MissingCredentials)?;

        let url = format!("{}/oauth/token/", self.base_url);
        let params = [
            ("device_id", device_id.as_str()),
            ("device_os", "web"),
            ("grant_type", "refresh_token"),
            ("refresh_token", &refresh_token),
        ];

        let resp = self
            .client
            .post(&url)
            .form(&params)
            .send()
            .await
            .map_err(AuthError::HttpRequest)?;

        if resp.status() != StatusCode::OK {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(AuthError::HttpStatus { status, body });
        }

        let data: TokenPair = resp.json().await.map_err(AuthError::HttpRequest)?;

        st.access_token = Some(data.access_token.clone());
        st.refresh_token = Some(data.refresh_token.clone());

        Ok(TokenPair {
            access_token: data.access_token,
            refresh_token: data.refresh_token,
            expires_in: data.expires_in,
        })
    }

    /// Clear static access token (disables static token auth).
    pub async fn clear_access_token(&self) {
        let mut st = self.state.lock().await;
        st.static_access_token = None;
    }

    /// Clear refresh token and device ID (disables refresh flow).
    pub async fn clear_refresh_and_device_id(&self) {
        let mut st = self.state.lock().await;
        st.refresh_token = None;
        st.device_id = None;
        st.access_token = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;
    use reqwest::Client;
    use reqwest::header::{AUTHORIZATION, HeaderMap};

    fn make_provider(server_url: &str) -> AuthProvider {
        AuthProvider::new(Client::new(), server_url)
    }

    #[tokio::test]
    async fn test_set_access_token_only_and_apply_auth_header() {
        let provider = make_provider("http://localhost");
        provider
            .set_access_token_only("my_token".into())
            .await
            .unwrap();

        let mut headers = HeaderMap::new();
        provider.apply_auth_header(&mut headers).await.unwrap();

        assert_eq!(headers.get(AUTHORIZATION).unwrap(), "Bearer my_token");
    }

    #[tokio::test]
    async fn test_refresh_tokens_and_apply_auth_header() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/oauth/token/")
            .match_body(mockito::Matcher::AllOf(vec![
                mockito::Matcher::UrlEncoded("grant_type".into(), "refresh_token".into()),
                mockito::Matcher::UrlEncoded("device_id".into(), "abc123".into()),
                mockito::Matcher::UrlEncoded("refresh_token".into(), "ref123".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{
            "access_token": "new_access",
            "refresh_token": "new_refresh",
            "expires_in": 3600
        }"#,
            )
            .create_async()
            .await;

        let provider = make_provider(&server.url());
        provider
            .set_refresh_token_and_device_id("ref123".into(), "abc123".into())
            .await
            .unwrap();

        let pair = provider.refresh_tokens().await.unwrap();
        assert_eq!(pair.access_token, "new_access");
        assert_eq!(pair.refresh_token, "new_refresh");
        assert_eq!(pair.expires_in, 3600);

        let mut headers = HeaderMap::new();
        provider.apply_auth_header(&mut headers).await.unwrap();
        assert_eq!(headers.get(AUTHORIZATION).unwrap(), "Bearer new_access");

        mock.assert_async().await;
    }

    #[tokio::test]
    async fn test_apply_auth_header_without_refresh_no_header() {
        let provider = make_provider("http://localhost");
        provider
            .set_refresh_token_and_device_id("ref".into(), "dev".into())
            .await
            .unwrap();

        let mut headers = HeaderMap::new();
        provider.apply_auth_header(&mut headers).await.unwrap();

        assert!(headers.get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn test_clear_access_token() {
        let provider = make_provider("http://localhost");
        provider
            .set_access_token_only("my_token".into())
            .await
            .unwrap();
        provider.clear_access_token().await;

        let mut headers = HeaderMap::new();

        provider.apply_auth_header(&mut headers).await.unwrap();

        assert!(headers.get(AUTHORIZATION).is_none());
    }

    #[tokio::test]
    async fn test_clear_refresh_and_device_id() {
        let provider = make_provider("http://localhost");
        provider
            .set_refresh_token_and_device_id("my_token".into(), "my_device_id".into())
            .await
            .unwrap();
        provider.clear_refresh_and_device_id().await;

        let mut headers = HeaderMap::new();

        provider.apply_auth_header(&mut headers).await.unwrap();

        assert!(headers.get(AUTHORIZATION).is_none());
    }
}
