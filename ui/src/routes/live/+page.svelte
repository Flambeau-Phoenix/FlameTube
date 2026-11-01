<script lang="ts">
	import { onMount } from 'svelte';
	import { getCatalogue, playChannel, liveStatus, type LiveChannel } from '$lib/iptv';
	import * as api from '$lib/api';
	import { playback, prefs, videoReady } from '$lib/player.svelte';
	import { watch, setWatchHole, clearWatchHole } from '$lib/watch-hole.svelte';
	import VideoControls from '$lib/components/VideoControls.svelte';
	let channels = $state<LiveChannel[]>([]), selected = $state<LiveChannel | null>(null);
	let sourceIndex = $state(0), query = $state(''), category = $state(''), country = $state('');
	let english = $state(true), favoritesOnly = $state(false), favorites = $state<string[]>([]);
	let loading = $state(true), refreshing = $state(false), error = $state(''), playbackError = $state('');
	let source = $state(''), warning = $state(''), limit = $state(120), decoded = $state(false);
	let box = $state<HTMLElement | null>(null);
	let request = 0;
	let health = $state<Record<string, { ok: boolean; at: number }>>({});
	let hideFailed = $state(true);
	const quickCategories = ['News', 'Movies', 'Sports', 'Entertainment', 'Documentary', 'Kids', 'Music'];
	function recordHealth(id: string, ok: boolean) {
		health[id] = { ok, at: Date.now() };
		localStorage.setItem('flametube_live_health', JSON.stringify(health));
	}
	const countryNames = new Intl.DisplayNames(['en'], { type: 'region' });
	const countryLabel = (code: string) => { try { return countryNames.of(code) ?? code; } catch { return code; } };
	const categories = $derived([...new Set(channels.flatMap(c => c.categories))].sort());
	const countries = $derived([...new Set(channels.map(c => c.country).filter(Boolean))].sort((a,b) => countryLabel(a).localeCompare(countryLabel(b))));
	const filtered = $derived(channels.filter(c => (!english || c.languages.includes('eng')) &&
		(!country || c.country === country) && (!category || c.categories.includes(category)) &&
		(!favoritesOnly || favorites.includes(c.id)) && (!hideFailed || !health[c.id] || health[c.id].ok || Date.now() - health[c.id].at > 86400000) && `${c.name} ${c.categories.join(' ')}`.toLowerCase().includes(query.toLowerCase()))
		.sort((a,b) => Number(b.country === 'US') - Number(a.country === 'US') || a.name.localeCompare(b.name)));
	const current = $derived(!!selected && playback.now?.videoId === selected.id);
	async function load(refresh = false) {
		refreshing = refresh; error = '';
		try {
			const result = await getCatalogue(refresh); channels = result.channels; source = result.source; warning = result.warning ?? '';
			if (!selected && playback.now?.videoId.startsWith('iptv:')) selected = channels.find(c => c.id === playback.now?.videoId) ?? null;
		} catch (e) { error = String(e); } finally { loading = false; refreshing = false; }
	}
	async function play(channel: LiveChannel, index = 0) {
		const mine = ++request;
		selected = channel; sourceIndex = index; playbackError = ''; decoded = false;
		try { await playChannel(channel.id, index); } catch (e) { if (mine === request) playbackError = String(e); }
	}
	function favorite(id: string) {
		favorites = favorites.includes(id) ? favorites.filter(v => v !== id) : [...favorites, id];
		localStorage.setItem('flametube_live_favorites', JSON.stringify(favorites));
	}
	function measure() {
		if (!box || !current || !prefs.nativeVideo || !selected || !videoReady[selected.id]) { setWatchHole(null); return; }
		const r = box.getBoundingClientRect(); setWatchHole({ x:r.x, y:r.y, w:r.width, h:r.height });
	}
	function fullscreen() {
		if (document.fullscreenElement) void document.exitFullscreen(); else if (box) void box.requestFullscreen();
	}
	$effect(() => { void box; void current; void selected; void prefs.nativeVideo; void videoReady[selected?.id ?? '']; measure(); });
	onMount(() => {
		try { const saved = JSON.parse(localStorage.getItem('flametube_live_favorites') ?? '[]'); if (Array.isArray(saved)) favorites = saved.filter(v => typeof v === 'string'); } catch { /* ignore damaged preferences */ }
		try { const saved = JSON.parse(localStorage.getItem('flametube_live_health') ?? '{}'); health = Object.fromEntries(Object.entries(saved).filter(([,v]) => v && typeof v === 'object' && typeof (v as {ok?:unknown}).ok === 'boolean' && Number.isFinite((v as {at?:number}).at))) as typeof health; } catch { /* ignore damaged preferences */ }
		void load(); watch.want = true;
		let alive = true, off: (() => void) | undefined;
		void api.onPlaybackError(msg => { if (current && selected) { playbackError = msg; recordHealth(selected.id, false); } }).then(fn => { if (alive) off = fn; else fn(); });
		const resize = new ResizeObserver(measure); resize.observe(document.documentElement);
		const timer = setInterval(() => {
			if (box) resize.observe(box);
			if (current && selected) {
				const id = selected.id;
				void liveStatus(id).then(v => {
					if (!alive || selected?.id !== id) return;
					decoded = !!v;
					if (v) { sourceIndex = v.streamIndex; if (!health[id]?.ok) recordHealth(id, true); }
				}).catch(() => {});
			}
			measure();
		}, 1500);
		document.addEventListener('scroll', measure, true); document.addEventListener('fullscreenchange', measure);
		return () => { alive = false; request++; off?.(); resize.disconnect(); clearInterval(timer); clearWatchHole();
			document.removeEventListener('scroll', measure, true); document.removeEventListener('fullscreenchange', measure); };
	});
