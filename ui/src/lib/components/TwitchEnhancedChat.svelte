<script lang="ts">
	import { onMount } from 'svelte';
	import type { TwitchTarget } from '$lib/twitch';
	import { openExternal } from '$lib/api';
	import { toast, ui } from '$lib/player.svelte';
	import { mode } from 'mode-watcher';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		Moon02Icon,
		Sun01Icon,
		Settings02Icon,
		Cancel01Icon,
		ViewIcon
	} from '@hugeicons/core-free-icons';

	let {
		target,
		parent,
		onClose
	}: {
		target: TwitchTarget;
		parent: string;
		onClose: () => void;
	} = $props();

	// Chat preferences
	let chatDark = $state(true);
	let chatWidth = $state<'normal' | 'compact' | 'wide'>('normal');
	let chatPosition = $state<'right' | 'bottom'>('right');
	let revision = $state(0);
	let optionsOpen = $state(false);

	onMount(() => {
		try {
			const saved = JSON.parse(localStorage.getItem('flametube_twitch_chat_prefs') ?? '{}');
			if (typeof saved.chatDark === 'boolean') chatDark = saved.chatDark;
			else chatDark = mode.current !== 'light';
			if (['normal', 'compact', 'wide'].includes(saved.chatWidth)) chatWidth = saved.chatWidth;
			if (['right', 'bottom'].includes(saved.chatPosition)) chatPosition = saved.chatPosition;
		} catch {
			chatDark = mode.current !== 'light';
		}
	});

	function savePrefs() {
		try {
			localStorage.setItem(
				'flametube_twitch_chat_prefs',
				JSON.stringify({ chatDark, chatWidth, chatPosition })
			);
		} catch {
			// Ignore storage write errors
		}
	}

	function toggleTheme() {
		chatDark = !chatDark;
		revision++;
		savePrefs();
	}

	function setWidth(w: 'normal' | 'compact' | 'wide') {
		chatWidth = w;
		savePrefs();
	}

	function togglePosition() {
		chatPosition = chatPosition === 'right' ? 'bottom' : 'right';
		savePrefs();
	}

	async function popoutChat() {
		const url = `https://www.twitch.tv/popout/${target.id}/chat?popout=`;
		try {
			await openExternal(url);
			toast('Opened Twitch popout chat in browser (7TV / FFZ compatible)');
		} catch {
			toast('Could not open browser');
		}
	}

	async function openChatterino() {
		const url = `chatterino://twitch/${target.id}`;
		try {
			await openExternal(url);
			toast(`Sent to Chatterino for #${target.id}`);
		} catch {
			toast('Could not launch Chatterino');
		}
	}

	const chatSrc = $derived.by(() => {
		const p = new URLSearchParams({ parent });
		if (chatDark) p.set('darkpopout', '');
		return `https://www.twitch.tv/embed/${target.id}/chat?${p.toString()}`;
	});

	const widthClass = $derived(
		chatPosition === 'bottom'
			? 'w-full h-[460px]'
			: chatWidth === 'compact'
			? 'w-[290px] h-[580px]'
			: chatWidth === 'wide'
			? 'w-[420px] h-[580px]'
			: 'w-[340px] h-[580px]'
	);
</script>

<div class="flex flex-col rounded-2xl border border-border bg-card shadow-lg overflow-hidden transition-all {widthClass}">
	<!-- Enhanced Chat Controls Toolbar -->
	<div class="flex items-center justify-between border-b border-border/80 bg-muted/40 px-3 py-2 text-xs">
		<div class="flex items-center gap-1.5 font-semibold text-foreground">
			<span class="h-2 w-2 rounded-full bg-[#9146FF]"></span>
			<span>Twitch Stream Chat</span>
		</div>

		<!-- Action buttons -->
		<div class="flex items-center gap-1">
			<!-- Theme Toggle (Dark/Light chat) -->
			<button
				type="button"
				class="flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={toggleTheme}
				title={chatDark ? 'Switch to light chat' : 'Switch to dark chat'}
			>
				<HugeiconsIcon icon={chatDark ? Sun01Icon : Moon02Icon} class="h-3.5 w-3.5" />
			</button>

			<!-- Position Toggle (Right vs Bottom) -->
			<button
				type="button"
				class="flex h-7 px-2 items-center justify-center rounded-lg text-[11px] font-medium text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={togglePosition}
				title="Toggle chat position (Side / Bottom)"
			>
				{chatPosition === 'right' ? 'Dock Bottom' : 'Dock Right'}
			</button>

			<!-- Width selector when on right side -->
			{#if chatPosition === 'right'}
				<div class="flex items-center rounded-lg bg-foreground/5 p-0.5 text-[10px] font-semibold">
					<button
						type="button"
						class="rounded px-1.5 py-0.5 transition-colors {chatWidth === 'compact' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
						onclick={() => setWidth('compact')}
						title="Compact width (290px)"
					>
						S
					</button>
					<button
						type="button"
						class="rounded px-1.5 py-0.5 transition-colors {chatWidth === 'normal' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
						onclick={() => setWidth('normal')}
						title="Standard width (340px)"
					>
						M
					</button>
					<button
						type="button"
						class="rounded px-1.5 py-0.5 transition-colors {chatWidth === 'wide' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
						onclick={() => setWidth('wide')}
						title="Wide width (420px)"
					>
						L
					</button>
				</div>
			{/if}

			<!-- Popout Chat (7TV/FFZ compatible) -->
			<button
				type="button"
				class="flex h-7 px-2 items-center gap-1 rounded-lg text-[11px] font-medium text-[#9146FF] hover:bg-[#9146FF]/10 transition-colors"
				onclick={popoutChat}
				title="Open Popout Chat in browser (Supports 7TV, FrankerFaceZ, BetterTTV emotes)"
			>
				<span>Popout ↗</span>
			</button>

			<!-- Chatterino Launcher -->
			<button
				type="button"
				class="flex h-7 px-1.5 items-center rounded-lg text-[11px] font-medium text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={openChatterino}
				title="Open in Chatterino desktop app"
			>
				<span>IRC</span>
			</button>

			<!-- Reload Chat -->
			<button
				type="button"
				class="flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={() => revision++}
				title="Reload chat"
			>
				<span class="text-xs">↻</span>
			</button>

			<!-- Close Chat -->
			<button
				type="button"
				class="flex h-7 w-7 items-center justify-center rounded-lg text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={onClose}
				title="Close chat"
			>
				<HugeiconsIcon icon={Cancel01Icon} class="h-3.5 w-3.5" />
			</button>
		</div>
	</div>

	<!-- Chat Iframe -->
	<div class="relative flex-1 bg-background">
		{#key `${target.id}:${revision}:${chatDark}`}
			<iframe
				title="Twitch chat"
				src={chatSrc}
				class="h-full w-full border-0"
				referrerpolicy="strict-origin-when-cross-origin"
			></iframe>
		{/key}
	</div>

	<!-- Enhanced Chat Footer / Links -->
	<div class="flex items-center justify-between border-t border-border/60 bg-muted/30 px-3 py-1.5 text-[11px] text-muted-foreground">
		<span class="truncate">Enhanced chat options active.</span>
		<div class="flex items-center gap-2 shrink-0">
			<a
				href="#settings"
				class="hover:text-primary hover:underline"
				onclick={(e) => {
					e.preventDefault();
					ui.settingsOpen = true;
				}}
			>
				Settings
			</a>
			<span>·</span>
			<button
				type="button"
				class="hover:text-[#9146FF] hover:underline"
				onclick={popoutChat}
			>
				7TV/FFZ Popout
			</button>
		</div>
	</div>
</div>
