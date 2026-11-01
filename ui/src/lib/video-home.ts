import type { Feed, FeedItem } from './api';
const sessions = new Map<string, { seen: Set<string>; cursor?: string }>();
const keyOf = (item: FeedItem) => item.kind === 'video' ? `v:${item.videoId}` :
	item.kind === 'channel' ? `c:${item.channelId}` : `p:${item.playlistId}`;

/** Refetch first, then advance the previous cursor when YouTube repeats its first page. */
export async function freshHome(key: string, first: () => Promise<Feed>,
	more: (cursor: string) => Promise<Feed>): Promise<{ feed: Feed; repeated: boolean }> {
	const previous = sessions.get(key);
	let feed = await first();
	const unseen = (f: Feed) => ({ ...f, sections: f.sections.map(s => ({ ...s,
		items: s.items.filter(i => !previous?.seen.has(keyOf(i))) })).filter(s => s.items.length) });
	let fresh = unseen(feed);
	let cursor = feed.continuation;
	let nextCursor = previous?.cursor ?? cursor;
	for (let attempt = 0; previous && fresh.sections.flatMap(s => s.items).length < 8 && nextCursor && attempt < 3; attempt++) {
		try {
			const next = await more(nextCursor);
			feed = { ...next, sections: [...fresh.sections, ...next.sections] };
			fresh = unseen(feed); cursor = next.continuation; nextCursor = cursor;
		} catch { break; }
	}
	const repeated = !!previous && !fresh.sections.some(s => s.items.length);
	const answer = repeated ? feed : fresh;
	const seen = new Set(previous?.seen), unique = new Set<string>();
	answer.sections = answer.sections.map(s => ({ ...s, items: s.items.filter(i => {
		const id = keyOf(i); if (unique.has(id)) return false;
		unique.add(id); seen.add(id); return true;
	}) }));
	while (seen.size > 2000) seen.delete(seen.values().next().value!);
	sessions.set(key, { seen, cursor: cursor ?? feed.continuation });
	return { feed: { ...answer, continuation: cursor ?? feed.continuation }, repeated };
}
export function clearHomeSessions() { sessions.clear(); }
