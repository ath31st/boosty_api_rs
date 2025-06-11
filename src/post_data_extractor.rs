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
}

impl Post {
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
