// The watch page's picture box, and Audio Mode's overlay state.
//
// The native picture (mpv under the webview) needs a transparent box to show through: the root
// paints the page background everywhere *except* this rect, which is the same trick the player view
// uses (`video.svelte.ts` `setHole`). Kept in its own module because the watch page is a route, and
// a route cannot hold the state the layout above it has to read.
import * as api from './api';
import { prefs } from './player.svelte';

export type Rect = { x: number; y: number; w: number; h: number };

export const watch = $state({
	/** Where mpv should draw, viewport CSS pixels; null when no picture is coming. */
	hole: null as Rect | null,
	/** True while the watch page is asking for a picture. The hole opens after native output
	 *  confirms it has a surface underneath the page. */
	want: false
});

let seq = 0;
let sent: string | null = null;

/** Report the box mpv should draw into, or null to take the picture away. */
export function setWatchHole(r: Rect | null) {
	if (r && (r.w < 1 || r.h < 1)) r = null;
	const key = r && `${Math.round(r.x)},${Math.round(r.y)},${Math.round(r.w)},${Math.round(r.h)}`;
	if (key === sent) return;
	sent = key;
	const mine = ++seq;
	watch.hole = null;
	if (!prefs.nativeVideo) {
		sent = null;
		return;
	}
	// Close the old cutout before moving native output; open the new one only after confirmation.
	const send = () => {
		if (mine !== seq) return;
		api.nativeVideoRect(r && [r.x, r.y, r.w, r.h]).then((up) => {
			if (mine !== seq || !r) return;
			watch.hole = up ? r : null;
			if (!up) prefs.nativeVideo = false;
		}).catch(() => {
			if (mine !== seq) return;
			sent = null; // allow a later measurement to retry
		});
	};
	if (document.hidden) send();
	else requestAnimationFrame(() => requestAnimationFrame(send));
}

/** Leaving the page: take the picture down and forget where it was. */
export function clearWatchHole() {
	watch.want = false;
	setWatchHole(null);
}
