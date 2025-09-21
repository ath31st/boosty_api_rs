use serde::Deserialize;

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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReactionCounter {
    #[serde(rename = "type")]
    pub type_: String,
    pub count: u32,
}
