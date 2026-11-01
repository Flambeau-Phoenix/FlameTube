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

Discovery, followed-channel synchronization, and app-owned Twitch authentication
are future work. The Browse button opens Twitch's directory in the user's browser.
Adding Helix discovery requires a registered Twitch app and an appropriate token
flow; never distribute a client secret in desktop source or binaries.

Official references:

- [Video and clips](https://dev.twitch.tv/docs/embed/video-and-clips/)
- [Embed requirements](https://dev.twitch.tv/docs/embed/)
- [Twitch API setup](https://dev.twitch.tv/docs/api/get-started/)
- [Tauri configuration](https://v2.tauri.app/reference/config/)
