<script lang="ts">
	import { ImagePlus } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { cn } from "$lib/utils";
	import { views, zine } from "$lib/zine.svelte";

	let { onChooseFile }: { onChooseFile: () => void } = $props();

	const view = $derived(views[zine.viewIndex]);
	const dim = "disabled:pointer-events-none disabled:opacity-50";

	// The viewer's file input targets the selected page, so select before opening it.
	function pickImage(index: number) {
		zine.selectedIndex = index;
		onChooseFile();
	}
</script>

<section class="flex divide-x-2 divide-border border-t-2 border-border bg-card">
	{#each view as index (index)}
		{@const page = zine.pages[index]}
		{@const hasImage = page.imageUrl !== null}
		<div
			class={cn(
				"flex min-w-0 flex-1 flex-wrap items-center gap-3 px-4 py-3",
				view.length > 1 && index === zine.selectedIndex && "bg-primary/20"
			)}
		>
			<button
				type="button"
				class="flex aspect-[3/4] h-16 shrink-0 cursor-pointer items-center justify-center overflow-hidden border-2 border-border bg-muted transition hover:bg-primary/30"
				aria-label={hasImage ? `Replace image for ${page.label}` : `Add image for ${page.label}`}
				title={hasImage ? "Click to replace image" : "Click to add image"}
				onclick={() => pickImage(index)}
			>
				{#if page.imageUrl}
					<img src={page.imageUrl} alt="" class="size-full object-cover" />
				{:else}
					<ImagePlus class="size-6" />
				{/if}
			</button>

			<span class="font-head">{page.label}</span>

			<div class="flex gap-2" role="group" aria-label="{page.label} image mode">
				<Button
					size="sm"
					class={dim}
					variant={page.mode === "fit" ? "default" : "outline"}
					disabled={!hasImage}
					onclick={() => zine.update(index, { mode: "fit" })}>Fit</Button
				>
				<Button
					size="sm"
					class={dim}
					variant={page.mode === "fill" ? "default" : "outline"}
					disabled={!hasImage}
					onclick={() => zine.update(index, { mode: "fill" })}>Fill</Button
				>
			</div>
			{#if page.mode === "fill" && hasImage}
				<span class="text-sm">Drag image to reframe</span>
			{/if}

			<Input
				class="min-w-40 flex-1"
				placeholder="Caption"
				aria-label="{page.label} caption"
				value={page.caption}
				oninput={(e) => zine.update(index, { caption: e.currentTarget.value })}
			/>

			<div class="flex gap-2">
				<Button size="sm" variant="outline" class={dim} disabled={!hasImage} onclick={() => zine.clearImage(index)}>
					Remove
				</Button>
			</div>
		</div>
	{/each}
</section>
