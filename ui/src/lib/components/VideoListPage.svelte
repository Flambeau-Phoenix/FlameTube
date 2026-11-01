<script lang="ts">
	// Library rows for Video mode: history, Watch Later, your playlists.
	//
	// One component for the three because they are the same page shape (a feed under a title), and
	// the difference is only which browse id it asks for.
	import * as api from '$lib/api';
	import type { Feed } from '$lib/api';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { auth } from '$lib/player.svelte';
	import { getCached, putCached } from '$lib/pagecache';
	import { openFeedItem } from '$lib/video-nav';
	import { t } from '$lib/i18n.svelte';

	let { id, title }: { id: string; title: string } = $props();

	const cacheKey = $derived(`video:list:${id}`);

	let feed = $state<Feed | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let more = $state(false);

	async function load() {
		loading = true;
		error = null;
		const cached = getCached<Feed>(cacheKey);
		if (cached) {
			feed = cached;
			loading = false;
		}
		try {
			const f = await api.ytFeed(id);
			feed = f;
			putCached(cacheKey, f);
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
			putCached(cacheKey, feed);
		} catch (e) {
			error = String(e);
		} finally {
			more = false;
		}
	}

	$effect(() => {
		id;
		auth.epoch;
		void load();
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	<h1 class="mb-5 font-heading text-2xl font-bold tracking-tight">{title}</h1>
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