</script>

<div class="mx-auto w-full max-w-[110rem] px-6 py-6">
	<header class="mb-5 flex flex-wrap items-center justify-between gap-3">
		<div><h1 class="font-heading text-2xl font-bold">Live TV</h1><p class="mt-1 text-sm text-muted-foreground">Free public streams · English channels, United States first</p></div>
		<button class="rounded-full border px-4 py-2 text-sm disabled:opacity-50" disabled={refreshing} onclick={() => load(true)}>{refreshing ? 'Updating…' : 'Update channel list'}</button>
	</header>
	{#if selected}
		<section class="mb-6">
			<div class="relative aspect-video max-h-[65vh] w-full overflow-hidden rounded-xl {watch.hole ? 'bg-transparent' : 'bg-black'}" bind:this={box}>
				{#if !prefs.nativeVideo}<p class="absolute inset-0 flex items-center justify-center px-6 text-center text-white">Live TV needs the native video renderer on this platform.</p>
				{:else if !decoded && !playbackError}<p class="absolute inset-0 flex items-center justify-center text-white">Connecting to {selected.name}…</p>{/if}
			</div>
			<div class="mt-3 flex flex-wrap items-center gap-3"><h2 class="font-heading text-lg font-semibold">{selected.name}</h2><button class="rounded-lg border px-3 py-2 text-sm" aria-pressed={favorites.includes(selected.id)} onclick={() => favorite(selected!.id)}>{favorites.includes(selected.id) ? '★ Saved' : '☆ Favorite'}</button>
				<label class="ml-auto text-sm">Source <select class="ml-2 max-w-72 rounded-lg border bg-background p-2" value={sourceIndex} onchange={e => play(selected!, Number(e.currentTarget.value))}>{#each selected.streams as stream, index}<option value={index}>{index + 1}. {stream.label}</option>{/each}</select></label>
				<button class="rounded-lg border px-3 py-2 text-sm" onclick={() => play(selected!, (sourceIndex + 1) % selected!.streams.length)}>{selected.streams.length > 1 ? 'Try alternate source' : 'Reconnect'}</button>
			</div>
			{#if playbackError}<p class="my-2 rounded-lg border border-destructive p-3 text-sm" role="alert">This stream could not play. It may be offline, region-restricted, or unavailable at this time. Try another source or channel. {playbackError}</p>{/if}
			<VideoControls live {fullscreen} />
		</section>
	{/if}
	<div class="mb-4 flex flex-wrap items-center gap-3">
		<label class="min-w-52 flex-1"><span class="sr-only">Search channels</span><input class="w-full rounded-lg border bg-background px-3 py-2" placeholder="Search channels or categories" bind:value={query} oninput={() => limit = 120} /></label>
		<label class="text-sm">Category <select class="rounded-lg border bg-background p-2" bind:value={category} onchange={() => limit = 120}><option value="">All categories</option>{#each categories as value}<option value={value}>{value}</option>{/each}</select></label>
		<label class="text-sm">Country <select class="max-w-52 rounded-lg border bg-background p-2" bind:value={country} onchange={() => limit = 120}><option value="">All countries</option>{#each countries as code}<option value={code}>{countryLabel(code)}</option>{/each}</select></label>
		<label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={english} />English only</label>
		<label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={favoritesOnly} />Favorites</label>
		<label class="flex items-center gap-2 text-sm"><input type="checkbox" bind:checked={hideFailed} />Hide recently failed</label>
	</div>
	<div class="mb-4 flex flex-wrap gap-2">{#each quickCategories as value}<button class="rounded-full border px-3 py-1.5 text-sm {category === value ? 'bg-primary text-primary-foreground' : ''}" aria-pressed={category === value} onclick={() => { category = category === value ? '' : value; limit = 120; }}>{value}</button>{/each}</div>
	<p class="mb-4 text-xs text-muted-foreground">{filtered.length} channels · {source}. Listings do not guarantee availability. Country indicates channel origin. No TV guide data is bundled.</p>
	{#if warning || error}<p class="mb-4 rounded-lg border p-3 text-sm" role="status">{warning || error}</p>{/if}
	{#if loading}<p role="status">Loading channels…</p>{:else if !filtered.length}<p class="py-12 text-center text-muted-foreground">No channels match these filters. Try another category or turn off English only to include channels with unknown language.</p>
	{:else}
		<div class="grid grid-cols-1 gap-3 sm:grid-cols-2 xl:grid-cols-3 2xl:grid-cols-4">
			{#each filtered.slice(0, limit) as channel (channel.id)}
				<article class="flex items-center gap-3 rounded-xl border bg-card p-3 {selected?.id === channel.id ? 'ring-2 ring-primary' : ''}">
					<button class="flex min-w-0 flex-1 items-center gap-3 text-left" onclick={() => play(channel)}>
						<div class="flex h-14 w-20 shrink-0 items-center justify-center rounded-lg bg-white/90 p-1 text-black">{#if channel.logo}<img class="max-h-full max-w-full object-contain" src={channel.logo} alt="" loading="lazy" onerror={e => (e.currentTarget as HTMLImageElement).style.visibility = 'hidden'} />{:else}<span class="text-xs">LIVE TV</span>{/if}</div>
						<div class="min-w-0"><h3 class="line-clamp-2 text-sm font-semibold">{channel.name}</h3><p class="mt-1 truncate text-xs text-muted-foreground">{channel.categories.join(' · ') || 'Uncategorized'}</p><p class="text-xs text-muted-foreground">{channel.country ? countryLabel(channel.country) : 'Unknown origin'} · {channel.streams.length} source{channel.streams.length === 1 ? '' : 's'}</p><p class="mt-1 text-[11px] text-muted-foreground" title={health[channel.id] ? new Date(health[channel.id].at).toLocaleString() : 'No local playback result'}>{health[channel.id] ? `${health[channel.id].ok ? 'Played here' : 'Last attempt failed'} · ${new Date(health[channel.id].at).toLocaleDateString()}` : 'Untested here'}</p></div>
					</button>
					<button class="self-start p-1" aria-label={`${favorites.includes(channel.id) ? 'Remove' : 'Save'} ${channel.name} favorite`} aria-pressed={favorites.includes(channel.id)} onclick={() => favorite(channel.id)}>{favorites.includes(channel.id) ? '★' : '☆'}</button>
				</article>
			{/each}
		</div>
		{#if filtered.length > limit}<button class="mx-auto mt-5 block rounded-full border px-5 py-2" onclick={() => limit += 120}>Show more channels</button>{/if}
	{/if}
	<p class="mt-6 text-xs text-muted-foreground">Channel directory: <button class="underline" onclick={() => api.openExternal('https://github.com/iptv-org/iptv')}>iptv-org</button>. Availability depends on the broadcaster and your region.</p>
</div>
