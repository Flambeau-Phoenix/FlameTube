<script lang="ts">
	import { onMount, untrack } from 'svelte';
	import * as api from '$lib/api';
	import type { Feed } from '$lib/api';
	import { auth } from '$lib/player.svelte';
	import { freshHome, freshMore, seedWatchedVideos } from '$lib/video-home';
	import { openFeedItem } from '$lib/video-nav';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import TwitchFollowedBar from '$lib/components/TwitchFollowedBar.svelte';
	import { twitchSession } from '$lib/twitch.svelte';

	let feed = $state<Feed | null>(null), loading = $state(true);
	let error = $state<string | null>(null), repeated = $state(false), more = $state(false);
	let topic = $state('Science and technology');
	let request = 0;
	const topics = [
		'Science and technology',
		'Travel and nature',
		'Gaming',
		'Documentaries',
		'Cooking',
		'News',
		'Music & Performance',
		'Design & Architecture',
		'History'
	];
	const signedIn = $derived(!!auth.account?.signedIn);
	const scopeKey = $derived(`${auth.epoch}:${signedIn ? 'home' : topic}`);

	onMount(() => {
		void api
			.getPlayCounts()
			.then((counts) => seedWatchedVideos(Object.keys(counts)))
			.catch(() => {});
	});

	async function load() {
		const mine = ++request;
		loading = true;
		error = null;
		const loggedIn = signedIn;
		const key = `${auth.epoch}:${loggedIn ? 'home' : topic}`;
		try {
			const result = await freshHome(
				key,
				() => (loggedIn ? api.ytFeed(api.YT_HOME) : api.ytSearch(topic, 'videos')),
				(cursor) => api.ytMore(loggedIn ? 'browse' : 'search', cursor)
			);
			if (mine !== request) return;
			feed = result.feed;
			repeated = result.repeated;
		} catch (e) {
			if (mine === request) error = String(e);
		} finally {
			if (mine === request) loading = false;
		}
	}

	async function loadMore() {
		if (!feed?.continuation || more) return;
		const mine = request;
		const loggedIn = signedIn;
		const key = scopeKey;
		more = true;
		try {
			const next = await freshMore(key, feed.continuation, (cursor) =>
				api.ytMore(loggedIn ? 'browse' : 'search', cursor)
			);
			if (mine === request && feed) {
				feed = { ...next, sections: [...feed.sections, ...next.sections] };
			}
		} catch (e) {
			error = String(e);
		} finally {
			more = false;
		}
	}

	$effect(() => {
		// Startup account restoration is asynchronous. Wait for it before choosing personal home
		// versus topic discovery, and reload if the account changes while this page stays mounted.
		const account = auth.account;
		auth.epoch;
		if (!account) return;
		untrack(() => void load());
		return () => {
			request++;
		};
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	{#if twitchSession.status?.signedIn}
		<div class="mb-4">
			<TwitchFollowedBar navigate={true} compact={true} />
		</div>
	{/if}

	<div class="mb-5 flex flex-wrap items-center justify-between gap-3">
		<div>
			<h1 class="font-heading text-2xl font-bold">{signedIn ? 'Your YouTube home' : 'Explore videos'}</h1>
			<p class="mt-1 text-sm text-muted-foreground">
				{signedIn
					? 'Fresh recommendations tailored to you, with watched videos filtered out.'
					: 'Browse YouTube by topic, or use Sign in above for your personal recommendations.'}
			</p>
		</div>
		<button
			class="cursor-pointer rounded-full border px-4 py-2 text-sm transition-colors hover:bg-foreground/5 disabled:opacity-50"
			disabled={loading}
			onclick={load}
		>
			Refresh videos
		</button>
	</div>
	{#if !signedIn}
		<div class="mb-5 flex flex-wrap gap-2">
			{#each topics as value}
				<button
					class="cursor-pointer rounded-full border px-3 py-1.5 text-sm transition-colors {topic === value
						? 'bg-primary text-primary-foreground'
						: 'hover:bg-foreground/5'}"
					aria-pressed={topic === value}
					onclick={() => {
						topic = value;
						feed = null;
						void load();
					}}
				>
					{value}
				</button>
			{/each}
		</div>
	{/if}
	{#if repeated}
		<p class="mb-4 text-sm text-muted-foreground">
			YouTube returned overlapping suggestions. Scroll down or click Refresh for more unwatched videos.
		</p>
	{/if}
	{#if error}<ErrorState message={error} onRetry={load} />{/if}
	{#if loading}
		<VideoGridSkeleton />
	{:else}
		<VideoFeed {feed} loadingMore={more} onopen={openFeedItem} onmore={loadMore} />
	{/if}
</div>
