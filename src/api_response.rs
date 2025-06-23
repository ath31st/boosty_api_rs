use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub user: User,
    pub is_pinned: bool,
    pub is_blocked: bool,
    pub has_access: bool,
    pub data: Vec<MediaData>,
    pub is_record: bool,
    pub content_counters: Vec<ContentCounter>,
    pub donators: Donators,
    pub show_views_counter: bool,
    pub created_at: i64,
    pub is_published: bool,
    pub is_liked: bool,
    pub tags: Vec<Tag>,
    pub is_comments_denied: bool,
    pub count: Count,
    pub publish_time: i64,
    pub title: String,
    pub sort_order: i64,
    pub price: i32,
    pub id: String,
    pub comments: Comments,
    pub donations: i32,
    pub teaser: Vec<MediaData>,
    pub is_waiting_video: bool,
    pub int_id: Option<i64>,
    pub is_deleted: bool,
    pub updated_at: i64,
    pub signed_query: String,
    pub advertiser_info: Option<serde_json::Value>,
    pub currency_prices: CurrencyPrices,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub blog_url: String,
    pub avatar_url: String,
    pub name: String,
    pub has_avatar: bool,
    pub id: i64,
    pub flags: Flags,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    pub show_post_donations: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoData {
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OkVideoData {
    pub upload_status: String,
    pub width: u32,
    pub status: String,
    pub title: String,
    pub url: String,
    pub preview_id: Option<String>,
    pub player_urls: Vec<PlayerUrl>,
    pub id: String,
    pub vid: String,
    pub preview: String,
    pub height: u32,
    pub time_code: i32,
    pub show_views_counter: bool,
    pub duration: u32,
    pub complete: bool,
    pub views_counter: u32,
    pub default_preview: String,
    pub failover_host: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioData {
    pub show_views_counter: bool,
    pub upload_status: Option<String>,
    pub complete: bool,
    pub time_code: u32,
    pub size: u32,
    pub id: String,
    pub url: String,
    pub artist: String,
    pub album: String,
    pub file_type: String,
    pub title: String,
    pub track: String,
    pub duration: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageData {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub preview: Option<String>,
    pub id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextData {
    pub modificator: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkData {
    pub explicit: bool,
    pub content: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUrl {
    #[serde(rename = "type")]
    pub type_: String,
    pub url: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentCounter {
    #[serde(rename = "type")]
    pub type_: String,
    pub count: u32,
    pub size: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Donators {
    pub extra: ExtraFlag,
    pub data: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFlag {
    pub is_last: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Comments {
    pub extra: ExtraFlag,
    pub data: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Count {
    pub comments: u32,
    pub reactions: Reactions,
    pub likes: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Reactions {
    pub dislike: u32,
    pub heart: u32,
    pub fire: u32,
    pub angry: u32,
    pub wonder: u32,
    pub laught: u32,
    pub sad: u32,
    pub like: u32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyPrices {
    #[serde(alias = "RUB")]
    pub rub: f32,
    #[serde(alias = "USD")]
    pub usd: f32,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    pub title: String,
    pub id: i64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum MediaData {
    #[serde(rename = "video", rename_all = "camelCase")]
    Video(VideoData),
    #[serde(rename = "ok_video", rename_all = "camelCase")]
    OkVideo(OkVideoData),
    #[serde(rename = "audio_file", rename_all = "camelCase")]
    Audio(AudioData),
    #[serde(rename = "image", rename_all = "camelCase")]
    Image(ImageData),
    #[serde(rename = "text", rename_all = "camelCase")]
    Text(TextData),
    #[serde(rename = "link", rename_all = "camelCase")]
    Link(LinkData),
    #[serde(other)]
    Unknown,
}
