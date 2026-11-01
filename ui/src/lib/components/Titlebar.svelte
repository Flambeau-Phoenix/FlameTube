<script lang="ts">
	// Custom titlebar. The window runs undecorated by default (tauri.conf `decorations: false`);
	// with a system frame (`win.chrome`, issue #65) this bar stays as a toolbar and only drops its
	// own window buttons, since everything else on it is app navigation, not window management.
	// Everything on the bar is a drag region except the buttons; double-click maximizes (Tauri's
	// drag region itself). Account and window controls live on the right.
	import { afterNavigate } from '$app/navigation';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		ArrowLeft01Icon,
		ArrowRight01Icon,
		Refresh03Icon,
		PlayIcon,
		MinusSignIcon,
		SquareIcon,
		CopyIcon,
		Cancel01Icon,
		MinimizeScreenIcon,
		CameraVideoIcon,
		UserGroup02Icon,
		Link04Icon
	} from '@hugeicons/core-free-icons';
	import AccountMenu from './AccountMenu.svelte';
	import { appIcon } from '$lib/appicon.svelte';
	import { np, openMiniPlayer, playback, refreshView, ui } from '$lib/player.svelte';
	import { nowPlayingHref } from '$lib/now-playing';
	import { twitchSession } from '$lib/twitch.svelte';
	import { win } from '$lib/win.svelte';
	import { lt } from '$lib/lt.svelte';
	import { t } from '$lib/i18n.svelte';
	import { videoMode, setMode } from '$lib/video-mode.svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	const twitchSection = $derived(page.url.pathname === '/twitch');
	const liveSection = $derived(page.url.pathname === '/live' || page.url.pathname.startsWith('/live/'));

	// `w` is this window; `win` (imported) is the shared frame state.
	const w = getCurrentWindow();
	function openSection(href: string, video = true) {
		np.open = false;
		ui.theaterOpen = false;
		setMode(video);
		void goto(href);
	}
	function returnToNowPlaying() {
		if (twitchSession.returnPending && twitchSession.target) {
			np.open = false;
			ui.theaterOpen = false;
			void goto('/twitch');
			return;
		}
		const id = playback.now?.videoId;
		if (!id) return;
		const href = nowPlayingHref(id, playback.queue.items[playback.queue.currentIndex]);
		ui.theaterOpen = false;
		if (href) {
			np.open = false;
			setMode(true);
			void goto(href);
		} else {
			setMode(false);
			np.open = true;
			void goto('/');
		}
	}

	// Back/forward. `depth` is how many history entries deep the session is, `deepest` how far it
	// has ever been, so both buttons grey out instead of doing nothing. popstate carries a signed
	// delta (the mouse's side buttons come through here); anything else is a push, which wipes the
	// entries ahead of us.
	let depth = $state(0);
	let deepest = $state(0);
	afterNavigate((nav) => {
		if (nav.type === 'enter') depth = deepest = 0;
		else if (nav.delta !== undefined) depth = Math.max(0, depth + nav.delta);
		else deepest = depth += 1;
	});

</script>

