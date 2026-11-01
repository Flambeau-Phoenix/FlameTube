//! Tauri commands for Video mode: regular YouTube (www.youtube.com) through the `WEB` client.
//!
//! Separate from `commands.rs` only for size. Same rules: nothing YouTube-shaped leaks past here,
//! and the UI reaches these through `ui/src/lib/api.ts` alone. Signed-out feeds work; anything that
//! writes (like, subscribe, Watch Later) needs the account and says so the same way the music side
//! does.

use std::sync::Arc;

use innertube::video::{ChannelPage, Feed, VideoCard, WatchPage};
use innertube::{Rating, VIDEO_CLIENT};
use tauri::State;

use crate::state::AppState;

type St<'a> = State<'a, Arc<AppState>>;

#[tauri::command]
pub fn playback_video_info(state: St<'_>) -> Option<(i64, i64)> {
    state.player.video_dimensions()
}

/// The Video mode client, or a message the UI can show. The identity is always present (it is
/// bundled `clients.json`), so a miss is a build bug rather than something a user did.
pub(crate) fn video_client(
    state: &Arc<AppState>,
) -> Result<&innertube::YouTubeClient, String> {
    state.clients.get(VIDEO_CLIENT).ok_or_else(|| "video client missing".into())
}

fn login_client(state: &Arc<AppState>) -> Result<&innertube::YouTubeClient, String> {
    if !state.it.is_logged_in() {
        return Err("Sign in first to use this.".into());
    }
    video_client(state)
}

/// A feed or playlist page: `browse_id` is `FEwhat_to_watch`, `FEsubscriptions`, `FEhistory`,
/// `VLWL`, a `VL…` playlist or a `UC…` channel. `params` selects a channel tab.
#[tauri::command]
pub async fn yt_feed(
    state: St<'_>,
    browse_id: String,
    params: Option<String>,
) -> Result<Feed, String> {
    let client = video_client(&state)?;
    state.it.yt_feed(client, &browse_id, params.as_deref()).await.map_err(|e| e.to_string())
}

/// The next page of a feed, a search or a watch page's related rail. `path` is `browse`, `search`
/// or `next`, whichever the page that issued `token` came from.
#[tauri::command]
pub async fn yt_more(state: St<'_>, path: String, token: String) -> Result<Feed, String> {
    let client = video_client(&state)?;
    state.it.yt_continuation(client, &path, &token).await.map_err(|e| e.to_string())
}

/// Search. `filter` is `videos`, `channels`, `playlists` or nothing. Always sent signed in: the
/// ranking is the point of signing in, and YouTube records a submitted query either way.
#[tauri::command]
pub async fn yt_search(
    state: St<'_>,
    query: String,
    filter: Option<String>,
) -> Result<Feed, String> {
    let client = video_client(&state)?;
    state.it.yt_search(client, &query, filter.as_deref()).await.map_err(|e| e.to_string())
}

/// Search-box completions. Anonymous on purpose (the site's own box sends them that way).
#[tauri::command]
pub async fn yt_suggest(query: String) -> Result<Vec<String>, String> {
    let it = innertube::InnerTube::new(innertube::Session::default(), None)
        .map_err(|e| e.to_string())?;
    it.yt_suggestions(&query).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn yt_channel(
    state: St<'_>,
    channel_id: String,
    params: Option<String>,
) -> Result<ChannelPage, String> {
    let client = video_client(&state)?;
    state.it.yt_channel(client, &channel_id, params.as_deref()).await.map_err(|e| e.to_string())
}

/// A watch page: metadata, the related rail, YouTube's own autoplay pick and, when opened from a
/// playlist, the playlist around the video.
#[tauri::command]
pub async fn yt_watch(
    state: St<'_>,
    video_id: String,
    playlist_id: Option<String>,
    index: Option<usize>,
) -> Result<WatchPage, String> {
    let client = video_client(&state)?;
    state
        .it
        .yt_watch(client, &video_id, playlist_id.as_deref(), index)
        .await
        .map_err(|e| e.to_string())
}

/// Like, dislike or clear a video's rating. Same endpoint and same mutually exclusive states as
/// the music side, sent as the WEB client.
#[tauri::command]
pub async fn yt_rate(state: St<'_>, video_id: String, rating: Rating) -> Result<(), String> {
    let client = login_client(&state)?;
    state.it.rate(client, &video_id, rating).await.map_err(|e| e.to_string())
}

/// Subscribe to a channel, or leave it.
#[tauri::command]
pub async fn yt_subscribe(
    state: St<'_>,
    channel_id: String,
    subscribed: bool,
) -> Result<(), String> {
    let client = login_client(&state)?;
    state.it.subscribe(client, &channel_id, subscribed).await.map_err(|e| e.to_string())
}

/// Save a video to Watch Later, or take it out. Watch Later is playlist `WL`, edited through the
/// same endpoint as any other playlist.
#[tauri::command]
pub async fn yt_watch_later(
    state: St<'_>,
    video_id: String,
    saved: bool,
) -> Result<(), String> {
    let client = login_client(&state)?;
    let result = if saved {
        state.it.playlist_add(client, "WL", &video_id, false).await.map(|_| ())
    } else {
        state.it.playlist_remove_video(client, "WL", &video_id).await
    };
    result.map_err(|e| e.to_string())
}

/// Play a video, or a whole playlist of them. `video_id` is the one clicked; `items` is the page's
/// list (a playlist, or just the single card), and `playlist_id` the YouTube playlist when the
/// video was opened from one. Autoplay after it continues with YouTube's own pick.
#[tauri::command]
pub async fn yt_play(
    state: St<'_>,
    video_id: String,
    items: Vec<VideoCard>,
    playlist_id: Option<String>,
) -> Result<(), String> {
    let state = state.inner().clone();
    state.play_video(video_id, items, playlist_id).await
}
