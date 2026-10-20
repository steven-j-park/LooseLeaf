<script lang="ts">
	import { ArrowUpDown } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { imageCount, zine } from "$lib/zine.svelte";
	import CaptionInput from "./CaptionInput.svelte";
	import SlotOptions from "./SlotOptions.svelte";

	/** Settings for an inner page: up to two stacked images and a plain caption. */
	let { index, onChooseFile }: { index: number; onChooseFile: (page: number, slot: number) => void } = $props();

	const page = $derived(zine.pages[index]);
	const count = $derived(imageCount(page));
</script>

<div class="flex items-center gap-3">
	<span class="font-head">{page.label}</span>
	<CaptionInput
		value={page.caption}
		placeholder="Caption"
		label="{page.label} caption"
		oninput={(caption) => zine.updatePage(index, { caption })}
	/>
	<Button
		size="sm"
		variant="outline"
		class="disabled:pointer-events-none disabled:opacity-50"
		disabled={count < 2}
		title="Swap image positions"
		aria-label="Swap {page.label} image positions"
		onclick={() => zine.swapImages(index)}
	>
		<ArrowUpDown /> Swap
	</Button>
</div>

<SlotOptions {index} slot={0} {onChooseFile} />
<!-- The second slot appears once the first has an image (slots are kept compact). -->
{#if count > 0}
	<SlotOptions {index} slot={1} {onChooseFile} />
{/if}
