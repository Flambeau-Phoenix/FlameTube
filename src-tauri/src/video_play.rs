//! Video mode playback: an ordinary YouTube video as the thing that plays.
//!
//! Reuses the music pipeline end to end — the queue, mpv, the picture in its native window, the
//! gapless lookahead — with two differences carried on the queue and the items themselves:
//!
//! - **The picture is the point.** A Video mode item always gets its video track (unless Audio Mode
//!   is on), where a music row shows one only when it happens to be a music video.
//! - **Autoplay is YouTube's own pick**, not a music radio. The watch page names it (`autoplay`),
//!   so the tail is extended with exactly what youtube.com would have played next rather than a
//!   `RDAMVM…` radio, which for a talk or a tutorial returns songs.
//!
//! The queue records which mode seeded it, so a music radio started afterwards does not leave the
//! video logic extending it and vice versa.

use std::sync::Arc;

use innertube::video::{FeedItem, VideoCard};
use innertube::{SongItem, VIDEO_CLIENT};

use crate::db::Db;
use crate::state::AppState;

/// Queue marker: this queue came from Video mode, and autoplay continues with YouTube's own pick.
const VIDEO_SEED: &str = "YTWATCH";

/// A watch-page or search card as the queue item the player runs on.
pub fn song_from_card(card: &VideoCard) -> SongItem {
    SongItem {
        video_id: card.video_id.clone(),
        title: card.title.clone(),
        // The channel stands where an artist does: the player bar, the OS media card and the
        // queue all read `artists`, and a video has nobody else to name.
        artists: card.channel.clone().unwrap_or_default(),
        artist_id: card.channel_id.clone(),
        artist_runs: Vec::new(),
        album: None,
        album_id: None,
        duration: card.duration.clone(),
        play_count: None,
        thumbnail: Some(format!("https://i.ytimg.com/vi/{}/hqdefault.jpg", card.video_id)),
        set_video_id: None,
        added_by: None,
        added_by_avatars: Vec::new(),
        added_at: None,
        rating: None,
        queued_by: None,
        queued: false,
        queued_end: false,
        queued_from: None,
        autoplay: false,
        // Always true: a normal video is not an audio track, and the picture path keys off it.
        is_video: true,
        is_episode: false,
        counterpart: None,
        counterpart_thumbnail: None,
        counterpart_segments: Vec::new(),
        is_upload: false,
        yt: true,
        explicit: false,
        library: None,
    }
}

/// Cards out of anything that hands back a mixed feed, videos only. Used by the UI-side callers
/// through the commands; kept here so the filter lives beside the type it reads.
#[allow(dead_code)]
pub fn videos(items: &[FeedItem]) -> Vec<&VideoCard> {
    items
        .iter()
        .filter_map(|i| match i {
            FeedItem::Video(v) => Some(v),
            _ => None,
        })
        .collect()
}

/// Whether a queue seed marks Video mode.
pub(crate) fn is_video_seed(seed: &str) -> bool {
    seed == VIDEO_SEED || seed.starts_with("YTWATCH:")
}

/// Audio Mode, from the stored setting. While it is on, a Video mode play resolves no video track
/// at all: nothing is downloaded and nothing is decoded, where the browser extension this follows
/// still fetched a 144p picture.
pub(crate) fn audio_mode(db: &Db) -> bool {
    db.get_setting("audio_mode").as_deref() == Some("true")
}

/// Whether Video mode registers plays and watchtime with YouTube. Default on, like the music
/// side's history setting: a personalised home and recommendations are the reason to sign in.
pub(crate) fn history_ping(db: &Db) -> bool {
    db.get_setting("video_history_ping").as_deref() != Some("false")
}

/// Video quality cap in pixels, from the stored setting. 1080 by default: the smallest cap that is
/// still a normal picture, and the one a lightweight client should ship.
pub(crate) fn quality(db: &Db) -> i32 {
    db.get_setting("video_quality")
        .and_then(|v| v.parse::<i32>().ok())
        .filter(|h| (144..=2160).contains(h))
        .unwrap_or(1080)
}

impl AppState {
    #[allow(dead_code)]
    pub(crate) fn is_video_client(&self) -> bool {
        self.clients.get(VIDEO_CLIENT).is_some()
    }

    /// Whether the queue that is playing came from Video mode.
    pub(crate) async fn video_queue(&self) -> bool {
        self.queue_is_video().await
    }

    /// Play a video, or a playlist of them: the whole list becomes the queue unless only `video_id`
    /// was given. `playlist` is the watch page's playlist, when the video was opened from one.
    ///
    /// Autoplay continues with YouTube's pick, and a playlist continues in its own order first —
    /// both by seeding the queue with [`VIDEO_SEED`], which `extend_queue_radio` understands.
    pub async fn play_video(
        self: &Arc<Self>,
        video_id: String,
        mut items: Vec<VideoCard>,
        playlist_id: Option<String>,
    ) -> Result<(), String> {
        tracing::info!(%video_id, items = items.len(), "DBG play_video entry");
        if self.lt.is_guest().await {
            self.emit_guest_hint();
            return Ok(());
        }
        let mut start = items.iter().position(|i| i.video_id == video_id);
        if start.is_none() {
            items.insert(0, self.video_card(&video_id).await?);
            start = Some(0);
        }
        let songs: Vec<SongItem> = items.iter().map(song_from_card).collect();
        let start = start.unwrap_or(0);
        let queue_seed = match playlist_id.as_deref() {
            Some(pl) => format!("{VIDEO_SEED}:{pl}"),
            None => VIDEO_SEED.to_owned(),
        };
        let name = if playlist_id.is_some() {
            None
        } else {
            Some("Up Next".to_owned())
        };
        self.play_tracks_with_seed(songs, Some(start), Some(queue_seed), name).await;
        Ok(())
    }

    /// A single card, for a video no page handed over: search-and-pick on the watch page's own
    /// data would cost a round trip that `play_video`'s caller already made.
    async fn video_card(&self, video_id: &str) -> Result<VideoCard, String> {
        let client = self.clients.get(VIDEO_CLIENT).ok_or("video client missing")?;
        let page = self.it.yt_watch(client, video_id, None, None).await.map_err(|e| e.to_string())?;
        Ok(VideoCard {
            video_id: page.video_id.clone(),
            title: page.title.clone(),
            channel: page.channel.clone(),
            channel_id: page.channel_id.clone(),
            channel_thumbnail: page.channel_thumbnail.clone(),
            thumbnail: format!("https://i.ytimg.com/vi/{}/mqdefault.jpg", page.video_id),
            views: page.views.clone(),
            published: page.published.clone(),
            ..VideoCard::default()
        })
    }

    /// What YouTube's watch page names as next for `video_id`, as a queue item.
    pub(crate) async fn next_video_item(&self, video_id: &str) -> Option<SongItem> {
        let client = self.clients.get(VIDEO_CLIENT)?;
        let page = self.it.yt_watch(client, video_id, None, None).await.ok()?;
        let card = page.autoplay.or_else(|| {
            page.related.into_iter().find_map(|i| match i {
                FeedItem::Video(v) if !v.is_live && !v.is_short => Some(v),
                _ => None,
            })
        })?;
        if card.video_id == video_id {
            return None;
        }
        Some(song_from_card(&card))
    }

}
