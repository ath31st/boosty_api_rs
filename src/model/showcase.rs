use serde::Deserialize;

use crate::model::Post;

/// Showcase response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowcaseResponse {
    pub data: ShowcaseData,
    pub extra: Extra,
}

/// Showcase data
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShowcaseData {
    pub showcase_items: Vec<Showcase>,
}

/// Showcase extra
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub offset: i64,
    pub blog_id: i64,
    pub counters: Counters,
    pub is_enabled: bool,
    pub is_last: bool,
}

/// Showcase counters
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Counters {
    /// Visible total
    pub visible_total: i64,
    /// Visible posts count
    pub visible_posts_count: i64,
    /// Visible bundles count
    pub visible_bundles_count: i64,
}

/// Showcase item
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Showcase {
    /// Showcase item id
    pub showcase_item_id: i64,
    /// Type can be "post", "bundle"
    pub item_type: String,
    /// Is item visible
    pub is_visible: bool,
    /// Item id
    pub item_id: String,
    /// Post
    pub post: Post,
    /// Item position
    pub position: i64,
}
