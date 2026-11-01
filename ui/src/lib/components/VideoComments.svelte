<script lang="ts">
	// Video mode's comments. The docked CommentsPanel follows whatever is *playing*, which is right
	// for Music mode but wrong on a watch page: these comments belong to the video on screen, which
	// may not have started playing yet. So the page passes the id in explicitly.
	//
	// The row, composer and delete-dialog wiring mirror CommentsView.svelte (the player panel); this
	// one is the page-shaped container around the same store.
	import { untrack } from 'svelte';
	import { HugeiconsIcon } from '@hugeicons/svelte';
	import { ArrowDown01Icon, ArrowUp01Icon } from '@hugeicons/core-free-icons';
	import { Button } from '$lib/components/ui/button';
	import { Skeleton } from '$lib/components/ui/skeleton';
	import * as AlertDialog from '$lib/components/ui/alert-dialog';
	import {
		cancelDelete,
		comments,
		composing,
		confirmDelete,
		followTrack,
		loadComments,
		loadMoreComments,
		loadMoreReplies,
		sending,
		setCommentSort,
		submitCreate,
		toggleReplies,
		type ThreadView
	} from '$lib/comments.svelte';
	import { auth } from '$lib/player.svelte';
	import { t } from '$lib/i18n.svelte';
	import CommentRow from './CommentRow.svelte';
	import CommentComposer from './CommentComposer.svelte';

	let { videoId, title = '' }: { videoId: string; title?: string } = $props();

	const holding = $derived(composing());
	const count = $derived(comments.header?.count_text);
	const SORTS = ['top', 'newest'] as const;

	$effect(() => {
		const id = videoId;
		const name = title;
		void holding;
		void auth.epoch;
		if (id) untrack(() => followTrack(id, name));
	});

	let scroller = $state<HTMLElement | null>(null);

	function watch(node: HTMLElement) {
		const io = new IntersectionObserver(
			(entries) => {
				if (entries.some((e) => e.isIntersecting)) loadMoreComments();
			},
			{ root: scroller, rootMargin: '200px' }
		);
		io.observe(node);
		return () => io.disconnect();
	}
</script>

{#snippet skeletons()}
	{#each [0, 1, 2] as i (i)}
		<div class="flex gap-3 py-2">
			<Skeleton class="size-8 shrink-0 rounded-full" />
			<div class="min-w-0 flex-1 space-y-2">
				<Skeleton class="h-3 w-1/3 rounded" />
				<Skeleton class="h-3 w-full rounded" />
				<Skeleton class="h-3 w-2/3 rounded" />
			</div>
		</div>
	{/each}
{/snippet}

{#snippet replies(v: ThreadView)}
	{#if v.replies.length || v.repliesToken}
		<button
			class="mt-1 flex cursor-pointer items-center gap-1 text-xs font-medium text-primary"
			onclick={() => toggleReplies(v)}
		>
			<HugeiconsIcon
				icon={ArrowDown01Icon}
				altIcon={ArrowUp01Icon}
				showAlt={v.open}
				class="h-3.5 w-3.5"
			/>
			{v.open ? t('comments.hide_replies') : t('comments.show_replies')}
		</button>
	{/if}
	{#if v.open}
		<div class="mt-1 pl-6">
			{#each v.replies as r (r.id)}
				{#if !comments.hidden[r.id]}<CommentRow comment={r} reply />{/if}
			{/each}
			{#if v.loading}
				<Skeleton class="h-3 w-1/2 rounded" />
			{:else if v.repliesToken}
				<button
					class="cursor-pointer text-xs font-medium text-primary"
					onclick={() => loadMoreReplies(v)}
				>
					{t('comments.show_more_replies')}
				</button>
			{/if}
		</div>
	{/if}
{/snippet}

<div class="flex max-h-[36rem] flex-col">
	<div class="flex items-center justify-between gap-2 pb-2">
		<span class="text-sm font-semibold">
			{t('comments.title')}
			{#if count}
				<span class="ml-1 font-normal text-muted-foreground">
					{count === '1' ? t('comments.count_one') : t('comments.count', { count })}
				</span>
			{/if}
		</span>
		{#if comments.header?.sorts.length}
			<div class="flex shrink-0 items-center gap-3 text-xs" role="group" aria-label={t('sort.label')}>
				{#each SORTS as key (key)}
					<button
						type="button"
						aria-pressed={comments.header.sorts.find((s) => s.selected)?.key === key}
						class="cursor-pointer py-1 text-muted-foreground hover:text-foreground aria-pressed:font-medium aria-pressed:text-foreground"
						onclick={() => setCommentSort(key)}
					>
						{t(`comments.sort_${key}`)}
					</button>
				{/each}
			</div>
		{/if}
	</div>

	<div class="min-h-0 flex-1 overflow-y-auto" bind:this={scroller}>
		{#if comments.header?.composer && (comments.status === 'ready' || comments.status === 'loading')}
			<div class="border-b py-3">
				<CommentComposer
					bind:value={comments.create.text}
					placeholder={comments.header.composer.placeholder ?? t('comments.add_placeholder')}
					label={t('comments.composer_label')}
					submitLabel={t('comments.post')}
					pending={comments.create.pending}
					avatar="size-8"
					collapsible
					onsubmit={submitCreate}
				/>
			</div>
		{/if}
		{#if comments.status === 'idle' || comments.status === 'loading'}
			{@render skeletons()}
		{:else if comments.status === 'reload' || comments.status === 'error'}
			<Button variant="secondary" size="sm" class="mt-3" onclick={() => loadComments(videoId, true)}>
				{t('comments.retry')}
			</Button>
		{:else if comments.state === 'disabled'}
			<p class="py-6 text-sm text-muted-foreground">{t('comments.disabled')}</p>
		{:else if comments.state === 'empty' || comments.threads.length === 0}
			<p class="py-6 text-sm text-muted-foreground">{t('comments.empty')}</p>
		{:else}
			{#each comments.threads as v (v.comment.id)}
				{#if !comments.hidden[v.comment.id]}
					<CommentRow comment={v.comment}>
						{@render replies(v)}
					</CommentRow>
				{/if}
			{/each}
			{#if comments.loadingMore}
				{@render skeletons()}
			{:else if comments.continuation}
				{#key comments.threads.length}
					<div class="h-px" {@attach watch}></div>
				{/key}
			{/if}
		{/if}
	</div>
</div>

<!-- One dialog for the list, as in CommentsView: the row's menu sets `confirmDelete`. -->
<AlertDialog.Root open={!!comments.confirmDelete}>
	<AlertDialog.Content>
		<AlertDialog.Header>
			<AlertDialog.Title>{t('comments.delete_title')}</AlertDialog.Title>
			<AlertDialog.Description>{t('comments.delete_desc')}</AlertDialog.Description>
		</AlertDialog.Header>
		<AlertDialog.Footer>
			<AlertDialog.Cancel onclick={cancelDelete}>{t('common.cancel')}</AlertDialog.Cancel>
			<AlertDialog.Action onclick={confirmDelete} disabled={sending()}>
				{t('comments.delete')}
			</AlertDialog.Action>
		</AlertDialog.Footer>
	</AlertDialog.Content>
</AlertDialog.Root>
