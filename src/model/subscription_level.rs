use serde::Deserialize;
use std::collections::HashMap;

/// API response containing a list of subscription levels.
#[derive(Deserialize, Debug)]
pub struct SubscriptionLevelResponse {
    /// Array of subscription level items.
    pub data: Vec<SubscriptionLevel>,
}

/// Represents a single subscription level from the API.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionLevel {
    /// Unique identifier.
    pub id: u64,
    /// Name of the subscription level.
    pub name: String,
    /// Base price in default currency.
    pub price: f64,
    /// Price in different currencies.
    pub currency_prices: HashMap<String, f64>,
    /// Whether the subscription is limited.
    pub is_limited: bool,
    /// Whether the subscription is archived.
    pub is_archived: bool,
    /// Whether the subscription is deleted.
    pub deleted: bool,
    /// Whether the subscription is hidden.
    pub is_hidden: bool,
    /// Timestamp of creation (unix epoch).
    pub created_at: i64,
    /// ID of the owner (creator).
    pub owner_id: u64,
    /// Promo info.
    pub promos: Vec<Promo>,
    /// Content data blocks (text, image).
    pub data: Vec<DataBlock>,
    /// External application bindings.
    pub external_apps: ExternalApps,
}

/// Represents a promotional campaign attached to a subscription level.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Promo {
    /// Unique identifier of the promo.
    pub id: u64,
    /// Type of the promo (e.g., `"discount"`).
    #[serde(rename = "type")]
    pub type_: String,
    /// Optional human-readable description.
    pub description: Option<String>,
    /// Start timestamp of the promo (Unix epoch).
    pub start_time: i64,
    /// End timestamp of the promo, or `None` if it does not expire.
    pub end_time: Option<i64>,
    /// Whether the promo is already finished.
    pub is_finished: bool,
    /// Access rights granted by this promo.
    pub access: Access,
    /// Activation counters for the promo.
    pub count: Count,
    /// Discount details associated with the promo.
    pub discount: Discount,
}

/// Access rights granted by a promo campaign.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Access {
    /// Whether holders of other subscription levels can access.
    pub access_other_level_subscriber: bool,
    /// Whether new subscribers receive this promo.
    pub new_subscriber: bool,
    /// Whether existing paid subscribers receive this promo.
    pub old_paid_subscriber: bool,
}

/// Activation limits for a promo campaign.
#[derive(Deserialize, Debug)]
pub struct Count {
    /// Number of times the promo has been activated.
    pub activation: u64,
    /// Maximum number of activations allowed, or `None` if unlimited.
    pub max_activation: Option<u64>,
}

/// Details about the discount offered by a promo.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Discount {
    /// Discounted price.
    pub price: u64,
    /// Discount percentage.
    pub percent: u32,
    /// Price equivalents in various currencies.
    #[serde(rename = "currencyPrices")]
    pub currency_prices: HashMap<String, f64>,
}

/// Represents a content block (text or image).
#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum DataBlock {
    /// Text block.
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "text")]
    Text {
        /// Text content.
        content: String,
        /// Optional content marker ("", "BLOCK_END").
        modificator: String,
    },
    /// Image block.
    #[serde(rename_all = "camelCase")]
    #[serde(rename = "image")]
    Image {
        /// Image ID.
        id: String,
        /// Image URL.
        url: String,
        /// Rendition type (e.g. "source").
        rendition: String,
        /// Width in pixels.
        width: u32,
        /// Height in pixels.
        height: u32,
        /// File size in bytes.
        size: u64,
    },
}

/// External applications data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExternalApps {
    /// Discord integration.
    pub discord: DiscordApp,
    /// Telegram integration.
    pub telegram: TelegramApp,
}

/// Discord app data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscordApp {
    /// Whether Discord is configured.
    pub is_configured: bool,
    /// Optional Discord metadata.
    pub data: Option<DiscordData>,
}

/// Discord-specific data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscordData {
    /// Role assigned for the subscription.
    pub role: DiscordRole,
}

/// Discord role description.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DiscordRole {
    /// Discord role ID.
    pub id: String,
    /// Discord role name.
    pub name: String,
}

/// Telegram app data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TelegramApp {
    /// Whether Telegram is configured.
    pub is_configured: bool,
}
