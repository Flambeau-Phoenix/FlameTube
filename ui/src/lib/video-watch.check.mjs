// Exercise the watch page's actual async handlers with out-of-order API replies.
import { readFileSync } from 'node:fs';
import { stripTypeScriptTypes } from 'node:module';
import { runInNewContext } from 'node:vm';
import assert from 'node:assert/strict';

const source = readFileSync(new URL('../routes/video/watch/[id]/+page.svelte', import.meta.url), 'utf8');
const handlers = source.slice(source.indexOf('\tasync function load()'), source.indexOf('\n\tfunction cardOf('));
const start = source.indexOf('\t// Start once per navigation.');
const end = source.indexOf('\n\t// The same muted', start);
assert(start > 0 && end > start);
const requests = [];
const plays = [];
let startEffect;
const context = {
	videoId: 'A', listId: null, listIndex: null, loadSeq: 0, startedKey: null,
	page: null, playback: { now: null },
	getCached: () => null, putCached() {},
	api: {
		ytWatch: () => new Promise((resolve, reject) => requests.push({ resolve, reject })),
		ytPlay: async (id) => { plays.push(id); }
	},
	cardOf: (p) => p, toast: { error() {} },
	$effect: (f) => { startEffect = f; }, untrack: (f) => f()
};
const code = stripTypeScriptTypes(handlers + source.slice(start, end), { mode: 'strip' });
const h = runInNewContext(code + '\n({load, read: () => page})', context);
const a = h.load();
context.videoId = 'B';
const b = h.load();
requests[1].resolve({ videoId: 'B', related: [] });
await b;
requests[0].resolve({ videoId: 'A', related: [] });
await a;
assert.equal(h.read().videoId, 'B', 'a stale response must not replace the selected video');
startEffect();
startEffect(); // background revalidation / queue event
assert.deepEqual(plays, ['B'], 'one navigation must queue only one play');
context.playback.now = { videoId: 'C' };
startEffect();
assert.deepEqual(plays, ['B'], 'autoplay advancing must not restart the old video');
context.videoId = 'C';
context.startedKey = null;
const c = h.load();
context.videoId = 'D';
const d = h.load();
requests[2].reject('old request failed');
await c;
assert.equal(context.loading, true, 'an old failure must not end the latest loading state');
assert.equal(context.error, null, 'an old failure must not replace the latest error state');
requests[3].resolve({ videoId: 'D', related: [] });
await d;
startEffect();
assert.deepEqual(plays, ['B', 'D'], 'a new navigation must start the newly selected video');
console.log('ok — watch navigation, stale responses and autoplay');

const holeSource = readFileSync(new URL('./watch-hole.svelte.ts', import.meta.url), 'utf8')
	.replace(/^import .*;$/gm, '').replaceAll('export ', '');
const rectRequests = [];
const holeContext = {
	$state: (v) => v, prefs: { nativeVideo: true }, document: { hidden: true },
	api: { nativeVideoRect: () => new Promise((resolve) => rectRequests.push(resolve)) }
};
const holes = runInNewContext(stripTypeScriptTypes(holeSource, { mode: 'strip' }) +
	'\n({ setWatchHole, clearWatchHole, watch })', holeContext);
const rect = { x: 10, y: 20, w: 640, h: 360 };
holes.setWatchHole(rect);
assert.equal(holes.watch.hole, null, 'do not expose the desktop before native output is confirmed');
holes.clearWatchHole();
rectRequests[0](true);
await Promise.resolve();
assert.equal(holes.watch.hole, null, 'a late reply must not reopen a hole after navigation');
holes.setWatchHole(rect);
rectRequests[2](true);
await Promise.resolve();
assert.equal(holes.watch.hole, rect, 'confirmed native output opens the measured hole');
holes.setWatchHole({ ...rect, x: 30 });
rectRequests[3](false);
await Promise.resolve();
assert.equal(holeContext.prefs.nativeVideo, false, 'unavailable native output enables the fallback');
assert.equal(holes.watch.hole, null, 'fallback must not leave a transparent hole');
console.log('ok — native output confirmation, navigation and fallback');
