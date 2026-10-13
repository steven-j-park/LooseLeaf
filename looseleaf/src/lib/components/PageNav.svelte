<script lang="ts">
	import { PanelLeftClose, PanelLeftOpen } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { cn } from "$lib/utils";
	import { views, zine } from "$lib/zine.svelte";
</script>

<nav
	class={cn(
		"flex flex-col gap-2 overflow-y-auto border-r-2 border-border bg-card p-2",
		zine.navExpanded ? "w-48" : "w-14"
	)}
	aria-label="Pages"
>
	<Button
		variant="ghost"
		size="icon"
		class="self-end"
		aria-label={zine.navExpanded ? "Collapse page list" : "Expand page list"}
		onclick={() => (zine.navExpanded = !zine.navExpanded)}
	>
		{#if zine.navExpanded}<PanelLeftClose />{:else}<PanelLeftOpen />{/if}
	</Button>

	{#each views as view, v (v)}
		{@const active = v === zine.viewIndex}
		{@const label =
			view.length > 1
				? `Pages ${view[0] + 1}–${view[1] + 1}`
				: zine.pages[view[0]].label}
		<button
			type="button"
			aria-current={active ? "page" : undefined}
			aria-label={label}
			onclick={() => (zine.selectedIndex = view[0])}
			class={cn(
				"flex cursor-pointer items-center border-2 border-border text-left font-head transition",
				active ? "bg-primary shadow-md" : "bg-background hover:bg-muted",
				zine.navExpanded ? "gap-2 p-1.5" : "h-10 shrink-0 justify-center self-center px-1",
				!zine.navExpanded && (view.length > 1 ? "w-12" : "w-10")
			)}
		>
			{#if zine.navExpanded}
				<span class="flex shrink-0 border border-border bg-muted">
					{#each view as i, n (i)}
						<span
							class={cn(
								"flex aspect-[3/4] w-10 items-center justify-center overflow-hidden",
								n > 0 && "border-l border-dashed border-border"
							)}
						>
							{#if zine.pages[i].imageUrl}
								<img src={zine.pages[i].imageUrl} alt="" class="size-full object-cover" />
							{/if}
						</span>
					{/each}
				</span>
				<span class="text-sm">{label}</span>
			{:else}
				{view.length > 1 ? `${view[0] + 1}–${view[1] + 1}` : view[0] + 1}
			{/if}
		</button>
	{/each}
</nav>
