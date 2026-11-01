//! Regular YouTube (www.youtube.com, the `WEB` client): the feeds, search, watch page and channel
//! pages behind the app's Video mode.
//!
//! YouTube Music's parsers ([`crate::models::browse`]) read `music*Renderer` shapes only. The web
//! site sends a different family: `videoRenderer` and its grid/compact/playlist cousins, and since
//! 2025 the `lockupViewModel` that is replacing all of them. Both are read here, by walking the
//! response for any renderer we know rather than following one fixed path, because YouTube moves
//! the same cards between `richGridRenderer`, `sectionListRenderer` and `itemSectionRenderer`
//! wrappers from one A/B bucket to the next.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::clients::YouTubeClient;
use crate::models::context::Context;
use crate::transport::{Error, InnerTube};

pub const HOME: &str = "FEwhat_to_watch";
pub const SUBSCRIPTIONS: &str = "FEsubscriptions";
pub const HISTORY: &str = "FEhistory";
pub const WATCH_LATER: &str = "VLWL";
pub const LIKED: &str = "VLLL";
pub const PLAYLISTS: &str = "FEplaylist_aggregation";
pub const CHANNELS: &str = "FEchannels";

/// Search filters (`params`), as the web site's own filter menu sends them.
pub fn search_filter(kind: &str) -> Option<&'static str> {
    match kind {
        "videos" => Some("EgIQAQ%3D%3D"),
        "channels" => Some("EgIQAg%3D%3D"),
        "playlists" => Some("EgIQAw%3D%3D"),
        _ => None,
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct VideoCard {
    pub video_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_thumbnail: Option<String>,
    pub thumbnail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub views: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(default)]
    pub is_live: bool,
    #[serde(default)]
    pub is_short: bool,
    /// Index text inside a playlist ("3"), or the selected row of a watch-page playlist panel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub selected: bool,
    /// `playlistSetVideoId`, on rows of a playlist you own: what removing the row needs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_video_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChannelCard {
    pub channel_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PlaylistCard {
    pub playlist_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_video_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum FeedItem {
    Video(VideoCard),
    Channel(ChannelCard),
    Playlist(PlaylistCard),
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeedSection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub items: Vec<FeedItem>,
}

/// One page of a feed, a search, a playlist or a channel tab.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Feed {
    pub sections: Vec<FeedSection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<String>,
    /// The page's own title where it has one (a playlist, a channel tab).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelTab {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<String>,
    pub selected: bool,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelPage {
    pub channel_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub handle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_count: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed: Option<bool>,
    pub tabs: Vec<ChannelTab>,
    pub feed: Feed,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchPlaylist {
    pub playlist_id: String,
    pub title: String,
    pub items: Vec<VideoCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_index: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WatchPage {
    pub video_id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribers: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscribed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub views: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likes: Option<String>,
    /// `LIKE` / `DISLIKE` / `INDIFFERENT`, signed in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub like_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment_count: Option<String>,
    pub related: Vec<FeedItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_continuation: Option<String>,
    /// What YouTube's own autoplay would play next.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub autoplay: Option<VideoCard>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playlist: Option<WatchPlaylist>,
}

// --- endpoints ------------------------------------------------------------------------------

impl InnerTube {
    pub async fn yt_post(
        &self,
        client: &YouTubeClient,
        path: &str,
        mut extra: serde_json::Map<String, Value>,
    ) -> Result<Value, Error> {
        let context = serde_json::to_value::<Context>(self.context_for(client))?;
        extra.insert("context".into(), context);
        self.post(path, client, &Value::Object(extra), true).await
    }

    /// A `browse` page: a feed id (`FEwhat_to_watch`), a playlist (`VL…`) or a channel (`UC…`).
    pub async fn yt_browse_raw(
        &self,
        client: &YouTubeClient,
        browse_id: &str,
        params: Option<&str>,
    ) -> Result<Value, Error> {
        let mut body = serde_json::Map::new();
        body.insert("browseId".into(), browse_id.into());
        if let Some(p) = params {
            body.insert("params".into(), p.into());
        }
        self.yt_post(client, "browse", body).await
    }

    /// A feed or playlist page, parsed.
    pub async fn yt_feed(
        &self,
        client: &YouTubeClient,
        browse_id: &str,
        params: Option<&str>,
    ) -> Result<Feed, Error> {
        let value = self.yt_browse_raw(client, browse_id, params).await?;
        if browse_id == HOME && is_logged_out_home(&value) && self.is_logged_in() {
            tracing::debug!("video home came back as the signed-out page");
        }
        Ok(parse_feed(&value))
    }

    /// The next page of anything a [`Feed`] came from: `browse`, `search` and `next` all page
    /// through the same `continuation` body, only on their own path.
    pub async fn yt_continuation(
        &self,
        client: &YouTubeClient,
        path: &str,
        token: &str,
    ) -> Result<Feed, Error> {
        let path = match path {
            "search" | "next" => path,
            _ => "browse",
        };
        let mut body = serde_json::Map::new();
        body.insert("continuation".into(), token.into());
        let value = self.yt_post(client, path, body).await?;
        Ok(parse_feed(&value))
    }

    pub async fn yt_search(
        &self,
        client: &YouTubeClient,
        query: &str,
        filter: Option<&str>,
    ) -> Result<Feed, Error> {
        let mut body = serde_json::Map::new();
        body.insert("query".into(), query.into());
        if let Some(p) = filter.and_then(search_filter) {
            body.insert("params".into(), p.into());
        }
        let value = self.yt_post(client, "search", body).await?;
        Ok(parse_feed(&value))
    }

    /// Typeahead from YouTube's suggest service. Anonymous: it is the request the site's own search
    /// box fires per keystroke and carries nothing about the account.
    pub async fn yt_suggestions(&self, query: &str) -> Result<Vec<String>, Error> {
        let url = format!(
            "https://suggestqueries-clients6.youtube.com/complete/search?client=youtube&ds=yt&xhr=t&q={}",
            urlencoding::encode(query)
        );
        let text = self.http_get_text(&url).await?;
        Ok(parse_suggestions(&text))
    }

    pub async fn yt_channel(
        &self,
        client: &YouTubeClient,
        channel_id: &str,
        params: Option<&str>,
    ) -> Result<ChannelPage, Error> {
        let value = self.yt_browse_raw(client, channel_id, params).await?;
        Ok(parse_channel(&value, channel_id))
    }

    pub async fn yt_watch(
        &self,
        client: &YouTubeClient,
        video_id: &str,
        playlist_id: Option<&str>,
        index: Option<usize>,
    ) -> Result<WatchPage, Error> {
        let mut body = serde_json::Map::new();
        body.insert("videoId".into(), video_id.into());
        if let Some(p) = playlist_id {
            body.insert("playlistId".into(), p.into());
        }
        if let Some(i) = index {
            body.insert("playlistIndex".into(), i.into());
        }
        let value = self.yt_post(client, "next", body).await?;
        Ok(parse_watch(&value, video_id))
    }
}

// --- parsing --------------------------------------------------------------------------------

/// Text of a YouTube text object: `simpleText`, `runs`, or a view model's `content`.
pub(crate) fn text(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.to_owned()).filter(|s| !s.is_empty());
    }
    if let Some(s) = v.get("simpleText").and_then(Value::as_str) {
        return Some(s.to_owned()).filter(|s| !s.is_empty());
    }
    if let Some(s) = v.get("content").and_then(Value::as_str) {
        return Some(s.to_owned()).filter(|s| !s.is_empty());
    }
    let runs = v.get("runs")?.as_array()?;
    let s: String = runs.iter().filter_map(|r| r.get("text").and_then(Value::as_str)).collect();
    Some(s).filter(|s| !s.is_empty())
}

fn text_at(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(text)
}

fn https(url: &str) -> String {
    if let Some(rest) = url.strip_prefix("//") {
        format!("https://{rest}")
    } else {
        url.to_owned()
    }
}

/// The thumbnail closest to `want` pixels wide from a `thumbnails` / `sources` list.
fn pick_thumb(list: &Value, want: i64) -> Option<String> {
    let arr = list.as_array()?;
    arr.iter()
        .filter_map(|t| {
            let url = t.get("url")?.as_str()?;
            let w = t.get("width").and_then(Value::as_i64).unwrap_or(0);
            Some((url, w))
        })
        .min_by_key(|(_, w)| if *w == 0 { i64::MAX / 2 } else { (w - want).abs() })
        .map(|(u, _)| https(u))
}

fn thumbs_of(v: &Value, want: i64) -> Option<String> {
    v.pointer("/thumbnail/thumbnails")
        .or_else(|| v.pointer("/thumbnails/0/thumbnails"))
        .or_else(|| v.get("thumbnails"))
        .and_then(|t| pick_thumb(t, want))
}

/// Small, light card art for a video: YouTube's 320×180 `mqdefault`, whatever the response offered.
pub fn video_thumb(video_id: &str) -> String {
    format!("https://i.ytimg.com/vi/{video_id}/mqdefault.jpg")
}

/// The first `browseId` starting with `UC` anywhere under `v`.
fn channel_id_in(v: &Value) -> Option<String> {
    match v {
        Value::Object(map) => {
            if let Some(id) = map
                .get("browseEndpoint")
                .and_then(|b| b.get("browseId"))
                .and_then(Value::as_str)
                .filter(|id| id.starts_with("UC"))
            {
                return Some(id.to_owned());
            }
            map.values().find_map(channel_id_in)
        }
        Value::Array(arr) => arr.iter().find_map(channel_id_in),
        _ => None,
    }
}

/// Every value stored under `key`, depth first, in document order.
pub(crate) fn find_all<'a>(v: &'a Value, key: &str) -> Vec<&'a Value> {
    let mut out = Vec::new();
    fn walk<'a>(v: &'a Value, key: &str, out: &mut Vec<&'a Value>) {
        match v {
            Value::Object(map) => {
                for (k, child) in map {
                    if k == key {
                        out.push(child);
                    }
                    walk(child, key, out);
                }
            }
            Value::Array(arr) => arr.iter().for_each(|c| walk(c, key, out)),
            _ => {}
        }
    }
    walk(v, key, &mut out);
    out
}

