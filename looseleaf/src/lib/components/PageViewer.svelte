<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { cn } from "$lib/utils";
	import { ASPECTS, imageCount, isCover, views, zine, type CaptionSize } from "$lib/zine.svelte";

	const view = $derived(views[zine.viewIndex]);
	const spread = $derived(view.length > 1);

	let fileInput = $state<HTMLInputElement | null>(null);
	let target = { page: 0, slot: 0 };
	let dragOver = $state<number | null>(null);
	let drag: { x: number; y: number; ox: number; oy: number } | null = null;

	export function chooseFile(page = zine.selectedIndex, slot = 0) {
		target = { page, slot };
		fileInput?.click();
	}

	function onDrop(e: DragEvent, page: number, slot: number) {
		e.preventDefault();
		dragOver = null;
		zine.selectedIndex = page;
		const file = e.dataTransfer?.files?.[0];
		if (file) zine.addImage(page, file, slot);
	}

	function frameStyle(aspect: keyof typeof ASPECTS): string {
		const r = ASPECTS[aspect];
		if (!r) return "width: 100%; height: 100%";
		return `aspect-ratio: ${r}; width: min(100cqw, ${r * 100}cqh)`;
	}

	function onPointerDown(e: PointerEvent, page: number, slot: number) {
		const s = zine.pages[page].images[slot];
		if (s.mode !== "fill") return;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		drag = { x: e.clientX, y: e.clientY, ox: s.offsetX, oy: s.offsetY };
	}

	function onPointerMove(e: PointerEvent, page: number, slot: number) {
		const img = e.currentTarget as HTMLImageElement;
		const frame = img.parentElement;
		if (!drag || !frame || !img.naturalWidth) return;
		// Size of the image once scaled to cover the frame; overflow is what can be panned.
		const fw = frame.clientWidth;
		const fh = frame.clientHeight;
		const scale = Math.max(fw / img.naturalWidth, fh / img.naturalHeight);
		const overflowX = img.naturalWidth * scale - fw;
		const overflowY = img.naturalHeight * scale - fh;
		const clamp = (v: number) => Math.min(100, Math.max(0, v));
		// object-position p% shifts the image by -p% of overflow, so dragging right lowers p.
		zine.updateSlot(page, slot, {
			offsetX: overflowX > 0 ? clamp(drag.ox - ((e.clientX - drag.x) / overflowX) * 100) : 50,
			offsetY: overflowY > 0 ? clamp(drag.oy - ((e.clientY - drag.y) / overflowY) * 100) : 50,
		});
	}

	function onPointerUp() {
		drag = null;
	}

	const JUSTIFY = { left: "flex-start", center: "center", right: "flex-end" } as const;
	const ALIGN = { top: "flex-start", middle: "center", bottom: "flex-end" } as const;
	const CAPTION_FONT: Record<CaptionSize, string> = { sm: "6cqw", md: "9cqw", lg: "13cqw" };

	let capDrag: { x: number; y: number; ox: number; oy: number } | null = null;

	function onCaptionDown(e: PointerEvent, page: number) {
		const cs = zine.pages[page].captionStyle;
		if (!cs.custom) return;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		capDrag = { x: e.clientX, y: e.clientY, ox: cs.x, oy: cs.y };
	}

	function onCaptionMove(e: PointerEvent, page: number) {
		const area = (e.currentTarget as HTMLElement).parentElement;
		if (!capDrag || !area) return;
		const rect = area.getBoundingClientRect();
		const clamp = (v: number) => Math.min(100, Math.max(0, v));
		zine.updateCaptionStyle(page, {
			x: clamp(capDrag.ox + ((e.clientX - capDrag.x) / rect.width) * 100),
			y: clamp(capDrag.oy + ((e.clientY - capDrag.y) / rect.height) * 100),
		});
	}

	function onCaptionUp() {
		capDrag = null;
	}
</script>

