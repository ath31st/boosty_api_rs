use serde::Deserialize;

/// Currency price info.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CurrencyPrices {
    /// Price in Euro.
    #[serde(alias = "EUR", default)]
    pub eur: f64,
    /// Price in Russian Rubles.
    #[serde(alias = "RUB", default)]
    pub rub: f64,
    /// Price in US Dollars.
    #[serde(alias = "USD", default)]
    pub usd: f64,
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

/// Thumbnail image data.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    /// Unique image identifier.
    pub id: String,
    /// Image title.
    pub title: String,
    /// URL to the image.
    pub url: String,
    /// Image rendition type.
    pub rendition: String,
    /// Image type (e.g., "image").
    #[serde(rename = "type")]
    pub type_: String,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
    /// File size in bytes.
    pub size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_single_currency() {
        let prices: CurrencyPrices = serde_json::from_str(r#"{"RUB": 1000}"#).unwrap();
        assert_eq!(prices.rub, 1000.0);
        assert_eq!(prices.usd, 0.0);
        assert_eq!(prices.eur, 0.0);
    }

    #[test]
    fn deserializes_all_currencies() {
        let prices: CurrencyPrices =
            serde_json::from_str(r#"{"RUB": 150, "USD": 1.84, "EUR": 1.58}"#).unwrap();
        assert_eq!(prices.rub, 150.0);
        assert_eq!(prices.usd, 1.84);
        assert_eq!(prices.eur, 1.58);
    }
}