fn find_first<'a>(v: &'a Value, key: &str) -> Option<&'a Value> {
    match v {
        Value::Object(map) => {
            if let Some(found) = map.get(key) {
                return Some(found);
            }
            map.values().find_map(|c| find_first(c, key))
        }
        Value::Array(arr) => arr.iter().find_map(|c| find_first(c, key)),
        _ => None,
    }
}

fn duration_like(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit() || c == ':') && s.contains(':')
}

fn is_live_style(v: &Value) -> bool {
    find_all(v, "style").iter().chain(find_all(v, "badgeStyle").iter()).any(|s| {
        s.as_str().is_some_and(|s| s.contains("LIVE"))
    })
}

fn video_renderer(r: &Value) -> Option<VideoCard> {
    let video_id = r.get("videoId")?.as_str()?.to_owned();
    let title = text_at(r, "title").or_else(|| text_at(r, "headline")).unwrap_or_default();
    let byline = r
        .get("ownerText")
        .or_else(|| r.get("longBylineText"))
        .or_else(|| r.get("shortBylineText"));
    let duration = text_at(r, "lengthText").or_else(|| {
        find_all(r, "thumbnailOverlayTimeStatusRenderer")
            .into_iter()
            .find_map(|o| text_at(o, "text"))
            .filter(|t| duration_like(t))
    });
    let is_live = r
        .get("badges")
        .is_some_and(|b| b.to_string().contains("LIVE"))
        || r.get("thumbnailOverlays").is_some_and(is_live_style);
    Some(VideoCard {
        thumbnail: video_thumb(&video_id),
        channel: byline.and_then(text),
        channel_id: byline.and_then(channel_id_in),
        channel_thumbnail: r
            .pointer("/channelThumbnailSupportedRenderers/channelThumbnailWithLinkRenderer")
            .or_else(|| r.get("channelThumbnail"))
            .and_then(|c| thumbs_of(c, 68)),
        duration,
        views: text_at(r, "shortViewCountText").or_else(|| text_at(r, "viewCountText")),
        published: text_at(r, "publishedTimeText"),
        is_live,
        is_short: false,
        index: text_at(r, "index").or_else(|| text_at(r, "indexText")),
        selected: r.get("selected").and_then(Value::as_bool).unwrap_or(false),
        set_video_id: r.get("setVideoId").and_then(Value::as_str).map(str::to_owned),
        video_id,
        title,
    })
}

