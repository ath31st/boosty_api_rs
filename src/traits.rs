use crate::media_content::{ContentItem, VideoQuality};

/// Common trait for entities with content.
pub trait HasContent {
    /// Extracts content items, picking the highest available OK.ru video quality.
    fn extract_content(&self) -> Vec<ContentItem>;

    /// Extracts content items, capping OK.ru video quality at `max_quality`.
    fn extract_content_with_video_quality(
        &self,
        max_quality: VideoQuality,
    ) -> Vec<ContentItem>;
}

/// Common trait for entities with title.
pub trait HasTitle {
    fn safe_title(&self) -> String;
}

/// Common trait for entities with availability.
pub trait IsAvailable {
    fn not_available(&self) -> bool;
}
