use crate::media_content::ContentItem;

/// Common trait for entities with content.
pub trait HasContent {
    fn extract_content(&self) -> Vec<ContentItem>;
}

/// Common trait for entities with title.
pub trait HasTitle {
    fn safe_title(&self) -> String;
}

/// Common trait for entities with availability.
pub trait IsAvailable {
    fn not_available(&self) -> bool;
}
