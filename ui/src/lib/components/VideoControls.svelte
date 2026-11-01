<script lang="ts">
	import { onMount } from 'svelte';
	import * as api from '$lib/api';
	import { playback, toast, commitVolume } from '$lib/player.svelte';
	import { videoSettings, setVideoQuality } from '$lib/video-mode.svelte';
	let { fullscreen, live = false }: { fullscreen: () => void; live?: boolean } = $props();
	let dimensions = $state<[number, number] | null>(null);
	let changing = $state(false);
	async function quality(event: Event) {
		changing = true;
		try { await setVideoQuality(Number((event.target as HTMLSelectElement).value)); }
		catch (e) { toast.error(String(e)); } finally { changing = false; }
	}
	onMount(() => {
		let alive = true;
		const update = () => api.playbackVideoInfo().then(v => { if (alive) dimensions = v; }).catch(() => {});
		void update(); const timer = setInterval(update, 1500);
		return () => { alive = false; clearInterval(timer); };
	});
	const fmt = (s: number) => `${Math.floor(s / 60)}:${String(Math.floor(s % 60)).padStart(2, '0')}`;
</script>

<div class="my-3 flex flex-wrap items-center gap-3 rounded-xl border bg-card p-3 text-sm">
	<button class="rounded-lg border px-3 py-2" onclick={() => api.togglePause()}>{playback.paused ? 'Play' : 'Pause'}</button>
	{#if live}<span class="font-semibold text-red-500">LIVE</span>{:else}
		<button class="rounded-lg border px-2 py-2" aria-label="Back 10 seconds" onclick={() => api.seek(Math.max(0, playback.position - 10))}>−10s</button>
		<button class="rounded-lg border px-2 py-2" aria-label="Forward 10 seconds" onclick={() => api.seek(Math.min(playback.duration, playback.position + 10))}>+10s</button>
		<label class="flex min-w-40 flex-1 items-center gap-2"><span class="sr-only">Seek video</span><input class="w-full accent-primary" type="range" min="0" max={playback.duration || 1} step="1" value={playback.position} onchange={e => api.seek(Number(e.currentTarget.value))} /><span class="whitespace-nowrap tabular-nums">{fmt(playback.position)} / {fmt(playback.duration)}</span></label>
		<label class="flex items-center gap-2">Quality <select class="rounded-lg border bg-background px-2 py-2" value={String(videoSettings.quality)} onchange={quality} disabled={changing}><option value="2160">Up to 4K</option><option value="1440">Up to 1440p</option><option value="1080">Up to 1080p</option><option value="720">Up to 720p</option><option value="480">Up to 480p</option><option value="360">Up to 360p</option></select></label>
		<label>Speed <select class="rounded-lg border bg-background p-2" value={playback.speed} onchange={e => api.setPlaybackParams(Number(e.currentTarget.value), playback.semitones, playback.reverb)}>{#each [0.5, 0.75, 1, 1.25, 1.5, 2] as speed}<option value={speed}>{speed}×</option>{/each}</select></label>
	{/if}
	<label class="flex items-center gap-2">Volume <input class="w-20 accent-primary" type="range" min="0" max="100" value={playback.volume} onchange={e => commitVolume(Number(e.currentTarget.value))} /></label>
	<span class="text-xs text-muted-foreground">{dimensions ? `${dimensions[0]} × ${dimensions[1]}` : 'Waiting for video'}</span>
	<button class="rounded-lg border px-3 py-2" onclick={fullscreen}>Fullscreen</button>
</div>
