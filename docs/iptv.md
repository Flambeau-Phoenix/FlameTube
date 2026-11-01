# Live TV catalog and playback

Treat iptv-org as a directory, not an application or a guarantee of live availability.
Start with its published category playlist and the separate metadata API. Raw
`streams/*.m3u` files in an iptv-org checkout are maintenance inputs grouped by source;
their filenames do not establish language, category, or viewing region.

1. Read iptv-org's playlist and stream-description documentation.
2. Import the published `index.category.m3u` playlist.
3. Join channel metadata by channel ID and feed languages by the full `channel@feed`
   identifier. Preserve feed suffixes: they can distinguish languages and regions.
4. Deduplicate repeated category entries and combine categories. Preserve each
   alternate stream's own Referer/User-Agent options.
5. Assign stable IDs so a catalog refresh does not invalidate local favorites.

FlameTube defaults to English-speaking channels from any country and orders US
channels first. English means feed metadata explicitly includes `eng`; unknown
language remains unknown and can be included by disabling the English filter.
Country describes origin, not geographic availability. Preserve upstream
geo-blocked and not-24/7 labels. Known adult entries are excluded.

`crates/iptv` is a platform-independent parser/model. `src-tauri/src/iptv.rs` adapts
it to the desktop with a bundled snapshot, bounded updates, known-channel-ID
playback, and fallback to the last good catalog. `scripts/sync-iptv.mjs` refreshes
metadata and provenance. Optionally pass an iptv-org checkout as its first argument
to import alternate HTTP(S) endpoints only for exact feed IDs already in the public
playlist. No local checkout is required at runtime.

HLS streams use mpv directly. Do not route them through YouTube resolution, music
radio continuation, or the googlevideo proxy: relative segment URLs and per-stream
headers depend on the original manifest. Playback supports alternate sources and
reconnect. A manifest returning HTTP 200 does not prove its segments or codecs work;
health observations are time-dependent, not permanent alive/dead guarantees.
Do not probe thousands of endpoints at startup.

The catalog is reusable in a separate TV application. Keep parsing, grouping,
source selection, and stable identifiers separate from the desktop shell. A TV UI
needs explicit remote focus, predictable Back behavior, restored focus, channel
switching, favorites, and an offline catalog. Android should use an appropriate
native TV interface and playback engine rather than inherit this desktop UI.
No Android application is created by this repository. EPG is not included; future
XMLTV ingestion should be bounded and preprocessed rather than parsing huge XML
documents on the UI thread.

Sources: [iptv-org playlists](https://github.com/iptv-org/iptv/blob/master/docs/playlists.md),
[iptv-org API](https://github.com/iptv-org/api), and the provenance and license in
`assets/iptv`. Public URLs do not transfer ownership of broadcast content.
