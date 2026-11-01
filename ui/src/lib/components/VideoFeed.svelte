<script lang="ts">
	// A feed of Video mode cards: grid rows by section, with the page's "Show more" hook.
	//
	// The grid is CSS (`auto-fill` at a 15rem minimum), so a window resize reflows it with no
	// JavaScript, and thumbnails stay lazy: YouTube's own home is a hundred requests before a pixel
	// is drawn, and that is exactly what a lightweight client should not do.
	import type { Feed, FeedItem } from '$lib/api';
	import VideoCard from './VideoCard.svelte';
	import SectionHeading from './SectionHeading.svelte';

	let {
		feed,
		onopen,
		onmore,
		moreLabel,
		compact = false
	}: {
		feed: Feed | null;
		onopen: (item: FeedItem) => void;
		/** Omitted for a feed with no next page. */
		onmore?: () => void;
		/** Titles a single-section feed whose section carries no name (a search, a channel tab). */
		moreLabel?: string;
		compact?: boolean;
	} = $props();
</script>

{#if feed}
	{#each feed.sections as section, i (i)}
		{#if section.items.length}
			<section class="mb-8">
				{#if section.title}
					<SectionHeading title={section.title} />
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
	{#if onmore && feed.continuation}
		<div class="mb-8 flex justify-center">
			<button
				class="cursor-pointer rounded-full bg-foreground/5 px-4 py-2 text-sm font-medium transition-colors hover:bg-foreground/10"
				onclick={onmore}>
				{moreLabel ?? 'Show more'}
			</button>
		</div>
	{/if}
{/if}
