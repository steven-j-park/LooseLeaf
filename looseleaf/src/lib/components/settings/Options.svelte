<script lang="ts">
	import {
		AlignCenter,
		AlignLeft,
		AlignRight,
		AlignVerticalJustifyCenter,
		AlignVerticalJustifyEnd,
		AlignVerticalJustifyStart,
		Move,
	} from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { H_POS, V_POS, zine, type CaptionSize, type HAlign, type VAlign } from "$lib/zine.svelte";
	import CaptionInput from "./CaptionInput.svelte";
	import SlotOptions from "./SlotOptions.svelte";

	/** Settings for a cover: one image plus an overlaid, styleable caption. */
	let { index, onChooseFile }: { index: number; onChooseFile: (page: number, slot: number) => void } = $props();

	const page = $derived(zine.pages[index]);
	const cs = $derived(page.captionStyle);

	const hOptions = [
		{ value: "left", label: "Align left", icon: AlignLeft },
		{ value: "center", label: "Align center", icon: AlignCenter },
		{ value: "right", label: "Align right", icon: AlignRight },
	] as const;
	const vOptions = [
		{ value: "top", label: "Align top", icon: AlignVerticalJustifyStart },
		{ value: "middle", label: "Align middle", icon: AlignVerticalJustifyCenter },
		{ value: "bottom", label: "Align bottom", icon: AlignVerticalJustifyEnd },
	] as const;
	const sizes: { value: CaptionSize; label: string }[] = [
		{ value: "sm", label: "S" },
		{ value: "md", label: "M" },
		{ value: "lg", label: "L" },
	];

	// Picking an alignment leaves custom placement; Custom starts from the current alignment.
	function setH(h: HAlign) {
		zine.updateCaptionStyle(index, { h, custom: false });
	}
	function setV(v: VAlign) {
		zine.updateCaptionStyle(index, { v, custom: false });
	}
	function toggleCustom() {
		if (cs.custom) zine.updateCaptionStyle(index, { custom: false });
		else zine.updateCaptionStyle(index, { custom: true, x: H_POS[cs.h], y: V_POS[cs.v] });
	}
</script>

<div class="flex items-center gap-3">
	<span class="font-head">{page.label}</span>
	<CaptionInput
		value={page.caption}
		placeholder="Caption"
		label="{page.label} caption"
		oninput={(caption) => zine.updatePage(index, { caption })}
	/>
	{#if index === 0}
		<CaptionInput
			value={page.subCaption}
			placeholder="Sub-caption"
			label="Front cover sub-caption"
			oninput={(subCaption) => zine.updatePage(index, { subCaption })}
		/>
		<CaptionInput
			value={page.author}
			placeholder="Author"
			label="Front cover author"
			oninput={(author) => zine.updatePage(index, { author })}
		/>
	{/if}
</div>

<div class="flex flex-wrap items-center gap-3" role="group" aria-label="{page.label} caption style">
	<div class="flex gap-1">
		{#each hOptions as o (o.value)}
			<Button
				size="icon"
				variant={!cs.custom && cs.h === o.value ? "default" : "outline"}
				title={o.label}
				aria-label={o.label}
				onclick={() => setH(o.value)}><o.icon /></Button
			>
		{/each}
	</div>
	<div class="flex gap-1">
		{#each vOptions as o (o.value)}
			<Button
				size="icon"
				variant={!cs.custom && cs.v === o.value ? "default" : "outline"}
				title={o.label}
				aria-label={o.label}
				onclick={() => setV(o.value)}><o.icon /></Button
			>
		{/each}
	</div>
	<Button
		size="sm"
		variant={cs.custom ? "default" : "outline"}
		title="Drag the caption into position on the cover"
		onclick={toggleCustom}><Move /> Custom</Button
	>
	<div class="flex gap-1" role="group" aria-label="Caption size">
		{#each sizes as z (z.value)}
			<Button
				size="icon"
				variant={cs.size === z.value ? "default" : "outline"}
				aria-label="Caption size {z.label}"
				onclick={() => zine.updateCaptionStyle(index, { size: z.value })}>{z.label}</Button
			>
		{/each}
	</div>
	<div class="flex gap-1" role="group" aria-label="Caption color">
		{#each ["white", "black"] as const as c (c)}
			<Button
				size="icon"
				variant={cs.color === c ? "default" : "outline"}
				title="{c === 'white' ? 'White' : 'Black'} caption"
				aria-label="{c} caption"
				onclick={() => zine.updateCaptionStyle(index, { color: c })}
			>
				<span class="size-4 rounded-full border-2 border-border" style="background: {c}"></span>
			</Button>
		{/each}
	</div>
	{#if cs.custom}
		<span class="text-xs">Drag the caption on the cover</span>
	{/if}
</div>

<SlotOptions {index} slot={0} {onChooseFile} />
