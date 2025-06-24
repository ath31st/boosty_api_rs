use reqwest::StatusCode;
use thiserror::Error;

/// Error during authentication with the Boosty API (e.g., token refresh).
#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Invalid token format")]
    InvalidTokenFormat,

    #[error("Missing credentials: neither static access token nor refresh token + device_id set")]
    MissingCredentials,

    #[error("Empty access token")]
    EmptyAccessToken,

    #[error("Empty refresh token")]
    EmptyRefreshToken,

    #[error("Empty device_id")]
    EmptyDeviceId,

    #[error("HTTP request error during token refresh: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Unexpected HTTP status {status} during token refresh, body: {body}")]
    HttpStatus { status: StatusCode, body: String },

    #[error("Failed to parse JSON response during token refresh: {0}")]
    ParseError(#[from] serde_json::Error),
}

/// Error when calling Boosty API endpoints (includes AuthError).
#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("HTTP request error when calling API: {0}")]
    HttpRequest(#[from] reqwest::Error),

    #[error("Unexpected HTTP status {status} when calling endpoint '{endpoint}'")]
    HttpStatus {
        status: StatusCode,
        endpoint: String,
    },

    #[error("Failed to parse response body into intermediate JSON: {0}")]
    JsonParse(reqwest::Error),

    #[error("Failed to parse response JSON: {error}")]
    JsonParseDetailed { error: String },

    #[error("Unauthorized (401): invalid or missing token")]
    Unauthorized,

    #[error("Resource not available")]
    NotAvailable,

    #[error("Failed to deserialize JSON into target type: {0}")]
    Deserialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

pub type ResultAuth<T> = Result<T, AuthError>;
pub type ResultApi<T> = Result<T, ApiError>;
