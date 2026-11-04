<script lang="ts">
	import { onMount, tick } from 'svelte';
	import type { TwitchTarget } from '$lib/twitch';
	import {
		openExternal,
		twitchChatConnect,
		twitchChatDisconnect,
		twitchChatPoll,
		twitchChatSend,
		onTwitchChatBatch,
		type TwitchChatMessage,
		type TwitchChatSnapshot
	} from '$lib/api';
	import { twitchSession } from '$lib/twitch.svelte';
	import { toast, ui } from '$lib/player.svelte';
	import { mode } from 'mode-watcher';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		Moon02Icon,
		Sun01Icon,
		Cancel01Icon
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

	const RING_BUFFER_UI_CAPACITY = 500;

	// Chat preferences
	let chatDark = $state(true);
	let chatWidth = $state<'normal' | 'compact' | 'wide'>('normal');
	let chatPosition = $state<'right' | 'bottom'>('right');
	let chatMode = $state<'irc' | 'embed'>('irc');
	let revision = $state(0);

	// Native Rust IRC + Ring-Buffer state
	let messages = $state<TwitchChatMessage[]>([]);
	let lastSeq = $state(0);
	let connected = $state(false);
	let connecting = $state(false);
	let chatError = $state<string | null>(null);
	let draftMessage = $state('');
	let sending = $state(false);
	let autoScroll = $state(true);
	let chatListEl = $state<HTMLElement | null>(null);

	const fallbackColors = [
		'#FF4A4A',
		'#9146FF',
		'#00E5FF',
		'#10B981',
		'#F59E0B',
		'#EC4899',
		'#3B82F6',
		'#A855F7',
		'#14B8A6'
	];

	function usernameColor(msg: TwitchChatMessage): string {
		if (msg.color && msg.color.trim().length > 0) return msg.color;
		let hash = 0;
		for (let i = 0; i < msg.username.length; i++) {
			hash = (hash * 31 + msg.username.charCodeAt(i)) | 0;
		}
		return fallbackColors[Math.abs(hash) % fallbackColors.length];
	}

	function formatTime(ms: number): string {
		if (!ms) return '';
		const d = new Date(ms);
		return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
	}

	function appendMessages(incoming: TwitchChatMessage[]) {
		if (incoming.length === 0) return;
		const forCurrentChannel =
			target.kind === 'channel'
				? incoming.filter((m) => !m.channel || m.channel.toLowerCase() === target.id.toLowerCase())
				: [];
		if (forCurrentChannel.length === 0) return;
		const seenIds = new Set(messages.map((m) => m.id));
		const fresh = forCurrentChannel.filter((m) => !seenIds.has(m.id));
		if (fresh.length > 0) {
			const combined = [...messages, ...fresh];
			messages =
				combined.length > RING_BUFFER_UI_CAPACITY
					? combined.slice(combined.length - RING_BUFFER_UI_CAPACITY)
					: combined;
			for (const m of fresh) {
				if (m.seq > lastSeq) lastSeq = m.seq;
			}
			if (autoScroll) {
				void tick().then(() => scrollToBottom());
			}
		}
	}

	function applySnapshot(snap: TwitchChatSnapshot) {
		if (target.kind !== 'channel' || snap.channel.toLowerCase() !== target.id.toLowerCase()) {
			return;
		}
		connected = snap.connected;
		appendMessages(snap.messages);
		if (snap.latestSeq > lastSeq) {
			lastSeq = snap.latestSeq;
		}
	}

	function scrollToBottom() {
		if (!chatListEl) return;
		chatListEl.scrollTop = chatListEl.scrollHeight;
		autoScroll = true;
	}

	function handleChatScroll() {
		if (!chatListEl) return;
		const { scrollTop, scrollHeight, clientHeight } = chatListEl;
		const distanceFromBottom = scrollHeight - (scrollTop + clientHeight);
		autoScroll = distanceFromBottom < 48;
	}

	async function connectNativeChat(channelLogin: string) {
		if (!channelLogin) return;
		connecting = true;
		chatError = null;
		messages = [];
		lastSeq = 0;
		try {
			const snap = await twitchChatConnect(channelLogin);
			applySnapshot(snap);
		} catch (e) {
			chatError = String(e);
			connected = false;
		} finally {
			connecting = false;
		}
	}

	async function handleSendMessage(e: Event) {
		e.preventDefault();
		const text = draftMessage.trim();
		if (!text || sending || target.kind !== 'channel') return;
		sending = true;
		try {
			await twitchChatSend(target.id, text);
			draftMessage = '';
			scrollToBottom();
		} catch (err) {
			toast.error(String(err));
		} finally {
			sending = false;
		}
	}

	onMount(() => {
		try {
			const saved = JSON.parse(localStorage.getItem('flametube_twitch_chat_prefs') ?? '{}');
			if (typeof saved.chatDark === 'boolean') chatDark = saved.chatDark;
			else chatDark = mode.current !== 'light';
			if (['normal', 'compact', 'wide'].includes(saved.chatWidth)) chatWidth = saved.chatWidth;
			if (['right', 'bottom'].includes(saved.chatPosition)) chatPosition = saved.chatPosition;
			if (['irc', 'embed'].includes(saved.chatMode)) chatMode = saved.chatMode;
		} catch {
			chatDark = mode.current !== 'light';
		}

		let unlisten: (() => void) | null = null;
		void onTwitchChatBatch((batch) => {
			connected = true;
			appendMessages(batch);
		}).then((fn) => {
			unlisten = fn;
		});

		// Fallback lightweight poll to stay synchronized with the Rust ring buffer
		const pollTimer = setInterval(() => {
			if (chatMode === 'irc' && target.kind === 'channel') {
				void twitchChatPoll(lastSeq)
					.then((snap) => applySnapshot(snap))
					.catch(() => {});
			}
		}, 1000);

		return () => {
			clearInterval(pollTimer);
			unlisten?.();
			void twitchChatDisconnect().catch(() => {});
		};
	});

	// Reconnect native IRC chat whenever the target channel, revision, or mode changes
	$effect(() => {
		const kind = target.kind;
		const id = target.id;
		const modeNow = chatMode;
		void revision;
		if (kind === 'channel' && modeNow === 'irc') {
			void connectNativeChat(id);
		} else {
			void twitchChatDisconnect().catch(() => {});
		}
	});

	function savePrefs() {
		try {
			localStorage.setItem(
				'flametube_twitch_chat_prefs',
				JSON.stringify({ chatDark, chatWidth, chatPosition, chatMode })
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

	function setMode(m: 'irc' | 'embed') {
		chatMode = m;
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
	<div class="flex flex-wrap items-center justify-between gap-1.5 border-b border-border/80 bg-muted/40 px-3 py-2 text-xs">
		<div class="flex items-center gap-1.5 font-semibold text-foreground">
			<span class="h-2 w-2 rounded-full {chatMode === 'irc' ? (connected ? 'bg-emerald-500 animate-pulse' : connecting ? 'bg-amber-500 animate-pulse' : 'bg-red-500') : 'bg-[#9146FF]'}"></span>
			<span class="truncate">#{target.id}</span>
		</div>

		<!-- Mode Switcher (Native Rust IRC vs Web Embed) + Action buttons -->
		<div class="flex flex-wrap items-center gap-1">
			<div class="flex items-center rounded-lg bg-foreground/5 p-0.5 text-[10px] font-semibold">
				<button
					type="button"
					class="cursor-pointer rounded px-1.5 py-0.5 transition-colors {chatMode === 'irc' ? 'bg-[#9146FF] text-white' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => setMode('irc')}
					title="Native Rust IRC Chat (Tokio + Ring-Buffer)"
				>
					Native IRC
				</button>
				<button
					type="button"
					class="cursor-pointer rounded px-1.5 py-0.5 transition-colors {chatMode === 'embed' ? 'bg-[#9146FF] text-white' : 'text-muted-foreground hover:text-foreground'}"
					onclick={() => setMode('embed')}
					title="Twitch Web Embed Chat"
				>
					Embed
				</button>
			</div>

			<!-- Theme Toggle (Dark/Light chat) -->
			<button
				type="button"
				class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-lg text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={toggleTheme}
				title={chatDark ? 'Switch to light chat' : 'Switch to dark chat'}
			>
				<HugeiconsIcon icon={chatDark ? Sun01Icon : Moon02Icon} class="h-3.5 w-3.5" />
			</button>

			<!-- Position Toggle (Right vs Bottom) -->
			<button
				type="button"
				class="flex h-7 cursor-pointer px-2 items-center justify-center rounded-lg text-[11px] font-medium text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={togglePosition}
				title="Toggle chat position (Side / Bottom)"
			>
				{chatPosition === 'right' ? 'Bottom' : 'Side'}
			</button>

			<!-- Width selector when on right side -->
			{#if chatPosition === 'right'}
				<div class="flex items-center rounded-lg bg-foreground/5 p-0.5 text-[10px] font-semibold">
					<button
						type="button"
						class="cursor-pointer rounded px-1.5 py-0.5 transition-colors {chatWidth === 'compact' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
						onclick={() => setWidth('compact')}
						title="Compact width (290px)"
					>
						S
					</button>
					<button
						type="button"
						class="cursor-pointer rounded px-1.5 py-0.5 transition-colors {chatWidth === 'normal' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
						onclick={() => setWidth('normal')}
						title="Standard width (340px)"
					>
						M
					</button>
					<button
						type="button"
						class="cursor-pointer rounded px-1.5 py-0.5 transition-colors {chatWidth === 'wide' ? 'bg-primary text-primary-foreground' : 'text-muted-foreground hover:text-foreground'}"
						onclick={() => setWidth('wide')}
						title="Wide width (420px)"
					>
						L
					</button>
				</div>
			{/if}

			<!-- Reload Chat -->
			<button
				type="button"
				class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-lg text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={() => revision++}
				title="Reconnect / Reload chat"
			>
				<span class="text-xs">↻</span>
			</button>

			<!-- Close Chat -->
			<button
				type="button"
				class="flex h-7 w-7 cursor-pointer items-center justify-center rounded-lg text-muted-foreground hover:bg-foreground/10 hover:text-foreground transition-colors"
				onclick={onClose}
				title="Close chat"
			>
				<HugeiconsIcon icon={Cancel01Icon} class="h-3.5 w-3.5" />
			</button>
		</div>
	</div>

	{#if chatMode === 'irc'}
		<!-- Native Rust IRC Chat View -->
		<div class="relative flex min-h-0 flex-1 flex-col {chatDark ? 'bg-zinc-950 text-zinc-100' : 'bg-white text-zinc-900'}">
			{#if chatError}
				<div class="border-b border-red-500/30 bg-red-500/10 px-3 py-1.5 text-[11px] text-red-400 flex items-center justify-between gap-2">
					<span class="truncate">{chatError}</span>
					<button
						type="button"
						class="cursor-pointer font-semibold underline shrink-0"
						onclick={() => revision++}
					>
						Retry
					</button>
				</div>
			{/if}

			<div
				bind:this={chatListEl}
				onscroll={handleChatScroll}
				class="flex-1 overflow-y-auto px-3 py-2 space-y-1.5 text-[12.5px] leading-snug select-text"
			>
				{#if messages.length === 0}
					<div class="flex h-full flex-col items-center justify-center text-center p-4 text-xs text-muted-foreground space-y-1.5">
						{#if connecting}
							<span class="inline-block h-4 w-4 rounded-full border-2 border-[#9146FF] border-t-transparent animate-spin"></span>
							<p>Joining #{target.id} via Rust IRC…</p>
						{:else if connected}
							<p class="font-medium text-foreground/80">Connected to #{target.id}</p>
							<p class="text-[11px] opacity-75">Waiting for chat messages…</p>
						{:else}
							<p>Disconnected from chat.</p>
						{/if}
					</div>
				{:else}
					{#each messages as msg (msg.id)}
						<div class="break-words rounded px-1 py-0.5 hover:bg-foreground/5 transition-colors">
							<span class="mr-1.5 text-[10px] opacity-45 tabular-nums">{formatTime(msg.timestampMs)}</span>
							{#if msg.badges && msg.badges.length > 0}
								{#each msg.badges as badge}
									<span class="mr-1 inline-block rounded bg-[#9146FF]/20 px-1 py-0.2 text-[9px] font-bold uppercase tracking-wider text-[#b583ff]">
										{badge}
									</span>
								{/each}
							{/if}
							<span class="font-bold" style="color: {usernameColor(msg)}">{msg.username}</span><span class="opacity-70">: </span>
							<span class={msg.isAction ? 'italic opacity-90' : ''}>{msg.text}</span>
						</div>
					{/each}
				{/if}
			</div>

			{#if !autoScroll && messages.length > 0}
				<button
					type="button"
					class="absolute bottom-14 left-1/2 -translate-x-1/2 cursor-pointer rounded-full bg-[#9146FF] px-3 py-1 text-[11px] font-semibold text-white shadow-md hover:bg-[#772CE8] transition-all"
					onclick={scrollToBottom}
				>
					Chat paused · Jump to bottom ↓
				</button>
			{/if}

			<!-- Send Message Input Bar -->
			<form
				class="flex items-center gap-1.5 border-t border-border/60 bg-muted/20 p-2"
				onsubmit={handleSendMessage}
			>
				<input
					type="text"
					maxlength="500"
					bind:value={draftMessage}
					placeholder={twitchSession.status?.signedIn
						? `Send a message as @${twitchSession.status.login}…`
						: 'Sign in with Twitch to send chat messages…'}
					disabled={!twitchSession.status?.signedIn || sending}
					class="min-w-0 flex-1 rounded-lg border border-border/80 bg-background px-2.5 py-1.5 text-xs text-foreground placeholder:text-muted-foreground focus-visible:outline-2 focus-visible:outline-[#9146FF] disabled:opacity-50"
				/>
				{#if twitchSession.status?.signedIn}
					<button
						type="submit"
						disabled={sending || !draftMessage.trim()}
						class="cursor-pointer rounded-lg bg-[#9146FF] px-3 py-1.5 text-xs font-semibold text-white hover:bg-[#772CE8] disabled:opacity-40 transition-colors"
					>
						Chat
					</button>
				{:else}
					<button
						type="button"
						onclick={() => (twitchSession.authOpen = true)}
						class="cursor-pointer rounded-lg bg-[#9146FF] px-2.5 py-1.5 text-xs font-semibold text-white hover:bg-[#772CE8] transition-colors shrink-0"
					>
						Sign in
					</button>
				{/if}
			</form>
		</div>
	{:else}
		<!-- Chat Iframe (Fallback Web Embed) -->
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
	{/if}

	<!-- Enhanced Chat Footer / Links -->
	<div class="flex items-center justify-between border-t border-border/60 bg-muted/30 px-3 py-1.5 text-[11px] text-muted-foreground">
		<span class="truncate">
			{#if chatMode === 'irc'}
				{connected ? `IRC active · ${messages.length}/${RING_BUFFER_UI_CAPACITY} buffered` : 'IRC standby'}
			{:else}
				Web embed active
			{/if}
		</span>
		<div class="flex items-center gap-2 shrink-0">
			<button
				type="button"
				class="cursor-pointer hover:text-foreground hover:underline"
				onclick={openChatterino}
				title="Open in Chatterino desktop app"
			>
				Chatterino
			</button>
			<span>·</span>
			<button
				type="button"
				class="cursor-pointer hover:text-[#9146FF] hover:underline"
				onclick={popoutChat}
			>
				Popout ↗
			</button>
			<span>·</span>
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
		</div>
	</div>
</div>
