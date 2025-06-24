use serde::Deserialize;

/// Represents a single post fetched from the Boosty API.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    /// The author or owner of the post.
    pub user: User,
    /// Is the post pinned by the author.
    pub is_pinned: bool,
    /// Is the post blocked.
    pub is_blocked: bool,
    /// Does the current user have access to this post.
    pub has_access: bool,
    /// Media content associated with the post.
    pub data: Vec<MediaData>,
    /// Is the post a record (e.g. audio/video record).
    pub is_record: bool,
    /// Counters for different content types inside the post.
    pub content_counters: Vec<ContentCounter>,
    /// Information about donators who supported this post.
    pub donators: Donators,
    /// Whether to show the views counter on the post.
    pub show_views_counter: bool,
    /// Creation timestamp (unix epoch).
    pub created_at: i64,
    /// Is the post published.
    pub is_published: bool,
    /// Is the post liked by the current user.
    pub is_liked: bool,
    /// Tags assigned to the post.
    pub tags: Vec<Tag>,
    /// Are comments disabled for this post.
    pub is_comments_denied: bool,
    /// Various counts (likes, comments, reactions).
    pub count: Count,
    /// Publish timestamp (unix epoch).
    pub publish_time: i64,
    /// Title of the post.
    pub title: String,
    /// Sorting order index.
    pub sort_order: i64,
    /// Price to access the post (if any).
    pub price: i32,
    /// Unique post identifier.
    pub id: String,
    /// Comments associated with the post.
    pub comments: Comments,
    /// Total amount of donations received for this post.
    pub donations: i32,
    /// Teaser media data shown before accessing full content.
    pub teaser: Vec<MediaData>,
    /// Is the post waiting for video processing.
    pub is_waiting_video: bool,
    /// Optional internal numeric ID.
    pub int_id: Option<i64>,
    /// Is the post deleted.
    pub is_deleted: bool,
    /// Last updated timestamp (unix epoch).
    pub updated_at: i64,
    /// Signed query string for accessing protected content.
    pub signed_query: String,
    /// Optional advertiser metadata (unstructured).
    pub advertiser_info: Option<serde_json::Value>,
    /// Price details in various currencies.
    pub currency_prices: CurrencyPrices,
}

/// Represents a user or author of a post.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// URL to the user's blog.
    pub blog_url: String,
    /// URL to the user's avatar image.
    pub avatar_url: String,
    /// User's display name.
    pub name: String,
    /// Whether the user has set an avatar.
    pub has_avatar: bool,
    /// Unique user identifier.
    pub id: i64,
    /// Various boolean flags for user settings.
    pub flags: Flags,
}

/// User-specific flags.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Flags {
    /// Should post donation info be shown.
    pub show_post_donations: bool,
}

/// Video media data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VideoData {
    /// URL to the video.
    pub url: String,
}

/// Video hosted on OK.ru platform.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OkVideoData {
    /// Upload status of the video.
    pub upload_status: Option<String>,
    /// Width of the video in pixels.
    pub width: u32,
    /// Current status.
    pub status: String,
    /// Title of the video.
    pub title: String,
    /// URL to the video.
    pub url: String,
    /// Optional preview ID.
    pub preview_id: Option<String>,
    /// List of player URLs for various formats.
    pub player_urls: Vec<PlayerUrl>,
    /// Video identifier.
    pub id: String,
    /// Video ID string.
    pub vid: String,
    /// URL to the preview image.
    pub preview: String,
    /// Height of the video in pixels.
    pub height: u32,
    /// Timecode for the video (seconds or frames).
    pub time_code: i32,
    /// Whether to show views counter.
    pub show_views_counter: bool,
    /// Duration of the video in seconds.
    pub duration: u32,
    /// Is the video processing complete.
    pub complete: bool,
    /// Number of views.
    pub views_counter: u32,
    /// URL to the default preview image.
    pub default_preview: String,
    /// Host used as failover.
    pub failover_host: String,
}

/// Audio media data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudioData {
    /// Whether to show views counter.
    pub show_views_counter: bool,
    /// Upload status of the audio.
    pub upload_status: Option<String>,
    /// Is the audio processing complete.
    pub complete: bool,
    /// Timecode or length of the audio (seconds).
    pub time_code: u32,
    /// File size in bytes.
    pub size: u32,
    /// Unique audio identifier.
    pub id: String,
    /// URL to the audio file.
    pub url: String,
    /// Artist name.
    pub artist: String,
    /// Album name.
    pub album: String,
    /// File MIME type or extension.
    pub file_type: String,
    /// Title of the audio track.
    pub title: String,
    /// Track number in the album.
    pub track: String,
    /// Duration in seconds.
    pub duration: u32,
}

/// Image media data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ImageData {
    /// URL to the image.
    pub url: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// Optional URL to preview image.
    pub preview: Option<String>,
    /// Unique image identifier.
    pub id: String,
}

/// Text media data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TextData {
    /// Modifier string (e.g. formatting info).
    pub modificator: String,
    /// Text content.
    pub content: String,
}

/// Link media data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LinkData {
    /// Whether the link is explicit.
    pub explicit: bool,
    /// Text content of the link.
    pub content: String,
    /// URL of the link.
    pub url: String,
}

/// Video player URL with type info.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PlayerUrl {
    /// Type of player URL (e.g. "mp4", "hls").
    #[serde(rename = "type")]
    pub type_: String,
    /// URL to the video stream.
    pub url: String,
}

/// Counter for specific content type inside a post.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentCounter {
    /// Content type name.
    #[serde(rename = "type")]
    pub type_: String,
    /// Count of items.
    pub count: u32,
    /// Total size in bytes.
    pub size: u64,
}

/// Donators info wrapper.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Donators {
    /// Additional metadata flags.
    pub extra: ExtraFlag,
    /// Raw data array for donators (untyped JSON).
    pub data: Vec<serde_json::Value>,
}

/// Extra flags container.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExtraFlag {
    /// Is this the last page or item.
    pub is_last: bool,
}

/// Comments wrapper.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Comments {
    /// Additional metadata flags.
    pub extra: ExtraFlag,
    /// Raw data array for comments (untyped JSON).
    pub data: Vec<serde_json::Value>,
}

/// Post counts summary.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Count {
    /// Number of comments.
    pub comments: u32,
    /// Various reaction counts.
    pub reactions: Reactions,
    /// Number of likes.
    pub likes: u32,
}

/// Reactions summary.
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

/// Currency price info.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyPrices {
    /// Price in Russian Rubles.
    #[serde(alias = "RUB")]
    pub rub: f32,
    /// Price in US Dollars.
    #[serde(alias = "USD")]
    pub usd: f32,
}

/// Post tag info.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// Tag title or name.
    pub title: String,
    /// Tag ID.
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
