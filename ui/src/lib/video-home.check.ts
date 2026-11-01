const assert = { equal: (a: unknown,b: unknown) => { if(a !== b) throw Error(`${a} != ${b}`); }, deepEqual: (a: unknown,b: unknown) => { if(JSON.stringify(a) !== JSON.stringify(b)) throw Error('Values differ'); } };
import { freshHome, clearHomeSessions } from './video-home.ts';

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
console.log('ok — home refresh advances beyond repeats and isolates accounts');
