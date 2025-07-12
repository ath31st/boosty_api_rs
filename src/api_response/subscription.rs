use crate::api_response::subscription_level::Promo;
use serde::Deserialize;
use std::collections::HashMap;

/// API response containing a paginated list of subscriptions.
#[derive(Deserialize, Debug)]
pub struct SubscriptionsResponse {
    /// List of subscriptions.
    pub data: Vec<Subscription>,
    /// Total number of subscriptions.
    pub total: u64,
    /// Number of items per page.
    pub limit: u64,
    /// Offset of the current page.
    pub offset: u64,
}

/// Represents a single user subscription.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Subscription {
    /// Subscription ID.
    pub id: u64,
    /// ID of the subscription level.
    pub level_id: u64,
    /// Optional ID of the parent subscription (for upgrades/downgrades).
    pub parent_id: Option<u64>,
    /// Display name of the subscription.
    pub name: String,
    /// Standard price (in base currency units).
    pub price: u64,
    /// Custom price, if applied.
    pub custom_price: u64,
    /// Billing period in months.
    pub period: u8,
    /// Start timestamp (Unix epoch).
    pub on_time: i64,
    /// Optional end timestamp (if unsubscribed).
    pub off_time: Option<i64>,
    /// Optional next payment timestamp (if recurring).
    pub next_pay_time: Option<i64>,
    /// Whether the subscription is paused.
    pub is_pause: bool,
    /// Whether the subscription is suspended.
    pub is_suspended: bool,
    /// Whether the subscription is archived.
    pub is_archived: bool,
    /// Whether paid via Apple system.
    pub is_apple_payed: bool,
    /// Whether the fee is paid by the user.
    pub is_fee_paid: bool,
    /// ID of the subscriber (user).
    pub owner_id: u64,

    /// Associated subscription level info.
    pub subscription_level: SubscriptionLevelInfo,
    /// Associated blog info.
    pub blog: BlogInfo,

    /// Recommended promo for this subscription, if available.
    pub recommended_promo: Option<Promo>,
}

/// Basic info about the associated subscription level.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionLevelInfo {
    /// Subscription level ID.
    pub id: u64,
    /// Name of the level.
    pub name: String,
    /// Base price in main currency.
    pub price: u64,
    /// Price per currency (e.g., USD, EUR).
    pub currency_prices: HashMap<String, f64>,
    /// Whether the level has limited availability.
    pub is_limited: bool,
    /// Whether the level is archived.
    pub is_archived: bool,
    /// Whether the level is hidden.
    pub is_hidden: bool,
    /// Whether the level is marked as deleted.
    pub deleted: bool,
    /// ID of the blog owner (creator).
    pub owner_id: u64,
    /// Timestamp of creation (Unix epoch).
    pub created_at: i64,
    /// List of content blocks (JSON, e.g., images/text).
    pub data: Vec<serde_json::Value>,
}

/// Blog information associated with the subscription.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlogInfo {
    /// URL of the blog.
    pub blog_url: String,
    /// Title of the blog.
    pub title: String,
    /// Cover image URL.
    pub cover_url: String,
    /// Whether the blog contains adult content.
    pub has_adult_content: bool,
    /// Blog owner information.
    pub owner: BlogOwner,
    /// Feature flags for the blog.
    pub flags: BlogFlags,
}

/// Basic information about the blog owner.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlogOwner {
    /// Owner ID.
    pub id: u64,
    /// Display name.
    pub name: String,
    /// Whether the user has a custom avatar.
    pub has_avatar: bool,
    /// URL of the avatar.
    pub avatar_url: String,
}

/// Blog feature flags.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct BlogFlags {
    /// Whether post-level donations are enabled.
    pub show_post_donations: bool,
    /// Whether adult content is marked.
    pub has_adult_content: bool,
    /// Whether the blog has active subscription levels.
    pub has_subscription_levels: bool,
    /// Whether changing adult content flag is forbidden.
    pub forbidden_change_has_adult_content: bool,
    /// Whether the blog has active targets (goals).
    pub has_targets: bool,
    /// Whether the blog is owned by another user (viewed from outside).
    pub is_alien: bool,
    /// Whether the blog is indexable.
    pub allow_index: bool,
    /// Whether the blog is indexable by Google.
    pub allow_google_index: bool,
    /// Whether donation messages are accepted.
    pub accept_donation_messages: bool,
    /// Whether RSS feed is enabled.
    pub is_rss_feed_enabled: bool,
}