fn reel_item(r: &Value) -> Option<VideoCard> {
    let video_id = r
        .get("videoId")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| find_first(r, "reelWatchEndpoint")?.get("videoId")?.as_str().map(str::to_owned))?;
    let title = text_at(r, "headline")
        .or_else(|| r.pointer("/overlayMetadata/primaryText").and_then(text))
        .unwrap_or_default();
    Some(VideoCard {
        thumbnail: video_thumb(&video_id),
        views: text_at(r, "viewCountText")
            .or_else(|| r.pointer("/overlayMetadata/secondaryText").and_then(text)),
        is_short: true,
        video_id,
        title,
        ..Default::default()
    })
}

/// `lockupViewModel`: the 2025 card for videos and playlists alike.
fn lockup(r: &Value) -> Option<FeedItem> {
    let content_id = r.get("contentId")?.as_str()?.to_owned();
    let kind = r.get("contentType").and_then(Value::as_str).unwrap_or_default();
    let meta = r.pointer("/metadata/lockupMetadataViewModel");
    let title = meta.and_then(|m| m.get("title")).and_then(text).unwrap_or_default();
    let rows: Vec<Vec<String>> = meta
        .and_then(|m| m.pointer("/metadata/contentMetadataViewModel/metadataRows"))
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .map(|row| {
                    row.get("metadataParts")
                        .and_then(Value::as_array)
                        .map(|parts| {
                            parts.iter().filter_map(|p| p.get("text").and_then(text)).collect()
                        })
                        .unwrap_or_default()
                })
                .collect()
        })
        .unwrap_or_default();
    let image = r.get("contentImage");
    let badges: Vec<String> = image
        .map(|i| find_all(i, "thumbnailBadgeViewModel"))
        .unwrap_or_default()
        .into_iter()
        .filter_map(|b| b.get("text").and_then(text))
        .collect();
    let channel_id = meta.and_then(channel_id_in);
    if kind.contains("PLAYLIST") || kind.contains("PODCAST") {
        let first_video_id = find_first(r, "watchEndpoint")
            .and_then(|w| w.get("videoId"))
            .and_then(Value::as_str)
            .map(str::to_owned);
        let thumb = image.and_then(|i| find_first(i, "sources")).and_then(|s| pick_thumb(s, 480));
        return Some(FeedItem::Playlist(PlaylistCard {
            playlist_id: content_id,
            title,
            thumbnail: thumb,
            count: badges.into_iter().find(|b| b.chars().any(|c| c.is_ascii_digit())),
            channel: rows.first().and_then(|r| r.first()).cloned(),
            first_video_id,
        }));
    }
    if !kind.contains("VIDEO") && !kind.is_empty() {
        return None;
    }
    let is_live = image.is_some_and(is_live_style)
        || badges.iter().any(|b| b.eq_ignore_ascii_case("live"));
    // Row 0 is the channel (absent on a channel's own pages), the last one views · date.
    let (channel, stats) = match rows.len() {
        0 => (None, Vec::new()),
        1 if channel_id.is_none() => (None, rows[0].clone()),
        1 => (rows[0].first().cloned(), Vec::new()),
        _ => (rows[0].first().cloned(), rows[rows.len() - 1].clone()),
    };
    Some(FeedItem::Video(VideoCard {
        thumbnail: video_thumb(&content_id),
        channel,
        channel_id,
        channel_thumbnail: meta
            .and_then(|m| m.get("image"))
            .and_then(|i| find_first(i, "sources"))
            .and_then(|s| pick_thumb(s, 68)),
        duration: badges.iter().find(|b| duration_like(b)).cloned(),
        views: stats.first().cloned(),
        published: stats.get(1).cloned(),
        is_live,
        video_id: content_id,
        title,
        ..Default::default()
    }))
}

