//! Debug tool: run Video mode's parsers against live www.youtube.com responses, signed out.
//!   cargo run -p innertube --example video_probe -- [query] [videoId] [channelId]

use innertube::video::FeedItem;
use innertube::{Clients, InnerTube, Session, VIDEO_CLIENT};

fn summary(items: &[FeedItem]) -> String {
    let (mut v, mut c, mut p, mut s) = (0, 0, 0, 0);
    for i in items {
        match i {
            FeedItem::Video(x) if x.is_short => s += 1,
            FeedItem::Video(_) => v += 1,
            FeedItem::Channel(_) => c += 1,
            FeedItem::Playlist(_) => p += 1,
        }
    }
    format!("{v} videos, {s} shorts, {c} channels, {p} playlists")
}

#[tokio::main]
async fn main() {
    let mut args = std::env::args().skip(1);
    let query = args.next().unwrap_or_else(|| "lofi hip hop".into());
    let video_id = args.next().unwrap_or_else(|| "jfKfPfyJRdk".into());
    let channel_id = args.next().unwrap_or_else(|| "UCSJ4gkVC6NrvII8umztf0Ow".into());

    let it = InnerTube::new(Session::default(), None).unwrap();
    let vd = it.fetch_visitor_data().await.ok();
    let it = InnerTube::new(Session { visitor_data: vd, ..Session::default() }, None).unwrap();
    let clients = Clients::bundled();
    let web = clients.get(VIDEO_CLIENT).unwrap();

    match it.yt_search(web, &query, None).await {
        Ok(f) => {
            let items: Vec<FeedItem> = f.sections.iter().flat_map(|s| s.items.clone()).collect();
            println!("search: {} (more: {})", summary(&items), f.continuation.is_some());
            if let Some(FeedItem::Video(v)) = items.iter().find(|i| matches!(i, FeedItem::Video(_))) {
                println!("  first: {:?}", v);
            }
            if let Some(tok) = f.continuation {
                match it.yt_continuation(web, "search", &tok).await {
                    Ok(f2) => {
                        let items: Vec<FeedItem> =
                            f2.sections.iter().flat_map(|s| s.items.clone()).collect();
                        println!("search page 2: {}", summary(&items));
                    }
                    Err(e) => println!("search page 2 failed: {e}"),
                }
            }
        }
        Err(e) => println!("search failed: {e}"),
    }

    match it.yt_watch(web, &video_id, None, None).await {
        Ok(w) => {
            println!(
                "watch: title={:?} channel={:?} ({:?}) subs={:?} views={:?} published={:?} likes={:?} comments={:?}",
                w.title, w.channel, w.channel_id, w.subscribers, w.views, w.published, w.likes, w.comment_count
            );
            println!(
                "  description: {} chars; related: {}; more: {}; autoplay: {:?}",
                w.description.as_deref().map_or(0, str::len),
                summary(&w.related),
                w.related_continuation.is_some(),
                w.autoplay.as_ref().map(|a| &a.title)
            );
            if let Some(FeedItem::Video(v)) = w.related.first() {
                println!("  first related: {:?}", v);
            }
            if let Some(tok) = w.related_continuation {
                match it.yt_continuation(web, "next", &tok).await {
                    Ok(f2) => {
                        let items: Vec<FeedItem> =
                            f2.sections.iter().flat_map(|s| s.items.clone()).collect();
                        println!("related page 2: {}", summary(&items));
                    }
                    Err(e) => println!("related page 2 failed: {e}"),
                }
            }
        }
        Err(e) => println!("watch failed: {e}"),
    }

    match it.comments(web, &video_id).await {
        Ok(c) => println!("comments: {:?}, {} threads", c.state, c.threads.len()),
        Err(e) => println!("comments failed: {e}"),
    }

    match it.yt_channel(web, &channel_id, None).await {
        Ok(c) => {
            println!(
                "channel: {:?} handle={:?} subs={:?} videos={:?} avatar={} banner={} tabs={:?}",
                c.title,
                c.handle,
                c.subscribers,
                c.video_count,
                c.avatar.is_some(),
                c.banner.is_some(),
                c.tabs.iter().map(|t| (&t.title, t.params.is_some(), t.selected)).collect::<Vec<_>>()
            );
            let items: Vec<FeedItem> = c.feed.sections.iter().flat_map(|s| s.items.clone()).collect();
            println!("  home tab: {}", summary(&items));
            if let Some(videos) = c.tabs.iter().find(|t| t.title == "Videos") {
                match it.yt_channel(web, &channel_id, videos.params.as_deref()).await {
                    Ok(v) => {
                        let items: Vec<FeedItem> =
                            v.feed.sections.iter().flat_map(|s| s.items.clone()).collect();
                        println!("  videos tab: {} (more: {})", summary(&items), v.feed.continuation.is_some());
                        if let Some(FeedItem::Video(x)) = items.first() {
                            println!("  first: {:?}", x);
                        }
                    }
                    Err(e) => println!("  videos tab failed: {e}"),
                }
            }
        }
        Err(e) => println!("channel failed: {e}"),
    }

    match it.yt_feed(web, "VLPLFgquLnL59alCl_2TQvOiD5Vgm1hCaGSI", None).await {
        Ok(f) => {
            let items: Vec<FeedItem> = f.sections.iter().flat_map(|s| s.items.clone()).collect();
            println!("playlist: {:?} {} (more: {})", f.title, summary(&items), f.continuation.is_some());
        }
        Err(e) => println!("playlist failed: {e}"),
    }

    match it.yt_feed(web, innertube::video::HOME, None).await {
        Ok(f) => {
            let items: Vec<FeedItem> = f.sections.iter().flat_map(|s| s.items.clone()).collect();
            println!("home (signed out): {}", summary(&items));
        }
        Err(e) => println!("home failed: {e}"),
    }

    match it.yt_suggestions("minecraft").await {
        Ok(s) => println!("suggestions: {s:?}"),
        Err(e) => println!("suggestions failed: {e}"),
    }
}
