use crate::model::Flags;
use serde::Deserialize;

/// Represents a user or author of a post.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct User {
    /// URL to the user's blog.
    pub blog_url: String,
    /// URL to the user's avatar image.
    pub avatar_url: String,
    /// User's display name.
    pub name: String,
    /// Whether the user has set an avatar.
    pub has_avatar: bool,
    /// Unique user identifier.
    pub id: i64,
    /// Various boolean flags for user settings.
    pub flags: Flags,
}
