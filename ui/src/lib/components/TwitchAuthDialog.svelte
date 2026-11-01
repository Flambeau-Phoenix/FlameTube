<script lang="ts">
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import {
		twitchSession,
		refreshTwitchStatus,
		refreshFollowedLive,
		signOutTwitch
	} from '$lib/twitch.svelte';
	import {
		twitchSetClientId,
		twitchDeviceStart,
		twitchDeviceWait,
		twitchDeviceCancel,
		openExternal,
		type TwitchDeviceCode
	} from '$lib/api';
	import { copyText } from '$lib/clipboard';
	import { toast, ui } from '$lib/player.svelte';
	import TwitchIcon from '$lib/components/TwitchIcon.svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import {
		Tick02Icon,
		Copy01Icon,
		Cancel01Icon,
		Logout01Icon,
		ArrowRight01Icon,
		Alert02Icon
	} from '@hugeicons/core-free-icons';

	let clientIdInput = $state('');
	let editingId = $state(false);
	let savingId = $state(false);
	let signingIn = $state(false);
	let deviceCode = $state<TwitchDeviceCode | null>(null);
	let copied = $state(false);
	let error = $state('');

	const status = $derived(twitchSession.status);
	const signedIn = $derived(!!status?.signedIn);
	const clientIdSet = $derived(!!status?.clientIdSet);

	$effect(() => {
		if (twitchSession.authOpen) {
			error = '';
			clientIdInput = status?.clientId ?? '';
			if (!clientIdSet) {
				editingId = true;
			}
		} else {
			if (signingIn) {
				void cancelSignIn();
			}
		}
	});

	async function saveClientId() {
		const id = clientIdInput.trim();
		if (!id) {
			error = 'Please enter a Twitch Client ID';
			return;
		}
		savingId = true;
		error = '';
		try {
			const s = await twitchSetClientId(id);
			twitchSession.status = s;
			editingId = false;
			toast('Twitch Client ID saved');
		} catch (e) {
			error = String(e);
		} finally {
			savingId = false;
		}
	}

	async function startSignIn() {
		signingIn = true;
		error = '';
		deviceCode = null;
		copied = false;
		try {
			const dc = await twitchDeviceStart();
			deviceCode = dc;
			// Automatically copy code to clipboard for convenience
			try {
				await copyText(dc.userCode);
				copied = true;
			} catch {
				// Non-fatal
			}

			// Start waiting for the user to authorize on twitch.tv/activate
			const login = await twitchDeviceWait(dc.deviceCode, dc.interval, dc.expiresIn, dc.generation);
			signingIn = false;
			deviceCode = null;
			await refreshTwitchStatus();
			await refreshFollowedLive(true);
			twitchSession.authOpen = false;
			toast(`Connected to Twitch as @${login}!`);
		} catch (e) {
			const msg = String(e);
			if (!msg.includes('cancelled')) {
				error = msg;
			}
			signingIn = false;
			deviceCode = null;
		}
	}

	async function cancelSignIn() {
		try {
			await twitchDeviceCancel();
		} catch {
			// Ignore cancel errors
		}
		signingIn = false;
		deviceCode = null;
	}

	async function openActivateUrl() {
		if (!deviceCode) return;
		try {
			await copyText(deviceCode.userCode);
			copied = true;
			await openExternal(deviceCode.verificationUri);
		} catch {
			error = 'Could not open browser. Please visit ' + deviceCode.verificationUri;
		}
	}

	async function openDevPortal() {
		try {
			await openExternal('https://dev.twitch.tv/console/apps');
		} catch {
			error = 'Could not open browser. Please visit dev.twitch.tv/console/apps';
		}
	}

	async function handleSignOut() {
		try {
			await signOutTwitch();
			toast('Signed out from Twitch');
		} catch (e) {
			error = String(e);
		}
	}
</script>

