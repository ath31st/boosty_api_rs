use serde::Deserialize;

/// API response containing a list of targets.
#[derive(Deserialize, Debug)]
pub struct TargetResponse {
    /// Array of target items.
    pub data: Vec<Target>,
}

/// Represents a single target from the API.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Target {
    /// Description of the goal.
    pub description: String,
    /// ID of the blogger.
    pub blogger_id: i64,
    /// Priority level of the target.
    pub priority: u32,
    /// Creation timestamp (unix epoch).
    pub created_at: i64,
    /// Unique identifier for the target.
    pub id: u64,
    /// Total sum required.
    pub target_sum: f64,
    /// Current collected sum.
    pub current_sum: f64,
    /// Optional finish timestamp.
    pub finish_time: Option<i64>,
    /// Blogger's username or URL.
    pub blogger_url: String,
    /// Type of target (e.g., "money").
    #[serde(rename = "type")]
    pub type_: String,
}
