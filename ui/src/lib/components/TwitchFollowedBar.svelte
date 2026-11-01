<script lang="ts">
	import { onMount } from 'svelte';
	import {
		twitchSession,
		refreshTwitchStatus,
		refreshFollowedLive,
		tuneChannel
	} from '$lib/twitch.svelte';
	import TwitchIcon from '$lib/components/TwitchIcon.svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		PlayIcon,
		UserIcon,
		ArrowRight01Icon,
		ArrowLeft01Icon
	} from '@hugeicons/core-free-icons';

	let {
		navigate = false,
		compact = false,
		class: cls = ''
	}: {
		navigate?: boolean;
		compact?: boolean;
		class?: string;
	} = $props();

	let scrollContainer = $state<HTMLElement | null>(null);
	let canScrollLeft = $state(false);
	let canScrollRight = $state(false);

	const status = $derived(twitchSession.status);
	const signedIn = $derived(!!status?.signedIn);
	const channels = $derived(twitchSession.liveChannels);
	const loading = $derived(twitchSession.loadingLive);
	const error = $derived(twitchSession.liveError);
	const activeTarget = $derived(twitchSession.target);

	onMount(() => {
		if (!twitchSession.status) {
			void refreshTwitchStatus();
		} else if (signedIn && channels.length === 0) {
			void refreshFollowedLive();
		}
		updateScrollState();
	});

	function formatViewers(count: number): string {
		if (count >= 1_000_000) return (count / 1_000_000).toFixed(1) + 'M';
		if (count >= 1_000) return (count / 1_000).toFixed(1) + 'K';
		return count.toString();
	}

	function updateScrollState() {
		if (!scrollContainer) return;
		const { scrollLeft, scrollWidth, clientWidth } = scrollContainer;
		canScrollLeft = scrollLeft > 8;
		canScrollRight = scrollLeft + clientWidth < scrollWidth - 8;
	}

	function scrollBy(delta: number) {
		if (!scrollContainer) return;
		scrollContainer.scrollBy({ left: delta, behavior: 'smooth' });
	}

	function handleChannelClick(login: string) {
		void tuneChannel(login, navigate);
	}
</script>

