# Boosty API

A minimal, async-ready client for fetching post data from a remote blogging API that requires either a bearer token
or a refresh token + device ID combo for authentication. This crate is designed with resiliency in mind: it
transparently handles token expiration and retries requests when needed.

## Table of Contents

- [Disclaimer](#disclaimer)
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

## Features

- Simple, high-level API for fetching individual posts or batches of posts.
- Built-in token management using access and refresh tokens (similar in concept to OAuth2 flows).
- Automatic token refreshing and retrying failed requests (when tokens are provided).
- Deserialization into rich, typed Rust structs using `serde`.
- Clean separation of concerns between authentication and API interaction.
- Currently, supports content types: **video**, **audio**, **image**, **text**, **link** and **file**.  

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
boosty_api = { git = "https://github.com/ath31st/boosty_api_rs.git", tag = "0.8.2" }
```

## Example fetching single post

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

    let post = api_client.fetch_post("some-blog-name", "post-id").await?;
    println!("{:#?}", post);

    Ok(())
}
```

## Example fetching multiple posts

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
    let posts = api_client.fetch_posts("blog_name", limit).await?;
    println!("{:#?}", posts);

    Ok(())
}
```

## Extracting content from a post

```rust
use boosty_api::post_data_extractor::ContentItem;

let content_items = post.extract_content();
for item in content_items {
  match item {
    ContentItem::Image { url, id } => {
      println ! ("Image URL: {}, ID: {}", url, id);
    }
    ContentItem::Video { url } => {
      println ! ("Video URL: {}", url);
    }
    ContentItem::OkVideo { url, video_title } => {
      println ! ("OK Video URL: {}, Title: {}", url, video_title);
    }
    ContentItem::Audio { url, audio_title, file_type } => {
      println ! ("Audio URL: {}, Title: {}, Type: {}", url, audio_title, file_type);
    }
    ContentItem::Text { modificator, content } => {
      println ! ("Text: {}, Modificator: {}", content, modificator);
    }
    ContentItem::Link { explicit, content, url } => {
      println ! ("Link: {}, Content: {}, Explicit: {}", url, content, explicit);
    }
    ContentItem::File { url, title, size } => {
      println ! ("File: {}, URL: {}, Size: {}", title, url, size);
    }
    ContentItem::Unknown => {
      println ! ("Unknown content type");
    }
  }
}
```

## Authentication

To get access token or refresh token and device_id, you need to log in to the service, then press F12 in the browser and go to the application tab, where you can select local storage. The required keys are _clentId and auth.

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

- `api_client`: Main entry point; fetches post(s), manages headers, and authentication logic
- `auth_provider`: Refresh-token and access-token management
- `api_response`: Deserialization models for all content types (video, image, audio, etc.)
- `error`: Uniform error types for API and auth operations
- `post_data_extractor`: Utility module

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

This project is licensed under the MIT License.
