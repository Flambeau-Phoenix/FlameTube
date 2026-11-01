<script lang="ts">
	// A channel: header, tabs, and whichever tab is open.
	import { goto } from '$app/navigation';
	import * as api from '$lib/api';
	import type { ChannelPage } from '$lib/api';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { getCached, putCached } from '$lib/pagecache';
	import { openFeedItem } from '$lib/video-nav';

	import { page as route } from '$app/state';

	let id = $derived(route.params.id ?? '');

	let channel = $state<ChannelPage | null>(null);
	let tab = $state<string | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let more = $state(false);

	async function load() {
		loading = true;
		error = null;
		const cacheKey = `video:channel:${id}:${tab ?? ''}`;
		const cached = getCached<ChannelPage>(cacheKey);
		if (cached) {
			channel = cached;
			loading = false;
		}
		try {
			const c = await api.ytChannel(id, tab);
			// Header fields only come with the first tab's response; a later tab answers without
			// them, so keep what the first one said.
			channel = channel && tab ? { ...channel, ...c, feed: c.feed } : c;
			putCached(cacheKey, channel);
		} catch (e) {
			if (!cached) error = String(e);
		} finally {
			loading = false;
		}
	}

	async function loadMore() {
		const feed = channel?.feed;
		if (!feed?.continuation || more) return;
		more = true;
		try {
			const next = await api.ytMore('browse', feed.continuation);
			channel = {
				...channel!,
				feed: {
					...feed,
					sections: [...feed.sections, ...next.sections],
					continuation: next.continuation
				}
			};
		} catch (e) {
			error = String(e);
		} finally {
			more = false;
		}
	}

	async function toggleSubscribe() {
		const on = !channel?.subscribed;
		try {
			await api.ytSubscribe(id, on);
			channel = { ...channel!, subscribed: on };
		} catch {
			// A failed write leaves the button as it was, which is the truth.
		}
	}

	$effect(() => {
		id;
		tab;
		void load();
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	{#if loading && !channel}
		<VideoGridSkeleton />
	{:else if error && !channel}
		<ErrorState message={error} onRetry={load} />
	{:else if channel}
		{#if channel.banner}
			<div class="mb-4 h-32 w-full overflow-hidden rounded-2xl md:h-40">
				<img src={channel.banner} alt="" class="h-full w-full object-cover" loading="lazy" />
			</div>
		{/if}
		<div class="mb-6 flex flex-wrap items-center gap-4">
			{#if channel.avatar}
				<img src={channel.avatar} alt="" class="h-20 w-20 rounded-full object-cover" />
			{/if}
			<div class="min-w-0 flex-1">
				<h1 class="font-heading text-2xl font-bold tracking-tight">{channel.title}</h1>
				<p class="mt-1 truncate text-sm text-muted-foreground">
					{[channel.handle, channel.subscribers, channel.videoCount].filter(Boolean).join(' · ')}
				</p>
			</div>
			<button
				class="cursor-pointer rounded-full px-4 py-2 text-sm font-medium transition-colors {channel.subscribed
					? 'bg-foreground/10 hover:bg-foreground/15'
					: 'bg-primary text-primary-foreground'}"
				onclick={toggleSubscribe}
			>
				{channel.subscribed ? 'Subscribed' : 'Subscribe'}
			</button>
		</div>

		{#if channel.tabs.length > 1}
			<div class="mb-5 flex flex-wrap gap-2">
				{#each channel.tabs as tb (tb.title)}
					<button
						class="cursor-pointer rounded-full px-3.5 py-1.5 text-sm font-medium transition-colors {tb.params ===
						tab
							? 'bg-primary text-primary-foreground'
							: 'bg-foreground/5 text-foreground/75 hover:bg-foreground/10'}"
						onclick={() => (tab = tb.params ?? null)}>{tb.title}</button
					>
				{/each}
			</div>
		{/if}

		<VideoFeed feed={channel.feed} onopen={openFeedItem} onmore={loadMore} />
	{/if}
</div>
