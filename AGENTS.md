# FlameTube development

Rust/Tauri desktop host, Svelte frontend, and libmpv native playback. Keep the
YouTube, Music, Live TV, and Twitch top-level destinations visible in that order.
Fresh profiles start on YouTube. Preserve the existing app layout and ensure the
Now Playing control returns to the active player without restarting native playback.

Use `scripts/setup-windows.ps1` and `scripts/build-windows.ps1` for Windows builds.
Frontend checks are `pnpm check`, `pnpm test`, and `pnpm build` from `ui`.
The portable IPTV model is tested with `cargo test -p flametube-iptv --locked`.
Read `docs/iptv.md` before changing catalog ingestion and `docs/twitch.md` before
changing Twitch playback. Platform availability must be tested before claiming support.

Keep profiles, cookies, tokens, keys, recordings, local databases, binaries, and
machine-specific paths out of Git. Preserve GPL and third-party attribution.
Do not enable dormant Discord or scrobbling services. Signed self-updates require
a project-owned release pipeline and signing configuration before being enabled.
