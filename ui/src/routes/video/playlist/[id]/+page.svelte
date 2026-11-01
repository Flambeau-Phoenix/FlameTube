<script lang="ts">
	// A playlist: its title, then its videos in order. Playing any of them queues the playlist, so
	// autoplay continues in its own order rather than jumping back to the home feed.
	import * as api from '$lib/api';
	import type { Feed } from '$lib/api';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { openFeedItem } from '$lib/video-nav';
	import { getCached, putCached } from '$lib/pagecache';

	import { page as route } from '$app/state';

	let id = $derived(route.params.id ?? '');

	let feed = $state<Feed | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let more = $state(false);

	async function load() {
		loading = true;
		error = null;
		const cacheKey = `video:playlist:${id}`;
		const cached = getCached<Feed>(cacheKey);
		if (cached) {
			feed = cached;
			loading = false;
		}
		try {
			const f = await api.ytFeed('VL' + id);
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
		} catch (e) {
			error = String(e);
		} finally {
			more = false;
		}
	}

	$effect(() => {
		id;
		void load();
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	<h1 class="mb-5 font-heading text-2xl font-bold tracking-tight">{feed?.title ?? ''}</h1>
	{#if loading && !feed}
		<VideoGridSkeleton />
	{:else if error && !feed}
		<ErrorState message={error} onRetry={load} />
	{:else}
		<VideoFeed {feed} onopen={(i) => openFeedItem(i)} onmore={loadMore} />
	{/if}
</div>
