use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    media_content::{self, ContentItem},
    model::{MediaData, ReactionCounter, Reactions},
    traits::{HasContent, IsAvailable},
};

/// Comments response.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommentsResponse {
    pub data: Vec<Comment>,
    pub extra: Extra,
}

/// Extra flags container.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub is_first: bool,
    pub is_last: bool,
}

/// Single comment.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub int_id: u64,
    pub post: PostRef,
    pub author: Author,
    pub created_at: u64,
    pub updated_at: Option<u64>,
    pub is_deleted: bool,
    pub is_blocked: bool,
    pub is_updated: bool,
    pub reply_count: u32,
    pub replies: Option<Replies>,
    pub data: Vec<MediaData>,
    pub reactions: Reactions,
    pub reaction_counters: Vec<ReactionCounter>,

    pub parent_id: Option<u64>,
    pub reply_id: Option<u64>,
    pub reply_to_user: Option<Author>,
}

/// Post reference.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PostRef {
    pub id: String,
}

/// Comment author.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Author {
    pub id: u64,
    pub name: String,
    pub has_avatar: bool,
    pub avatar_url: String,
}

/// Comment replies.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Replies {
    pub data: Vec<Comment>,
    pub extra: Extra,
}

impl IsAvailable for Comment {
    /// Returns true if the comment is not accessible or has no media data.
    ///
    /// # Returns
    ///
    /// - `true` if user has no access (`has_access == false`) OR `data` is empty.
    /// - `false` otherwise.
    fn not_available(&self) -> bool {
        self.data.is_empty()
    }
}

impl HasContent for Comment {
    /// Extracts media items from comment into a vector of `ContentItem`.
    ///
    /// # Returns
    ///
    /// Vector of `ContentItem` items.
    fn extract_content(&self) -> Vec<ContentItem> {
        media_content::extract_content(&self.data)
    }
}

/// Comment block.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CommentBlock {
    /// Text block.
    #[serde(rename = "text")]
    Text(TextBlock),
    /// Smile block.
    #[serde(rename = "smile")]
    Smile(SmileBlock),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextBlock {
    pub content: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub modificator: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmileBlock {
    pub name: String,
}

impl CommentBlock {
    pub fn text(text: &str) -> Self {
        CommentBlock::Text(TextBlock {
            content: json!([text, "unstyled", []]).to_string(),
            modificator: "".into(),
        })
    }

    pub fn text_end() -> Self {
        CommentBlock::Text(TextBlock {
            content: "".into(),
            modificator: "BLOCK_END".into(),
        })
    }

    pub fn smile(name: &str) -> Self {
        CommentBlock::Smile(SmileBlock { name: name.into() })
    }
}
