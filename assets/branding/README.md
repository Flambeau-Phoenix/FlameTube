# FlameTube logo

`FlameTube.svg` and `FlameTube.png` are the user-supplied logo sources.
The interface uses the SVG copied to `ui/src/lib/assets/favicon.svg`.
Desktop bundle icons in `src-tauri/icons` were generated from the SVG with Tauri's icon CLI:

```powershell
npx --yes @tauri-apps/cli@2 icon assets/branding/FlameTube.svg --output target/flametube-icons
```

Copy the matching desktop icon filenames from that output into `src-tauri/icons`.
Rebuild the frontend and Windows executable to update all embedded icons.
