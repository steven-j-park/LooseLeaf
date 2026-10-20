<script lang="ts">
	import { ImagePlus } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { ASPECTS, zine, type Aspect } from "$lib/zine.svelte";

	let {
		index,
		slot: s,
		onChooseFile,
	}: { index: number; slot: number; onChooseFile: (page: number, slot: number) => void } = $props();

	const page = $derived(zine.pages[index]);
	const slot = $derived(page.images[s]);
	const hasImage = $derived(slot.path !== null);
	const dim = "disabled:pointer-events-none disabled:opacity-50";
	const aspects = Object.keys(ASPECTS) as Aspect[];

	function pickImage() {
		zine.selectedIndex = index;
		onChooseFile(index, s);
	}
</script>

<div class="flex flex-wrap items-center gap-3">
	<button
		type="button"
		class="flex aspect-[3/4] h-12 shrink-0 cursor-pointer items-center justify-center overflow-hidden border-2 border-border bg-muted transition hover:bg-primary/30"
		aria-label={hasImage ? `Replace image ${s + 1} for ${page.label}` : `Add image ${s + 1} for ${page.label}`}
		title={hasImage ? "Click to replace image" : "Click to add image"}
		onclick={pickImage}
	>
		{#if slot.path}
			<img src={slot.thumbUrl} alt="" class="size-full object-cover" />
		{:else}
			<ImagePlus class="size-5" />
		{/if}
	</button>

	<div class="flex gap-2" role="group" aria-label="{page.label} image {s + 1} mode">
		<Button
			size="sm"
			class={dim}
			variant={slot.mode === "fit" ? "default" : "outline"}
			disabled={!hasImage}
			onclick={() => zine.updateSlot(index, s, { mode: "fit" })}>Fit</Button
		>
		<Button
			size="sm"
			class={dim}
			variant={slot.mode === "fill" ? "default" : "outline"}
			disabled={!hasImage}
			onclick={() => zine.updateSlot(index, s, { mode: "fill" })}>Fill</Button
		>
	</div>

	<label class="flex items-center gap-1.5 text-sm">
		Aspect
		<select
			class="h-8 border-2 border-border bg-background px-2 disabled:opacity-50"
			disabled={!hasImage}
			value={slot.aspect}
			onchange={(e) => zine.updateSlot(index, s, { aspect: e.currentTarget.value as Aspect })}
		>
			{#each aspects as a (a)}
				<option value={a}>{a === "auto" ? "Auto" : a}</option>
			{/each}
		</select>
	</label>

	{#if slot.mode === "fill" && hasImage}
		<span class="text-xs">Drag image to reframe</span>
	{/if}

	<Button size="sm" variant="outline" class={dim} disabled={!hasImage} onclick={() => zine.clearImage(index, s)}>
		Remove
	</Button>
</div>
