//! Portable IPTV catalogue model. No Tauri, filesystem, player or platform dependencies.
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    #[serde(default)]
    pub country: String,
    #[serde(default)]
    pub languages: Vec<String>,
    #[serde(default)]
    pub nsfw: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stream {
    pub url: String,
    pub label: String,
    pub referrer: Option<String>,
    pub user_agent: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    pub id: String,
    pub tvg_id: String,
    pub name: String,
    pub logo: Option<String>,
    pub categories: Vec<String>,
    pub country: String,
    pub languages: Vec<String>,
    pub streams: Vec<Stream>,
}

pub fn public_url(value: &str) -> bool {
    let Ok(u) = url::Url::parse(value) else { return false };
    if !matches!(u.scheme(), "http" | "https")
        || u.host_str().is_none()
        || !u.username().is_empty()
        || u.password().is_some()
    {
        return false;
    }
    let host = u.host_str().unwrap_or_default().trim_matches(['[', ']']);
    if host.eq_ignore_ascii_case("localhost") || host.ends_with(".local") {
        return false;
    }
    if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        return match ip {
            std::net::IpAddr::V4(v) => {
                !(v.is_private()
                    || v.is_loopback()
                    || v.is_link_local()
                    || v.is_unspecified()
                    || v.is_multicast())
            }
            std::net::IpAddr::V6(v) => {
                !(v.is_loopback()
                    || v.is_unspecified()
                    || v.is_multicast()
                    || (v.segments()[0] & 0xfe00 == 0xfc00)
                    || (v.segments()[0] & 0xffc0 == 0xfe80))
            }
        };
    }
    true
}

// Split the title only at an unquoted comma; logos and stream names may contain commas.
fn header(line: &str) -> Option<(&str, &str)> {
    let mut quoted = false;
    for (i, c) in line.char_indices() {
        if c == '"' {
            quoted = !quoted;
        }
        if c == ',' && !quoted {
            return Some((&line[..i], line[i + 1..].trim()));
        }
    }
    None
}
fn attr(line: &str, key: &str) -> String {
    let pattern = format!("{key}=\"");
    let Some((_, value)) = line.split_once(&pattern) else { return String::new() };
    value.split('"').next().unwrap_or_default().to_owned()
}
fn clean_header(value: String) -> Option<String> {
    (!value.is_empty() && !value.contains(['\r', '\n'])).then_some(value)
}
fn stable_id(key: &str) -> String {
    let hash =
        key.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100000001b3));
    format!("iptv:{hash:016x}")
}

/// Keep alternative endpoints for channels still in the current public listing. Never resurrect
/// a removed channel, or replace the fresh primary endpoint with an older snapshot endpoint.
pub fn merge_alternates(fresh: &mut [Channel], snapshot: &[Channel]) {
    let old: HashMap<_, _> = snapshot.iter().map(|c| (&c.id, c)).collect();
    for channel in fresh {
        if let Some(previous) = old.get(&channel.id) {
            for stream in &previous.streams {
                if !channel.streams.iter().any(|s| s.url == stream.url) {
                    channel.streams.push(stream.clone());
                }
            }
        }
    }
}

