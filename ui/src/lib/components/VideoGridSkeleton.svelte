<!-- Placeholder cards while a Video mode feed is on the wire. Fixed count: a skeleton that matches
     the real grid's column maths, so the page does not jump when the cards land. -->
<div class="grid grid-cols-[repeat(auto-fill,minmax(15rem,1fr))] gap-3" aria-hidden="true">
	{#each Array(12) as _}
		<div class="flex flex-col gap-2 p-2">
			<div class="aspect-video w-full animate-pulse rounded-lg bg-foreground/5"></div>
			<div class="h-3.5 w-11/12 animate-pulse rounded bg-foreground/5"></div>
			<div class="h-3 w-2/3 animate-pulse rounded bg-foreground/5"></div>
		</div>
	{/each}
</div>
