<script lang="ts">
	import { onMount } from 'svelte';
	import type { TwitchTarget } from '$lib/twitch';
	import { twitchSession } from '$lib/twitch.svelte';
	import {
		twitchBroadcasterProfile,
		openExternal,
		type TwitchBroadcasterProfile as ProfileData
	} from '$lib/api';
	import { copyText } from '$lib/clipboard';
	import { toast, ui } from '$lib/player.svelte';
	import TwitchIcon from '$lib/components/TwitchIcon.svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		PlayIcon,
		UserIcon,
		Share01Icon,
		Settings02Icon,
		InformationCircleIcon
	} from '@hugeicons/core-free-icons';

	let {
		target,
		favorites = [],
		onFavorite
	}: {
		target: TwitchTarget | null;
		favorites?: string[];
		onFavorite?: () => void;
	} = $props();

	let profile = $state<ProfileData | null>(null);
	let loading = $state(false);
	let profileRequest = 0;
	let error = $state<string | null>(null);

	const channelLogin = $derived(
		target?.kind === 'channel' ? target.id.toLowerCase() : null
	);

	// Cross-reference with live channels already fetched
	const liveInfo = $derived(
		channelLogin
			? twitchSession.liveChannels.find((c) => c.login.toLowerCase() === channelLogin)
			: null
	);

	const isFavorited = $derived(channelLogin ? favorites.includes(channelLogin) : false);

	$effect(() => {
		if (channelLogin) {
			void loadProfile(channelLogin);
		} else {
			profileRequest++;
			profile = null;
		}
	});

	async function loadProfile(login: string) {
		const mine = ++profileRequest;
		loading = true;
		error = null;
		try {
			if (twitchSession.status?.signedIn) {
				const result = await twitchBroadcasterProfile(login);
				if (mine !== profileRequest || channelLogin !== login) return;
				profile = result;
			} else {
				// Fallback when signed out
				profile = {
					id: login,
					login,
					displayName: liveInfo?.name || login,
					description: '',
					profileImageUrl: liveInfo?.thumbnail || `https://static-cdn.jtvnw.net/jtv_user_pictures/${login}-profile_image-300x300.png`,
					broadcasterType: '',
					viewCount: 0,
					gameName: liveInfo?.game || '',
					title: liveInfo?.title || '',
					isLive: !!liveInfo,
					viewerCount: liveInfo?.viewers || 0,
					startedAt: liveInfo?.startedAt || ''
				};
			}
		} catch (e) {
			if (mine !== profileRequest || channelLogin !== login) return;
			// Non-fatal error; use basic card
			profile = {
				id: login,
				login,
				displayName: liveInfo?.name || login,
				description: '',
				profileImageUrl: liveInfo?.thumbnail || `https://static-cdn.jtvnw.net/jtv_user_pictures/${login}-profile_image-300x300.png`,
				broadcasterType: '',
				viewCount: 0,
				gameName: liveInfo?.game || '',
				title: liveInfo?.title || '',
				isLive: !!liveInfo,
				viewerCount: liveInfo?.viewers || 0,
				startedAt: liveInfo?.startedAt || ''
			};
		} finally {
			if (mine === profileRequest) loading = false;
		}
	}

	function formatUptime(startedAt: string): string {
		if (!startedAt) return '';
		try {
			const start = new Date(startedAt).getTime();
			const now = Date.now();
			const diffMins = Math.max(0, Math.floor((now - start) / 60000));
			const hours = Math.floor(diffMins / 60);
			const mins = diffMins % 60;
			if (hours > 0) return `${hours}h ${mins}m`;
			return `${mins}m`;
		} catch {
			return '';
		}
	}

	function formatViewers(count: number): string {
		if (count >= 1_000_000) return (count / 1_000_000).toFixed(1) + 'M';
		if (count >= 1_000) return (count / 1_000).toFixed(1) + 'K';
		return count.toLocaleString();
	}

	async function shareChannel() {
		if (!channelLogin) return;
		const url = `https://www.twitch.tv/${channelLogin}`;
		try {
			await copyText(url);
			toast('Channel link copied to clipboard!');
		} catch {
			toast('Could not copy link');
		}
	}

	async function openChannelUrl() {
		if (!channelLogin) return;
		try {
			await openExternal(`https://www.twitch.tv/${channelLogin}`);
		} catch {
			toast('Could not open browser');
		}
	}

	async function openSubsUrl() {
		if (!channelLogin) return;
		try {
			await openExternal(`https://www.twitch.tv/subs/${channelLogin}`);
		} catch {
			toast('Could not open browser');
		}
	}
</script>

