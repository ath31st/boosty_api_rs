use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchTagsFullResponse {
    pub extra: Extra,
    pub data: SearchTagsData,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub offset: String,
    pub is_last: bool,
}

/// Search tags response.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchTagsData {
    pub search_tags: Vec<SearchTag>,
}

/// Tags response.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TagsResponse {
    pub data: Vec<Tag>,
}

/// Tag search response.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SearchTag {
    /// Tag rank.
    pub rank: i64,
    /// Tag response.
    pub tag: Tag,
}

/// Tag response.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Tag {
    /// Tag title or name.
    pub title: String,
    /// Tag ID.
    pub id: i64,
}