<main class="flex min-h-0 items-center justify-center bg-muted p-6">
	<input
		bind:this={fileInput}
		type="file"
		accept="image/*"
		class="hidden"
		onchange={(e) => {
			const file = e.currentTarget.files?.[0];
			if (file) {
				zine.selectedIndex = target.page;
				zine.setImage(target.page, target.slot, file);
			}
			e.currentTarget.value = "";
		}}
	/>

	<div class="flex h-full w-full min-w-0 flex-col items-center gap-2">
		<div class="min-h-0 w-full flex-1" style="container-type: size">
			<div
				class={cn(
					"mx-auto flex overflow-hidden border-2 border-border bg-background shadow-xl",
					spread ? "aspect-[3/2]" : "aspect-[3/4]"
				)}
				style="width: min(100cqw, {spread ? 150 : 75}cqh)"
			>
				{#each view as index, n (index)}
					{@const page = zine.pages[index]}
					{@const count = imageCount(page)}
					<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
					<div
						class={cn(
							"relative flex min-w-0 flex-1 flex-col",
							isCover(index) && "[container-type:inline-size]",
							count > 1 && "gap-2 p-2",
							n > 0 && "border-l-2 border-dashed border-border",
							spread && index === zine.selectedIndex && "outline-4 -outline-offset-4 outline-primary",
							dragOver === index && "bg-primary/30"
						)}
						aria-label={page.label}
						onclick={() => (zine.selectedIndex = index)}
						ondragover={(e) => {
							e.preventDefault();
							dragOver = index;
						}}
						ondragleave={() => (dragOver = null)}
						ondrop={(e) => onDrop(e, index, 0)}
					>
						{#if count === 0}
							<div class="flex size-full flex-col items-center justify-center gap-3 p-4 text-center">
								<p class="font-head">Drop an image here</p>
								<Button
									size="sm"
									onclick={(e) => {
										e.stopPropagation();
										chooseFile(index, 0);
									}}>Choose file</Button
								>
							</div>
						{:else}
							{#each page.images as slot, s (s)}
								{#if slot.imageUrl}
									<div
										class="flex min-h-0 flex-1 items-center justify-center"
										style="container-type: size"
										ondragover={(e) => e.stopPropagation()}
										ondrop={(e) => {
											e.stopPropagation();
											onDrop(e, index, s);
										}}
									>
										<div
											class={cn("relative overflow-hidden", slot.aspect !== "auto" && "outline outline-1 outline-border/40")}
											style={frameStyle(slot.aspect)}
										>
											<img
												src={slot.imageUrl}
												alt={page.caption || `${page.label} image ${s + 1}`}
												draggable="false"
												class={cn(
													"size-full select-none",
													slot.mode === "fill"
														? "cursor-grab touch-none object-cover active:cursor-grabbing"
														: "object-contain"
												)}
												style="object-position: {slot.offsetX}% {slot.offsetY}%"
												onpointerdown={(e) => onPointerDown(e, index, s)}
												onpointermove={(e) => onPointerMove(e, index, s)}
												onpointerup={onPointerUp}
												onpointercancel={onPointerUp}
											/>
										</div>
									</div>
								{/if}
							{/each}
						{/if}
						{#if isCover(index) && (page.caption || page.subCaption || page.author)}
							{@const cs = page.captionStyle}
							<div
								class="pointer-events-none absolute inset-0 flex p-[5%]"
								style={cs.custom ? "" : `justify-content: ${JUSTIFY[cs.h]}; align-items: ${ALIGN[cs.v]}`}
							>
								<div
									class={cn(
										"max-w-[90%] font-head leading-none break-words whitespace-pre-wrap",
										cs.custom && "pointer-events-auto absolute cursor-move touch-none select-none"
									)}
									style="color: {cs.color}; font-size: {CAPTION_FONT[cs.size]}; text-align: {cs.h}; {cs.custom
										? `left: ${cs.x}%; top: ${cs.y}%; transform: translate(-50%, -50%)`
										: ''}"
									onpointerdown={(e) => onCaptionDown(e, index)}
									onpointermove={(e) => onCaptionMove(e, index)}
									onpointerup={onCaptionUp}
									onpointercancel={onCaptionUp}
								>
									{#if page.caption}<p>{page.caption}</p>{/if}
									{#if page.subCaption}<p style="font-size: 0.7em">{page.subCaption}</p>{/if}
									{#if page.author}<p style="font-size: 0.5em">{page.author}</p>{/if}
								</div>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
		<p class="h-6 max-w-full truncate text-sm">
			{view
				.filter((i) => !isCover(i))
				.map((i) => zine.pages[i].caption)
				.filter(Boolean)
				.join(" · ")}
		</p>
	</div>
</main>
