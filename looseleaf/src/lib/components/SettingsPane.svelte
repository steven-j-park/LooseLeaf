<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { zine } from "$lib/zine.svelte";

	let { onChooseFile }: { onChooseFile: () => void } = $props();

	const page = $derived(zine.selected);
	const hasImage = $derived(page.imageUrl !== null);
	const dim = "disabled:pointer-events-none disabled:opacity-50";
</script>

<section class="flex flex-wrap items-center gap-4 border-t-2 border-border bg-card px-4 py-3">
	<span class="font-head">{page.label}</span>

	<div class="flex gap-2" role="group" aria-label="Image mode">
		<Button
			size="sm"
			class={dim}
			variant={page.mode === "fit" ? "default" : "outline"}
			disabled={!hasImage}
			onclick={() => zine.update(zine.selectedIndex, { mode: "fit" })}>Fit</Button
		>
		<Button
			size="sm"
			class={dim}
			variant={page.mode === "fill" ? "default" : "outline"}
			disabled={!hasImage}
			onclick={() => zine.update(zine.selectedIndex, { mode: "fill" })}>Fill</Button
		>
	</div>
	{#if page.mode === "fill" && hasImage}
		<span class="text-sm">Drag the image to reframe</span>
	{/if}

	<Input
		class="min-w-48 flex-1"
		placeholder="Caption"
		value={page.caption}
		oninput={(e) => zine.update(zine.selectedIndex, { caption: e.currentTarget.value })}
	/>

	<div class="flex gap-2">
		<Button size="sm" variant="outline" class={dim} disabled={!hasImage} onclick={onChooseFile}>Replace</Button>
		<Button size="sm" variant="outline" class={dim} disabled={!hasImage} onclick={() => zine.clearImage(zine.selectedIndex)}>
			Remove
		</Button>
	</div>
</section>
