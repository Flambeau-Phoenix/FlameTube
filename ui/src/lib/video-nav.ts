// Adds the Video mode routes' shared bits: this file keeps the pages themselves free of repeated
// navigation and cache code.
import { goto } from '$app/navigation';
import type { FeedItem } from './api';

/** Every card in Video mode opens the same three ways. */
export function openFeedItem(item: FeedItem): void {
	if (item.kind === 'video') goto(`/video/watch/${encodeURIComponent(item.videoId)}`);
	else if (item.kind === 'channel') goto(`/video/channel/${encodeURIComponent(item.channelId)}`);
	else goto(`/video/playlist/${encodeURIComponent(item.playlistId)}`);
}
