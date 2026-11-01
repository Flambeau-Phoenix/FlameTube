<script lang="ts">
	// One video in a grid: the picture, the length, and the line of channel + stats.
	//
	// Light on purpose. This is a feed of dozens of cards, so the thumbnail is YouTube's 320×180
	// mqdefault (a video's own frame, not the cover ladder a music card needs), loaded lazily, and
	// nothing here fetches until the card is on screen.
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { PlayIcon, UserIcon, Playlist01Icon, AlertCircleIcon } from '@hugeicons/core-free-icons';
	import type { FeedItem } from '$lib/api';

	let {
		item,
		onopen
	}: {
		item: FeedItem;
		/** Navigating is the page's business: the card only says what was picked. */
		onopen: (item: FeedItem) => void;
	} = $props();

	let failed = $state(false);

	// Narrowed views of the union: a Svelte template does not narrow a discriminated union through
	// a prop, so each field the card reads is asked for once, here.
	const video = $derived(item.kind === 'video' ? item : null);
	const channel = $derived(item.kind === 'channel' ? item : null);
	const playlist = $derived(item.kind === 'playlist' ? item : null);
	const kind = $derived(item.kind);
	const title = $derived(item.title);
	// A video card asks i.ytimg.com for the 320x180 frame of the id it was given: YouTube serves
	// that size for every video, so the feed response need not carry a URL. Channels and
	// playlists carry their own art.
	const thumb = $derived(
		item.kind === 'video'
			? `https://i.ytimg.com/vi/${item.videoId}/mqdefault.jpg`
			: (item.thumbnail ?? null)
	);
</script>

<button
	class="group flex w-full cursor-pointer flex-col gap-2 rounded-xl p-2 text-left transition-colors hover:bg-accent/10"
	onclick={() => onopen(item)}
>
	<div
		class="relative aspect-video w-full overflow-hidden bg-foreground/5 {kind === 'channel'
			? 'rounded-full'
			: 'rounded-lg'}"
	>
		{#if thumb && !failed}
			<img
				src={thumb}
				alt=""
				loading="lazy"
				decoding="async"
				class="h-full w-full object-cover {kind === 'channel' ? 'rounded-full' : ''}"
				onerror={() => (failed = true)}
			/>
		{:else}
			<div class="flex h-full w-full items-center justify-center text-muted-foreground/40">
				<HugeiconsIcon
					icon={kind === 'channel'
						? UserIcon
						: kind === 'playlist'
							? Playlist01Icon
							: AlertCircleIcon}
					class="h-8 w-8"
				/>
			</div>
		{/if}
		{#if video?.duration}
			<span
				class="absolute right-1.5 bottom-1.5 rounded bg-black/80 px-1 py-0.5 text-[11px] font-medium text-white"
				>{video.duration}</span
			>
		{/if}
		{#if video?.isLive}
			<span
				class="absolute right-1.5 bottom-1.5 rounded bg-red-600 px-1 py-0.5 text-[11px] font-semibold tracking-wide text-white uppercase"
				>LIVE</span
			>
		{/if}
		{#if video?.isShort}
			<span
				class="absolute left-1.5 bottom-1.5 rounded bg-black/70 px-1 py-0.5 text-[10px] font-medium text-white"
				>Short</span
			>
		{/if}
		{#if playlist?.count}
			<span
				class="absolute right-1.5 bottom-1.5 flex items-center gap-1 rounded bg-black/80 px-1 py-0.5 text-[11px] font-medium text-white"
			>
				<HugeiconsIcon icon={Playlist01Icon} class="h-3 w-3" />
				{playlist.count}
			</span>
		{/if}
		<!-- Affordance only: the whole card is the button. -->
		<span
			class="pointer-events-none absolute inset-0 flex items-center justify-center bg-black/0 opacity-0 transition-opacity group-hover:bg-black/20 group-hover:opacity-100"
		>
			<span class="flex h-10 w-10 items-center justify-center rounded-full bg-black/60 text-white">
				<HugeiconsIcon icon={PlayIcon} class="h-5 w-5" />
			</span>
		</span>
	</div>
	<div class="flex min-w-0 gap-2 px-0.5">
		{#if video?.channelThumbnail}
			<img
				src={video.channelThumbnail}
				alt=""
				loading="lazy"
				class="mt-0.5 h-6 w-6 shrink-0 rounded-full object-cover"
			/>
		{/if}
		<div class="min-w-0 flex-1">
			<p class="line-clamp-2 text-sm leading-snug font-medium">{title}</p>
			<p class="mt-0.5 truncate text-xs text-muted-foreground">
				{#if video}
					{[video.channel, video.views, video.published].filter(Boolean).join(' · ')}
				{:else if channel}
					{[channel.handle, channel.subscribers].filter(Boolean).join(' · ')}
				{:else if playlist}
					{[playlist.channel, playlist.count].filter(Boolean).join(' · ')}
				{/if}
			</p>
		</div>
	</div>
</button>
