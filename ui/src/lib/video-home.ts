import type { Feed, FeedItem, FeedSection } from './api';

const WATCHED_STORAGE_KEY = 'flametube_watched_videos';
const MAX_WATCHED = 4000;
const sessions = new Map<string, { seen: Set<string>; cursor?: string }>();
const watched = new Set<string>();

function loadWatchedFromStorage() {
	if (typeof localStorage === 'undefined') return;
	try {
		const raw = localStorage.getItem(WATCHED_STORAGE_KEY);
		if (!raw) return;
		const parsed: unknown = JSON.parse(raw);
		if (Array.isArray(parsed)) {
			for (const id of parsed) {
				if (typeof id === 'string' && id) watched.add(id);
			}
		}
	} catch {
		// Ignore unavailable storage
	}
}

function saveWatchedToStorage() {
	if (typeof localStorage === 'undefined') return;
	try {
		localStorage.setItem(WATCHED_STORAGE_KEY, JSON.stringify([...watched]));
	} catch {
		// Ignore quota/storage errors
	}
}

loadWatchedFromStorage();

const keyOf = (item: FeedItem) =>
	item.kind === 'video'
		? `v:${item.videoId}`
		: item.kind === 'channel'
			? `c:${item.channelId}`
			: `p:${item.playlistId}`;

/** Mark a video ID as watched so recommendations and Up Next stop recirculating it. */
export function markVideoWatched(videoId: string) {
	if (!videoId) return;
	watched.delete(videoId);
	watched.add(videoId);
	while (watched.size > MAX_WATCHED) {
		const oldest = watched.values().next().value;
		if (oldest === undefined) break;
		watched.delete(oldest);
	}
	saveWatchedToStorage();
}

/** Seed watched video IDs from local SQLite play history (`play_counts`). */
export function seedWatchedVideos(videoIds: Iterable<string>) {
	let changed = false;
	for (const id of videoIds) {
		if (id && !watched.has(id)) {
			watched.add(id);
			changed = true;
		}
	}
	while (watched.size > MAX_WATCHED) {
		const oldest = watched.values().next().value;
		if (oldest === undefined) break;
		watched.delete(oldest);
	}
	if (changed) saveWatchedToStorage();
}

export function isVideoWatched(videoId: string): boolean {
	return watched.has(videoId);
}

function isHiddenItem(item: FeedItem, seen?: Set<string>): boolean {
	if (item.kind === 'video' && watched.has(item.videoId)) return true;
	if (seen?.has(keyOf(item))) return true;
	return false;
}

/** Refetch first, then advance the previous cursor when YouTube repeats its first page or returns watched videos. */
export async function freshHome(
	key: string,
	first: () => Promise<Feed>,
	more: (cursor: string) => Promise<Feed>
): Promise<{ feed: Feed; repeated: boolean }> {
	const previous = sessions.get(key);
	let feed = await first();
	const totalInitial = feed.sections.reduce((acc, s) => acc + s.items.length, 0);
	const unseen = (f: Feed) => ({
		...f,
		sections: f.sections
			.map((s) => ({
				...s,
				items: s.items.filter((i) => !isHiddenItem(i, previous?.seen))
			}))
			.filter((s) => s.items.length > 0)
	});
	let fresh = unseen(feed);
	const freshInitial = fresh.sections.reduce((acc, s) => acc + s.items.length, 0);
	const hadFiltered = Boolean(previous) || freshInitial < totalInitial;

	let cursor = feed.continuation;
	let nextCursor = previous?.cursor ?? cursor;
	for (
		let attempt = 0;
		hadFiltered &&
		fresh.sections.flatMap((s) => s.items).length < 8 &&
		nextCursor &&
		attempt < 4;
		attempt++
	) {
		try {
			const next = await more(nextCursor);
			feed = { ...next, sections: [...fresh.sections, ...next.sections] };
			fresh = unseen(feed);
			cursor = next.continuation;
			nextCursor = cursor;
		} catch {
			break;
		}
	}

	// Filter watched videos even if the session repeated
	const unwatchedOnly = (f: Feed): Feed => ({
		...f,
		sections: f.sections
			.map((s) => ({
				...s,
				items: s.items.filter((i) => !(i.kind === 'video' && watched.has(i.videoId)))
			}))
			.filter((s) => s.items.length > 0)
	});

	const repeated = !!previous && !fresh.sections.some((s) => s.items.length);
	const fallback = unwatchedOnly(feed);
	const answer = repeated ? (fallback.sections.length ? fallback : feed) : fresh;
	const seen = new Set(previous?.seen);
	const unique = new Set<string>();
	answer.sections = answer.sections
		.map((s) => ({
			...s,
			items: s.items.filter((i) => {
				const id = keyOf(i);
				if (unique.has(id)) return false;
				unique.add(id);
				seen.add(id);
				return true;
			})
		}))
		.filter((s) => s.items.length > 0);
	while (seen.size > 3000) seen.delete(seen.values().next().value!);
	sessions.set(key, { seen, cursor: cursor ?? feed.continuation });
	return { feed: { ...answer, continuation: cursor ?? feed.continuation }, repeated };
}

