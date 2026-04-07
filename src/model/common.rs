use serde::Deserialize;

/// Currency price info.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyPrices {
    /// Price in Euro.
    #[serde(alias = "EUR", default)]
    pub eur: f32,
    /// Price in Russian Rubles.
    #[serde(alias = "RUB")]
    pub rub: f32,
    /// Price in US Dollars.
    #[serde(alias = "USD")]
    pub usd: f32,
}

/// Counter for a specific content type inside a bundle.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ContentCounter {
    /// Content type name (e.g., "video", "text", "image").
    #[serde(rename = "type")]
    pub type_: String,
    /// Count of items.
    pub count: u32,
    /// Total size in bytes.
    pub size: u64,
}
