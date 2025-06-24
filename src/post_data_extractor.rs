use crate::api_response::{MediaData, PlayerUrl, Post};

#[derive(Debug)]
pub enum ContentItem {
    Image {
        url: String,
        id: String,
    },
    Video {
        url: String,
    },
    OkVideo {
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
                MediaData::Video(vd) => result.push(ContentItem::Video {
                    url: vd.url.clone(),
                }),
                MediaData::OkVideo(vd) => {
                    if let Some(best_url) = pick_higher_quality_for_video(&vd.player_urls) {
                        result.push(ContentItem::OkVideo {
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

pub(crate) fn pick_higher_quality_for_video(player_urls: &[PlayerUrl]) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api_response::*;

    fn dummy_post(data: Vec<MediaData>, has_access: bool) -> Post {
        Post {
            has_access,
            data,
            user: User {
                blog_url: "".into(),
                avatar_url: "".into(),
                name: "".into(),
                has_avatar: false,
                id: 0,
                flags: Flags {
                    show_post_donations: false,
                },
            },
            is_pinned: false,
            is_blocked: false,
            is_record: false,
            content_counters: vec![],
            donators: Donators {
                extra: ExtraFlag { is_last: false },
                data: vec![],
            },
            show_views_counter: false,
            created_at: 0,
            is_published: true,
            is_liked: false,
            tags: vec![],
            is_comments_denied: false,
            count: Count {
                comments: 0,
                likes: 0,
                reactions: Reactions {
                    dislike: 0,
                    heart: 0,
                    fire: 0,
                    angry: 0,
                    wonder: 0,
                    laught: 0,
                    sad: 0,
                    like: 0,
                },
            },
            publish_time: 0,
            title: "".into(),
            sort_order: 0,
            price: 0,
            id: "".into(),
            comments: Comments {
                extra: ExtraFlag { is_last: false },
                data: vec![],
            },
            donations: 0,
            teaser: vec![],
            is_waiting_video: false,
            int_id: None,
            is_deleted: false,
            updated_at: 0,
            signed_query: "".into(),
            advertiser_info: None,
            currency_prices: CurrencyPrices { rub: 0.0, usd: 0.0 },
        }
    }

    #[test]
    fn test_not_available_cases() {
        assert!(dummy_post(vec![], true).not_available());
        assert!(dummy_post(vec![], false).not_available());
        assert!(dummy_post(vec![MediaData::Unknown], false).not_available());
        assert!(!dummy_post(vec![MediaData::Unknown], true).not_available());
    }

    #[test]
    fn test_extract_image() {
        let img = ImageData {
            url: "image_url".into(),
            width: 100,
            height: 100,
            preview: None,
            id: "img123".into(),
        };
        let post = dummy_post(vec![MediaData::Image(img)], true);
        let content = post.extract_content();

        assert!(
            matches!(content[0], ContentItem::Image { ref url, ref id } if url == "image_url" && id == "img123")
        );
    }

    #[test]
    fn test_extract_video() {
        let video = VideoData {
            url: "video_url".into(),
        };
        let post = dummy_post(vec![MediaData::Video(video)], true);
        let content = post.extract_content();

        assert!(matches!(content[0], ContentItem::Video { ref url } if url == "video_url"));
    }

    #[test]
    fn test_extract_ok_video_with_priority() {
        let ok_video = OkVideoData {
            upload_status: "".into(),
            width: 0,
            status: "".into(),
            title: "vid".into(),
            url: "".into(),
            preview_id: None,
            player_urls: vec![
                PlayerUrl {
                    type_: "low".into(),
                    url: "low_url".into(),
                },
                PlayerUrl {
                    type_: "full_hd".into(),
                    url: "hd_url".into(),
                },
            ],
            id: "".into(),
            vid: "".into(),
            preview: "".into(),
            height: 0,
            time_code: 0,
            show_views_counter: false,
            duration: 0,
            complete: false,
            views_counter: 0,
            default_preview: "".into(),
            failover_host: "".into(),
        };

        let post = dummy_post(vec![MediaData::OkVideo(ok_video)], true);
        let content = post.extract_content();

        assert!(
            matches!(content[0], ContentItem::OkVideo { ref url, ref video_title } if url == "hd_url" && video_title == "vid")
        );
    }

    #[test]
    fn test_extract_audio() {
        let audio = AudioData {
            show_views_counter: false,
            upload_status: None,
            complete: true,
            time_code: 0,
            size: 0,
            id: "".into(),
            url: "audio_url".into(),
            artist: "".into(),
            album: "".into(),
            file_type: "mp3".into(),
            title: "AudioTitle".into(),
            track: "TrackTitle".into(),
            duration: 0,
        };
        let post = dummy_post(vec![MediaData::Audio(audio)], true);
        let content = post.extract_content();

        assert!(
            matches!(content[0], ContentItem::Audio { ref url, ref audio_title, ref file_type }
        if url == "audio_url" && audio_title == "TrackTitle" && file_type == "mp3")
        );
    }

    #[test]
    fn test_extract_text() {
        let text = TextData {
            modificator: "bold".into(),
            content: "hello world".into(),
        };
        let post = dummy_post(vec![MediaData::Text(text)], true);
        let content = post.extract_content();

        assert!(
            matches!(content[0], ContentItem::Text { ref content, ref modificator }
        if content == "hello world" && modificator == "bold")
        );
    }

    #[test]
    fn test_extract_link() {
        let link = LinkData {
            explicit: true,
            content: "Check this".into(),
            url: "https://test.com".into(),
        };
        let post = dummy_post(vec![MediaData::Link(link)], true);
        let content = post.extract_content();

        assert!(
            matches!(content[0], ContentItem::Link { explicit: true, ref content, ref url }
        if content == "Check this" && url == "https://test.com")
        );
    }

    #[test]
    fn test_extract_unknown() {
        let post = dummy_post(vec![MediaData::Unknown], true);
        let content = post.extract_content();

        assert!(matches!(content[0], ContentItem::Unknown));
    }

    #[test]
    fn test_pick_higher_quality() {
        let urls = vec![
            PlayerUrl {
                type_: "medium".into(),
                url: "medium_url".into(),
            },
            PlayerUrl {
                type_: "ultra_hd".into(),
                url: "ultra_url".into(),
            },
            PlayerUrl {
                type_: "low".into(),
                url: "low_url".into(),
            },
        ];
        let result = pick_higher_quality_for_video(&urls);
        assert_eq!(result.unwrap(), "ultra_url");
    }

    #[test]
    fn test_pick_higher_quality_fallback() {
        let urls = vec![
            PlayerUrl {
                type_: "other".into(),
                url: "".into(),
            },
            PlayerUrl {
                type_: "weird".into(),
                url: "fallback_url".into(),
            },
        ];
        let result = pick_higher_quality_for_video(&urls);
        assert_eq!(result.unwrap(), "fallback_url");
    }
}
