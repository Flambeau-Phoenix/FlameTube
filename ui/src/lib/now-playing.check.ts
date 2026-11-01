import { nowPlayingHref } from './now-playing.ts';
function equal(actual: unknown, expected: unknown) {
	if (actual !== expected) throw Error(`Expected ${expected}, received ${actual}`);
}
equal(nowPlayingHref(undefined), null);
equal(nowPlayingHref('iptv:example'), '/live');
equal(nowPlayingHref('current', { video_id:'current', yt:true }), '/video/watch/current');
equal(nowPlayingHref('new', { video_id:'old', yt:true }), null);
equal(nowPlayingHref('music-video', { video_id:'music-video', yt:false }), null);
equal(nowPlayingHref('a/b?c', { video_id:'a/b?c', yt:true }), '/video/watch/a%2Fb%3Fc');
console.log('ok — Now Playing follows the active video/live item and preserves music navigation');
