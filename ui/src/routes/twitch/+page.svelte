<script lang="ts">
	import { onMount } from 'svelte';
	import { parseTwitch, twitchEmbed, twitchUrl, type TwitchTarget } from '$lib/twitch';
	import { twitchSession } from '$lib/twitch.svelte';
	import { playback, np, ui } from '$lib/player.svelte';
	import { openExternal, togglePause } from '$lib/api';
	let input = $state(''), error = $state(''), busy = $state(false), ready = $state(false), chat = $state(false);
	let favorites = $state<string[]>([]), revision = $state(0);
	const target = $derived(twitchSession.target);
	const parent = typeof location === 'undefined' ? 'localhost' : location.hostname;
	const button = 'rounded-md border border-border px-3 py-2 text-sm hover:bg-foreground/10 focus-visible:outline-2 focus-visible:outline-primary disabled:opacity-40';
	onMount(() => {
		try {
			const saved: unknown = JSON.parse(localStorage.getItem('flametube_twitch_favorites') ?? '[]');
			if (Array.isArray(saved)) favorites = [...new Set(saved.filter((v): v is string => typeof v === 'string' && /^[a-z0-9_]{1,25}$/.test(v)))].slice(0, 200);
		} catch { /* An empty favorites list remains usable if storage is unavailable. */ }
		if (target) void tune(target);
		else ready = true;
	});
	async function tune(next: TwitchTarget | null) {
		if (!next) { error = 'Enter a channel name or a Twitch channel, video, or clip link.'; return; }
		busy = true;
		ready = false;
		error = '';
		try {
			if (playback.now && !playback.paused) await togglePause();
			np.open = false;
			ui.theaterOpen = false;
			twitchSession.target = next;
			twitchSession.returnPending = true;
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
<section class="mx-auto w-full max-w-[1600px] space-y-5 p-5">
	<div class="flex flex-wrap items-center justify-between gap-3">
		<div><h1 class="text-2xl font-semibold">Twitch</h1><p class="mt-1 text-sm text-muted-foreground">Live channels, past broadcasts, and clips.</p></div>
		<button class={button} onclick={() => external('https://www.twitch.tv/directory')}>Browse on Twitch ↗</button>
	</div>
	<form class="flex gap-2" onsubmit={(e) => { e.preventDefault(); void tune(parseTwitch(input)); }}>
		<label class="sr-only" for="twitch-channel">Channel name or Twitch link</label>
		<input id="twitch-channel" maxlength="512" bind:value={input} placeholder="Channel name or Twitch link" autocomplete="off" spellcheck="false" class="min-w-0 flex-1 rounded-md border border-border bg-background px-3 py-2 text-sm focus-visible:outline-2 focus-visible:outline-primary" />
		<button class="rounded-md bg-primary px-5 py-2 text-sm font-semibold text-primary-foreground disabled:opacity-40" disabled={busy}>Watch</button>
	</form>
	{#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
	{#if favorites.length}
		<div aria-label="Favorite Twitch channels" class="flex flex-wrap gap-2">
			{#each favorites as id}<button class={button} disabled={busy} onclick={() => tune({ kind: 'channel', id })}>{id}</button>{/each}
		</div>
	{/if}
	{#if target}
		<div class="flex flex-wrap items-center gap-2">
			<h2 class="mr-auto text-lg font-semibold">{target.kind === 'channel' ? target.id : target.kind === 'video' ? `Video ${target.id}` : 'Twitch clip'}</h2>
			{#if target.kind === 'channel'}
				<button class={button} aria-pressed={favorites.includes(target.id)} onclick={favorite}>{favorites.includes(target.id) ? '★ Favorited' : '☆ Favorite'}</button>
				<button class={button} aria-pressed={chat} onclick={() => chat = !chat}>{chat ? 'Hide chat' : 'Show chat'}</button>
			{/if}
			<button class={button} onclick={() => revision++}>Reload player</button>
			<button class={button} onclick={() => external(twitchUrl(target))}>Open on Twitch ↗</button>
			<button class={button} onclick={() => { twitchSession.target = null; twitchSession.returnPending = false; }}>Stop</button>
		</div>
		<div class="flex flex-wrap gap-3">
			{#if ready}
			{#key `${target.kind}:${target.id}:${revision}`}
				<iframe title="Twitch player" src={twitchEmbed(target, parent)} allow="autoplay; fullscreen; picture-in-picture; encrypted-media" allowfullscreen class="min-h-[300px] min-w-[400px] flex-1 aspect-video rounded-lg bg-black" referrerpolicy="strict-origin-when-cross-origin"></iframe>
			{/key}
			{/if}
			{#if ready && chat && target.kind === 'channel'}
				<iframe title="Twitch chat" src={`https://www.twitch.tv/embed/${target.id}/chat?parent=${encodeURIComponent(parent)}`} class="h-[500px] w-[320px] rounded-lg border border-border" referrerpolicy="strict-origin-when-cross-origin"></iframe>
			{/if}
		</div>
		<p class="text-xs text-muted-foreground">Quality, volume, and fullscreen are in the Twitch player. Leaving this tab stops its player; Now Playing reopens it. If a stream is offline or unavailable here, try Open on Twitch.</p>
	{:else}
		<div class="rounded-lg border border-border p-8">
			<h2 class="text-lg font-semibold">Choose a channel to start watching</h2>
			<p class="mt-2 max-w-xl text-sm text-muted-foreground">Paste a Twitch link above or enter a channel name. Save favorite channels for quick access. Chat loads when you open it.</p>
		</div>
	{/if}
</section>