pub fn parse_m3u(text: &str, metadata: &HashMap<String, Metadata>) -> Vec<Channel> {
    if !text.trim_start_matches('\u{feff}').trim_start().starts_with("#EXTM3U") {
        return Vec::new();
    }
    let mut channels = BTreeMap::<String, Channel>::new();
    let mut pending: Option<(String, String, String, String, Option<String>, Option<String>)> =
        None;
    for line in text.lines().map(|s| s.trim().trim_start_matches('\u{feff}')) {
        if line.starts_with("#EXTINF:") {
            pending = header(line).map(|(attrs, name)| {
                (
                    attr(attrs, "tvg-id"),
                    name.to_owned(),
                    attr(attrs, "tvg-logo"),
                    attr(attrs, "group-title"),
                    clean_header(attr(attrs, "http-referrer")),
                    clean_header(attr(attrs, "http-user-agent")),
                )
            });
        } else if let Some((_, _, _, _, refer, ua)) = pending.as_mut() {
            if let Some(value) = line.strip_prefix("#EXTVLCOPT:http-referrer=") {
                *refer = clean_header(value.to_owned());
                continue;
            }
            if let Some(value) = line.strip_prefix("#EXTVLCOPT:http-user-agent=") {
                *ua = clean_header(value.to_owned());
                continue;
            }
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            let (tvg, name, logo, category, refer, ua) = pending.take().unwrap();
            if !public_url(line) || name.is_empty() {
                continue;
            }
            let base = tvg.split('@').next().unwrap_or_default();
            let meta =
                metadata.get(&tvg).or_else(|| metadata.get(base)).cloned().unwrap_or_default();
            if meta.nsfw || category.eq_ignore_ascii_case("xxx") {
                continue;
            }
            let key = if tvg.is_empty() { format!("name:{name}") } else { tvg.clone() };
            let ch = channels.entry(key.clone()).or_insert_with(|| Channel {
                id: stable_id(&key),
                tvg_id: tvg,
                name: name.clone(),
                logo: public_url(&logo).then_some(logo),
                categories: Vec::new(),
                country: meta.country,
                languages: meta.languages,
                streams: Vec::new(),
            });
            for group in category.split(';').filter(|s| !s.is_empty()) {
                if !ch.categories.iter().any(|s| s == group) {
                    ch.categories.push(group.to_owned());
                }
            }
            if !ch
                .streams
                .iter()
                .any(|s| s.url == line && s.referrer == refer && s.user_agent == ua)
            {
                ch.streams.push(Stream {
                    url: line.to_owned(),
                    label: name,
                    referrer: refer,
                    user_agent: ua,
                });
            }
        }
    }
    let mut out: Vec<_> = channels.into_values().collect();
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bundled_catalogue_has_confirmed_english_metadata_and_stable_ids() {
        let metadata =
            serde_json::from_str(include_str!("../../../assets/iptv/metadata.json")).unwrap();
        let channels = parse_m3u(include_str!("../../../assets/iptv/catalog.m3u"), &metadata);
        assert!(channels.len() > 5000);
        assert!(channels.iter().filter(|c| c.languages.iter().any(|l| l == "eng")).count() > 500);
        assert!(channels.iter().all(|c| !c.streams.is_empty()));
        let ids: std::collections::HashSet<_> = channels.iter().map(|c| &c.id).collect();
        assert_eq!(ids.len(), channels.len());
    }
    #[test]
    fn preserves_headers_commas_alternatives_and_categories() {
        let text = "#EXTM3U\r\n#EXTINF:-1 tvg-id=\"Example.us@HD\" tvg-logo=\"https://cdn.test/a,b.png\" group-title=\"News\",Example, News (720p)\r\n#EXTVLCOPT:http-referrer=https://example.test/\r\n#EXTVLCOPT:http-user-agent=Example Player\r\nhttps://example.test/live.m3u8\r\n#EXTINF:-1 tvg-id=\"Example.us@HD\" group-title=\"General\",Example (1080p)\nhttps://backup.test/live.m3u8\n#EXTINF:-1 tvg-id=\"Example.us@HD\",Example\nhttps://backup.test/live.m3u8";
        let metadata = HashMap::from([(
            "Example.us@HD".into(),
            Metadata { country: "US".into(), languages: vec!["eng".into()], nsfw: false },
        )]);
        let c = parse_m3u(text, &metadata);
        assert_eq!(c.len(), 1);
        assert_eq!(c[0].streams.len(), 2);
        assert_eq!(c[0].name, "Example, News (720p)");
        assert_eq!(c[0].categories, vec!["News", "General"]);
        assert_eq!(c[0].languages, vec!["eng"]);
        assert_eq!(c[0].streams[0].user_agent.as_deref(), Some("Example Player"));
        assert_eq!(c[0].id, parse_m3u(text, &metadata)[0].id);
    }
    #[test]
    fn rejects_unsafe_urls_and_malformed_rows() {
        for url in [
            "file:///C:/test",
            "javascript:alert(1)",
            "http://localhost/a",
            "http://192.168.1.1/a",
            "http://[::1]/a",
            "http://[fe80::1]/a",
            "https://u:p@site.test/a",
        ] {
            assert!(!public_url(url), "{url}");
        }
        assert!(public_url("https://cdn.test/live.m3u8?token=public"));
        assert!(
            parse_m3u("#EXTM3U\n#EXTINF:-1 broken\nhttps://cdn.test/x", &HashMap::new()).is_empty()
        );
    }
    #[test]
    fn nsfw_is_not_imported() {
        let meta = HashMap::from([("X.us".into(), Metadata { nsfw: true, ..Default::default() })]);
        assert!(parse_m3u("#EXTM3U\n#EXTINF:-1 tvg-id=\"X.us\",X\nhttps://cdn.test/live", &meta)
            .is_empty());
    }

    #[test]
    fn refresh_keeps_alternates_without_resurrecting_removed_channels() {
        let metadata = HashMap::new();
        let mut current =
            parse_m3u("#EXTM3U\n#EXTINF:-1 tvg-id=\"A.us@HD\",A\nhttps://new.test/a", &metadata);
        let old = parse_m3u("#EXTM3U\n#EXTINF:-1 tvg-id=\"A.us@HD\",A\nhttps://old.test/a\n#EXTINF:-1 tvg-id=\"Retired.us\",Retired\nhttps://old.test/b", &metadata);
        merge_alternates(&mut current, &old);
        assert_eq!(current.len(), 1);
        assert_eq!(current[0].streams.len(), 2);
        assert_eq!(current[0].streams[0].url, "https://new.test/a");
    }
}