<Dialog.Root bind:open={twitchSession.authOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<div class="flex items-center gap-2.5">
				<div class="flex h-9 w-9 items-center justify-center rounded-lg bg-[#9146FF]/15 text-[#9146FF]">
					<TwitchIcon class="h-5 w-5" />
				</div>
				<div>
					<Dialog.Title class="text-lg font-semibold">Twitch Integration</Dialog.Title>
					<Dialog.Description class="text-xs text-muted-foreground">
						{signedIn ? `Connected as @${status?.login}` : 'Followed live channels and stream player'}
					</Dialog.Description>
				</div>
			</div>
		</Dialog.Header>

		{#if error}
			<div class="flex items-start gap-2 rounded-lg border border-destructive/30 bg-destructive/10 p-3 text-xs text-destructive">
				<HugeiconsIcon icon={Alert02Icon} class="mt-0.5 h-4 w-4 shrink-0" />
				<span class="flex-1">{error}</span>
			</div>
		{/if}

		{#if signedIn}
			<div class="space-y-4 py-2">
				<div class="rounded-xl border border-border bg-card p-4">
					<div class="flex items-center justify-between">
						<div class="flex items-center gap-3">
							<div class="flex h-10 w-10 items-center justify-center rounded-full bg-[#9146FF] font-semibold text-white">
								{status?.login?.slice(0, 1).toUpperCase() ?? 'T'}
							</div>
							<div>
								<p class="font-semibold text-foreground">@{status?.login}</p>
								<p class="text-xs text-muted-foreground">Ready to watch followed live streams</p>
							</div>
						</div>
						<div class="flex items-center gap-1.5 rounded-full bg-emerald-500/10 px-2.5 py-1 text-xs font-medium text-emerald-500">
							<span class="h-1.5 w-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
							<span>Connected</span>
						</div>
					</div>
				</div>

				<div class="flex flex-col gap-2">
					<Button
						variant="outline"
						class="w-full justify-center"
						onclick={() => { void refreshFollowedLive(true); toast('Refreshing live channels…'); }}
					>
						Refresh Live Channels
					</Button>
					<Button
						variant="ghost"
						class="w-full text-muted-foreground hover:text-foreground text-xs"
						onclick={() => editingId = !editingId}
					>
						{editingId ? 'Hide Client ID' : 'Change Twitch Client ID'}
					</Button>
					{#if editingId}
						<div class="mt-2 space-y-2 rounded-lg border border-border/80 bg-muted/40 p-3 text-xs">
							<label for="edit-client-id" class="font-medium text-foreground">Client ID</label>
							<Input
								id="edit-client-id"
								bind:value={clientIdInput}
								placeholder="Twitch Client ID (30 alphanumeric chars)"
								class="h-8 text-xs font-mono"
							/>
							<Button size="sm" class="w-full" disabled={savingId} onclick={saveClientId}>
								{savingId ? 'Saving…' : 'Update Client ID'}
							</Button>
						</div>
					{/if}
				</div>
			</div>

			<Dialog.Footer class="flex sm:justify-between items-center gap-2 pt-2">
				<Button variant="destructive" size="sm" onclick={handleSignOut} class="gap-1.5">
					<HugeiconsIcon icon={Logout01Icon} class="h-3.5 w-3.5" />
					<span>Sign Out</span>
				</Button>
				<Button variant="outline" size="sm" onclick={() => twitchSession.authOpen = false}>
					Done
				</Button>
			</Dialog.Footer>

		{:else if signingIn && deviceCode}
			<!-- Device Code in progress -->
			<div class="space-y-4 py-2">
				<div class="rounded-xl border border-[#9146FF]/30 bg-[#9146FF]/5 p-4 text-center">
					<p class="text-xs text-muted-foreground mb-1">Your Twitch activation code:</p>
					<div class="my-2 inline-flex items-center gap-2 rounded-lg bg-background px-4 py-2 font-mono text-2xl font-bold tracking-widest text-[#9146FF] border border-border">
						<span>{deviceCode.userCode}</span>
						<button
							class="text-muted-foreground hover:text-foreground transition-colors p-1"
							title="Copy code"
							onclick={async () => { await copyText(deviceCode?.userCode ?? ''); copied = true; }}
						>
							<HugeiconsIcon icon={copied ? Tick02Icon : Copy01Icon} class="h-5 w-5 {copied ? 'text-emerald-500' : ''}" />
						</button>
					</div>
					{#if copied}
						<p class="text-[11px] text-emerald-500 font-medium">Code copied to clipboard!</p>
					{/if}
					<p class="mt-2 text-xs text-muted-foreground">
						Enter this code at <span class="font-mono text-foreground font-medium">twitch.tv/activate</span> to authorize FlameTube.
					</p>
				</div>

				<div class="flex items-center justify-center gap-2 text-xs text-muted-foreground">
					<span class="h-2 w-2 rounded-full bg-[#9146FF] animate-ping"></span>
					<span>Waiting for authorization on Twitch…</span>
				</div>

				<Button
					class="w-full bg-[#9146FF] text-white hover:bg-[#772CE8] gap-1.5"
					onclick={openActivateUrl}
				>
					<span>Open twitch.tv/activate</span>
					<HugeiconsIcon icon={ArrowRight01Icon} class="h-4 w-4" />
				</Button>
			</div>

			<Dialog.Footer class="pt-2">
				<Button variant="ghost" size="sm" class="w-full" onclick={cancelSignIn}>
					Cancel
				</Button>
			</Dialog.Footer>

		{:else}
			<!-- Not signed in -->
			<div class="space-y-4 py-2 text-xs">
				{#if !clientIdSet || editingId}
					<div class="rounded-xl border border-border bg-card p-4 space-y-3">
						<div class="flex items-center justify-between">
							<span class="font-semibold text-foreground text-sm">Twitch Client ID</span>
							<button
								type="button"
								class="text-[#9146FF] hover:underline flex items-center gap-1 font-medium"
								onclick={openDevPortal}
							>
								<span>Get a free Client ID ↗</span>
							</button>
						</div>
						<p class="text-muted-foreground leading-relaxed">
							Twitch requires an application Client ID to look up followed channels. Create a free app in the Twitch Developer Console (Redirect URL: <code class="rounded bg-muted px-1 py-0.5 font-mono text-[11px]">http://localhost</code>) and paste the Client ID below.
						</p>
						<div class="flex gap-2">
							<Input
								bind:value={clientIdInput}
								placeholder="Paste Client ID (e.g. 30 chars)"
								class="h-9 font-mono text-xs"
							/>
							<Button size="sm" disabled={savingId} onclick={saveClientId}>
								{savingId ? 'Saving…' : 'Save'}
							</Button>
						</div>
					</div>
				{:else}
					<div class="flex items-center justify-between rounded-xl border border-border bg-muted/30 p-3">
						<div class="flex items-center gap-2">
							<HugeiconsIcon icon={Tick02Icon} class="h-4 w-4 text-emerald-500" />
							<span class="font-medium text-foreground">Client ID configured</span>
						</div>
						<button
							type="button"
							class="text-xs text-muted-foreground hover:text-foreground underline"
							onclick={() => editingId = true}
						>
							Edit
						</button>
					</div>

					<div class="rounded-xl border border-border bg-card p-4 space-y-2">
						<p class="font-medium text-foreground text-sm">Sign in to Twitch</p>
						<p class="text-muted-foreground leading-relaxed">
							Sign in with your Twitch account to see who you follow live directly in FlameTube. Only the read-only follow list is accessed; your credentials stay securely with Twitch.
						</p>
					</div>

					<Button
						class="w-full bg-[#9146FF] text-white hover:bg-[#772CE8] gap-2 py-5 text-sm font-semibold shadow-md"
						disabled={signingIn}
						onclick={startSignIn}
					>
						<TwitchIcon class="h-5 w-5" />
						<span>{signingIn ? 'Starting sign-in…' : 'Sign in with Twitch'}</span>
					</Button>
				{/if}
			</div>

			<Dialog.Footer class="pt-2">
				<Button variant="outline" size="sm" onclick={() => twitchSession.authOpen = false}>
					Close
				</Button>
			</Dialog.Footer>
		{/if}
	</Dialog.Content>
</Dialog.Root>