fn channel_renderer(r: &Value) -> Option<ChannelCard> {
    let channel_id = r.get("channelId")?.as_str()?.to_owned();
    // Since 2024 `videoCountText` carries the subscriber count and `subscriberCountText` the @handle.
    let a = text_at(r, "subscriberCountText");
    let b = text_at(r, "videoCountText");
    let (handle, subscribers) = match (&a, &b) {
        (Some(h), s) if h.starts_with('@') => (Some(h.clone()), s.clone()),
        _ => (None, a.or(b)),
    };
    Some(ChannelCard {
        title: text_at(r, "title").unwrap_or_default(),
        thumbnail: thumbs_of(r, 88),
        subscribers,
        handle,
        channel_id,
    })
}

fn playlist_renderer(r: &Value) -> Option<PlaylistCard> {
    let playlist_id = r.get("playlistId")?.as_str()?.to_owned();
    let first_video_id = r
        .pointer("/navigationEndpoint/watchEndpoint/videoId")
        .and_then(Value::as_str)
        .map(str::to_owned);
    Some(PlaylistCard {
        title: text_at(r, "title").unwrap_or_default(),
        thumbnail: thumbs_of(r, 480)
            .or_else(|| first_video_id.as_deref().map(video_thumb)),
        count: text_at(r, "videoCountShortText")
            .or_else(|| r.get("videoCount").and_then(Value::as_str).map(str::to_owned))
            .or_else(|| text_at(r, "videoCountText")),
        channel: text_at(r, "shortBylineText").or_else(|| text_at(r, "longBylineText")),
        playlist_id,
        first_video_id,
    })
}

