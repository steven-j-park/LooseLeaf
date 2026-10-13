<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { cn } from "$lib/utils";
	import { views, zine } from "$lib/zine.svelte";

	const view = $derived(views[zine.viewIndex]);
	const spread = $derived(view.length > 1);

	let fileInput = $state<HTMLInputElement | null>(null);
	let dragOver = $state<number | null>(null);
	let drag: { x: number; y: number; ox: number; oy: number } | null = null;

	export function chooseFile() {
		fileInput?.click();
	}

	function onFiles(files: FileList | null | undefined, index = zine.selectedIndex) {
		const file = files?.[0];
		if (file) zine.setImage(index, file);
	}

	function onDrop(e: DragEvent, index: number) {
		e.preventDefault();
		dragOver = null;
		zine.selectedIndex = index;
		onFiles(e.dataTransfer?.files, index);
	}

	function onPointerDown(e: PointerEvent, index: number) {
		const page = zine.pages[index];
		if (page.mode !== "fill") return;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		drag = { x: e.clientX, y: e.clientY, ox: page.offsetX, oy: page.offsetY };
	}

	function onPointerMove(e: PointerEvent, index: number) {
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
		zine.update(index, {
			offsetX: overflowX > 0 ? clamp(drag.ox - ((e.clientX - drag.x) / overflowX) * 100) : 50,
			offsetY: overflowY > 0 ? clamp(drag.oy - ((e.clientY - drag.y) / overflowY) * 100) : 50,
		});
	}

	function onPointerUp() {
		drag = null;
	}
</script>

<main class="flex min-h-0 items-center justify-center bg-muted p-6">
	<input
		bind:this={fileInput}
		type="file"
		accept="image/*"
		class="hidden"
		onchange={(e) => {
			onFiles(e.currentTarget.files);
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
					<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
					<div
						class={cn(
							"relative min-w-0 flex-1",
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
						ondrop={(e) => onDrop(e, index)}
					>
						{#if page.imageUrl}
							<img
								src={page.imageUrl}
								alt={page.caption || page.label}
								draggable="false"
								class={cn(
									"size-full select-none",
									page.mode === "fill" ? "cursor-grab touch-none object-cover active:cursor-grabbing" : "object-contain"
								)}
								style="object-position: {page.offsetX}% {page.offsetY}%"
								onpointerdown={(e) => onPointerDown(e, index)}
								onpointermove={(e) => onPointerMove(e, index)}
								onpointerup={onPointerUp}
								onpointercancel={onPointerUp}
							/>
						{:else}
							<div class="flex size-full flex-col items-center justify-center gap-3 p-4 text-center">
								<p class="font-head">Drop an image here</p>
								<Button
									size="sm"
									onclick={(e) => {
										e.stopPropagation();
										zine.selectedIndex = index;
										chooseFile();
									}}>Choose file</Button
								>
							</div>
						{/if}
					</div>
				{/each}
			</div>
		</div>
		<p class="h-6 max-w-full truncate text-sm">
			{view.map((i) => zine.pages[i].caption).filter(Boolean).join(" · ")}
		</p>
	</div>
</main>
