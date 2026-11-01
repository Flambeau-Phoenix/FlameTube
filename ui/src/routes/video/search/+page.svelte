<script lang="ts">
	// Video mode's search. Same box as Music mode's, but the results are YouTube's videos,
	// channels and playlists rather than songs, and a filter row picks between them.
	import { onMount } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Search01Icon, Cancel01Icon } from '@hugeicons/core-free-icons';
	import * as api from '$lib/api';
	import type { Feed } from '$lib/api';
	import VideoFeed from '$lib/components/VideoFeed.svelte';
	import VideoGridSkeleton from '$lib/components/VideoGridSkeleton.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { openFeedItem } from '$lib/video-nav';
	import { t } from '$lib/i18n.svelte';

	const FILTERS: { id: string; label: string }[] = [
		{ id: 'videos', label: 'Videos' },
		{ id: 'channels', label: 'Channels' },
		{ id: 'playlists', label: 'Playlists' }
	];

	let query = $state('');
	let filter = $state<string | null>(null);
	let feed = $state<Feed | null>(null);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let more = $state(false);
	let suggestions = $state<string[]>([]);
	let suggestFor = $state('');
	let seq = 0;

	async function run() {
		const q = query.trim();
		if (!q) return;
		const mine = ++seq;
		loading = true;
		error = null;
		try {
			const f = await api.ytSearch(q, filter);
			if (mine === seq) feed = f;
		} catch (e) {
			if (mine === seq) error = String(e);
		} finally {
			if (mine === seq) loading = false;
		}
	}

	async function loadMore() {
		if (!feed?.continuation || more) return;
		more = true;
		try {
			const next = await api.ytMore('search', feed.continuation);
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

	// Completions are anonymous and cheap, so they can follow the typing; the real search only runs
	// on submit or on a filter change with a query already entered.
	$effect(() => {
		const q = query.trim();
		if (!q || q === suggestFor) return;
		const timer = setTimeout(async () => {
			suggestFor = q;
			try {
				suggestions = await api.ytSuggest(q);
			} catch {
				suggestions = [];
			}
		}, 200);
		return () => clearTimeout(timer);
	});

	function pick(s: string) {
		query = s;
		suggestions = [];
		void run();
	}

	function setFilter(id: string | null) {
		filter = id;
		if (query.trim()) void run();
	}

	onMount(() => {
		// A query passed in from elsewhere (?q=…) runs straight away.
		const q = new URLSearchParams(location.search).get('q');
		if (q) {
			query = q;
			void run();
		}
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	<form
		class="mb-4 flex items-center gap-2 rounded-full bg-foreground/5 px-4 py-2"
		onsubmit={(e) => {
			e.preventDefault();
			suggestions = [];
			void run();
		}}
	>
		<HugeiconsIcon icon={Search01Icon} class="h-4 w-4 shrink-0 text-muted-foreground" />
		<input
			class="min-w-0 flex-1 bg-transparent text-sm outline-none placeholder:text-muted-foreground"
			placeholder={t('nav.search')}
			bind:value={query}
			onfocus={() => (suggestFor = '')}
		/>
		{#if query}
			<button
				type="button"
				class="cursor-pointer text-muted-foreground hover:text-foreground"
				aria-label={t('common.cancel')}
				onclick={() => {
					query = '';
					feed = null;
					suggestions = [];
				}}
			>
				<HugeiconsIcon icon={Cancel01Icon} class="h-4 w-4" />
			</button>
		{/if}
	</form>

	{#if suggestions.length && !feed}
		<ul class="mb-4 flex flex-col">
			{#each suggestions as s (s)}
				<li>
					<button
						class="w-full cursor-pointer rounded-lg px-3 py-2 text-left text-sm hover:bg-foreground/5"
						onclick={() => pick(s)}>{s}</button
					>
				</li>
			{/each}
		</ul>
	{/if}

	<div class="mb-5 flex gap-2">
		<button
			class="cursor-pointer rounded-full px-3.5 py-1.5 text-sm font-medium transition-colors {filter ===
			null
				? 'bg-primary text-primary-foreground'
				: 'bg-foreground/5 text-foreground/75 hover:bg-foreground/10'}"
			onclick={() => setFilter(null)}>All</button
		>
		{#each FILTERS as f (f.id)}
			<button
				class="cursor-pointer rounded-full px-3.5 py-1.5 text-sm font-medium transition-colors {filter ===
				f.id
					? 'bg-primary text-primary-foreground'
					: 'bg-foreground/5 text-foreground/75 hover:bg-foreground/10'}"
				onclick={() => setFilter(f.id)}>{f.label}</button
			>
		{/each}
	</div>

	{#if loading && !feed}
		<VideoGridSkeleton />
	{:else if error && !feed}
		<ErrorState message={error} onRetry={run} />
	{:else}
		<VideoFeed {feed} onopen={openFeedItem} onmore={loadMore} />
	{/if}
</div>
