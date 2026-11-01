//! Debug tool: save a raw www.youtube.com InnerTube response.
//!   cargo run -p innertube --example video_dump -- <path> <key> <value> <out.json>

use innertube::{Clients, InnerTube, Session, VIDEO_CLIENT};

#[tokio::main]
async fn main() {
    let a: Vec<String> = std::env::args().skip(1).collect();
    let it = InnerTube::new(Session::default(), None).unwrap();
    let vd = it.fetch_visitor_data().await.ok();
    let it = InnerTube::new(Session { visitor_data: vd, ..Session::default() }, None).unwrap();
    let clients = Clients::bundled();
    let mut body = serde_json::Map::new();
    body.insert(a[1].clone(), a[2].clone().into());
    let v = it.yt_post(clients.get(VIDEO_CLIENT).unwrap(), &a[0], body).await.unwrap();
    std::fs::write(&a[3], serde_json::to_string_pretty(&v).unwrap()).unwrap();
}
