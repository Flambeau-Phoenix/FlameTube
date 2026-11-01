# Twitch integration

The fourth top-level destination uses Twitch's official player in the system
WebView already supplied by Tauri. It adds no Electron runtime or native stream
extraction dependency. Twitch's player still downloads its own web resources;
this is not a pure-Rust video decoder.

Enter a channel name, channel URL, `/videos/<id>` URL, or clip URL. Inputs are parsed
into identifiers and embedded only through official Twitch hosts. Favorites contain
channel names only, saved in the local profile. Chat loads only when requested.
No Twitch client secret, app token, or shared login is included.

Starting Twitch pauses native YouTube/Music/Live TV playback. Navigating away
destroys the iframe to stop playback and network activity. The selected target stays
in memory so Now Playing can reopen it; live playback resumes at the current live
point. Twitch's own controls provide quality, volume, and fullscreen. Browser fallback
and reload controls are available for offline channels or embedding failures.

The iframe uses the actual app hostname as Twitch's required `parent`. The Windows
clean profile uses Tauri's HTTPS custom-protocol scheme. The remote frames receive
no Tauri capability grant; native commands remain restricted to the local UI.
Avoid overlays over the Twitch player and retain its minimum 400 × 300 size.

Followed live channels and broadcaster profiles use Twitch Helix through the Rust
host. Register a **public** application at the Twitch developer console and enter
its Client ID in Connect Twitch. Authorize the displayed device code on Twitch's
activation page. Only `user:read:follows` is requested. No client secret is needed.
Tokens are stored in the local app database, never embedded into the source or sent
to frontend settings. Treat that database and profile backups as private. Sign-out
removes the local API session; it does not sign out Twitch's separate browser/chat cookies.

Session commits reject cancelled generations; refresh is serialized and tokens are
validated on startup and hourly while the app runs. Channel cards ignore stale
profile responses when switching targets. Chat supports theme, width, docking, and
a narrowly validated Chatterino launch link, if Chatterino is installed.

The Browse button opens Twitch's directory in the browser. API connection and
embedded chat authentication remain separate flows. End-to-end device authorization
requires the user's Twitch application and account and is not automated by the test suite.

Official references:

- [Video and clips](https://dev.twitch.tv/docs/embed/video-and-clips/)
- [Embed requirements](https://dev.twitch.tv/docs/embed/)
- [Twitch API setup](https://dev.twitch.tv/docs/api/get-started/)
- [Tauri configuration](https://v2.tauri.app/reference/config/)
