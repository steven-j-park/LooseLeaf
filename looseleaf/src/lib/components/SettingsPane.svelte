<script lang="ts">
	import { ArrowUpDown, ImagePlus } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { Input } from "$lib/components/ui/input";
	import { cn } from "$lib/utils";
	import { ASPECTS, imageCount, maxImages, views, zine, type Aspect } from "$lib/zine.svelte";

	let { onChooseFile }: { onChooseFile: (page: number, slot: number) => void } = $props();

	const view = $derived(views[zine.viewIndex]);
	const dim = "disabled:pointer-events-none disabled:opacity-50";
	const aspects = Object.keys(ASPECTS) as Aspect[];

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
					placeholder="Caption"
					aria-label="{page.label} caption"
					value={page.caption}
					oninput={(e) => zine.updatePage(index, { caption: e.currentTarget.value })}
				/>
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

			{#each page.images.slice(0, maxImages(index)) as slot, s (s)}
				{@const hasImage = slot.imageUrl !== null}
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
						{#if slot.imageUrl}
							<img src={slot.imageUrl} alt="" class="size-full object-cover" />
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
