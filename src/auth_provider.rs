use crate::error::{AuthError, ResultAuth};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Response body for token refresh endpoint.
#[derive(Deserialize)]
struct RefreshResponse {
    /// New access token returned by server.
    access_token: String,
    /// New refresh token returned by server.
    refresh_token: String,
    /// Lifetime of access token in seconds.
    expires_in: i64,
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
    /// Current valid access token from refresh flow.
    access_token: Option<String>,
    /// Expiration instant for `access_token`.
    expires_at: Option<Instant>,
}

/// Provider managing authentication: either static token or refresh-token flow.
#[derive(Clone)]
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
            expires_at: None,
        };
        Self {
            client,
            base_url: base_url.into(),
            state: Arc::new(Mutex::new(state)),
        }
    }

    /// Apply authorization header to given headers map.
    ///
    /// If a static access token is set, uses it. Otherwise, if refresh flow is configured,
    /// obtains (or refreshes) the access token and applies it.
    pub async fn apply_auth_header(&self, headers: &mut HeaderMap) -> ResultAuth<()> {
        // First check static token
        let static_tok_opt = {
            let st = self.state.lock().await;
            st.static_access_token.clone()
        };

        if let Some(tok) = static_tok_opt {
            let hv = HeaderValue::from_str(&format!("Bearer {tok}"))
                .map_err(|_| AuthError::InvalidTokenFormat)?;
            headers.insert(AUTHORIZATION, hv);
            return Ok(());
        }

        // If static not set but refresh+device_id present, use refresh flow
        if self.has_refresh_and_device_id().await {
            let tok = self.get_access_token().await?;
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
        st.expires_at = None;
        Ok(())
    }

    /// Set refresh token and device ID for refresh flow, disabling static token.
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
        st.expires_at = None;
        Ok(())
    }

    /// Get a valid access token, refreshing if needed.
    ///
    /// If static token is set, returns it directly. Otherwise, uses refresh flow.
    /// Returns `AuthError::MissingCredentials` if neither static nor refresh flow configured.
    pub async fn get_access_token(&self) -> ResultAuth<String> {
        let st = self.state.lock().await;
        if let Some(tok) = &st.static_access_token {
            return Ok(tok.clone());
        }
        let refresh = st.refresh_token.clone();
        let device_id = st.device_id.clone();
        drop(st);

        match (refresh, device_id) {
            (Some(_), Some(_)) => {
                let mut st2 = self.state.lock().await;
                // Determine if need to refresh: if no expires_at or close to expiry (<=30s left)
                let need_refresh = match st2.expires_at {
                    Some(exp) => Instant::now() + Duration::from_secs(30) >= exp,
                    None => true,
                };
                if need_refresh {
                    self.refresh_internal(&mut st2).await?;
                }
                // After refresh_internal, access_token must be Some
                Ok(st2.access_token.clone().unwrap())
            }
            _ => Err(AuthError::MissingCredentials),
        }
    }

    /// Internal method to perform token refresh via HTTP request.
    ///
    /// Updates `st.access_token`, `st.refresh_token`, and `st.expires_at`.
    async fn refresh_internal(&self, st: &mut AuthState) -> ResultAuth<()> {
        let refresh_token = st.refresh_token.clone().unwrap();
        let device_id = st.device_id.clone().unwrap();

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

        let data: RefreshResponse = resp.json().await.map_err(AuthError::HttpRequest)?;
        let now = Instant::now();

        st.access_token = Some(data.access_token.clone());
        st.refresh_token = Some(data.refresh_token.clone());
        st.expires_at = Some(now + Duration::from_secs(data.expires_in as u64));
        Ok(())
    }

    /// Check if both refresh token and device ID are set.
    pub async fn has_refresh_and_device_id(&self) -> bool {
        let st = self.state.lock().await;
        st.refresh_token.is_some() && st.device_id.is_some()
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
        st.expires_at = None;
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
    async fn test_apply_auth_header_with_refresh_token_flow() {
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

        let mut headers = HeaderMap::new();
        provider.apply_auth_header(&mut headers).await.unwrap();

        assert_eq!(headers.get(AUTHORIZATION).unwrap(), "Bearer new_access");
        mock.assert_async().await;
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
