import { parseTwitch, twitchEmbed, twitchUrl } from './twitch.ts';
function assert(value: unknown, message: string) { if (!value) throw Error(message); }
for (const value of ['TwitchDev', 'https://www.twitch.tv/twitchdev', 'twitch.tv/twitchdev?ref=home'])
	assert(parseTwitch(value)?.id === 'twitchdev', `channel: ${value}`);
assert(parseTwitch('https://www.twitch.tv/videos/123456')?.kind === 'video', 'VOD');
assert(parseTwitch('https://clips.twitch.tv/FriendlyClip-123')?.kind === 'clip', 'clip');
assert(parseTwitch('https://twitch.tv/name/clip/FriendlyClip-123')?.kind === 'clip', 'channel clip');
for (const value of ['https://twitch.tv.evil.test/name', 'https://evil.test/name', 'javascript:alert(1)', 'https://twitch.tv/directory', 'http://twitch.tv/name', 'https://user:pass@twitch.tv/name', 'https://twitch.tv:9443/name', '<script>'])
	assert(parseTwitch(value) === null, `rejected: ${value}`);
const target = { kind: 'channel' as const, id: 'twitchdev' };
const embed = new URL(twitchEmbed(target, 'tauri.localhost'));
assert(embed.origin === 'https://player.twitch.tv', 'official player');
assert(embed.searchParams.get('parent') === 'tauri.localhost', 'parent matches host');
assert(embed.searchParams.get('channel') === 'twitchdev', 'channel parameter');
assert(twitchUrl(target) === 'https://www.twitch.tv/twitchdev', 'external URL');
assert(new URL(twitchEmbed({ kind: 'video', id: '123' }, 'localhost')).searchParams.get('video') === 'v123', 'VOD prefix');
console.log('ok — Twitch inputs stay on official hosts and embed parameters match the target');
