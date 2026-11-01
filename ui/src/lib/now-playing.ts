/** Return to the active item, independent of browsing history or the last opened watch page. */
export function nowPlayingHref(videoId: string | undefined,
	row?: { video_id: string; yt?: boolean }): string | null {
	if (!videoId) return null;
	if (videoId.startsWith('iptv:')) return '/live';
	if (row?.video_id === videoId && row.yt) return `/video/watch/${encodeURIComponent(videoId)}`;
	return null; // Music keeps its existing Now Playing view.
}
