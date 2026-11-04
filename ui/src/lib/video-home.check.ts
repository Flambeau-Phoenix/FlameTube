const assert = { equal: (a: unknown,b: unknown) => { if(a !== b) throw Error(`${a} != ${b}`); }, deepEqual: (a: unknown,b: unknown) => { if(JSON.stringify(a) !== JSON.stringify(b)) throw Error('Values differ'); } };
import {
	freshHome,
	freshMore,
	markVideoWatched,
	separateShortsAndVideos,
	clearHomeSessions
} from './video-home.ts';

const feed = (ids: string[], continuation?: string) => ({ sections: [{ items: ids.map(videoId => ({ kind: 'video' as const, videoId, title: videoId, thumbnail: '' })) }], continuation });
clearHomeSessions();
await freshHome('account-a', async () => feed(['a','b'], 'cursor-1'), async () => { throw Error('must not page on first load'); });
const cursors: string[] = [];
const refreshed = await freshHome('account-a', async () => feed(['a','b'], 'cursor-1'), async c => {
	cursors.push(c); return feed(['b','c','d','d']);
});
assert.deepEqual(cursors, ['cursor-1']);
assert.deepEqual(refreshed.feed.sections.flatMap(s => s.items).map(i => i.kind === 'video' ? i.videoId : ''), ['c','d']);
assert.equal(refreshed.repeated, false);
const other = await freshHome('account-b', async () => feed(['a','b']), async () => { throw Error('wrong account cursor'); });
assert.equal(other.feed.sections[0].items.length,2);
const same = await freshHome('account-b', async () => feed(['a','b']), async () => feed([]));
assert.equal(same.repeated,true);

// Watched videos are filtered out across sessions and pagination
clearHomeSessions();
markVideoWatched('watched-1');
const filteredHome = await freshHome('account-c', async () => feed(['watched-1', 'fresh-1', 'fresh-2', 'fresh-3', 'fresh-4', 'fresh-5', 'fresh-6', 'fresh-7', 'fresh-8']), async () => feed([]));
assert.equal(filteredHome.feed.sections[0].items.some(i => i.kind === 'video' && i.videoId === 'watched-1'), false);
const morePage = await freshMore('account-c', 'c-more', async () => feed(['watched-1', 'fresh-1', 'next-1', 'next-2', 'next-3', 'next-4', 'next-5', 'next-6']));
assert.deepEqual(morePage.sections.flatMap(s => s.items).map(i => i.kind === 'video' ? i.videoId : ''), ['next-1', 'next-2', 'next-3', 'next-4', 'next-5', 'next-6']);

// Shorts and full-length videos are cleanly separated
const separated = separateShortsAndVideos([{
	title: 'Recommended',
	items: [
		{ kind: 'video', videoId: 'v1', title: 'Full 1', thumbnail: '', isShort: false },
		{ kind: 'video', videoId: 's1', title: 'Short 1', thumbnail: '', isShort: true }
	]
}]);
assert.equal(separated.videoSections[0].items.length, 1);
assert.equal(separated.shorts.length, 1);
console.log('ok — home refresh advances beyond repeats, filters watched videos, and separates shorts');
