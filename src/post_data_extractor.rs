use crate::api_response::{MediaData, PlayerUrl, Post};

#[derive(Debug)]
pub enum ContentItem {
    Image {
        post_title: String,
        url: String,
        id: String,
    },
    Video {
        post_title: String,
        url: String,
        video_title: String,
    },
    Text {
        modificator: String,
        content: String,
    },
    Link {
        explicit: bool,
        content: String,
        url: String,
    },
    Unknown,
}

impl Post {
    pub fn not_available(&self) -> bool {
        !self.has_access || self.data.is_empty()
    }

    pub fn extract_content(&self) -> Vec<ContentItem> {
        let mut result = Vec::new();
        let post_title = self.title.clone();

        for media in &self.data {
            match media {
                MediaData::Image(img) => {
                    result.push(ContentItem::Image {
                        post_title: post_title.clone(),
                        url: img.url.clone(),
                        id: img.id.clone(),
                    });
                }
                MediaData::Video(vd) | MediaData::OkVideo(vd) => {
                    if let Some(best_url) = pick_higher_quality_for_video(&vd.player_urls) {
                        result.push(ContentItem::Video {
                            post_title: post_title.clone(),
                            url: best_url,
                            video_title: vd.title.clone(),
                        });
                    }
                }
                MediaData::Text(text) => result.push(ContentItem::Text {
                    content: text.content.clone(),
                    modificator: text.modificator.clone(),
                }),
                MediaData::Link(link) => result.push(ContentItem::Link {
                    explicit: link.explicit,
                    content: link.content.clone(),
                    url: link.url.clone(),
                }),
                MediaData::Unknown => {
                    result.push(ContentItem::Unknown);
                }
            }
        }
        result
    }
}

fn pick_higher_quality_for_video(player_urls: &[PlayerUrl]) -> Option<String> {
    const PRIORITY: &[&str] = &["ultra_hd", "full_hd", "high", "medium", "low"];

    for &pref in PRIORITY {
        if let Some(pu) = player_urls
            .iter()
            .find(|pu| pu.type_.as_str() == pref && !pu.url.is_empty())
        {
            return Some(pu.url.clone());
        }
    }

    player_urls
        .iter()
        .find(|pu| !pu.url.is_empty())
        .map(|pu| pu.url.clone())
}
