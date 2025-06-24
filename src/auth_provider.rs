use crate::error::{AuthError, ResultAuth};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

#[derive(Debug)]
struct AuthState {
    static_access_token: Option<String>,
    device_id: Option<String>,
    refresh_token: Option<String>,
    access_token: Option<String>,
    expires_at: Option<Instant>,
}

#[derive(Clone)]
pub struct AuthProvider {
    client: Client,
    base_url: String,
    state: Arc<Mutex<AuthState>>,
}

impl AuthProvider {
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

    pub async fn apply_auth_header(&self, headers: &mut HeaderMap) -> ResultAuth<()> {
        let static_tok_opt = {
            let st = self.state.lock().await;
            st.static_access_token.clone()
        };

        if let Some(tok) = static_tok_opt {
            let hv = HeaderValue::from_str(&format!("Bearer {}", tok))
                .map_err(|_| AuthError::InvalidTokenFormat)?;
            headers.insert(AUTHORIZATION, hv);
            return Ok(());
        }

        if self.has_refresh_and_device_id().await {
            let tok = self.get_access_token().await?;
            let hv = HeaderValue::from_str(&format!("Bearer {}", tok))
                .map_err(|_| AuthError::InvalidTokenFormat)?;
            headers.insert(AUTHORIZATION, hv);
        }
        Ok(())
    }

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
                let need_refresh = match st2.expires_at {
                    Some(exp) => Instant::now() + Duration::from_secs(30) >= exp,
                    None => true,
                };
                if need_refresh {
                    self.refresh_internal(&mut st2).await?;
                }
                Ok(st2.access_token.clone().unwrap())
            }
            _ => Err(AuthError::MissingCredentials),
        }
    }

    pub async fn force_refresh(&self) -> ResultAuth<String> {
        let mut st = self.state.lock().await;

        if st.refresh_token.is_none() {
            return Err(AuthError::EmptyRefreshToken);
        }
        if st.device_id.is_none() {
            return Err(AuthError::EmptyDeviceId);
        }

        self.refresh_internal(&mut st).await?;
        Ok(st.access_token.clone().unwrap())
    }

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

    pub async fn has_refresh_and_device_id(&self) -> bool {
        let st = self.state.lock().await;
        st.refresh_token.is_some() && st.device_id.is_some()
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
    async fn test_force_refresh_fails_with_missing_data() {
        let provider = make_provider("http://localhost");
        let err = provider.force_refresh().await.unwrap_err();
        assert!(matches!(err, AuthError::EmptyRefreshToken));

        provider
            .set_refresh_token_and_device_id("ref".into(), "".into())
            .await
            .unwrap_err();
    }
}
