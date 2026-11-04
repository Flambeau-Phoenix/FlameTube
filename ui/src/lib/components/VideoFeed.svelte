<script lang="ts">
	// A feed of Video mode cards: grid rows by section, with clear separation between
	// full-length videos and Shorts, plus automatic infinite-scroll continuation.
	import type { Feed, FeedItem } from '$lib/api';
	import { separateShortsAndVideos } from '$lib/video-home';
	import VideoCard from './VideoCard.svelte';
	import SectionHeading from './SectionHeading.svelte';

	let {
		feed,
		onopen,
		onmore,
		moreLabel,
		compact = false,
		loadingMore = false
	}: {
		feed: Feed | null;
		onopen: (item: FeedItem) => void;
		/** Omitted for a feed with no next page. */
		onmore?: () => void;
		/** Titles a single-section feed whose section carries no name (a search, a channel tab). */
		moreLabel?: string;
		compact?: boolean;
		loadingMore?: boolean;
	} = $props();

	let viewFilter = $state<'videos' | 'shorts' | 'all'>('all');
	let sentinel = $state<HTMLElement | null>(null);

	const partitioned = $derived(
		feed ? separateShortsAndVideos(feed.sections) : { videoSections: [], shorts: [] }
	);
	const videoCount = $derived(
		partitioned.videoSections.reduce((acc, s) => acc + s.items.length, 0)
	);
	const shortsCount = $derived(partitioned.shorts.length);
	const hasBoth = $derived(videoCount > 0 && shortsCount > 0);

	// Automatic infinite scroll when sentinel enters viewport
	$effect(() => {
		const el = sentinel;
		const canMore = Boolean(onmore && feed?.continuation);
		if (!el || !canMore || typeof IntersectionObserver === 'undefined') return;
		const io = new IntersectionObserver(
			(entries) => {
				if (entries.some((e) => e.isIntersecting) && !loadingMore) {
					onmore?.();
				}
			},
			{ rootMargin: '420px' }
		);
		io.observe(el);
		return () => io.disconnect();
	});
</script>

{#if feed}
	{#if hasBoth}
		<div class="mb-5 flex flex-wrap items-center gap-2 border-b border-border/60 pb-3">
			<span class="mr-1 text-xs font-semibold tracking-wide text-muted-foreground uppercase">
				Format:
			</span>
			<button
				type="button"
				class="cursor-pointer rounded-full px-3.5 py-1 text-xs font-semibold transition-colors {viewFilter ===
				'all'
					? 'bg-primary text-primary-foreground'
					: 'bg-foreground/5 text-foreground/80 hover:bg-foreground/10'}"
				onclick={() => (viewFilter = 'all')}
			>
				All Separated ({videoCount + shortsCount})
			</button>
			<button
				type="button"
				class="cursor-pointer rounded-full px-3.5 py-1 text-xs font-semibold transition-colors {viewFilter ===
				'videos'
					? 'bg-primary text-primary-foreground'
					: 'bg-foreground/5 text-foreground/80 hover:bg-foreground/10'}"
				onclick={() => (viewFilter = 'videos')}
			>
				Full-Length Videos ({videoCount})
			</button>
			<button
				type="button"
				class="cursor-pointer rounded-full px-3.5 py-1 text-xs font-semibold transition-colors {viewFilter ===
				'shorts'
					? 'bg-red-600 text-white'
					: 'bg-red-500/10 text-red-500 hover:bg-red-500/20'}"
				onclick={() => (viewFilter = 'shorts')}
			>
				Shorts Only ({shortsCount})
			</button>
		</div>
	{/if}

	{#if viewFilter !== 'shorts'}
		{#each partitioned.videoSections as section, i (i)}
			{#if section.items.length}
				<section class="mb-8">
					{#if section.title}
						<SectionHeading title={section.title} />
					{:else if hasBoth && i === 0}
						<SectionHeading title="Full-Length Videos" />
					{/if}
					<div
						class="grid gap-3 {compact
							? 'grid-cols-1 sm:grid-cols-2'
							: 'grid-cols-[repeat(auto-fill,minmax(15rem,1fr))]'}"
					>
						{#each section.items as item, j (item.kind + ':' + (item.kind === 'video'
							? item.videoId
							: item.kind === 'channel'
								? item.channelId
								: item.playlistId) + ':' + j)}
							<VideoCard {item} {onopen} />
						{/each}
					</div>
				</section>
			{/if}
		{/each}
	{/if}

	{#if viewFilter !== 'videos' && partitioned.shorts.length > 0}
		<section class="mb-8 rounded-2xl border border-red-500/20 bg-red-500/[0.03] p-4">
			<div class="mb-3 flex items-center justify-between gap-2">
				<div class="flex items-center gap-2">
					<span class="rounded-md bg-red-600 px-2 py-0.5 text-xs font-bold tracking-wider text-white uppercase">
						Shorts
					</span>
					<span class="text-xs text-muted-foreground">
						Separated from full-length videos ({partitioned.shorts.length})
					</span>
				</div>
				{#if hasBoth && viewFilter === 'all'}
					<button
						type="button"
						class="cursor-pointer text-xs text-muted-foreground hover:text-foreground hover:underline"
						onclick={() => (viewFilter = 'videos')}
					>
						Hide Shorts
					</button>
				{/if}
			</div>
			<div class="grid grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] gap-3">
				{#each partitioned.shorts as item, j ((item.kind === 'video' ? item.videoId : j) + ':' + j)}
					<VideoCard {item} {onopen} vertical={true} />
				{/each}
			</div>
		</section>
	{/if}

	{#if onmore && feed.continuation}
		<div bind:this={sentinel} class="mb-8 flex justify-center">
			<button
				class="cursor-pointer rounded-full bg-foreground/5 px-5 py-2 text-sm font-medium transition-colors hover:bg-foreground/10 disabled:opacity-50"
				disabled={loadingMore}
				onclick={onmore}
			>
				{loadingMore ? 'Loading more videos…' : (moreLabel ?? 'Show more')}
			</button>
		</div>
	{/if}
{/if}
