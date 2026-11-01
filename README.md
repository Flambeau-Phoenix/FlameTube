# FlameTube

A desktop video and music app built with Rust, Tauri, Svelte, and libmpv.

The top bar opens **YouTube · Music · Live TV · Twitch**. A fresh profile starts on
YouTube. **Now Playing** returns to the current YouTube video, music player, live
channel, or recently opened Twitch player.

- YouTube browsing, search, and native video playback with accessible quality controls.
- YouTube Music browsing and playback through the existing music library interface.
- Live TV with English-language filtering, US-first sorting, categories, search,
  favorites, and alternate streams from the attributed iptv-org catalog.
- Twitch channels, past broadcasts, and clips through Twitch's official player;
  local favorite channels and optional chat. Leaving Twitch unloads its player;
  Now Playing reopens the selected channel or link.
- Discord activity and Last.fm/scrobbling integrations are disabled and absent
  from the settings interface.

## Clean base

This repository begins with a source-only snapshot. It contains no imported account
profile, sign-in cookies, viewing history, personal favorites, recordings, local
databases, build outputs, or previous development Git history. Runtime data is created
locally. The app identifier is `app.flametube.desktop`, separate from Limusic's profile.

YouTube sign-in is optional. Twitch channel/link playback does not require a bundled
API key; account-dependent Twitch features remain controlled by Twitch. No shared
credentials are included. Automatic signed updates are not configured for this base.

## Build on Windows x64

Install Rust with the MSVC toolchain, Visual Studio 2022 Build Tools with Desktop
development with C++, Node.js 26, Git, and 7-Zip. WebView2 is required at runtime.
From the repository root in PowerShell:

```powershell
.\scripts\setup-windows.ps1
.\scripts\build-windows.ps1
```

Setup downloads a pinned, SHA-256-verified libmpv development archive and generates
its MSVC import library. Build installs locked frontend dependencies, runs frontend
checks, and produces an unsigned NSIS installer in `target/release/bundle/nsis`.
For a development executable, use `build-windows.ps1 -Debug -NoBundle`; the executable
is `target/debug/limusic-app.exe` (the internal crate name is retained).

Frontend checks can also run independently:

```powershell
cd ui
npx --yes pnpm@10 install --frozen-lockfile
npx --yes pnpm@10 check
npx --yes pnpm@10 test
npx --yes pnpm@10 build
```

Portable catalog tests: `cargo test -p flametube-iptv --locked`.
Linux and macOS platform source is retained, but this clean base is validated on
Windows; packaging on other platforms requires the usual Tauri and libmpv dependencies.

## Project map

- `ui/src/lib/components/Titlebar.svelte`: primary navigation and Now Playing.
- `ui/src/routes/video/`: YouTube browsing and watch pages.
- `ui/src/routes/live/`: Live TV interface.
- `ui/src/routes/twitch/`: Twitch interface.
- `src-tauri/`: desktop application and native playback adapter.
- `crates/iptv/`: portable catalog parser and model.
- `scripts/sync-iptv.mjs`: refreshes the bundled catalog and metadata.
- `docs/iptv.md`: catalog integration notes and reusable TV design lessons.
- `docs/twitch.md`: Twitch integration boundaries and limitations.

## Credits and licensing

FlameTube is a modified fork of [Limusic by SimoHypers](https://github.com/SimoHypers/limusic).
The application remains licensed under **GPL-3.0-or-later**; see [LICENSE](LICENSE).
Original source notices and internal crate names are retained. FlameTube is an
independent project and is not affiliated with YouTube, Twitch, or their owners.

The bundled IPTV directory comes from [iptv-org](https://github.com/iptv-org/iptv).
Its license and provenance are in `assets/iptv/LICENSE` and `assets/iptv/source.json`.
Channel availability, schedules, codecs, and geographic access vary. Stream URLs
are directory entries; no broadcast content is bundled or hosted by FlameTube.
libmpv and other dependencies retain their respective licenses.
