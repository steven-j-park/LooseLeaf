<script lang="ts">
	import {
		AlignCenter,
		AlignLeft,
		AlignRight,
		AlignVerticalJustifyCenter,
		AlignVerticalJustifyEnd,
		AlignVerticalJustifyStart,
		ArrowUpDown,
		Move,
		ImagePlus,
	} from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { cn } from "$lib/utils";
	import { ASPECTS, H_POS, V_POS, imageCount, isCover, maxImages, views, zine, type Aspect, type HAlign, type VAlign, type CaptionSize } from "$lib/zine.svelte";

	let { onChooseFile }: { onChooseFile: (page: number, slot: number) => void } = $props();

	const view = $derived(views[zine.viewIndex]);
	const dim = "disabled:pointer-events-none disabled:opacity-50";
	const aspects = Object.keys(ASPECTS) as Aspect[];

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
	function setH(index: number, h: HAlign) {
		zine.updateCaptionStyle(index, { h, custom: false });
	}
	function setV(index: number, v: VAlign) {
		zine.updateCaptionStyle(index, { v, custom: false });
	}
	function toggleCustom(index: number) {
		const cs = zine.pages[index].captionStyle;
		if (cs.custom) zine.updateCaptionStyle(index, { custom: false });
		else zine.updateCaptionStyle(index, { custom: true, x: H_POS[cs.h], y: V_POS[cs.v] });
	}

	function pickImage(page: number, slot: number) {
		zine.selectedIndex = page;
		onChooseFile(page, slot);
	}
</script>

<section class="flex divide-x-2 divide-border border-t-2 border-border bg-card">
	{#each view as index (index)}
		{@const page = zine.pages[index]}
		<div
			class={cn(
				"flex min-w-0 flex-1 flex-col gap-2 px-4 py-3",
				view.length > 1 && index === zine.selectedIndex && "bg-primary/20"
			)}
		>
			<div class="flex items-center gap-3">
				<span class="font-head">{page.label}</span>
				<Input
					class="min-w-40 flex-1"
					autocapitalize="off"
					autocorrect="off"
					spellcheck={false}
					placeholder="Caption"
					aria-label="{page.label} caption"
					value={page.caption}
					oninput={(e) => zine.updatePage(index, { caption: e.currentTarget.value })}
				/>
				{#if index === 0}
				<Input
					class="min-w-40 flex-1"
					autocapitalize="off"
					autocorrect="off"
					spellcheck={false}
					placeholder="Sub-caption"
					aria-label="Front cover sub-caption"
					value={page.subCaption}
					oninput={(e) => zine.updatePage(index, { subCaption: e.currentTarget.value })}
				/>
				<Input
					class="min-w-40 flex-1"
					autocapitalize="off"
					autocorrect="off"
					spellcheck={false}
					placeholder="Author"
					aria-label="Front cover author"
					value={page.author}
					oninput={(e) => zine.updatePage(index, { author: e.currentTarget.value })}
				/>
				{/if}
				{#if maxImages(index) > 1}
				<Button
					size="sm"
					variant="outline"
					class={dim}
					disabled={imageCount(page) < 2}
					title="Swap image positions"
					aria-label="Swap {page.label} image positions"
					onclick={() => zine.swapImages(index)}
				>
					<ArrowUpDown /> Swap
				</Button>
				{/if}
			</div>

			{#if isCover(index)}
				{@const cs = page.captionStyle}
				<div class="flex flex-wrap items-center gap-3" role="group" aria-label="{page.label} caption style">
					<div class="flex gap-1">
						{#each hOptions as o (o.value)}
							<Button
								size="icon"
								variant={!cs.custom && cs.h === o.value ? "default" : "outline"}
								title={o.label}
								aria-label={o.label}
								onclick={() => setH(index, o.value)}><o.icon /></Button
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
								onclick={() => setV(index, o.value)}><o.icon /></Button
							>
						{/each}
					</div>
					<Button
						size="sm"
						variant={cs.custom ? "default" : "outline"}
						title="Drag the caption into position on the cover"
						onclick={() => toggleCustom(index)}><Move /> Custom</Button
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
			{/if}

			{#each page.images.slice(0, maxImages(index)) as slot, s (s)}
				{@const hasImage = slot.path !== null}
				<div class={cn("flex flex-wrap items-center gap-3", !hasImage && s === 1 && imageCount(page) === 0 && "hidden")}>
					<button
						type="button"
						class="flex aspect-[3/4] h-12 shrink-0 cursor-pointer items-center justify-center overflow-hidden border-2 border-border bg-muted transition hover:bg-primary/30"
						aria-label={hasImage
							? `Replace image ${s + 1} for ${page.label}`
							: `Add image ${s + 1} for ${page.label}`}
						title={hasImage ? "Click to replace image" : "Click to add image"}
						onclick={() => pickImage(index, s)}
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
			{/each}
		</div>
	{/each}
</section>
