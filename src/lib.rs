//! # Boosty API Client Crate
//!
//! Asynchronous client for interacting with the **Boosty API**.
//!
//! ## Overview
//! This crate provides a fully typed, async-ready client for fetching and parsing data
//! from Boosty, including posts, media content, and related entities.
//!
//! ## Features
//! - **Async API client** (`api_client`) for fetching posts and related data.
//! - **OAuth-like token management** (`auth_provider`) supporting both static and refreshable tokens.
//! - **Strongly typed API responses** (`api_response`).
//! - **Unified error handling** (`error`).
//! - **Content extraction utilities** (`media_content`, `traits`).
//!
//! ## Module Summary
//! - [`api_client`] — Boosty API HTTP client built on top of `reqwest`.
//! - [`api_response`] — Typed models that represent API JSON responses.
//! - [`auth_provider`] — Internal authorization provider (token refresh / static bearer).
//! - [`error`] — Error definitions covering network, parsing, and domain errors.
//! - [`media_content`] — Defines [`ContentItem`] and helpers for extracting typed content.
//! - [`traits`] — Common traits for entities that expose content, title, or availability.
pub mod api_client;
pub mod api_response;
mod auth_provider;
pub mod error;
pub mod media_content;
pub mod traits;
