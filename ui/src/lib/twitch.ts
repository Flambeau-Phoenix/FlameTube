export type TwitchTarget = { kind: 'channel' | 'video' | 'clip'; id: string };

/** Accept Twitch identifiers, never an arbitrary iframe URL. */
export function parseTwitch(input: string): TwitchTarget | null {
	const text = input.trim();
	if (/^[a-zA-Z0-9_]{1,25}$/.test(text)) return { kind: 'channel', id: text.toLowerCase() };
	try {
		const url = new URL(text.includes('://') ? text : `https://${text}`);
		if (url.protocol !== 'https:' || url.username || url.password || url.port) return null;
		const host = url.hostname.toLowerCase();
		const parts = url.pathname.split('/').filter(Boolean);
		if (host === 'clips.twitch.tv' && parts.length === 1 && /^[\w-]+$/.test(parts[0]))
			return { kind: 'clip', id: parts[0] };
		if (!['twitch.tv', 'www.twitch.tv', 'm.twitch.tv'].includes(host)) return null;
		if (parts.length === 2 && parts[0] === 'videos' && /^\d+$/.test(parts[1]))
			return { kind: 'video', id: parts[1] };
		if (parts.length === 3 && parts[1] === 'clip' && /^[\w-]+$/.test(parts[2]))
			return { kind: 'clip', id: parts[2] };
		if (parts.length === 1 && /^[a-zA-Z0-9_]{1,25}$/.test(parts[0]) &&
			!['directory', 'downloads', 'settings', 'search', 'videos', 'login', 'signup'].includes(parts[0].toLowerCase()))
			return { kind: 'channel', id: parts[0].toLowerCase() };
	} catch { /* Invalid input is shown beside the form. */ }
	return null;
}

export function twitchUrl(target: TwitchTarget): string {
	return target.kind === 'channel' ? `https://www.twitch.tv/${target.id}` :
		target.kind === 'video' ? `https://www.twitch.tv/videos/${target.id}` : `https://clips.twitch.tv/${target.id}`;
}

export function twitchEmbed(target: TwitchTarget, parent: string): string {
	const params = new URLSearchParams({ parent, autoplay: 'true' });
	if (target.kind === 'clip') {
		params.set('clip', target.id);
		return `https://clips.twitch.tv/embed?${params}`;
	}
	params.set(target.kind === 'channel' ? 'channel' : 'video', target.kind === 'video' ? `v${target.id}` : target.id);
	return `https://player.twitch.tv/?${params}`;
}
