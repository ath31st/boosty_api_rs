//! Boosty API client crate.
//!
//! Provides:
//! - Async-ready client for fetching posts from Boosty.
//! - OAuth-like token management (static bearer or refresh flow).
//! - Typed response models (`api_response`).
//! - Error handling (`error`).
//! - Content extraction utilities (`post_data_extractor`).
pub mod api_client;
pub mod api_response;
mod auth_provider;
pub mod error;
pub mod media_content;
