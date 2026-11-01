<script lang="ts">
	import { onMount } from 'svelte';
	import { parseTwitch, twitchEmbed, twitchUrl, type TwitchTarget } from '$lib/twitch';
	import { twitchSession, tuneTwitch, refreshFollowedLive } from '$lib/twitch.svelte';
	import { playback, np, ui } from '$lib/player.svelte';
	import { openExternal } from '$lib/api';
	import TwitchFollowedBar from '$lib/components/TwitchFollowedBar.svelte';
	import TwitchBroadcasterProfile from '$lib/components/TwitchBroadcasterProfile.svelte';
	import TwitchEnhancedChat from '$lib/components/TwitchEnhancedChat.svelte';
	import TwitchIcon from '$lib/components/TwitchIcon.svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { Settings02Icon, Message01Icon } from '@hugeicons/core-free-icons';

	let input = $state(''), error = $state(''), busy = $state(false), ready = $state(false), chat = $state(true);
	let favorites = $state<string[]>([]), revision = $state(0);
	const target = $derived(twitchSession.target);
	const status = $derived(twitchSession.status);
	const parent = typeof location === 'undefined' ? 'localhost' : location.hostname;
	const button = 'rounded-md border border-border px-3 py-2 text-sm hover:bg-foreground/10 focus-visible:outline-2 focus-visible:outline-primary disabled:opacity-40 transition-colors';

	onMount(() => {
		try {
			const saved: unknown = JSON.parse(localStorage.getItem('flametube_twitch_favorites') ?? '[]');
			if (Array.isArray(saved)) favorites = [...new Set(saved.filter((v): v is string => typeof v === 'string' && /^[a-z0-9_]{1,25}$/.test(v)))].slice(0, 200);
		} catch { /* An empty favorites list remains usable if storage is unavailable. */ }
		if (target) void tune(target);
		else ready = true;
	});

	$effect(() => {
		if (target) {
			input = twitchUrl(target);
			ready = true;
		}
	});

	async function tune(next: TwitchTarget | null) {
		if (!next) { error = 'Enter a channel name or a Twitch channel, video, or clip link.'; return; }
		busy = true;
		ready = false;
		error = '';
		try {
			await tuneTwitch(next);
			input = twitchUrl(next);
			revision++;
			ready = true;
		} catch { error = 'Could not pause the other player. Try again.'; }
		finally { busy = false; }
	}

	function favorite() {
		if (target?.kind !== 'channel') return;
		favorites = favorites.includes(target.id) ? favorites.filter((id) => id !== target.id) : [...favorites, target.id].slice(0, 200);
		try { localStorage.setItem('flametube_twitch_favorites', JSON.stringify(favorites)); }
		catch { error = 'Favorites could not be saved on this device.'; }
	}

	async function external(url: string) {
		try { await openExternal(url); } catch { error = 'Could not open your browser.'; }
	}
</script>

