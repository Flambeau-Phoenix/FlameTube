//! Desktop adapter for the portable IPTV catalogue. Only known catalogue IDs can be played.
use crate::{orchestrator::PlaybackData, state::AppState};
use flametube_iptv::{parse_m3u, Channel, Metadata};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock, RwLock};
use std::time::Duration;
use tauri::{Manager, State};

const PLAYLIST: &str = "https://iptv-org.github.io/iptv/index.category.m3u";
const SNAPSHOT: &str = include_str!("../../assets/iptv/catalog.m3u");
fn metadata() -> &'static HashMap<String, Metadata> {
    static META: OnceLock<HashMap<String, Metadata>> = OnceLock::new();
    META.get_or_init(|| {
        serde_json::from_str(include_str!("../../assets/iptv/metadata.json"))
            .expect("bundled IPTV metadata")
    })
}
fn channels() -> &'static RwLock<Vec<Channel>> {
    static STORE: OnceLock<RwLock<Vec<Channel>>> = OnceLock::new();
    STORE.get_or_init(|| RwLock::new(parse_m3u(SNAPSHOT, metadata())))
}
fn choices() -> &'static Mutex<HashMap<String, usize>> {
    static CHOICES: OnceLock<Mutex<HashMap<String, usize>>> = OnceLock::new();
    CHOICES.get_or_init(|| Mutex::new(HashMap::new()))
}
pub fn is_live(id: &str) -> bool {
    id.starts_with("iptv:")
}
pub fn next_source(id: &str) {
    *choices().lock().unwrap().entry(id.to_owned()).or_default() += 1;
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiveStatus {
    width: i64,
    height: i64,
    stream_index: usize,
}

#[tauri::command]
pub fn iptv_status(state: State<'_, Arc<AppState>>, id: String) -> Option<LiveStatus> {
    let data = playback_data(&id).ok()?;
    if state.player.current_path()? != data.stream_url {
        return None;
    }
    let (width, height) = state.player.video_dimensions()?;
    let store = channels().read().ok()?;
    let channel = store.iter().find(|c| c.id == id)?;
    let stream_index = *choices().lock().ok()?.get(&id).unwrap_or(&0) % channel.streams.len();
    Some(LiveStatus { width, height, stream_index })
}
pub fn playback_data(id: &str) -> Result<PlaybackData, String> {
    let guard = channels().read().unwrap();
    let channel = guard
        .iter()
        .find(|c| c.id == id)
        .ok_or("This live channel is no longer in the catalogue. Refresh Live TV.")?;
    let index = *choices().lock().unwrap().get(id).unwrap_or(&0);
    let stream = &channel.streams[index % channel.streams.len()];
    let mut headers = HashMap::new();
    if let Some(ua) = &stream.user_agent {
        headers.insert("User-Agent".into(), ua.clone());
    }
    if let Some(refer) = &stream.referrer {
        headers.insert("Referer".into(), refer.clone());
    }
    Ok(PlaybackData {
        video_id: id.into(),
        stream_url: stream.url.clone(),
        itag: 0,
        headers,
        expires_in_seconds: 0,
        loudness_db: None,
        playback_ping: None,
        watchtime_url: None,
        title: Some(channel.name.clone()),
        artists: Some("Live TV".into()),
        duration: None,
        thumbnail: channel.logo.clone(),
        is_video: Some(true),
        is_episode: Some(false),
        stream_client: "IPTV".into(),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalogue {
    channels: Vec<Channel>,
    source: &'static str,
    warning: Option<String>,
}

#[tauri::command]
pub async fn iptv_catalogue(
    state: State<'_, Arc<AppState>>,
    refresh: bool,
) -> Result<Catalogue, String> {
    static UPDATE: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
    let _turn = UPDATE.lock().await;
    let file = state.app.path().app_data_dir().map_err(|e| e.to_string())?.join("iptv-catalog.m3u");
    let mut source = "Bundled catalogue";
    let mut warning = None;
    let text = if refresh {
        let result = async {
            let r = crate::http::client()
                .get(PLAYLIST)
                .timeout(Duration::from_secs(30))
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?;
            if r.content_length().unwrap_or(0) > 16_000_000 {
                return Err("Catalogue is unexpectedly large".into());
            }
            let text = r.text().await.map_err(|e| e.to_string())?;
            if text.len() > 16_000_000 || !text.trim_start().starts_with("#EXTM3U") {
                return Err("Invalid catalogue response".into());
            }
            if parse_m3u(&text, metadata()).is_empty() {
                return Err("The updated catalogue contains no usable channels".into());
            }
            Ok::<_, String>(text)
        }
        .await;
        match result {
            Ok(text) => {
                let _ = std::fs::write(&file, &text);
                source = "Updated from iptv-org";
                Some(text)
            }
            Err(_) => {
                warning = Some("Could not update the catalogue. Your saved channels are still available; try again later.".into());
                std::fs::read_to_string(&file).ok()
            }
        }
    } else {
        std::fs::read_to_string(&file).ok()
    };
    if let Some(text) = text {
        let mut parsed = parse_m3u(&text, metadata());
        flametube_iptv::merge_alternates(&mut parsed, &parse_m3u(SNAPSHOT, metadata()));
        if !parsed.is_empty() {
            *channels().write().unwrap() = parsed;
            if !refresh {
                source = "Saved catalogue";
            }
        }
    }
    Ok(Catalogue { channels: channels().read().unwrap().clone(), source, warning })
}

#[tauri::command]
pub async fn iptv_play(
    state: State<'_, Arc<AppState>>,
    id: String,
    stream_index: usize,
) -> Result<(), String> {
    let channel = channels()
        .read()
        .unwrap()
        .iter()
        .find(|c| c.id == id)
        .cloned()
        .ok_or("Unknown live channel")?;
    if stream_index >= channel.streams.len() {
        return Err("Unknown channel source".into());
    }
    choices().lock().unwrap().insert(id.clone(), stream_index);
    let mut song = crate::video_play::song_from_card(&innertube::video::VideoCard {
        video_id: id,
        title: channel.name,
        channel: Some("Live TV".into()),
        ..Default::default()
    });
    song.thumbnail = channel.logo;
    state.inner().play_tracks_with_seed(vec![song], Some(0), None, Some("Live TV".into())).await;
    Ok(())
}
