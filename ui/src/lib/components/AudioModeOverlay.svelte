<script lang="ts">
	// What fills the video box in Audio Mode: the five-bar animation the youtube-audio-mode
	// extension drew over the hidden player, over the video's own blurred thumbnail or a theme.
	//
	// Nothing here reacts to the audio. The extension's bars never did either — they animate on a
	// CSS clock and pause with playback — and reacting would mean decoding the stream a second time
	// for a decoration. Cheap is the point of this mode.
	import { playback } from '$lib/player.svelte';
	import { audioMode, themeCss } from '$lib/video-mode.svelte';
	import { t } from '$lib/i18n.svelte';

	let {
		theme,
		thumbnail,
		title,
		channel
	}: {
		theme: string | undefined;
		thumbnail?: string;
		title: string;
		channel?: string;
	} = $props();
</script>

<div
	class="flex h-full w-full flex-col items-center justify-center gap-4 overflow-hidden rounded-2xl p-6 text-center"
	style="background:{themeCss(theme)}"
>
	{#if thumbnail}
		<!-- The cover, enlarged and blurred behind the gradient: the video's identity without the
		     pixels. One image, one filter, no per-frame work. -->
		<img
			src={thumbnail}
			alt=""
			aria-hidden="true"
			class="pointer-events-none absolute inset-0 h-full w-full scale-110 object-cover opacity-30 blur-2xl"
		/>
	{/if}
	<div class="relative flex flex-col items-center gap-3">
		<p class="text-xs font-semibold tracking-[0.2em] text-white/70 uppercase">
			{t('nav.audio_mode_active')}
		</p>
		<p class="line-clamp-2 max-w-xl text-lg font-semibold text-white">{title}</p>
		{#if channel}
			<p class="text-sm text-white/70">{channel}</p>
		{/if}
		<div class="audio-bars mt-2" class:paused={playback.paused}>
			<span></span><span></span><span></span><span></span><span></span>
		</div>
		<p class="text-[11px] text-white/50">{t('nav.audio_mode_hint')}</p>
	</div>
</div>

<style>
	.audio-bars {
		display: flex;
		align-items: flex-end;
		gap: 5px;
		height: 40px;
	}
	.audio-bars span {
		width: 6px;
		border-radius: 3px;
		background: rgba(255, 255, 255, 0.9);
		animation: bar 1.1s ease-in-out infinite;
	}
	.audio-bars span:nth-child(2) {
		animation-delay: 0.15s;
	}
	.audio-bars span:nth-child(3) {
		animation-delay: 0.3s;
	}
	.audio-bars span:nth-child(4) {
		animation-delay: 0.45s;
	}
	.audio-bars span:nth-child(5) {
		animation-delay: 0.6s;
	}
	.audio-bars.paused span {
		animation-play-state: paused;
		opacity: 0.45;
	}
	@keyframes bar {
		0%,
		100% {
			height: 8px;
		}
		50% {
			height: 36px;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.audio-bars span {
			animation: none;
			height: 18px;
		}
	}
</style>
