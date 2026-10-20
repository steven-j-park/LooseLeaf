<script lang="ts">
	import { getCurrentWebview } from "@tauri-apps/api/webview";
	import { isTauri } from "@tauri-apps/api/core";
	import { onMount } from "svelte";
	import { chooseImage, isImagePath } from "$lib/actions";
	import { Button } from "$lib/components/ui/button";
	import { cn } from "$lib/utils";
	import { ASPECTS, imageCount, isCover, views, zine, type CaptionSize } from "$lib/zine.svelte";

	const view = $derived(views[zine.viewIndex]);
	const spread = $derived(view.length > 1);
	const ratio = $derived(zine.pageRatio * view.length);

	let dragOver = $state<number | null>(null);
	let drag: { x: number; y: number; ox: number; oy: number } | null = null;

	export function chooseFile(page = zine.selectedIndex, slot = 0) {
		return chooseImage(page, slot);
	}

	/** Page / image slot under a window position from Tauri's drag-drop event (physical px). */
	function targetAt(pos: { x: number; y: number }): { page: number; slot: number } | null {
		const el = document.elementFromPoint(pos.x / devicePixelRatio, pos.y / devicePixelRatio);
		const pageEl = el?.closest<HTMLElement>("[data-page]");
		if (!pageEl) return null;
		const slotEl = el?.closest<HTMLElement>("[data-slot]");
		return { page: Number(pageEl.dataset.page), slot: Number(slotEl?.dataset.slot ?? 0) };
	}

	// Native file drops arrive as paths via Tauri (the webview doesn't deliver HTML5 file drops).
	onMount(() => {
		if (!isTauri()) return;
		let unlisten: (() => void) | undefined;
		getCurrentWebview()
			.onDragDropEvent(async ({ payload }) => {
				if (payload.type === "leave") {
					dragOver = null;
				} else if (payload.type === "enter" || payload.type === "over") {
					dragOver = targetAt(payload.position)?.page ?? null;
				} else if (payload.type === "drop") {
					dragOver = null;
					const target = targetAt(payload.position);
					const path = payload.paths.find(isImagePath);
					if (target && path) {
						zine.selectedIndex = target.page;
						await zine.addImage(target.page, path, target.slot);
					}
				}
			})
			.then((fn) => (unlisten = fn));
		return () => unlisten?.();
	});

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
	<div class="flex h-full w-full min-w-0 flex-col items-center gap-2">
		<div class="min-h-0 w-full flex-1" style="container-type: size">
			<div
				class="mx-auto flex overflow-hidden border-2 border-border bg-background shadow-xl"
				style="aspect-ratio: {ratio}; width: min(100cqw, {ratio * 100}cqh)"
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
						data-page={index}
						onclick={() => (zine.selectedIndex = index)}
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
								{#if slot.path}
									<div
										class="flex min-h-0 flex-1 items-center justify-center"
										style="container-type: size"
										data-slot={s}
									>
										<div
											class={cn("relative overflow-hidden", slot.aspect !== "auto" && "outline outline-1 outline-border/40")}
											style={frameStyle(slot.aspect)}
										>
											<img
												src={slot.previewUrl ?? ""}
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
										"max-w-[90%] font-head leading-none break-words whitespace-pre-wrap select-none",
										cs.custom && "pointer-events-auto absolute cursor-move touch-none"
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
	</div>
</main>