/** Fetch next continuation batch while filtering out already-seen and already-watched videos. */
export async function freshMore(
	key: string,
	cursor: string,
	more: (cursor: string) => Promise<Feed>
): Promise<Feed> {
	const state = sessions.get(key) ?? { seen: new Set<string>() };
	let nextCursor: string | undefined = cursor;
	const collectedSections: FeedSection[] = [];

	for (let attempt = 0; nextCursor && attempt < 3; attempt++) {
		const page = await more(nextCursor);
		nextCursor = page.continuation;
		for (const sec of page.sections) {
			const items = sec.items.filter((i) => {
				const id = keyOf(i);
				if (isHiddenItem(i, state.seen)) return false;
				state.seen.add(id);
				return true;
			});
			if (items.length > 0) {
				collectedSections.push({ ...sec, items });
			}
		}
		const count = collectedSections.reduce((acc, s) => acc + s.items.length, 0);
		if (count >= 6 || !nextCursor) break;
	}

	while (state.seen.size > 3000) state.seen.delete(state.seen.values().next().value!);
	state.cursor = nextCursor;
	sessions.set(key, state);
	return {
		sections: collectedSections,
		continuation: nextCursor
	};
}

/** Filter watched and duplicate items from a watch page's related rail. */
export function filterRelatedItems(items: FeedItem[], currentVideoId?: string): FeedItem[] {
	const seenIds = new Set<string>();
	if (currentVideoId) seenIds.add(`v:${currentVideoId}`);
	const filtered: FeedItem[] = [];
	for (const item of items) {
		const id = keyOf(item);
		if (seenIds.has(id)) continue;
		if (item.kind === 'video' && watched.has(item.videoId)) continue;
		seenIds.add(id);
		filtered.push(item);
	}
	return filtered.length > 0 ? filtered : items.filter((i) => !(i.kind === 'video' && i.videoId === currentVideoId));
}

/** Partition feed sections into full-length video sections and Shorts items for clear UI separation. */
export function separateShortsAndVideos(sections: FeedSection[]): {
	videoSections: FeedSection[];
	shorts: FeedItem[];
} {
	const videoSections: FeedSection[] = [];
	const shorts: FeedItem[] = [];
	const seenShorts = new Set<string>();

	for (const sec of sections) {
		const isShortsShelf =
			sec.title?.toLowerCase().includes('shorts') ?? false;
		const regularItems: FeedItem[] = [];
		for (const item of sec.items) {
			if (item.kind === 'video' && (item.isShort || isShortsShelf)) {
				if (!seenShorts.has(item.videoId)) {
					seenShorts.add(item.videoId);
					shorts.push({ ...item, isShort: true });
				}
			} else {
				regularItems.push(item);
			}
		}
		if (regularItems.length > 0) {
			videoSections.push({ ...sec, items: regularItems });
		}
	}

	return { videoSections, shorts };
}

export function clearHomeSessions() {
	sessions.clear();
	watched.clear();
	saveWatchedToStorage();
}
