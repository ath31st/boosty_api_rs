use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiResponse {
    user: User,
    is_pinned: bool,
    is_blocked: bool,
    has_access: bool,
    data: Vec<MediaData>,
    is_record: bool,
    content_counters: Vec<ContentCounter>,
    donators: Donators,
    show_views_counter: bool,
    created_at: i64,
    is_published: bool,
    is_liked: bool,
    tags: Vec<String>,
    is_comments_denied: bool,
    count: Count,
    publish_time: i64,
    title: String,
    sort_order: i64,
    price: i32,
    id: String,
    comments: Comments,
    donations: i32,
    teaser: Vec<String>,
    is_waiting_video: bool,
    int_id: i64,
    is_deleted: bool,
    updated_at: i64,
    signed_query: String,
    advertiser_info: Option<serde_json::Value>,
    currency_prices: CurrencyPrices,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct User {
    blog_url: String,
    avatar_url: String,
    name: String,
    has_avatar: bool,
    id: i64,
    flags: Flags,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Flags {
    show_post_donations: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VideoData {
    upload_status: String,
    #[serde(rename = "type")]
    type_: String,
    width: u32,
    status: String,
    title: String,
    url: String,
    preview_id: String,
    player_urls: Vec<PlayerUrl>,
    id: String,
    vid: String,
    preview: String,
    height: u32,
    time_code: i32,
    show_views_counter: bool,
    duration: u32,
    complete: bool,
    views_counter: u32,
    default_preview: String,
    failover_host: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImageData {
    url: String,
    width: u32,
    height: u32,
    preview: Option<String>,
    id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerUrl {
    #[serde(rename = "type")]
    type_: String,
    url: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ContentCounter {
    #[serde(rename = "type")]
    type_: String,
    count: u32,
    size: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Donators {
    extra: ExtraFlag,
    data: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtraFlag {
    is_last: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Comments {
    extra: ExtraFlag,
    data: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Count {
    comments: u32,
    reactions: Reactions,
    likes: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Reactions {
    dislike: u32,
    heart: u32,
    fire: u32,
    angry: u32,
    wonder: u32,
    laught: u32,
    sad: u32,
    like: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CurrencyPrices {
    rub: f32,
    usd: f32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
enum MediaData {
    Video(VideoData),
    Image(ImageData),
}