<svelte:head><title>Twitch · FlameTube</title></svelte:head>
<section class="mx-auto w-full max-w-[1600px] space-y-4 p-5">
	<!-- Top Header with Brand, Account Management, and User Settings Hyperlink -->
	<div class="flex flex-wrap items-center justify-between gap-3">
		<div class="flex items-center gap-3">
			<div class="flex h-10 w-10 items-center justify-center rounded-xl bg-[#9146FF]/15 text-[#9146FF]">
				<TwitchIcon class="h-6 w-6" />
			</div>
			<div>
				<div class="flex items-center gap-2">
					<h1 class="text-2xl font-bold tracking-tight">Twitch</h1>
					<a
						href="#settings"
						class="inline-flex items-center gap-1 rounded-full border border-border/80 bg-card/60 px-2.5 py-0.5 text-xs font-medium text-muted-foreground hover:text-primary hover:border-primary/50 transition-colors"
						onclick={(e) => {
							e.preventDefault();
							ui.settingsOpen = true;
						}}
						title="Open User Settings"
					>
						<HugeiconsIcon icon={Settings02Icon} class="h-3.5 w-3.5" />
						<span>User Settings</span>
					</a>
				</div>
				<p class="text-xs text-muted-foreground">Followed live channels, broadcasts, and clips.</p>
			</div>
		</div>

		<div class="flex items-center gap-2">
			{#if status?.signedIn}
				<button
					class="flex items-center gap-2 rounded-full border border-border bg-card px-3.5 py-1.5 text-xs font-semibold text-foreground hover:bg-foreground/10 transition-colors"
					onclick={() => twitchSession.authOpen = true}
					title="Manage Twitch account"
				>
					<span class="h-2 w-2 rounded-full bg-emerald-500 animate-pulse"></span>
					<span>@{status.login}</span>
				</button>
			{:else}
				<button
					class="flex items-center gap-2 rounded-full bg-[#9146FF] px-4 py-1.5 text-xs font-semibold text-white hover:bg-[#772CE8] transition-colors shadow-xs"
					onclick={() => twitchSession.authOpen = true}
				>
					<TwitchIcon class="h-3.5 w-3.5" />
					<span>Connect Twitch</span>
				</button>
			{/if}
			<button class={button} onclick={() => external('https://www.twitch.tv/directory')}>Browse on Twitch ↗</button>
		</div>
	</div>

	<!-- Search & Tune Input -->
	<form class="flex gap-2" onsubmit={(e) => { e.preventDefault(); void tune(parseTwitch(input)); }}>
		<label class="sr-only" for="twitch-channel">Channel name or Twitch link</label>
		<input
			id="twitch-channel"
			maxlength="512"
			bind:value={input}
			placeholder="Channel name or Twitch link (e.g. shroud or twitch.tv/shroud)"
			autocomplete="off"
			spellcheck="false"
			class="min-w-0 flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-primary"
		/>
		<button class="rounded-md bg-primary px-5 py-2 text-sm font-semibold text-primary-foreground disabled:opacity-40" disabled={busy}>Watch</button>
	</form>
	{#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}

	{#if favorites.length}
		<div aria-label="Favorite Twitch channels" class="flex flex-wrap items-center gap-2">
			<span class="text-xs text-muted-foreground mr-1">Favorites:</span>
			{#each favorites as id}
				<button class="rounded-lg border border-border/80 bg-card/60 px-2.5 py-1 text-xs hover:bg-foreground/10" disabled={busy} onclick={() => tune({ kind: 'channel', id })}>{id}</button>
			{/each}
		</div>
	{/if}

	<!-- Followed Live Channels Horizontal Line (Preferably above the video!) -->
	<div class="pt-1">
		<TwitchFollowedBar navigate={false} />
	</div>

	<!-- Stream Area: Video Player & Enhanced Chat -->
	{#if target}
		<div class="flex flex-wrap items-center gap-2 pt-1">
			<div class="mr-auto flex items-center gap-2">
				<span class="h-2 w-2 rounded-full bg-red-500 animate-pulse"></span>
				<h2 class="text-lg font-semibold">{target.kind === 'channel' ? target.id : target.kind === 'video' ? `Video ${target.id}` : 'Twitch clip'}</h2>
			</div>
			{#if target.kind === 'channel'}
				<button class={button} aria-pressed={favorites.includes(target.id)} onclick={favorite}>{favorites.includes(target.id) ? '★ Favorited' : '☆ Favorite'}</button>
				<button class={button} aria-pressed={chat} onclick={() => chat = !chat}>
					<span class="inline-flex items-center gap-1.5">
						<HugeiconsIcon icon={Message01Icon} class="h-3.5 w-3.5" />
						<span>{chat ? 'Hide Chat' : 'Enhanced Chat'}</span>
					</span>
				</button>
			{/if}
			<button class={button} onclick={() => revision++}>Reload player</button>
			<button class={button} onclick={() => external(twitchUrl(target))}>Open on Twitch ↗</button>
			<button class={button} onclick={() => { twitchSession.target = null; twitchSession.returnPending = false; }}>Stop</button>
		</div>

		<!-- Video Player + Enhanced Chat side-by-side or stacked -->
		<div class="flex flex-wrap items-start gap-4">
			{#if ready}
				<div class="min-h-[360px] min-w-[400px] flex-1 aspect-video rounded-2xl bg-black border border-border/70 overflow-hidden shadow-xl">
					{#key `${target.kind}:${target.id}:${revision}`}
						<iframe
							title="Twitch player"
							src={twitchEmbed(target, parent)}
							allow="autoplay; fullscreen; picture-in-picture; encrypted-media"
							allowfullscreen
							class="h-full w-full border-0"
							referrerpolicy="strict-origin-when-cross-origin"
						></iframe>
					{/key}
				</div>
			{/if}

			<!-- Enhanced Chat options and embed -->
			{#if ready && chat && target.kind === 'channel'}
				<TwitchEnhancedChat
					{target}
					{parent}
					onClose={() => chat = false}
				/>
			{/if}
		</div>

		<!-- Broadcaster's Profile is visible UNDER THE VIDEO -->
		<div class="pt-2">
			<TwitchBroadcasterProfile
				{target}
				{favorites}
				onFavorite={favorite}
			/>
		</div>
	{:else}
		<div class="rounded-2xl border border-border bg-card/60 p-10 text-center space-y-3">
			<div class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-[#9146FF]/15 text-[#9146FF]">
				<TwitchIcon class="h-8 w-8" />
			</div>
			<h2 class="text-xl font-bold tracking-tight">Choose a channel to start watching</h2>
			<p class="mx-auto max-w-md text-xs text-muted-foreground leading-relaxed">
				Select any of your followed live streamers in the horizontal line above, enter a channel name in the search bar, or pick from your saved favorites.
			</p>
			<div class="pt-2">
				<a
					href="#settings"
					class="inline-flex items-center gap-1.5 text-xs text-primary font-medium hover:underline"
					onclick={(e) => {
						e.preventDefault();
						ui.settingsOpen = true;
					}}
				>
					<HugeiconsIcon icon={Settings02Icon} class="h-3.5 w-3.5" />
					<span>Configure FlameTube in User Settings</span>
				</a>
			</div>
		</div>
	{/if}
</section>
