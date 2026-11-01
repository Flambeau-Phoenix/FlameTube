// Video mode's shared UI state: which mode the app is in, and the watch page's Audio Mode switch.
//
// One module for both, because everything that draws the picture reads them: the sidebar and
// titlebar pick their nav from `videoMode`, and the watch page, the player bar and the backend all
// need the same Audio Mode answer.
import * as api from './api';

const MODE_KEY = 'flametube_mode';

function readMode(): boolean {
	if (typeof localStorage === 'undefined') return true;
	return localStorage.getItem(MODE_KEY) !== 'music';
}

export const videoMode = $state({
	/** True while the app is showing Video mode: YouTube's feeds, search and watch pages. */
	on: readMode(),
	/** Whether the account can see Video mode's feeds at all (always true; kept explicit so the
	 *  UI has one place to ask). */
	available: true
});

/** Switch modes. The last one used is remembered, so a relaunch lands where the user left off. */
export function setMode(on: boolean) {
	if (videoMode.on === on) return;
	videoMode.on = on;
	if (typeof localStorage !== 'undefined') localStorage.setItem(MODE_KEY, on ? 'video' : 'music');
}

/**
 * Audio Mode: audio only, and the video track is not fetched at all (where the browser extension
 * this follows still downloaded a 144p picture). Global and sticky, like that extension's own
 * switch: someone listening to a talk wants the next one to stay audio too.
 */
export const audioMode = $state({ on: false });
export const videoSettings = $state({ quality: 1080 });

/** Seed from the stored setting once at startup (`initApp`). */
export function initVideoPrefs(settings: Record<string, string>) {
	videoMode.on = readMode();
	audioMode.on = settings.audio_mode === 'true';
	videoSettings.quality = Number(settings.video_quality ?? 1080);
}

export async function setVideoQuality(height: number) {
	await api.setSetting('video_quality', String(height));
	videoSettings.quality = height;
}

export function setAudioMode(on: boolean) {
	audioMode.on = on;
	// The backend resolves and attaches the picture, so it needs the setting, not just the UI.
	api.setSetting('audio_mode', on ? 'true' : 'false').catch(() => {});
}

/** The gradient the extension ships as its default, and the presets beside it. */
export const AUDIO_MODE_THEMES: { id: string; label: string; css: string }[] = [
	{ id: 'dusk', label: 'Dusk', css: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)' },
	{ id: 'ember', label: 'Ember', css: 'linear-gradient(135deg, #f12711 0%, #f5af19 100%)' },
	{ id: 'moss', label: 'Moss', css: 'linear-gradient(135deg, #134e5e 0%, #71b280 100%)' },
	{ id: 'ink', label: 'Ink', css: 'linear-gradient(135deg, #141e30 0%, #243b55 100%)' },
	{ id: 'rose', label: 'Rose', css: 'linear-gradient(135deg, #ee9ca7 0%, #ffdde1 100%)' },
	{ id: 'slate', label: 'Slate', css: 'linear-gradient(135deg, #3a6073 0%, #16222a 100%)' }
];

export function themeCss(id: string | undefined): string {
	const hit = AUDIO_MODE_THEMES.find((t) => t.id === id);
	return (hit ?? AUDIO_MODE_THEMES[0]).css;
}
