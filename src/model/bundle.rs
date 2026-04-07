use serde::Deserialize;

use crate::model::{ContentCounter, CurrencyPrices, Post, ReactionCounter, Thumbnail};

/// API response containing a list of bundles.
#[derive(Deserialize, Debug)]
pub struct BundlesResponse {
    pub data: BundlesData,
}

/// Bundles data from the Boosty API.
#[derive(Deserialize, Debug)]
pub struct BundlesData {
    pub bundles: Vec<Bundle>,
}

/// Represents a single bundle from the Boosty API.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bundle {
    /// Unique bundle identifier.
    pub id: String,
    /// Bundle title.
    pub title: String,
    /// Bundle description.
    pub description: String,
    /// Price in the default currency.
    pub price: i32,
    /// Price in different currencies.
    pub currency_prices: CurrencyPrices,
    /// Whether the bundle is hidden.
    pub hidden: bool,
    /// Whether the current user has access to this bundle.
    pub has_access: bool,
    /// Creation timestamp (unix epoch).
    pub created_at: i64,
    /// Last update timestamp (unix epoch).
    pub updated_at: i64,
    /// Publish timestamp (unix epoch).
    pub published_at: i64,
    /// Deletion timestamp (unix epoch), or null if not deleted.
    pub deleted_at: Option<i64>,
    /// Blog identifier.
    pub blog_id: u64,
    /// Number of published posts in this bundle.
    pub published_posts_counter: u32,
    /// Number of posts accessible to the current user.
    pub accessible_posts_counter: u32,
    /// Sorting order for posts in the bundle.
    pub sorting: String,
    /// Counters for different content types inside the bundle.
    pub content_counters: Vec<ContentCounter>,
    /// Thumbnail image for the bundle.
    pub thumbnail: Thumbnail,
}

/// API response containing bundle items (posts within a bundle).
#[derive(Deserialize, Debug)]
pub struct BundleItemsResponse {
    /// Data wrapper with bundle items.
    pub data: BundleItemsData,
    /// Extra pagination info.
    pub extra: BundleExtra,
}

/// Data wrapper for bundle items.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BundleItemsData {
    /// Array of bundle item posts.
    pub bundle_items: Vec<BundleItem>,
}

/// Extra pagination data for bundle items.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BundleExtra {
    /// Whether this is the last page.
    pub is_last: bool,
    /// Current offset.
    pub offset: usize,
}

/// Represents a single post within a bundle.
/// This is a `Post` with additional bundle-specific fields.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BundleItem {
    /// The underlying post.
    #[serde(flatten)]
    pub post: Post,
    /// Position in the bundle.
    pub position: u32,
    /// Bundle ID this post belongs to.
    pub bundle_id: String,
    /// Array of bundle IDs.
    pub bundle_ids: Vec<String>,
    /// Post ID (duplicate of `id`, but present in JSON).
    pub post_id: String,
    /// Change marker.
    pub change: String,
    /// Whether the post is a draft.
    pub is_draft: bool,
    /// Reaction counters (alternative format).
    pub reaction_counters: Vec<ReactionCounter>,
    /// Bundles this post belongs to.
    pub bundles: Vec<Bundle>,
}
