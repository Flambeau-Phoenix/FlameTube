<script lang="ts">
	// The watch page: the picture across the width, then the video's own column on the left (title,
	// channel, actions, Up next or the playlist, the description, and comments collapsed behind a
	// button) with the related rail on the right.
	//
	// Opening a video queues it in Video mode, so the player bar, the queue panel, the OS media card
	// and the gapless lookahead are all the ones Music mode already uses. The page never fetches a
	// stream URL itself: mpv plays the video and the box below is only a hole for it.
	import { onMount, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		ThumbsUpIcon,
		ThumbsDownIcon,
		Bookmark01Icon,
		HeadphonesIcon,
		ArrowDown01Icon,
		ArrowUp01Icon
	} from '@hugeicons/core-free-icons';
	import * as api from '$lib/api';
	import type { FeedItem, VideoCard as VideoCardData, WatchPage } from '$lib/api';
	import { playback, toast, prefs, videoReady } from '$lib/player.svelte';
	import { claimVideo, parkVideo, showVideo } from '$lib/video.svelte';
	import { audioMode, setAudioMode } from '$lib/video-mode.svelte';
	import { setWatchHole, watch, clearWatchHole } from '$lib/watch-hole.svelte';
	import { getCached, putCached } from '$lib/pagecache';
	import {
		filterRelatedItems,
		isVideoWatched,
		markVideoWatched,
		seedWatchedVideos,
		separateShortsAndVideos
	} from '$lib/video-home';
	import VideoCardView from '$lib/components/VideoCard.svelte';
	import VideoComments from '$lib/components/VideoComments.svelte';
	import VideoControls from '$lib/components/VideoControls.svelte';
	import AudioModeOverlay from '$lib/components/AudioModeOverlay.svelte';
	import ErrorState from '$lib/components/ErrorState.svelte';
	import { openFeedItem } from '$lib/video-nav';
	import { t } from '$lib/i18n.svelte';
	import TwitchFollowedBar from '$lib/components/TwitchFollowedBar.svelte';
	import { twitchSession } from '$lib/twitch.svelte';

	import { page as route } from '$app/state';

	let videoId = $derived(route.params.id ?? '');
	let page = $state<WatchPage | null>(null);
	let loading = $state(true);
	let error = $state<string | null>(null);
	let related = $state<FeedItem[]>([]);
	let relatedMore = $state<string | null>(null);
	let moreLoading = $state(false);
	let showRelatedShorts = $state(false);
	let descOpen = $state(false);
	let commentsOpen = $state(false);
	let saved = $state(false);
	let rating = $state<string | undefined>(undefined);
	let box = $state<HTMLElement | null>(null);
	// `?list=` and `?index=`, so a playlist link opens the video inside its playlist.
	const listId = $derived(route.url.searchParams.get('list'));
	const listIndex = $derived.by(() => {
		const i = route.url.searchParams.get('index');
		return i != null && /^\d+$/.test(i) ? Number(i) : null;
	});
	let loadSeq = 0;
	let startedKey: string | null = null;
	let fallbackBox = $state<HTMLElement | null>(null);
	let observedPlaying = $state(false);
	let clickTimer: ReturnType<typeof setTimeout> | null = null;

	// Separate related items into full-length videos and Shorts so Shorts never clutter Up Next.
	const separatedRelated = $derived.by(() => {
		const cleaned = filterRelatedItems(related, videoId);
		const split = separateShortsAndVideos([{ items: cleaned }]);
		const fullItems = split.videoSections.flatMap((s) => s.items);
		const shortItems = split.shorts;
		return { fullItems, shortItems };
	});

	// Choose an unwatched Up Next target: if YouTube's default autoplay item was already watched,
	// pick the first unwatched full-length related video instead.
	const effectiveAutoplay = $derived.by(() => {
		const candidate = page?.autoplay;
		if (candidate && candidate.videoId !== videoId && !isVideoWatched(candidate.videoId)) {
			return candidate;
		}
		const fallback = separatedRelated.fullItems.find(
			(item): item is Extract<FeedItem, { kind: 'video' }> =>
				item.kind === 'video' && item.videoId !== videoId && !isVideoWatched(item.videoId)
		);
		return fallback ?? candidate ?? null;
	});

	async function load() {
		const mine = ++loadSeq;
		const id = videoId;
		const list = listId;
		const index = listIndex;
		loading = true;
		error = null;
		page = null;
		const cacheKey = `video:watch:${id}:${list ?? ''}:${index ?? ''}`;
		const cached = getCached<WatchPage>(cacheKey);
		if (cached) {
			apply(cached);
			loading = false;
		}
		try {
			const p = await api.ytWatch(id, list, index);
			if (mine !== loadSeq) return;
			apply(p);
			putCached(cacheKey, p);
		} catch (e) {
			if (mine !== loadSeq) return;
			if (!cached) error = String(e);
		} finally {
			if (mine === loadSeq) loading = false;
		}
	}

	function apply(p: WatchPage) {
		page = p;
		related = p.related;
		relatedMore = p.relatedContinuation ?? null;
		rating = p.likeStatus;
		descOpen = false;
		commentsOpen = false;
	}

	/** Queue this video (and its playlist, when it was opened from one) and start it. */
	async function play() {
		if (!page || page.videoId !== videoId) return;
		if (typeof markVideoWatched === 'function') markVideoWatched(videoId);
		const items = page.playlist?.items?.length ? page.playlist.items : [cardOf(page)];
		try {
			await api.ytPlay(videoId, items, page.playlist?.playlistId ?? null);
		} catch (e) {
			toast.error(String(e));
		}
	}

	function cardOf(p: WatchPage): VideoCardData {
		return {
			videoId: p.videoId,
			title: p.title,
			channel: p.channel,
			channelId: p.channelId,
			channelThumbnail: p.channelThumbnail,
			thumbnail: `https://i.ytimg.com/vi/${p.videoId}/mqdefault.jpg`,
			duration: undefined,
			views: p.views,
			published: p.published,
			isLive: false,
			isShort: false
		} as VideoCardData;
	}

	async function loadMoreRelated() {
		if (!relatedMore || moreLoading) return;
		moreLoading = true;
		try {
			const feed = await api.ytMore('next', relatedMore);
			const incoming = feed.sections.flatMap((s) => s.items);
			const existingIds = new Set(
				related
					.filter((i): i is Extract<FeedItem, { kind: 'video' }> => i.kind === 'video')
					.map((i) => i.videoId)
			);
			const filtered = filterRelatedItems(incoming, videoId).filter(
				(i) => i.kind !== 'video' || !existingIds.has(i.videoId)
			);
			related = [...related, ...filtered];
			relatedMore = feed.continuation ?? null;
		} catch (e) {
			error = String(e);
		} finally {
			moreLoading = false;
		}
	}

	function toggleFullscreen() {
		if (!box) return;
		if (document.fullscreenElement) void document.exitFullscreen();
		else void box.requestFullscreen?.().catch(() => {});
	}

	function handleVideoSurfaceClick(e: MouseEvent) {
		// Ignore clicks on interactive controls inside AudioModeOverlay if any
		const target = e.target as HTMLElement | null;
		if (target?.closest('button, a, input, select')) return;
		if (clickTimer) clearTimeout(clickTimer);
		clickTimer = setTimeout(() => {
			clickTimer = null;
			void api.togglePause();
		}, 200);
	}

	function handleVideoSurfaceDblClick(e: MouseEvent) {
		const target = e.target as HTMLElement | null;
		if (target?.closest('button, a, input, select')) return;
		if (clickTimer) {
			clearTimeout(clickTimer);
			clickTimer = null;
		}
		toggleFullscreen();
	}

	async function toggleLike() {
		const next = rating === 'LIKE' ? 'indifferent' : 'like';
		try {
			await api.ytRate(videoId, next);
			rating = next === 'like' ? 'LIKE' : 'INDIFFERENT';
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function toggleDislike() {
		const next = rating === 'DISLIKE' ? 'indifferent' : 'dislike';
		try {
			await api.ytRate(videoId, next);
			rating = next === 'dislike' ? 'DISLIKE' : 'INDIFFERENT';
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function toggleSave() {
		try {
			await api.ytWatchLater(videoId, !saved);
			saved = !saved;
			if (saved) toast.success(t('nav.saved_to_watch_later'));
		} catch (e) {
			toast.error(String(e));
		}
	}

	async function toggleSubscribe() {
		if (!page?.channelId) return;
		const on = !page.subscribed;
		try {
			await api.ytSubscribe(page.channelId, on);
			page = { ...page, subscribed: on };
		} catch (e) {
			toast.error(String(e));
		}
	}

	/** The hole: wherever the 16:9 box is, and none while Audio Mode covers it. */
	function measure() {
		if (!box) return;
		if (audioMode.on || !watch.want || !prefs.nativeVideo ||
			playback.now?.videoId !== videoId || !videoReady[videoId]) {
			setWatchHole(null);
			return;
		}
		const r = box.getBoundingClientRect();
		setWatchHole({ x: r.left, y: r.top, w: r.width, h: r.height });
	}

	let ro: ResizeObserver | null = null;

	onMount(() => {
		void api
			.getPlayCounts()
			.then((counts) => seedWatchedVideos(Object.keys(counts)))
			.catch(() => {});
		ro = new ResizeObserver(measure);
		document.addEventListener('visibilitychange', measure);
		document.addEventListener('scroll', measure, true);
		document.addEventListener('fullscreenchange', measure);
		return () => {
			if (clickTimer) clearTimeout(clickTimer);
			loadSeq++;
			ro?.disconnect();
			document.removeEventListener('visibilitychange', measure);
			document.removeEventListener('scroll', measure, true);
			document.removeEventListener('fullscreenchange', measure);
			clearWatchHole();
		};
	});

	// The layout changed (a description opened, a panel closed): the box moved.
	$effect(() => {
		void descOpen;
		void commentsOpen;
		void audioMode.on;
		void prefs.nativeVideo;
		void watch.want;
		void playback.now?.videoId;
		void videoReady[videoId];
		if (box) {
			ro?.observe(box);
			ro?.observe(document.documentElement);
			measure();
		}
	});

	// A new video: reload, and start it — arriving here is how a video is picked.
	$effect(() => {
		void videoId;
		void listId;
		void listIndex;
		relatedMore = null;
		startedKey = null;
		observedPlaying = false;
		untrack(() => void load());
	});

	// Want the picture while this is a watch page and Audio Mode is off. This is the flag the
	// measure below reads; without it the box stayed opaque black over mpv's output.
	$effect(() => {
		watch.want = !audioMode.on;
		return () => {
			watch.want = false;
		};
	});

	// Start once per navigation. Watching playback.now here used to restart this video whenever
	// autoplay advanced the queue, and a revalidation could queue the same play twice.
	$effect(() => {
		if (!page || page.videoId !== videoId) return;
		const key = `${videoId}:${listId ?? ''}:${listIndex ?? ''}`;
		if (startedKey === key) return;
		startedKey = key;
		untrack(() => {
			if (playback.now?.videoId !== videoId) void play();
		});
	});

	// The same muted, synchronized element as Music mode when native output is unavailable.
	$effect(() => {
		if (!fallbackBox || prefs.nativeVideo || audioMode.on || !showVideo() ||
			playback.now?.videoId !== videoId) return;
		const holder = fallbackBox;
		claimVideo(holder);
		return () => parkVideo(holder);
	});

	// Keep the title, comments and picture on the video the queue advanced to. Only follow after
	// this page's video actually started, so the previous track cannot redirect a new selection.
	$effect(() => {
		const id = playback.now?.videoId;
		if (id === videoId) {
			observedPlaying = true;
			untrack(() => markVideoWatched(videoId));
			return;
		}
		const row = playback.queue.items[playback.queue.currentIndex];
		if (observedPlaying && id && row?.video_id === id && row.yt) {
			untrack(() => void goto(`/video/watch/${encodeURIComponent(id)}`));
		}
	});
</script>

<svelte:window
	onkeydown={(e) => {
		const target = e.target as HTMLElement | null;
		if (
			target &&
			(target.tagName === 'INPUT' ||
				target.tagName === 'TEXTAREA' ||
				target.tagName === 'SELECT' ||
				target.isContentEditable)
		) {
			return;
		}
		if (e.key === 'f' && !e.metaKey && !e.ctrlKey && box) {
			// Fullscreen on the box, not the document: the video is what should fill the screen.
			toggleFullscreen();
		} else if ((e.key === ' ' || e.key === 'k') && !e.metaKey && !e.ctrlKey && !e.altKey) {
			e.preventDefault();
			void api.togglePause();
		} else if (e.key === 'ArrowLeft' && !e.metaKey && !e.ctrlKey && !e.altKey) {
			e.preventDefault();
			void api.seek(Math.max(0, playback.position - 10));
		} else if (e.key === 'ArrowRight' && !e.metaKey && !e.ctrlKey && !e.altKey) {
			e.preventDefault();
			void api.seek(Math.min(playback.duration, playback.position + 10));
		}
	}}
/>

<div class="mx-auto w-full max-w-[110rem] px-4 py-4 md:px-6">
	{#if error && !page}
		<ErrorState message={error} onRetry={load} />
	{:else if page}
		<!-- Followed live channels horizontal bar above the video -->
		{#if twitchSession.status?.signedIn}
			<div class="mb-3">
				<TwitchFollowedBar navigate={true} compact={true} />
			</div>
		{/if}

		<!-- The picture, full width, its height capped so the page below it stays on screen.
		     Click toggles play/pause; double-click toggles fullscreen. -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="relative w-full cursor-pointer select-none overflow-hidden rounded-2xl {audioMode.on ? 'bg-black' : 'bg-transparent'}"
			style="aspect-ratio:16/9;max-height:calc(100vh - 13rem)"
			bind:this={box}
			onclick={handleVideoSurfaceClick}
			ondblclick={handleVideoSurfaceDblClick}
			title={playback.paused ? 'Click to play · Double-click for fullscreen' : 'Click to pause · Double-click for fullscreen'}
		>
			{#if !prefs.nativeVideo && !audioMode.on}
				<div class="absolute inset-0 bg-black [&_video]:h-full [&_video]:w-full [&_video]:object-contain" bind:this={fallbackBox}></div>
			{/if}
			{#if audioMode.on}
				<AudioModeOverlay
					theme={undefined}
					thumbnail={`https://i.ytimg.com/vi/${videoId}/hqdefault.jpg`}
					title={page.title}
					channel={page.channel}
				/>
			{:else}
				<!-- A hole: the root paints its background everywhere except here, and mpv draws the
				     picture into it. Nothing in this box may paint over it, so there is no overlay. -->
			{/if}
		</div>

		<VideoControls fullscreen={toggleFullscreen} />
		<div class="mt-5 flex flex-col gap-6 lg:flex-row">
			<!-- Left: everything about this video, and what plays after it. -->
			<div class="min-w-0 flex-1">
				<h1 class="font-heading text-xl leading-snug font-bold tracking-tight md:text-2xl">
					{page.title}
				</h1>
				<div class="mt-3 flex flex-wrap items-center gap-3">
					{#if page.channelId}
						<button
							class="flex cursor-pointer items-center gap-2.5 text-left"
							onclick={() => page && goto(`/video/channel/${encodeURIComponent(page.channelId!)}`)}
						>
							{#if page.channelThumbnail}
								<img src={page.channelThumbnail} alt="" class="h-10 w-10 rounded-full" />
							{/if}
							<span>
								<span class="block text-sm font-semibold">{page.channel}</span>
								{#if page.subscribers}
									<span class="block text-xs text-muted-foreground">{page.subscribers}</span>
								{/if}
							</span>
						</button>
						<button
							class="cursor-pointer rounded-full px-4 py-2 text-sm font-medium transition-colors {page.subscribed
								? 'bg-foreground/10 hover:bg-foreground/15'
								: 'bg-foreground text-background'}"
							onclick={toggleSubscribe}
						>
							{page.subscribed ? 'Subscribed' : 'Subscribe'}
						</button>
					{/if}
					<div class="ml-auto flex items-center gap-2">
						<button
							class="flex cursor-pointer items-center gap-1.5 rounded-full bg-foreground/5 px-3 py-2 text-sm hover:bg-foreground/10 {rating ===
							'LIKE'
								? 'text-primary'
								: ''}"
							onclick={toggleLike}
							aria-pressed={rating === 'LIKE'}
						>
							<HugeiconsIcon icon={ThumbsUpIcon} class="h-4 w-4" />
							{page.likes ?? ''}
						</button>
						<button
							class="cursor-pointer rounded-full bg-foreground/5 px-3 py-2 text-sm hover:bg-foreground/10 {rating ===
							'DISLIKE'
								? 'text-primary'
								: ''}"
							onclick={toggleDislike}
							aria-pressed={rating === 'DISLIKE'}
						>
							<HugeiconsIcon icon={ThumbsDownIcon} class="h-4 w-4" />
						</button>
						<button
							class="flex cursor-pointer items-center gap-1.5 rounded-full bg-foreground/5 px-3 py-2 text-sm hover:bg-foreground/10 {saved
								? 'text-primary'
								: ''}"
							onclick={toggleSave}
							aria-pressed={saved}
							title={t('nav.watch_later')}
						>
							<HugeiconsIcon icon={Bookmark01Icon} class="h-4 w-4" />
						</button>
						<!-- Audio Mode: this video, and the ones after it, without the picture. -->
						<button
							class="flex cursor-pointer items-center gap-1.5 rounded-full px-3 py-2 text-sm transition-colors {audioMode.on
								? 'bg-primary text-primary-foreground'
								: 'bg-foreground/5 hover:bg-foreground/10'}"
							onclick={() => setAudioMode(!audioMode.on)}
							aria-pressed={audioMode.on}
							title={t('nav.audio_mode')}
						>
							<HugeiconsIcon icon={HeadphonesIcon} class="h-4 w-4" />
							{t('nav.audio_mode')}
						</button>
					</div>
				</div>
				<p class="mt-2 text-xs text-muted-foreground">
					{[page.views, page.published].filter(Boolean).join(' · ')}
				</p>

				{#if page.playlist && page.playlist.items.length}
					<section class="mt-5 overflow-hidden rounded-xl border border-foreground/10">
						<header
							class="flex items-center justify-between gap-3 bg-foreground/5 px-3 py-2 text-sm font-semibold"
						>
							<span class="truncate">{page.playlist.title}</span>
							<span class="shrink-0 text-xs font-normal text-muted-foreground">
								{(page.playlist.currentIndex ?? 0) + 1} / {page.playlist.items.length}
							</span>
						</header>
						<ul class="max-h-72 overflow-y-auto">
							{#each page.playlist.items as row, i (row.videoId + i)}
								<li>
									<button
										class="flex w-full cursor-pointer items-center gap-2 px-3 py-2 text-left text-sm hover:bg-foreground/5 {row.videoId ===
										videoId
											? 'bg-foreground/10 font-medium'
											: ''}"
										onclick={() =>
											goto(
												`/video/watch/${encodeURIComponent(row.videoId)}?list=${encodeURIComponent(page!.playlist!.playlistId)}&index=${i}`
											)}
									>
										<span class="w-6 shrink-0 text-xs text-muted-foreground">{i + 1}</span>
										<span class="min-w-0 flex-1 truncate">{row.title}</span>
										<span class="shrink-0 text-xs text-muted-foreground">{row.duration ?? ''}</span>
									</button>
								</li>
							{/each}
						</ul>
					</section>
				{:else if effectiveAutoplay}
					<section class="mt-5 rounded-xl bg-foreground/5 p-3">
						<p class="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
							{t('nav.up_next')}
						</p>
						<button
							class="mt-1.5 flex w-full cursor-pointer items-center gap-2 text-left text-sm hover:underline"
							onclick={() =>
								effectiveAutoplay &&
								goto(`/video/watch/${encodeURIComponent(effectiveAutoplay.videoId)}`)}
						>
							<span class="min-w-0 flex-1 truncate">{effectiveAutoplay.title}</span>
							{#if effectiveAutoplay.channel}
								<span class="shrink-0 text-xs text-muted-foreground">{effectiveAutoplay.channel}</span>
							{/if}
						</button>
					</section>
				{/if}

				{#if page.description}
					<section class="mt-4 rounded-xl bg-foreground/5 p-3">
						<button
							class="flex w-full cursor-pointer items-start gap-2 text-left text-sm whitespace-pre-line"
							onclick={() => (descOpen = !descOpen)}
						>
							<span class="min-w-0 flex-1 {descOpen ? '' : 'line-clamp-2'}">{page.description}</span>
							<HugeiconsIcon
								icon={descOpen ? ArrowUp01Icon : ArrowDown01Icon}
								class="mt-0.5 h-4 w-4 shrink-0 text-muted-foreground"
							/>
						</button>
					</section>
				{/if}

				<!-- Comments collapsed by default: they are the heaviest thing on the page, and most
				     visits are for the video. -->
				<section class="mt-4">
					<button
						class="cursor-pointer rounded-full bg-foreground/5 px-4 py-2 text-sm font-medium hover:bg-foreground/10"
						onclick={() => (commentsOpen = !commentsOpen)}
					>
						{commentsOpen ? t('nav.hide_comments') : t('nav.show_comments')}
						{#if page.commentCount}
							({page.commentCount})
						{/if}
					</button>
					{#if commentsOpen}
						<div class="mt-3">
							<VideoComments videoId={videoId} />
						</div>
					{/if}
				</section>
			</div>

			<!-- Right: the related rail (Full-length videos separated from Shorts). -->
			<aside class="w-full shrink-0 lg:w-96">
				<div class="mb-2 flex items-center justify-between">
					<h2 class="font-heading text-sm font-semibold tracking-wide text-muted-foreground uppercase">
						{t('nav.up_next')}
					</h2>
					<span class="text-[11px] text-muted-foreground">
						{separatedRelated.fullItems.length} videos
					</span>
				</div>
				<div class="flex flex-col gap-1">
					{#each separatedRelated.fullItems as item, i (item.kind + ':' + (item.kind === 'video' ? item.videoId : i))}
						<VideoCardView {item} onopen={openFeedItem} />
					{/each}
				</div>

				{#if separatedRelated.shortItems.length > 0}
					<div class="mt-4 rounded-xl border border-border/60 bg-card/40 p-2.5">
						<div class="flex items-center justify-between">
							<div class="flex items-center gap-2">
								<span class="inline-flex items-center rounded-md bg-red-500/15 px-2 py-0.5 text-[10px] font-bold tracking-wider text-red-500 uppercase">
									Shorts
								</span>
								<span class="text-xs text-muted-foreground">
									{separatedRelated.shortItems.length} related shorts
								</span>
							</div>
							<button
								type="button"
								class="cursor-pointer text-xs font-medium text-primary hover:underline"
								onclick={() => (showRelatedShorts = !showRelatedShorts)}
							>
								{showRelatedShorts ? 'Hide' : 'Show'}
							</button>
						</div>
						{#if showRelatedShorts}
							<div class="mt-2.5 grid grid-cols-2 gap-2">
								{#each separatedRelated.shortItems as item, i (item.kind + ':' + (item.kind === 'video' ? item.videoId : i))}
									<VideoCardView {item} vertical={true} onopen={openFeedItem} />
								{/each}
							</div>
						{/if}
					</div>
				{/if}

				{#if relatedMore}
					<button
						class="mt-3 w-full cursor-pointer rounded-full bg-foreground/5 px-4 py-2 text-sm hover:bg-foreground/10 disabled:opacity-50"
						onclick={loadMoreRelated}
						disabled={moreLoading}
					>
						{moreLoading ? 'Loading more…' : t('common.see_all')}
					</button>
				{/if}
			</aside>
		</div>
	{:else if loading}
		<div class="aspect-video w-full animate-pulse rounded-2xl bg-foreground/5"></div>
	{/if}
</div>
