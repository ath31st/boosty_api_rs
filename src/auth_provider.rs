use anyhow::{Context, Result, bail};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use std::time::{Duration, Instant};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};

#[derive(Deserialize)]
struct RefreshResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
}

pub struct AuthProvider {
    client: Client,
    base_url: String,
    static_access_token: Option<String>,
    device_id: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    expires_at: Option<Instant>,
}

impl AuthProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            static_access_token: None,
            device_id: None,
            access_token: None,
            refresh_token: None,
            expires_at: None,
        }
    }

    pub async fn apply_auth_header(&mut self, headers: &mut HeaderMap) -> Result<()> {
        if let Some(static_tok) = &self.static_access_token {
            let hv = HeaderValue::from_str(&format!("Bearer {}", static_tok))
                .context("Invalid static token format")?;
            headers.insert(AUTHORIZATION, hv);
            return Ok(());
        }
        if self.refresh_token.is_some() && self.device_id.is_some() {
            let tok = self.get_access_token().await?;
            let hv = HeaderValue::from_str(&format!("Bearer {}", tok))
                .context("Invalid refreshed token format")?;
            headers.insert(AUTHORIZATION, hv);
            return Ok(());
        }
        Ok(())
    }

    pub fn set_access_token_only(&mut self, access: String) -> Result<()> {
        if access.is_empty() {
            bail!("Access token is empty");
        }
        self.static_access_token = Some(access);

        self.device_id = None;
        self.refresh_token = None;
        self.access_token = None;
        self.expires_at = None;
        Ok(())
    }

    pub fn set_refresh_token_and_device_id(
        &mut self,
        refresh: String,
        device_id: String,
    ) -> Result<()> {
        if refresh.is_empty() || device_id.is_empty() {
            bail!("Refresh token and/or device id are empty");
        }
        self.static_access_token = None;

        self.refresh_token = Some(refresh);
        self.device_id = Some(device_id);
        self.access_token = None;
        self.expires_at = None;
        Ok(())
    }

    pub async fn get_access_token(&mut self) -> Result<String> {
        if let Some(tok) = &self.static_access_token {
            return Ok(tok.clone());
        }

        if self.device_id.is_none() || self.refresh_token.is_none() {
            bail!("No credentials: neither static access token nor refresh + device_id set");
        }

        if let Some(exp) = self.expires_at {
            if Instant::now() + Duration::from_secs(30) >= exp {
                self.refresh_internal().await?;
            }
        }

        if self.access_token.is_none() {
            self.refresh_internal().await?;
        }

        Ok(self.access_token.clone().unwrap())
    }

    pub async fn force_refresh(&mut self) -> Result<String> {
        self.refresh_internal().await?;
        Ok(self.access_token.clone().unwrap())
    }

    fn get_refresh_token(&mut self) -> Result<String> {
        let refresh_token = match &self.refresh_token {
            Some(t) if !t.is_empty() => t.clone(),
            _ => bail!("Empty refresh token"),
        };
        Ok(refresh_token)
    }

    async fn refresh_internal(&mut self) -> Result<()> {
        let refresh_token = self.get_refresh_token()?;
        let device_id = self.device_id.as_ref().context("Empty device id")?;

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

        self.access_token = Some(data.access_token.clone());
        self.refresh_token = Some(data.refresh_token.clone());
        self.expires_at = Some(now + Duration::from_secs(data.expires_in as u64));

        Ok(())
    }
    
    pub fn has_refresh_and_device_id(&self) -> bool {
        self.refresh_token.is_some() && self.device_id.is_some()
    }
}
