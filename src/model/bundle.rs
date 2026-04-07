use serde::Deserialize;

use crate::model::{ContentCounter, CurrencyPrices, Thumbnail};

/// API response containing a list of bundles.
#[derive(Deserialize, Debug)]
pub struct BundlesResponse {
    /// Array of bundle items.
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