/// One card, if `key` names a renderer we read.
fn item_for(key: &str, r: &Value) -> Option<Option<FeedItem>> {
    let item = match key {
        "videoRenderer" | "gridVideoRenderer" | "compactVideoRenderer" | "playlistVideoRenderer"
        | "playlistPanelVideoRenderer" | "videoWithContextRenderer" => {
            video_renderer(r).map(FeedItem::Video)
        }
        "reelItemRenderer" | "shortsLockupViewModel" => reel_item(r).map(FeedItem::Video),
        "lockupViewModel" => lockup(r),
        "channelRenderer" | "gridChannelRenderer" => channel_renderer(r).map(FeedItem::Channel),
        "playlistRenderer" | "gridPlaylistRenderer" | "compactPlaylistRenderer" => {
            playlist_renderer(r).map(FeedItem::Playlist)
        }
        _ => return None,
    };
    Some(item)
}

/// `continuationItemViewModel` nests a `continuationCommand` inside another one, and only the
/// inner one carries the token.
fn continuation_token(c: &Value) -> Option<String> {
    find_all(c, "continuationCommand")
        .into_iter()
        .find_map(|cc| cc.get("token").and_then(Value::as_str))
        .map(str::to_owned)
}

/// Section titles we skip whole: they hold things a lightweight feed has no use for.
fn skip_section(r: &Value) -> bool {
    // Ads never come through the API we call, but promoted shelves and "Breaking news" do.
    matches!(
        r.get("targetId").and_then(Value::as_str),
        Some("search-feed-ad") | Some("promoted")
    )
}

struct Walk {
    sections: Vec<FeedSection>,
    continuation: Option<String>,
}

impl Walk {
    fn push(&mut self, item: FeedItem) {
        if self.sections.is_empty() {
            self.sections.push(FeedSection::default());
        }
        let last = self.sections.last_mut().unwrap();
        let dup = |a: &FeedItem| match (a, &item) {
            (FeedItem::Video(a), FeedItem::Video(b)) => a.video_id == b.video_id,
            _ => false,
        };
        if !last.items.iter().any(dup) {
            last.items.push(item);
        }
    }

    fn walk(&mut self, v: &Value) {
        match v {
            Value::Object(map) => {
                for (k, child) in map {
                    if let Some(item) = item_for(k, child) {
                        if let Some(item) = item {
                            self.push(item);
                        }
                        continue;
                    }
                    match k.as_str() {
                        // The first one wins: on a playlist page the rows' own spinner comes
                        // before the section list's.
                        "continuationItemRenderer" | "continuationItemViewModel" => {
                            if self.continuation.is_none() {
                                self.continuation = continuation_token(child);
                            }
                            continue;
                        }
                        // Ads ride along in the API whatever blocks them in a browser.
                        "adSlotRenderer" | "searchPyvRenderer" | "promotedSparklesWebRenderer"
                        | "inFeedAdLayoutRenderer" | "promotedVideoRenderer"
                        | "compactPromotedVideoRenderer" | "adSlotAndLayoutRenderer"
                        | "brandVideoShelfRenderer" | "statementBannerRenderer" => continue,
                        // Watch-page side panels and the like: not part of a feed.
                        "engagementPanels" | "header" | "topbar" | "frameworkUpdates"
                        | "responseContext" | "sidebar" => {
                            continue
                        }
                        "itemSectionRenderer" | "shelfRenderer" | "richSectionRenderer"
                        | "reelShelfRenderer" | "richShelfRenderer" => {
                            if skip_section(child) {
                                continue;
                            }
                            let title = child
                                .pointer("/header/itemSectionHeaderRenderer/title")
                                .or_else(|| child.get("title"))
                                .or_else(|| child.pointer("/content/richShelfRenderer/title"))
                                .and_then(text);
                            if title.is_some() {
                                self.sections.push(FeedSection { title, items: Vec::new() });
                            }
                            self.walk(child);
                            continue;
                        }
                        _ => {}
                    }
                    self.walk(child);
                }
            }
            Value::Array(arr) => arr.iter().for_each(|c| self.walk(c)),
            _ => {}
        }
    }
}

/// Any feed-shaped response: browse, search, a continuation page, a playlist.
pub fn parse_feed(root: &Value) -> Feed {
    let mut w = Walk { sections: Vec::new(), continuation: None };
    // A channel or playlist page has tabs; only the selected one holds this page's content.
    let selected_tab = find_all(root, "tabRenderer")
        .into_iter()
        .find(|t| t.get("selected").and_then(Value::as_bool) == Some(true) && t.get("content").is_some());
    match selected_tab {
        Some(tab) => w.walk(tab.get("content").unwrap()),
        None => {
            if let Some(c) = root.get("contents") {
                w.walk(c);
            }
            for key in [
                "onResponseReceivedActions",
                "onResponseReceivedCommands",
                "onResponseReceivedEndpoints",
                "continuationContents",
            ] {
                if let Some(c) = root.get(key) {
                    w.walk(c);
                }
            }
        }
    }
    w.sections.retain(|s| !s.items.is_empty());
    let title = root
        .pointer("/metadata/playlistMetadataRenderer/title")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| root.pointer("/header/pageHeaderRenderer/pageTitle").and_then(Value::as_str).map(str::to_owned));
    Feed { sections: w.sections, continuation: w.continuation, title }
}