{#if target && channelLogin}
	<div class="rounded-2xl border border-border bg-card/80 p-5 shadow-sm backdrop-blur-md transition-all">
		<div class="flex flex-col gap-5 md:flex-row md:items-start md:justify-between">
			<!-- Left: Broadcaster Info & Stream Title -->
			<div class="flex items-start gap-4">
				<!-- Avatar with Live Ring -->
				<div class="relative shrink-0">
					<img
						src={profile?.profileImageUrl || `https://static-cdn.jtvnw.net/jtv_user_pictures/${channelLogin}-profile_image-300x300.png`}
						alt={profile?.displayName || channelLogin}
						class="h-16 w-16 rounded-2xl object-cover ring-2 {liveInfo || profile?.isLive ? 'ring-red-500' : 'ring-border'} shadow-md"
						onerror={(e) => {
							// Fallback if image 404s
							(e.currentTarget as HTMLImageElement).src = `https://static-cdn.jtvnw.net/jtv_user_pictures/${channelLogin}-profile_image-300x300.png`;
						}}
					/>
					{#if liveInfo || profile?.isLive}
						<span class="absolute -bottom-1 -right-1 flex items-center gap-0.5 rounded-full bg-red-500 px-1.5 py-0.2 text-[9px] font-black uppercase text-white shadow-xs">
							LIVE
						</span>
					{/if}
				</div>

				<!-- Name, Title, and Categories -->
				<div class="space-y-1.5 min-w-0">
					<div class="flex flex-wrap items-center gap-2">
						<h3 class="text-xl font-bold tracking-tight text-foreground">
							{profile?.displayName || channelLogin}
						</h3>
						<span class="rounded-md bg-muted px-2 py-0.5 font-mono text-xs text-muted-foreground">
							@{channelLogin}
						</span>
						{#if profile?.broadcasterType === 'partner'}
							<span class="rounded-full bg-[#9146FF]/15 px-2 py-0.5 text-[10px] font-bold text-[#9146FF]">
								Partner
							</span>
						{/if}
					</div>

					<!-- Stream Title -->
					{#if liveInfo?.title || profile?.title}
						<p class="text-sm font-medium text-foreground/90 leading-snug line-clamp-2">
							{liveInfo?.title || profile?.title}
						</p>
					{/if}

					<!-- Metadata badges: Game, Viewers, Uptime -->
					<div class="flex flex-wrap items-center gap-2 pt-1 text-xs">
						{#if liveInfo?.game || profile?.gameName}
							<span class="inline-flex items-center gap-1 rounded-lg bg-primary/10 px-2.5 py-1 font-semibold text-primary">
								<span>🎮</span>
								<span>{liveInfo?.game || profile?.gameName}</span>
							</span>
						{/if}

						{#if liveInfo?.viewers || profile?.viewerCount}
							<span class="inline-flex items-center gap-1 rounded-lg bg-foreground/5 px-2.5 py-1 font-medium text-muted-foreground">
								<span class="h-1.5 w-1.5 rounded-full bg-red-500 animate-pulse"></span>
								<span>{formatViewers(liveInfo?.viewers || profile?.viewerCount || 0)} viewers</span>
							</span>
						{/if}

						{#if liveInfo?.startedAt || profile?.startedAt}
							{@const uptime = formatUptime(liveInfo?.startedAt || profile?.startedAt || '')}
							{#if uptime}
								<span class="inline-flex items-center gap-1 rounded-lg bg-foreground/5 px-2.5 py-1 font-medium text-muted-foreground">
									<span>⏱</span>
									<span>Uptime: {uptime}</span>
								</span>
							{/if}
						{/if}
					</div>
				</div>
			</div>

			<!-- Right: Broadcaster Action Buttons -->
			<div class="flex flex-wrap items-center gap-2 shrink-0">
				{#if onFavorite}
					<button
						type="button"
						class="flex items-center gap-1.5 rounded-xl border border-border px-3.5 py-2 text-xs font-semibold hover:bg-foreground/10 transition-colors {isFavorited ? 'text-amber-500 border-amber-500/40 bg-amber-500/10' : 'text-foreground'}"
						onclick={onFavorite}
					>
						<span>{isFavorited ? '★ Favorited' : '☆ Favorite'}</span>
					</button>
				{/if}

				<button
					type="button"
					class="flex items-center gap-1.5 rounded-xl border border-border px-3.5 py-2 text-xs font-semibold text-foreground hover:bg-foreground/10 transition-colors"
					onclick={shareChannel}
					title="Copy channel link"
				>
					<HugeiconsIcon icon={Share01Icon} class="h-3.5 w-3.5" />
					<span>Share</span>
				</button>

				<button
					type="button"
					class="flex items-center gap-1.5 rounded-xl bg-[#9146FF] px-4 py-2 text-xs font-semibold text-white hover:bg-[#772CE8] transition-colors shadow-xs"
					onclick={openChannelUrl}
				>
					<TwitchIcon class="h-3.5 w-3.5" />
					<span>Open on Twitch ↗</span>
				</button>

				<button
					type="button"
					class="flex items-center gap-1.5 rounded-xl border border-[#9146FF]/40 bg-[#9146FF]/10 px-3.5 py-2 text-xs font-semibold text-[#9146FF] hover:bg-[#9146FF]/20 transition-colors"
					onclick={openSubsUrl}
				>
					<span>Subscribe ↗</span>
				</button>
			</div>
		</div>

		<!-- Broadcaster Bio / Description -->
		{#if profile?.description}
			<div class="mt-4 pt-3 border-t border-border/60">
				<p class="text-xs text-muted-foreground leading-relaxed">
					{profile.description}
				</p>
			</div>
		{/if}

		<!-- User Settings Hyperlink Section -->
		<div class="mt-4 flex flex-wrap items-center justify-between border-t border-border/50 pt-3 text-xs text-muted-foreground">
			<div class="flex items-center gap-1">
				<span>Twitch integration active in FlameTube.</span>
				<span>Customize your themes, video preferences, and hotkeys in</span>
				<a
					href="#settings"
					class="font-semibold text-primary hover:underline inline-flex items-center gap-1 ml-0.5"
					onclick={(e) => {
						e.preventDefault();
						ui.settingsOpen = true;
					}}
				>
					<HugeiconsIcon icon={Settings02Icon} class="h-3.5 w-3.5" />
					<span>User Settings</span>
				</a>
			</div>

			<button
				type="button"
				class="text-[11px] text-[#9146FF] hover:underline font-medium"
				onclick={() => twitchSession.authOpen = true}
			>
				Manage Twitch Account ↗
			</button>
		</div>
	</div>
{/if}