<!-- `relative` makes this a stacking context, so the account/window dropdowns inside it are capped
     at this z: it must outrank the panels below (LyricsPanel/QueuePanel, z-30). Theater mode is
     the exception: it sits at z-40 so dialogs (z-50) can open over it, so the bar has to duck
     under it instead of the other way round. Modal backdrops sit at z-[49], under this bar, so the
     window can still be dragged and closed with a dialog open (#427). pointer-events-auto because
     an open dialog sets `pointer-events: none` on <body>; `data-window-control` marks the buttons
     that don't dismiss it (`onWindowChrome`). -->
<header
	data-tauri-drag-region
	class="pointer-events-auto relative {ui.theaterOpen ? 'z-0' : 'z-50'} flex h-9 shrink-0 select-none items-center justify-between border-b border-border/60 bg-background"
>
	<span
		class="pointer-events-none absolute inset-x-0 hidden text-center text-xs font-medium tracking-wide text-muted-foreground 2xl:block"
	>
		FlameTube
	</span>

	<!-- macOS overlay style floats the traffic lights over the top-left of the webview, so the row
	     starts clear of them. `trafficLightPosition` in tauri.macos.conf.json centres them in this
	     36px bar and insets them 14px (#189); they end near 68px, so 70px plus the logo's margin
	     leaves about as much space after them as before them. Its `y` is not a top margin: tao
	     sizes the titlebar to the 16px button plus `y` and the buttons keep their offset from its
	     bottom, so their centre lands at y + 2 (y: 10 moved them up 2px on a real Mac). -->
	<div class="flex h-full items-center {win.chrome === 'overlay' ? 'pl-[70px]' : ''}">
		<!-- pointer-events-none: the logo is decoration; clicks on it should drag the window. -->
		<img src={appIcon.src} alt="" class="pointer-events-none ml-3 mr-1 h-4 w-4" />
		<!-- Bigger and heavier than the icons on the right: these are navigation, and at their
		     weight the arrow read as decoration and got missed. -->
		<button
			class="flex h-full w-9 items-center justify-center text-foreground/80 transition-colors hover:bg-accent/10 hover:text-foreground disabled:pointer-events-none disabled:opacity-25"
			onclick={() => history.back()}
			disabled={depth === 0}
			title={t('common.back')}
			aria-label={t('common.back')}
		>
			<HugeiconsIcon icon={ArrowLeft01Icon} strokeWidth={2.5} class="h-5 w-5" />
		</button>
		<button
			class="flex h-full w-9 items-center justify-center text-foreground/80 transition-colors hover:bg-accent/10 hover:text-foreground disabled:pointer-events-none disabled:opacity-25"
			onclick={() => history.forward()}
			disabled={depth === deepest}
			title={t('common.forward')}
			aria-label={t('common.forward')}
		>
			<HugeiconsIcon icon={ArrowRight01Icon} strokeWidth={2.5} class="h-5 w-5" />
		</button>
		<!-- Third in the browser's own control group, and read as such without a label. It drops the
		     browse cache and remounts the page, which is the only way to ask YouTube for a different
		     home feed: refetching rotates roughly a third of "Quick picks" (#177). Never disabled,
		     since the page it reloads is always there; the route's own skeletons are the progress. -->
		<button
			class="flex h-full w-9 items-center justify-center text-foreground/80 transition-colors hover:bg-accent/10 hover:text-foreground"
			onclick={refreshView}
			title={t('common.refresh')}
			aria-label={t('common.refresh')}
		>
			<HugeiconsIcon icon={Refresh03Icon} strokeWidth={2.5} class="h-4 w-4" />
		</button>
		<button
			class="ml-2 flex h-7 items-center gap-1.5 rounded-full border border-border px-3 text-xs font-medium transition-colors hover:bg-accent/10 disabled:pointer-events-none disabled:opacity-30"
			onclick={returnToNowPlaying}
			disabled={!playback.now && !twitchSession.returnPending}
			title={twitchSession.returnPending ? 'Return to Twitch' : playback.now ? `Return to ${playback.now.title}` : 'Nothing playing'}
			aria-label="Now Playing"
		>
			<HugeiconsIcon icon={PlayIcon} class="h-3.5 w-3.5" />
			<span>Now Playing</span>
		</button>
	</div>

	<div class="flex h-full items-center">
		<!-- Main destinations remain visible beside the account controls. -->
		<div role="group" aria-label="Main sections" class="mr-2 flex h-8 items-center gap-1 rounded-full bg-foreground/5 p-0.5 text-sm font-semibold">
			<button
				class="cursor-pointer rounded-full px-2 py-1 xl:px-3 transition-colors {videoMode.on && !liveSection && !twitchSection ? 'bg-primary text-primary-foreground shadow-sm' : 'text-foreground hover:bg-foreground/10'}"
				aria-pressed={videoMode.on && !liveSection && !twitchSection}
				onclick={() => openSection('/video')}
			>YouTube</button>
			<button
				class="cursor-pointer rounded-full px-2 py-1 xl:px-3 transition-colors {!videoMode.on && !liveSection && !twitchSection ? 'bg-primary text-primary-foreground shadow-sm' : 'text-foreground hover:bg-foreground/10'}"
				aria-pressed={!videoMode.on && !liveSection && !twitchSection}
				onclick={() => openSection('/', false)}>{t('nav.music')}</button
			>
			<button
				class="cursor-pointer rounded-full px-2 py-1 xl:px-3 transition-colors {liveSection ? 'bg-primary text-primary-foreground shadow-sm' : 'text-foreground hover:bg-foreground/10'}"
				aria-pressed={liveSection}
				onclick={() => openSection('/live')}>Live TV</button
			>
			<button
				class="cursor-pointer rounded-full px-2 py-1 xl:px-3 transition-colors {twitchSection ? 'bg-primary text-primary-foreground shadow-sm' : 'text-foreground hover:bg-foreground/10'}"
				aria-pressed={twitchSection}
				onclick={() => openSection('/twitch')}>Twitch</button>
		</div>
		<!-- Account first, then the integrations, then the window controls. The drag region lives on
		     <header> only, so these children are ordinary buttons — don't add the attribute here. -->
		<AccountMenu />
		<div class="mx-1.5 h-4 w-px bg-border"></div>

		<!-- Paste a YouTube Music link and go to it: the only way into a playlist that is shared by
		     link and never appears in search or the library (#63). -->
		<button
			class="flex h-full w-8 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/10 hover:text-foreground"
			onclick={() => (ui.linkOpen = true)}
			title={t('dialogs.link.title')}
			aria-label={t('dialogs.link.title')}
		>
			<HugeiconsIcon icon={Link04Icon} class="h-4 w-4" />
		</button>

		<!-- Opens the same modal as the home hero's button (one dialog, mounted in +layout). -->
		<button
			class="flex h-full w-8 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/10 hover:text-foreground {lt.role !==
			'none'
				? 'text-primary'
				: ''}"
			onclick={() => (ui.ltOpen = true)}
			title={t('nav.listen_together')}
			aria-label={t('nav.listen_together')}
		>
			<span class="relative">
				<HugeiconsIcon icon={UserGroup02Icon} class="h-4 w-4" />
				{#if lt.role !== 'none'}
					<!-- Discord's status dot with a ping behind it: two layers, because animate-ping
					     scales and fades the element it's on, so a lone dot would blink out. -->
					<span class="absolute -right-0.5 -top-0.5 h-1.5 w-1.5">
						<span class="absolute inset-0 animate-ping rounded-full bg-emerald-500 opacity-75"
						></span>
						<span class="absolute inset-0 rounded-full bg-emerald-500 ring-[1.5px] ring-background"
						></span>
					</span>
				{/if}
			</span>
		</button>

		<!-- Theater mode: fullscreen, cover and lyrics, nothing else. Next to the mini player because
		     the pair are the same idea in opposite directions (shrink the app / become the screen),
		     and disabled with nothing playing, since there'd be nothing to show. -->
		<button
			class="flex h-full w-8 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/10 hover:text-foreground disabled:pointer-events-none disabled:opacity-25"
			onclick={() => (ui.theaterOpen = true)}
			disabled={!playback.now}
			title={t('player.theater_mode')}
			aria-label={t('player.theater_mode')}
		>
			<HugeiconsIcon icon={CameraVideoIcon} class="h-4 w-4" />
		</button>

		<!-- Mini player: hides the app to the tray and hands over to the floating widget (mini.rs).
		     It sits with the integrations rather than the window controls because it swaps what
		     you're using, not the size of this window. -->
		<button
			class="flex h-full w-8 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/10 hover:text-foreground"
			onclick={openMiniPlayer}
			title={t('a11y.toggle_mini')}
			aria-label={t('a11y.toggle_mini')}
		>
			<HugeiconsIcon icon={MinimizeScreenIcon} class="h-4 w-4" />
		</button>

		{#if win.chrome === 'off'}
			<div class="mx-1.5 h-4 w-px bg-border"></div>

			<!-- Minimize and maximize follow the desktop's button layout (#481). -->
			{#if win.minimize}
				<button
					class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/10 hover:text-foreground"
					data-window-control
					onclick={() => w.minimize()}
					aria-label={t('common.minimize')}
				>
					<HugeiconsIcon icon={MinusSignIcon} class="h-4 w-4" />
				</button>
			{/if}
			{#if win.maximize}
				<button
					class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:bg-accent/10 hover:text-foreground"
					data-window-control
					onclick={() => w.toggleMaximize()}
					aria-label={win.maximized ? t('common.restore') : t('common.maximize')}
				>
					<!-- Maximized: the restore glyph, two overlapping squares with the front one low-left.
					     That is CopyIcon turned half a turn. altIcon/showAlt, since `icon` is frozen at
					     mount (#428). -->
					<HugeiconsIcon
						icon={SquareIcon}
						altIcon={CopyIcon}
						showAlt={win.maximized}
						class="h-3.5 w-3.5 {win.maximized ? 'rotate-180' : ''}"
					/>
				</button>
			{/if}
			<button
				class="flex h-full w-11 items-center justify-center text-muted-foreground transition-colors hover:text-destructive"
				data-window-control
				onclick={() => w.close()}
				aria-label={t('common.close')}
			>
				<HugeiconsIcon icon={Cancel01Icon} class="h-4 w-4" />
			</button>
		{:else}
			<!-- The system frame owns closing; leave a little air before its own buttons. -->
			<div class="w-2"></div>
		{/if}
	</div>
</header>
