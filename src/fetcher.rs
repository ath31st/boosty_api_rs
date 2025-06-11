use crate::api_response::ApiResponse;
use crate::headers::Headers;
use anyhow::Result;
use reqwest::{Client, Response};
use serde_json::{Value, from_value};

pub const API_URL: &str = "https://api.boosty.to";

async fn get_request(path: &str, headers: Option<&Headers>) -> Result<Response> {
    let url = format!("{}/v1/{}", API_URL, path);
    let client = Client::new();
    let builder = client.get(&url);

    let builder = if let Some(h) = headers {
        builder.headers(h.map.clone())
    } else {
        builder
    };

    let response = builder.send().await?;
    Ok(response)
}

pub async fn fetch_post(
    blog_name: &str,
    post_id: &str,
    headers: Option<&Headers>,
) -> Result<ApiResponse> {
    let path = format!("blog/{}/post/{}", blog_name, post_id);
    let response = get_request(&path, headers).await?;
    let parsed = response.json::<ApiResponse>().await?;
    Ok(parsed)
}

pub async fn fetch_posts(
    blog_name: &str,
    limit: usize,
    headers: Option<&Headers>,
) -> Result<Vec<ApiResponse>> {
    let path = format!("blog/{}/post/?limit={}", blog_name, limit);
    let response = get_request(&path, headers).await?;
    let json: Value = response.json().await?;
    let parsed = from_value(json["data"].clone())?;
    Ok(parsed)
}
