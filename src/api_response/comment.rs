use serde::Deserialize;

use crate::api_response::{MediaData, ReactionCounter, Reactions};

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
    pub replies: Replies,
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
