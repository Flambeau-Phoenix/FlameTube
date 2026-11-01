<script lang="ts">
	import { untrack } from 'svelte';
	import * as api from '$lib/api';
	import type { Feed } from '$lib/api';
	import { auth } from '$lib/player.svelte';
	import { freshHome } from '$lib/video-home';
	import { openFeedItem } from '$lib/video-nav';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	let feed = $state<Feed | null>(null), loading = $state(true);
	let error = $state<string | null>(null), repeated = $state(false), more = $state(false);
	let topic = $state('Science and technology');
	let request = 0;
	const topics = ['Science and technology', 'Travel and nature', 'Gaming', 'Documentaries', 'Cooking', 'News'];
	const signedIn = $derived(!!auth.account?.signedIn);
	async function load() {
		const mine = ++request;
		loading = true; error = null;
		const loggedIn = signedIn;
		try {
			const result = await freshHome(`${auth.epoch}:${loggedIn ? 'home' : topic}`,
				() => loggedIn ? api.ytFeed(api.YT_HOME) : api.ytSearch(topic, 'videos'),
				cursor => api.ytMore(loggedIn ? 'browse' : 'search', cursor));
			if (mine !== request) return;
			feed = result.feed; repeated = result.repeated;
		} catch (e) { if (mine === request) error = String(e); }
		finally { if (mine === request) loading = false; }
	}
	async function loadMore() {
		if (!feed?.continuation || more) return;
		const mine = request; more = true;
		try {
			const next = await api.ytMore(signedIn ? 'browse' : 'search', feed.continuation);
			if (mine === request && feed) feed = { ...next, sections: [...feed.sections, ...next.sections] };
		} catch (e) { error = String(e); } finally { more = false; }
	}
	$effect(() => {
		// Startup account restoration is asynchronous. Wait for it before choosing personal home
		// versus topic discovery, and reload if the account changes while this page stays mounted.
		const account = auth.account;
		auth.epoch;
		if (!account) return;
		untrack(() => void load());
		return () => { request++; };
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	<div class="mb-5 flex flex-wrap items-center justify-between gap-3">
		<div>
			<h1 class="font-heading text-2xl font-bold">{signedIn ? 'Your YouTube home' : 'Explore videos'}</h1>
			<p class="mt-1 text-sm text-muted-foreground">{signedIn ? 'Recommendations from your YouTube account.' : 'Browse YouTube by topic, or use Sign in above for your personal recommendations.'}</p>
		</div>
		<button class="rounded-full border px-4 py-2 text-sm disabled:opacity-50" disabled={loading} onclick={load}>Refresh videos</button>
	</div>
	{#if !signedIn}
		<div class="mb-5 flex flex-wrap gap-2">
			{#each topics as value}<button class="rounded-full border px-3 py-1.5 text-sm {topic === value ? 'bg-primary text-primary-foreground' : ''}" aria-pressed={topic === value} onclick={() => { topic = value; feed = null; void load(); }}>{value}</button>{/each}
		</div>
	{/if}
	{#if repeated}<p class="mb-4 text-sm text-muted-foreground">YouTube returned the same recommendations. Watch some videos or choose another topic to change your feed.</p>{/if}
	{#if error}<ErrorState message={error} onRetry={load} />{/if}
	{#if loading}<VideoGridSkeleton />{:else}<VideoFeed {feed} onopen={openFeedItem} onmore={loadMore} />{/if}
</div>