<div class="group/bar relative flex w-full items-center gap-2 rounded-xl border border-border/80 bg-card/70 backdrop-blur-md p-1.5 shadow-xs transition-colors {cls}">
	<!-- Brand label + Quick status -->
	<div class="flex shrink-0 items-center gap-1.5 pl-1.5 pr-2 border-r border-border/60">
		<button
			type="button"
			class="flex items-center gap-1.5 rounded-lg px-2 py-1 text-xs font-semibold text-foreground transition-colors hover:bg-foreground/10"
			onclick={() => twitchSession.authOpen = true}
			title={signedIn ? `Twitch: @${status?.login} (Click to manage)` : 'Connect Twitch account'}
		>
			<TwitchIcon class="h-4 w-4 text-[#9146FF]" />
			<span class="hidden sm:inline">Twitch</span>
			{#if signedIn && channels.length > 0}
				<span class="flex items-center gap-1 rounded-full bg-red-500/15 px-1.5 py-0.2 text-[10px] font-bold text-red-500">
					<span class="h-1.5 w-1.5 rounded-full bg-red-500 animate-pulse"></span>
					<span>{channels.length}</span>
				</span>
			{/if}
		</button>

		<button
			type="button"
			class="flex h-6 w-6 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-foreground/10 hover:text-foreground"
			disabled={loading}
			onclick={() => void refreshFollowedLive(true)}
			title="Refresh followed channels"
		>
			<span class="text-xs transition-transform duration-300 {loading ? 'animate-spin' : ''}">↻</span>
		</button>
	</div>

	<!-- Scroll Left Arrow -->
	{#if canScrollLeft}
		<button
			type="button"
			class="absolute left-28 z-10 hidden md:flex h-7 w-7 items-center justify-center rounded-full bg-background/90 border border-border shadow-md text-foreground transition-opacity hover:bg-accent"
			onclick={() => scrollBy(-240)}
			aria-label="Scroll left"
		>
			<HugeiconsIcon icon={ArrowLeft01Icon} class="h-3.5 w-3.5" />
		</button>
	{/if}

	<!-- Main Horizontal Line of Followed Streamers -->
	<div
		bind:this={scrollContainer}
		onscroll={updateScrollState}
		class="flex flex-1 items-center gap-2 overflow-x-auto no-scrollbar scroll-smooth py-0.5"
	>
		{#if !signedIn}
			<!-- Signed-out banner inside the horizontal line -->
			<div class="flex items-center gap-2.5 py-1 px-2 text-xs">
				<span class="text-muted-foreground">See who you follow on Twitch live here:</span>
				<button
					type="button"
					class="inline-flex items-center gap-1.5 rounded-full bg-[#9146FF] px-3 py-1 text-xs font-semibold text-white shadow-xs transition-all hover:bg-[#772CE8]"
					onclick={() => twitchSession.authOpen = true}
				>
					<TwitchIcon class="h-3.5 w-3.5" />
					<span>Sign in with Twitch</span>
				</button>
			</div>

		{:else if loading && channels.length === 0}
			<!-- Loading skeleton pills -->
			{#each [1, 2, 3, 4] as _}
				<div class="flex shrink-0 items-center gap-2.5 rounded-xl border border-border/40 bg-muted/30 px-3 py-1.5 animate-pulse">
					<div class="h-7 w-7 rounded-full bg-muted"></div>
					<div class="space-y-1">
						<div class="h-3 w-16 rounded bg-muted"></div>
						<div class="h-2 w-10 rounded bg-muted"></div>
					</div>
				</div>
			{/each}

		{:else if channels.length === 0}
			<!-- No channels currently live -->
			<div class="flex items-center gap-2 py-1 px-2 text-xs text-muted-foreground">
				<span class="inline-block h-2 w-2 rounded-full bg-muted-foreground/40"></span>
				<span>No followed channels are live right now</span>
				<button
					type="button"
					class="text-xs text-[#9146FF] hover:underline font-medium ml-1"
					onclick={() => void refreshFollowedLive(true)}
				>
					Check again
				</button>
			</div>

		{:else}
			<!-- Live Followed Streamers in a Horizontal Row -->
			{#each channels as channel (channel.id || channel.login)}
				{@const isCurrent = activeTarget?.kind === 'channel' && activeTarget.id.toLowerCase() === channel.login.toLowerCase()}
				<button
					type="button"
					class="group/channel flex shrink-0 cursor-pointer items-center gap-2.5 rounded-xl border px-2.5 py-1.5 text-left transition-all duration-150 {isCurrent ? 'border-[#9146FF] bg-[#9146FF]/15 ring-2 ring-[#9146FF]/30' : 'border-border/60 bg-card/60 hover:border-primary/50 hover:bg-accent/40'}"
					onclick={() => handleChannelClick(channel.login)}
					title="{channel.name} · {channel.title} ({channel.game})"
				>
					<!-- Thumbnail / Avatar with live ring -->
					<div class="relative shrink-0">
						<img
							src={channel.thumbnail || `https://static-cdn.jtvnw.net/previews-ttv/live_user_${channel.login}-440x248.jpg`}
							alt={channel.name}
							class="h-7 w-7 rounded-full object-cover ring-2 {isCurrent ? 'ring-[#9146FF]' : 'ring-red-500'}"
							loading="lazy"
						/>
						<span class="absolute -bottom-0.5 -right-0.5 h-2.5 w-2.5 rounded-full border-2 border-background bg-red-500 animate-pulse"></span>
					</div>

					<!-- Streamer details -->
					<div class="min-w-0 max-w-[130px] {compact ? 'max-w-[100px]' : 'max-w-[140px]'} leading-tight">
						<div class="flex items-center gap-1">
							<span class="truncate text-xs font-semibold text-foreground group-hover/channel:text-primary transition-colors">
								{channel.name}
							</span>
						</div>
						<div class="truncate text-[10px] text-muted-foreground">
							{channel.game || 'Live'}
						</div>
					</div>

					<!-- Viewer count badge -->
					<div class="shrink-0 flex items-center gap-1 rounded-md bg-foreground/5 px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground group-hover/channel:text-foreground">
						<span class="h-1.5 w-1.5 rounded-full bg-red-500"></span>
						<span>{formatViewers(channel.viewers)}</span>
					</div>
				</button>
			{/each}
		{/if}
	</div>

	<!-- Scroll Right Arrow -->
	{#if canScrollRight}
		<button
			type="button"
			class="absolute right-2 z-10 hidden md:flex h-7 w-7 items-center justify-center rounded-full bg-background/90 border border-border shadow-md text-foreground transition-opacity hover:bg-accent"
			onclick={() => scrollBy(240)}
			aria-label="Scroll right"
		>
			<HugeiconsIcon icon={ArrowRight01Icon} class="h-3.5 w-3.5" />
		</button>
	{/if}
</div>
