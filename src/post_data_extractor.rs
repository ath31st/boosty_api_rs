use crate::api_response::{MediaData, PlayerUrl, Post};

#[derive(Debug)]
pub enum ContentItem {
    Image {
        url: String,
        id: String,
    },
    Video {
        url: String,
        video_title: String,
    },
    Audio {
        url: String,
        audio_title: String,
        file_type: String,
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

        for media in &self.data {
            match media {
                MediaData::Image(img) => {
                    result.push(ContentItem::Image {
                        url: img.url.clone(),
                        id: img.id.clone(),
                    });
                }
                MediaData::Video(vd) | MediaData::OkVideo(vd) => {
                    if let Some(best_url) = pick_higher_quality_for_video(&vd.player_urls) {
                        result.push(ContentItem::Video {
                            url: best_url,
                            video_title: vd.title.clone(),
                        });
                    }
                }
                MediaData::Audio(audio) => result.push(ContentItem::Audio {
                    url: audio.url.clone(),
                    audio_title: audio.track.clone(),
                    file_type: audio.file_type.clone(),
                }),
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
