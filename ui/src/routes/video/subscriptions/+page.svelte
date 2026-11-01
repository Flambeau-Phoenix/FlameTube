<script lang="ts">
	// Video mode's subscriptions feed: what the channels you follow have posted lately.
	import * as api from '$lib/api';
	import type { Feed } from '$lib/api';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { auth } from '$lib/player.svelte';
	import { getCached, putCached } from '$lib/pagecache';
	import { openFeedItem } from '$lib/video-nav';
	import { t } from '$lib/i18n.svelte';

	const CACHE_KEY = 'video:subscriptions';

	let feed = $state<Feed | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let more = $state(false);

	async function load() {
		loading = true;
		error = null;
		const cached = getCached<Feed>(CACHE_KEY);
		if (cached) {
			feed = cached;
			loading = false;
		}
		try {
			const f = await api.ytFeed(api.YT_SUBSCRIPTIONS);
			feed = f;
			putCached(CACHE_KEY, f);
		} catch (e) {
			if (!cached) error = String(e);
		} finally {
			loading = false;
		}
	}

	async function loadMore() {
		if (!feed?.continuation || more) return;
		more = true;
		try {
			const next = await api.ytMore('browse', feed.continuation);
			feed = {
				...feed,
				sections: [...feed.sections, ...next.sections],
				continuation: next.continuation
			};
			putCached(CACHE_KEY, feed);
		} catch (e) {
			error = String(e);
		} finally {
			more = false;
		}
	}

	$effect(() => {
		auth.epoch;
		feed = null;
		void load();
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	<h1 class="mb-5 font-heading text-2xl font-bold tracking-tight">{t('nav.subscriptions')}</h1>
	{#if !auth.account?.signedIn}
		<p class="text-sm text-muted-foreground">{t('nav.sign_in_video')}</p>
	{:else if loading && !feed}
		<VideoGridSkeleton />
	{:else if error && !feed}
		<ErrorState message={error} onRetry={load} />
	{:else}
		<VideoFeed {feed} onopen={openFeedItem} onmore={loadMore} />
	{/if}
</div>