/// The signed-out home is a prompt to search, not a feed.
fn is_logged_out_home(root: &Value) -> bool {
    find_first(root, "feedNudgeRenderer").is_some()
}

fn parse_suggestions(body: &str) -> Vec<String> {
    // `window.google.ac.h([...])` wrapped JSON (xhr=t answers bare JSON). Element [1] is a list of
    // `[suggestion, 0, [..]]`.
    let start = body.find('[').unwrap_or(0);
    let end = body.rfind(']').map(|e| e + 1).unwrap_or(body.len());
    let Ok(v) = serde_json::from_str::<Value>(&body[start..end]) else { return Vec::new() };
    v.get(1)
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|s| s.get(0).and_then(Value::as_str).map(str::to_owned))
                .take(10)
                .collect()
        })
        .unwrap_or_default()
}

fn subscribed_in(root: &Value) -> Option<bool> {
    find_all(root, "subscriptionStateEntity")
        .into_iter()
        .find_map(|e| e.get("subscribed").and_then(Value::as_bool))
        .or_else(|| {
            find_all(root, "subscribeButtonRenderer")
                .into_iter()
                .find_map(|s| s.get("subscribed").and_then(Value::as_bool))
        })
}

pub fn parse_channel(root: &Value, channel_id: &str) -> ChannelPage {
    let meta = root.pointer("/metadata/channelMetadataRenderer");
    let header = root.get("header");
    let vm = header.and_then(|h| find_first(h, "pageHeaderViewModel"));
    let rows: Vec<String> = vm
        .and_then(|v| v.pointer("/metadata/contentMetadataViewModel/metadataRows"))
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .flat_map(|r| {
                    r.get("metadataParts")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default()
                })
                .filter_map(|p| p.get("text").and_then(text))
                .collect()
        })
        .unwrap_or_default();
    let handle = rows.iter().find(|r| r.starts_with('@')).cloned().or_else(|| {
        header.and_then(|h| find_first(h, "channelHandleText")).and_then(text)
    });
    let subscribers = rows
        .iter()
        .find(|r| r.to_lowercase().contains("subscriber"))
        .cloned()
        .or_else(|| header.and_then(|h| find_first(h, "subscriberCountText")).and_then(text));
    let video_count = rows.iter().find(|r| r.to_lowercase().contains("video")).cloned();
    let avatar = meta
        .and_then(|m| m.pointer("/avatar/thumbnails"))
        .and_then(|t| pick_thumb(t, 176))
        .or_else(|| {
            vm.and_then(|v| v.get("image")).and_then(|i| find_first(i, "sources")).and_then(|s| pick_thumb(s, 176))
        });
    let banner = vm
        .and_then(|v| v.get("banner"))
        .and_then(|b| find_first(b, "sources"))
        .and_then(|s| pick_thumb(s, 1280))
        .or_else(|| header.and_then(|h| h.pointer("/c4TabbedHeaderRenderer/banner/thumbnails")).and_then(|t| pick_thumb(t, 1280)));
    let tabs = find_all(root, "tabRenderer")
        .into_iter()
        .filter_map(|t| {
            Some(ChannelTab {
                title: t.get("title")?.as_str()?.to_owned(),
                params: t
                    .pointer("/endpoint/browseEndpoint/params")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                selected: t.get("selected").and_then(Value::as_bool).unwrap_or(false),
            })
        })
        .collect();
    ChannelPage {
        channel_id: meta
            .and_then(|m| m.get("externalId"))
            .and_then(Value::as_str)
            .unwrap_or(channel_id)
            .to_owned(),
        title: meta
            .and_then(|m| m.get("title"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or_else(|| vm.and_then(|v| v.get("title")).and_then(|t| find_first(t, "content")).and_then(text))
            .unwrap_or_default(),
        avatar,
        banner,
        handle,
        subscribers,
        video_count,
        description: meta
            .and_then(|m| m.get("description"))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .filter(|d| !d.is_empty()),
        subscribed: subscribed_in(root),
        tabs,
        feed: parse_feed(root),
    }
}

pub fn parse_watch(root: &Value, video_id: &str) -> WatchPage {
    let two = root.pointer("/contents/twoColumnWatchNextResults");
    let primary = two.and_then(|t| find_first(t, "videoPrimaryInfoRenderer"));
    let secondary = two.and_then(|t| find_first(t, "videoSecondaryInfoRenderer"));
    let owner = secondary.and_then(|s| s.pointer("/owner/videoOwnerRenderer"));

    let description = secondary
        .and_then(|s| s.get("attributedDescription"))
        .and_then(text)
        .or_else(|| secondary.and_then(|s| s.get("description")).and_then(text))
        .or_else(|| {
            root.get("engagementPanels")
                .and_then(|p| find_first(p, "attributedDescriptionBodyText"))
                .and_then(text)
        });

    let likes = primary.and_then(like_count);
    let like_status = find_all(root, "likeStatusEntity")
        .into_iter()
        .find_map(|e| e.get("likeStatus").and_then(Value::as_str))
        .map(str::to_owned);

    let comment_count = root.get("engagementPanels").and_then(|panels| {
        panels.as_array()?.iter().find_map(|p| {
            let r = p.get("engagementPanelSectionListRenderer")?;
            if r.get("panelIdentifier").and_then(Value::as_str)
                != Some("engagement-panel-comments-section")
            {
                return None;
            }
            r.pointer("/header/engagementPanelTitleHeaderRenderer/contextualInfo").and_then(text)
        })
    });

    let mut related = Walk { sections: Vec::new(), continuation: None };
    if let Some(sec) = two.and_then(|t| t.pointer("/secondaryResults/secondaryResults")) {
        related.walk(sec);
    }
    let related_items: Vec<FeedItem> =
        related.sections.into_iter().flat_map(|s| s.items).collect();

    let playlist = two.and_then(|t| t.pointer("/playlist/playlist")).map(|p| {
        let items: Vec<VideoCard> = p
            .get("contents")
            .and_then(Value::as_array)
            .map(|c| {
                c.iter()
                    .filter_map(|i| i.get("playlistPanelVideoRenderer"))
                    .filter_map(video_renderer)
                    .collect()
            })
            .unwrap_or_default();
        WatchPlaylist {
            playlist_id: p.get("playlistId").and_then(Value::as_str).unwrap_or_default().to_owned(),
            title: text_at(p, "title").unwrap_or_default(),
            current_index: p
                .get("currentIndex")
                .and_then(Value::as_u64)
                .map(|i| i as usize)
                .or_else(|| items.iter().position(|i| i.selected)),
            items,
        }
    });

    // Autoplay: in a playlist the next row, otherwise what the site's own autoplay set names.
    let autoplay = playlist
        .as_ref()
        .and_then(|p| p.items.get(p.current_index? + 1).cloned())
        .or_else(|| {
            let id = two
                .and_then(|t| t.pointer("/autoplay/autoplay/sets/0/autoplayVideo/watchEndpoint/videoId"))
                .and_then(Value::as_str)?;
            related_items.iter().find_map(|i| match i {
                FeedItem::Video(v) if v.video_id == id => Some(v.clone()),
                _ => None,
            }).or_else(|| Some(VideoCard {
                video_id: id.to_owned(),
                thumbnail: video_thumb(id),
                ..Default::default()
            }))
        })
        .or_else(|| {
            related_items.iter().find_map(|i| match i {
                FeedItem::Video(v) if !v.is_live && !v.is_short => Some(v.clone()),
                _ => None,
            })
        });

    WatchPage {
        video_id: video_id.to_owned(),
        title: primary.and_then(|p| text_at(p, "title")).unwrap_or_default(),
        channel: owner.and_then(|o| text_at(o, "title")),
        channel_id: owner.and_then(channel_id_in),
        channel_thumbnail: owner.and_then(|o| thumbs_of(o, 88)),
        subscribers: owner.and_then(|o| text_at(o, "subscriberCountText")),
        subscribed: subscribed_in(root),
        views: primary.and_then(|p| {
            p.pointer("/viewCount/videoViewCountRenderer/viewCount")
                .and_then(text)
                .or_else(|| p.pointer("/viewCount/videoViewCountRenderer/shortViewCount").and_then(text))
        }),
        published: primary.and_then(|p| text_at(p, "dateText").or_else(|| text_at(p, "relativeDateText"))),
        description,
        likes,
        like_status,
        comment_count,
        related: related_items,
        related_continuation: related.continuation,
        autoplay,
        playlist,
    }
}

/// The like button's label ("12K"): the `buttonViewModel` whose icon is the thumbs-up.
fn like_count(primary: &Value) -> Option<String> {
    fn walk(v: &Value) -> Option<String> {
        match v {
            Value::Object(map) => {
                if map.get("iconName").and_then(Value::as_str) == Some("LIKE") {
                    if let Some(t) = map.get("title").and_then(Value::as_str) {
                        if t.chars().any(|c| c.is_ascii_digit()) {
                            return Some(t.to_owned());
                        }
                    }
                }
                map.values().find_map(walk)
            }
            Value::Array(arr) => arr.iter().find_map(walk),
            _ => None,
        }
    }
    walk(primary)
}

/// The comments continuation on a web watch page: the token on the comment section's spinner.
pub fn parse_web_comments_token(root: &Value) -> Option<String> {
    find_all(root, "itemSectionRenderer").into_iter().find_map(|s| {
        if s.get("sectionIdentifier").and_then(Value::as_str) != Some("comment-item-section") {
            return None;
        }
        s.get("contents")?.as_array()?.iter().find_map(|c| {
            c.get("continuationItemRenderer").and_then(continuation_token)
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_a_video_renderer() {
        let v = json!({"contents": {"twoColumnSearchResultsRenderer": {"primaryContents": {
            "sectionListRenderer": {"contents": [{"itemSectionRenderer": {"contents": [
                {"videoRenderer": {
                    "videoId": "abc123def45",
                    "title": {"runs": [{"text": "A "}, {"text": "video"}]},
                    "ownerText": {"runs": [{"text": "Chan", "navigationEndpoint": {"browseEndpoint": {"browseId": "UCxyz"}}}]},
                    "lengthText": {"simpleText": "12:34"},
                    "viewCountText": {"simpleText": "1,234 views"},
                    "publishedTimeText": {"simpleText": "3 days ago"}
                }},
                {"continuationItemRenderer": {"continuationEndpoint": {"continuationCommand": {"token": "TOK"}}}}
            ]}}]}}}}});
        let f = parse_feed(&v);
        assert_eq!(f.continuation.as_deref(), Some("TOK"));
        let FeedItem::Video(c) = &f.sections[0].items[0] else { panic!() };
        assert_eq!(c.title, "A video");
        assert_eq!(c.channel.as_deref(), Some("Chan"));
        assert_eq!(c.channel_id.as_deref(), Some("UCxyz"));
        assert_eq!(c.duration.as_deref(), Some("12:34"));
        assert_eq!(c.thumbnail, "https://i.ytimg.com/vi/abc123def45/mqdefault.jpg");
    }

    #[test]
    fn reads_a_lockup_view_model() {
        let v = json!({"contents": {"richGridRenderer": {"contents": [{"richItemRenderer": {"content": {
            "lockupViewModel": {
                "contentId": "vid00000001",
                "contentType": "LOCKUP_CONTENT_TYPE_VIDEO",
                "contentImage": {"thumbnailViewModel": {"overlays": [{"thumbnailOverlayBadgeViewModel": {
                    "thumbnailBadges": [{"thumbnailBadgeViewModel": {"text": "4:05"}}]}}]}},
                "metadata": {"lockupMetadataViewModel": {
                    "title": {"content": "Lockup title"},
                    "metadata": {"contentMetadataViewModel": {"metadataRows": [
                        {"metadataParts": [{"text": {"content": "Some Channel", "commandRuns": [{"onTap": {"innertubeCommand": {"browseEndpoint": {"browseId": "UCabc"}}}}]}}]},
                        {"metadataParts": [{"text": {"content": "10K views"}}, {"text": {"content": "1 year ago"}}]}
                    ]}}
                }}
            }}}}]}}});
        let f = parse_feed(&v);
        let FeedItem::Video(c) = &f.sections[0].items[0] else { panic!() };
        assert_eq!(c.title, "Lockup title");
        assert_eq!(c.channel.as_deref(), Some("Some Channel"));
        assert_eq!(c.channel_id.as_deref(), Some("UCabc"));
        assert_eq!(c.duration.as_deref(), Some("4:05"));
        assert_eq!(c.views.as_deref(), Some("10K views"));
        assert_eq!(c.published.as_deref(), Some("1 year ago"));
    }

    #[test]
    fn finds_the_web_comments_token() {
        let v = json!({"contents": {"twoColumnWatchNextResults": {"results": {"results": {"contents": [
            {"itemSectionRenderer": {"sectionIdentifier": "comment-item-section", "contents": [
                {"continuationItemRenderer": {"continuationEndpoint": {"continuationCommand": {"token": "CMT"}}}}
            ]}}
        ]}}}}});
        assert_eq!(parse_web_comments_token(&v).as_deref(), Some("CMT"));
    }

    #[test]
    fn parses_suggestions() {
        let body = r#"window.google.ac.h(["cats",[["cats videos",0,[512]],["cats funny",0]],{"k":1}])"#;
        assert_eq!(parse_suggestions(body), vec!["cats videos", "cats funny"]);
    }
}
