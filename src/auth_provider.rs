use anyhow::{Context, Result, bail};
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

    pub async fn apply_auth_header(&self, headers: &mut HeaderMap) -> Result<()> {
        let maybe_token = {
            let st = self.state.lock().await;
            if let Some(static_tok) = &st.static_access_token {
                Some(static_tok.clone())
            } else {
                None
            }
        };

        if let Some(tok) = maybe_token {
            let hv = HeaderValue::from_str(&format!("Bearer {}", tok))
                .context("Invalid static token format")?;
            headers.insert(AUTHORIZATION, hv);
            return Ok(());
        }

        if self.has_refresh_and_device_id().await {
            let tok = self.get_access_token().await?;
            let hv = HeaderValue::from_str(&format!("Bearer {}", tok))
                .context("Invalid refreshed token format")?;
            headers.insert(AUTHORIZATION, hv);
        }
        Ok(())
    }

    pub async fn set_access_token_only(&self, access: String) -> Result<()> {
        if access.is_empty() {
            bail!("Access token is empty");
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
    ) -> Result<()> {
        if refresh.is_empty() || device_id.is_empty() {
            bail!("Refresh token and/or device id are empty");
        }
        let mut st = self.state.lock().await;
        st.static_access_token = None;
        st.refresh_token = Some(refresh);
        st.device_id = Some(device_id);
        st.access_token = None;
        st.expires_at = None;
        Ok(())
    }

    pub async fn get_access_token(&self) -> Result<String> {
        let mut st = self.state.lock().await;
        if let Some(tok) = &st.static_access_token {
            return Ok(tok.clone());
        }
        let refresh = st.refresh_token.clone();
        let device_id = st.device_id.clone();
        if refresh.is_none() || device_id.is_none() {
            bail!("No credentials: neither static access token nor refresh + device_id set");
        }
        let need_refresh = match st.expires_at {
            Some(exp) => Instant::now() + Duration::from_secs(30) >= exp,
            None => true,
        };
        if need_refresh {
            self.refresh_internal(&mut st).await?;
        }
        Ok(st.access_token.clone().unwrap())
    }

    pub async fn force_refresh(&self) -> Result<String> {
        let mut st = self.state.lock().await;
        self.refresh_internal(&mut st).await?;
        Ok(st.access_token.clone().unwrap())
    }

    async fn refresh_internal(&self, st: &mut AuthState) -> Result<()> {
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
            .context("Send refresh tokens request failed")?;

        if resp.status() != StatusCode::OK {
            bail!("Refresh tokens failed: HTTP {}", resp.status());
        }

        let data: RefreshResponse = resp
            .json()
            .await
            .context("Parse refresh token response failed")?;
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
