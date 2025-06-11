use anyhow::Result;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

pub struct Headers {
    pub map: HeaderMap,
}

impl Headers {
    pub fn new_with_auth(access_token: &str) -> Result<Self> {
        let mut map = HeaderMap::new();
        let auth_value = HeaderValue::from_str(&format!("Bearer {}", access_token))?;
        map.insert(AUTHORIZATION, auth_value);
        Ok(Headers { map })
    }

    pub fn new() -> Self {
        Headers {
            map: HeaderMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: impl Into<reqwest::header::HeaderName>,
        value: impl Into<HeaderValue>,
    ) {
        self.map.insert(key.into(), value.into());
    }
}
