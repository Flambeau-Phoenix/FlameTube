// Refresh the portable, viewer-facing snapshot; never ship the upstream maintenance dump.
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';
const source = process.argv[2];
const out = fileURLToPath(new URL('../assets/iptv/', import.meta.url));
async function get(url, json = false) {
	const r = await fetch(url, { signal: AbortSignal.timeout(60000) });
	if (!r.ok) throw Error(`${r.status}: ${url}`);
	return json ? r.json() : r.text();
}
const [playlist, channels, feeds] = await Promise.all([
	get('https://iptv-org.github.io/iptv/index.category.m3u'),
	get('https://iptv-org.github.io/api/channels.json', true),
	get('https://iptv-org.github.io/api/feeds.json', true)
]);
if (!playlist.startsWith('#EXTM3U') || !playlist.includes('#EXTINF:')) throw Error('Invalid upstream playlist');
const byChannel = new Map(channels.map(c => [c.id, c]));
const metadata = {};
for (const feed of feeds) {
	const c = byChannel.get(feed.channel);
	if (c) metadata[`${feed.channel}@${feed.id}`] = { country: c.country, languages: feed.languages ?? [], nsfw: c.is_nsfw };
}
for (const c of channels) metadata[c.id] = { country: c.country, languages: [], nsfw: c.is_nsfw };
await mkdir(out, { recursive: true });
let catalogue = playlist;
let alternateStreams = 0;
if (source) {
 const publicIds = new Set([...playlist.matchAll(/tvg-id="([^"]+)"/g)].map(m => m[1]));
 const urls = new Set(playlist.split(/\r?\n/).filter(l => /^https?:/.test(l)));
 for (const file of (await readdir(join(source, 'streams'))).filter(f => f.endsWith('.m3u')).sort()) {
  const text = await readFile(join(source, 'streams', file), 'utf8');
  let row = [], allowed = false;
  for (const line of text.split(/\r?\n/)) {
   if (line.startsWith('#EXTINF:')) { row = [line]; allowed = publicIds.has(line.match(/tvg-id="([^"]+)"/)?.[1]); }
   else if (line.startsWith('#EXTVLCOPT:')) row.push(line);
   else if (/^https?:/.test(line) && allowed) {
    if (!urls.has(line)) { catalogue += '\n' + [...row, line].join('\n'); urls.add(line); alternateStreams++; }
    row = []; allowed = false;
   }
  }
 }
}
await writeFile(join(out, 'catalog.m3u'), catalogue);
const ids = new Set([...playlist.matchAll(/tvg-id="([^"]+)"/g)].flatMap(m => [m[1], m[1].split('@')[0]]));
await writeFile(join(out, 'metadata.json'), JSON.stringify(Object.fromEntries(Object.entries(metadata).filter(([id]) => ids.has(id)))));
const provenance = { upstream: 'https://github.com/iptv-org/iptv', playlist: 'https://iptv-org.github.io/iptv/index.category.m3u',
	fetchedAt: new Date().toISOString(), localAlternatesImported: !!source, alternateStreams };
if (source) {
	provenance.localPlaylistFiles = (await readdir(join(source, 'streams'))).filter(f => f.endsWith('.m3u')).length;
}
await writeFile(join(out, 'source.json'), JSON.stringify(provenance, null, 2) + '\n');
console.log(JSON.stringify({ ...provenance, streamRows: (playlist.match(/#EXTINF:/g) ?? []).length, metadata: Object.keys(metadata).length }));
