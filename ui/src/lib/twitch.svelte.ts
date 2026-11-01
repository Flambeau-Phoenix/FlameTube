import type { TwitchTarget } from './twitch';
import {
	twitchStatus,
	twitchFollowedLive,
	twitchSignOut,
	togglePause,
	type TwitchStatus,
	type TwitchLiveChannel
} from './api';
import { playback, np, ui } from './player.svelte';
import { goto } from '$app/navigation';

export const twitchSession = $state({
	target: null as TwitchTarget | null,
	returnPending: false,
	status: null as TwitchStatus | null,
	liveChannels: [] as TwitchLiveChannel[],
	loadingLive: false,
	liveError: null as string | null,
	authOpen: false,
	lastFetched: 0
});

let refreshTimer: ReturnType<typeof setInterval> | null = null;

export async function refreshTwitchStatus(): Promise<TwitchStatus | null> {
	try {
		const s = await twitchStatus();
		twitchSession.status = s;
		if (s.signedIn && document.visibilityState === 'visible') {
			void refreshFollowedLive();
		} else {
			twitchSession.liveChannels = [];
		}
		return s;
	} catch (e) {
		twitchSession.status = null;
		return null;
	}
}

export async function refreshFollowedLive(force = false): Promise<TwitchLiveChannel[]> {
	if (!twitchSession.status?.signedIn && !force) {
		return [];
	}
	// Avoid duplicate concurrent fetches
	if (twitchSession.loadingLive) {
		return twitchSession.liveChannels;
	}
	twitchSession.loadingLive = true;
	twitchSession.liveError = null;
	try {
		const list = await twitchFollowedLive();
		twitchSession.liveChannels = list;
		twitchSession.lastFetched = Date.now();
		return list;
	} catch (e) {
		const err = String(e);
		twitchSession.liveError = err;
		if (err.includes('expired') || err.includes('Sign in')) {
			twitchSession.status = { ...twitchSession.status, signedIn: false, clientIdSet: !!twitchSession.status?.clientIdSet };
			twitchSession.liveChannels = [];
		}
		return [];
	} finally {
		twitchSession.loadingLive = false;
	}
}

export async function tuneTwitch(next: TwitchTarget | null, navigate = false): Promise<void> {
	if (!next) {
		twitchSession.target = null;
		twitchSession.returnPending = false;
		return;
	}
	if (playback.now && !playback.paused) await togglePause();
	np.open = false;
	ui.theaterOpen = false;
	twitchSession.target = next;
	twitchSession.returnPending = true;
	if (navigate && typeof window !== 'undefined' && window.location.pathname !== '/twitch') {
		void goto('/twitch');
	}
}

export async function tuneChannel(login: string, navigate = false): Promise<void> {
	await tuneTwitch({ kind: 'channel', id: login.toLowerCase().trim() }, navigate);
}

export async function signOutTwitch(): Promise<void> {
	try {
		await twitchSignOut();
	} finally {
		twitchSession.status = {
			clientIdSet: !!twitchSession.status?.clientIdSet,
			clientId: twitchSession.status?.clientId,
			signedIn: false,
			login: null
		};
		twitchSession.liveChannels = [];
		twitchSession.liveError = null;
	}
}

export function initTwitch(): void {
	if (typeof window === 'undefined') return;
	void refreshTwitchStatus();
	if (!refreshTimer) {
		refreshTimer = setInterval(() => {
			if (twitchSession.status?.signedIn) {
				void refreshTwitchStatus();
			}
		}, 60000);
	}
}
