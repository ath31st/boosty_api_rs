# Boosty API

A minimal, async-ready client for getting post data from a remote blogging API that requires either a bearer token
or a refresh token + device ID combo for authentication. This crate is designed with resiliency in mind: it
transparently handles token expiration and retries requests when needed.

## Table of Contents

- [Disclaimer](#disclaimer)
- [Project Status](#project-status)
- [Features](#features)
- [Installation](#installation)
- [Example: Fetching a Single Post](#example-fetching-single-post)
- [Example: Fetching Multiple Posts](#example-fetching-multiple-posts)
- [Extracting Content from a Post](#extracting-content-from-a-post)
- [Authentication](#authentication)
    - [1. Static Bearer Token](#1-static-bearer-token)
    - [2. Refresh Token Flow](#2-refresh-token-flow)
- [Crate Structure](#crate-structure)
- [Error Handling](#error-handling)
- [API Documentation](#api-documentation)
- [Contributing](#contributing)
- [License](#license)

## Disclaimer

This crate is intended for research and personal use only.
By using it, you agree to:

- Access only your own content from the Boosty platform.
- Refrain from scraping, redistributing, or otherwise misusing content that you do not own.
- Comply with Boosty's [Terms of Service](https://boosty.to/terms) and any applicable copyright laws.

The author is not responsible for any misuse of this software.

## Project Status

🚧 This library is under active development.  
Breaking changes, refactoring, and architectural updates may occur frequently.  
Use with caution in production environments and pin specific versions if needed.

## Features

### 🔐 Authentication

- Static bearer token or refresh-token + device ID (OAuth2-like).
- Automatic token refresh and retry on expiration.
- Clean separation of `AuthProvider` logic.

### 🔁 Retry Behavior

The client automatically retries HTTP requests that fail due to transient network errors or expired access tokens.

- Retry logic is centralized in the `get_request()` method.
- On token expiration, the client performs a refresh (if refresh-token and device ID are set) and retries the request.
- Other error types (like 4xx or business-logic errors) are not retried.

### 📝 Post API

- Get single post: `get_post(blog, id)`.
- Get multiple posts: `get_posts(blog, limit)`.
- Strongly typed `Post` struct with `serde` support.
- Handles `"not available"` status gracefully.

### 🎯 Blog Metadata

- Get targets via `get_targets(blog)`.
- Get subscription levels via `get_subscription_levels(blog, show_free_level)`.

### 📜 Subscriptions

- Get current user subscriptions via `get_subscriptions(limit, with_follow)`, returning a paginated
  `SubscriptionsResponse`.

### ⚙️ Low-level Features

- Async-ready `ApiClient` using `reqwest`.
- Custom headers with real-world `User-Agent`, `DNT`, `Cache-Control`, etc.
- Unified error types: `ApiError`, `AuthError` with detailed variants.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
boosty_api = "0.14.0"
```

or

```bash
cargo add boosty_api
```

## Example getting single post

```rust
use boosty_api::api_client::ApiClient;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "https://api.example.com";

    let api_client = ApiClient::new(client, base_url);

    // Use static bearer token (optional)
    api_client.set_bearer_token("your-access-token").await?;

    // Or use refresh token + device ID
    // api.set_refresh_token_and_device_id("your-refresh-token", "your-device-id").await?;

    let post = api_client.get_post("some-blog-name", "post-id").await?;
    println!("{:#?}", post);

    Ok(())
}
```

## Example getting multiple posts

```rust
use boosty_api::api_client::ApiClient;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let base_url = "https://api.example.com";

    let api_client = ApiClient::new(client, base_url);

    // Use static bearer token (optional)
    api_client.set_bearer_token("your-access-token").await?;

    // Or use refresh token + device ID
    // api.set_refresh_token_and_device_id("your-refresh-token", "your-device-id").await?;
    let limit = 5;
    let posts = api_client.get_posts("blog_name", limit).await?;
    println!("{:#?}", posts);

    Ok(())
}
```

## Extracting content from a post

```rust
use boosty_api::post_data_extractor::ContentItem;

fn print_content(post: &boosty_api::Post) {
    let content_items = post.extract_content();
    for item in content_items {
        match item {
            ContentItem::Image { url, id } => {
                println!("Image URL: {url}, ID: {id}");
            }
            ContentItem::Video { url } => {
                println!("Video URL: {url}");
            }
            ContentItem::OkVideo { url, video_title } => {
                println!("OK Video URL: {url}, Title: {video_title}");
            }
            ContentItem::Audio { url, audio_title, file_type } => {
                println!("Audio URL: {url}, Title: {audio_title}, Type: {file_type}");
            }
            ContentItem::Text { modificator, content } => {
                println!("Text: {content}, Modificator: {modificator}");
            }
            ContentItem::Link { explicit, content, url } => {
                println!("Link: {url}, Content: {content}, Explicit: {explicit}");
            }
            ContentItem::File { url, title, size } => {
                println!("File: {title}, URL: {url}, Size: {size}");
            }
            ContentItem::Unknown => {
                println!("Unknown content type");
            }
        }
    }
}
```

## Authentication

To get access token or refresh token and device_id, you need to log in to the service, then press F12 in the browser and
go to the application tab, where you can select local storage. The required keys are _clentId and auth.

There are two options:

### 1. Static Bearer Token

```rust
api_client.set_bearer_token("access-token").await?;
```

### 2. Refresh Token Flow

```rust
api_client.set_refresh_token_and_device_id("refresh-token", "device-id").await?;
```

If a post is unavailable and refresh credentials are present, the client will automatically attempt a refresh.

## Crate Structure

- `api_client`: Main entry point; gets post(s), manages headers, and authentication logic
- `auth_provider`: Refresh-token and access-token management
- `api_response`: Deserialization models for all API content types (e.g. post, target); organized by domain
- `error`: Uniform error types for API and auth operations
- `post_data_extractor`: Utility module for extracting structured data from post models

## Error Handling

All API and auth operations return `Result<T, ApiError>` or `Result<T, AuthError>`, depending on context. Errors are
strongly typed and cover:

- HTTP request failures
- JSON (de)serialization issues
- Invalid or expired credentials
- Unsuccessful API status codes

## API Documentation

For detailed documentation, please refer to [docs.rs](https://docs.rs/boosty_api).

## Contributing

Contributions are welcome! Please open an issue or submit a pull request
on [GitHub](https://github.com/ath31st/boosty_api_rs).

## License

This project is licensed under the [MIT License](LICENSE).
