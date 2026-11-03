<script lang="ts">
	import { convertFileSrc, invoke } from "@tauri-apps/api/core";
	import { open } from "@tauri-apps/plugin-dialog";
	import { FolderOpen, RefreshCw, X } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { dropTargetAt, photoDrag } from "$lib/photoDrag.svelte";
	import { cn } from "$lib/utils";
	import { zine } from "$lib/zine.svelte";

	interface Photo {
		path: string;
		name: string;
		thumbUrl: string | null;
		failed: boolean;
	}

	/** Thumbnails generated at once; each runs on a backend blocking thread. */
	const CONCURRENCY = 4;
	/** Pointer travel (px) before a press on a thumbnail becomes a drag. */
	const DRAG_THRESHOLD = 5;

	const dir = $derived(zine.photoDir);
	/** Folder the current `photos` came from; scanning waits until the panel is opened. */
	let scannedDir: string | null = null;
	let photos = $state<Photo[]>([]);
	let scanning = $state(false);
	let error = $state("");
	/** Bumped on every scan so stale thumbnail work from a previous folder is dropped. */
	let generation = 0;

	const dirName = $derived(dir?.split(/[\\/]/).filter(Boolean).pop() ?? "");

	async function chooseDir() {
		const picked = await open({ directory: true, multiple: false, defaultPath: dir ?? undefined });
		if (typeof picked !== "string") return;
		zine.photoDir = picked;
	}

	// Rescan when the folder changes (picked here, or from an opened/new project) and the panel is open.
	$effect(() => {
		if (zine.photoTableOpen && dir !== scannedDir) scan();
	});

	async function scan() {
		scannedDir = dir;
		const gen = ++generation;
		error = "";
		if (!dir) {
			photos = [];
			return;
		}
		scanning = true;
		try {
			const paths = await invoke<string[]>("list_images", { dir });
			if (gen !== generation) return;
			photos = paths.map((path) => ({
				path,
				name: path.split(/[\\/]/).pop() ?? path,
				thumbUrl: null,
				failed: false,
			}));
		} catch (e) {
			if (gen !== generation) return;
			error = String(e);
			photos = [];
		} finally {
			if (gen === generation) scanning = false;
		}
		await loadThumbs(gen);
	}

	/** Build thumbnails via the same cached `prepare_image` used on import, so drops are instant. */
	async function loadThumbs(gen: number) {
		const list = photos;
		let next = 0;
		const worker = async () => {
			while (gen === generation && next < list.length) {
				const photo = list[next++];
				try {
					const info = await invoke<{ thumbPath: string }>("prepare_image", { path: photo.path });
					photo.thumbUrl = convertFileSrc(info.thumbPath);
				} catch {
					photo.failed = true;
				}
			}
		};
		await Promise.all(Array.from({ length: CONCURRENCY }, worker));
	}

	// ---- Dragging a thumbnail onto the zine ----

	let press: { photo: Photo; x: number; y: number } | null = null;
	/** Set when a press turned into a drag, so the click that follows is ignored. */
	let dragged = false;

	function onPointerDown(e: PointerEvent, photo: Photo) {
		if (e.button !== 0 || photo.failed) return;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
		press = { photo, x: e.clientX, y: e.clientY };
		dragged = false;
	}

	function onPointerMove(e: PointerEvent) {
		if (!press) return;
		if (!photoDrag.path) {
			if (Math.hypot(e.clientX - press.x, e.clientY - press.y) < DRAG_THRESHOLD) return;
			photoDrag.path = press.photo.path;
			photoDrag.thumbUrl = press.photo.thumbUrl;
			dragged = true;
		}
		photoDrag.x = e.clientX;
		photoDrag.y = e.clientY;
		photoDrag.target = dropTargetAt(e.clientX, e.clientY);
	}

	function endDrag() {
		press = null;
		photoDrag.path = null;
		photoDrag.thumbUrl = null;
		photoDrag.target = null;
	}

	async function onPointerUp() {
		const { path, target } = photoDrag;
		endDrag();
		if (path && target) {
			zine.selectedIndex = target.page;
			await zine.addImage(target.page, path, target.slot);
		}
	}

	/** Clicking (or Enter/Space) adds the photo to the selected page. */
	async function onClick(photo: Photo) {
		if (dragged) {
			dragged = false;
			return;
		}
		if (!photo.failed) await zine.addImage(zine.selectedIndex, photo.path);
	}
</script>

<aside
	class={cn(
		"absolute inset-y-0 right-0 z-20 flex w-72 flex-col border-l-2 border-border bg-card shadow-xl transition-transform duration-200",
		zine.photoTableOpen ? "translate-x-0" : "translate-x-full"
	)}
	aria-label="Photo table"
	inert={!zine.photoTableOpen}
>
	<div class="flex items-center justify-between gap-2 border-b-2 border-border p-2 pl-3">
		<h2 class="font-head">Photo table</h2>
		<Button variant="ghost" size="icon" aria-label="Close photo table" onclick={() => (zine.photoTableOpen = false)}>
			<X />
		</Button>
	</div>

	<div class="flex items-center gap-2 border-b-2 border-border p-2">
		<Button size="sm" class="min-w-0 flex-1" title={dir ?? "Choose a folder of photos"} onclick={chooseDir}>
			<FolderOpen />
			<span class="truncate">{dir ? dirName : "Choose folder"}</span>
		</Button>
		{#if dir}
			<Button
				variant="outline"
				size="icon"
				class="bg-background"
				aria-label="Rescan folder"
				title="Rescan folder"
				disabled={scanning}
				onclick={scan}><RefreshCw /></Button
			>
		{/if}
	</div>

	<div class="min-h-0 flex-1 overflow-y-auto p-3">
		{#if !dir}
			<p class="mt-8 text-center text-sm text-muted-foreground">
				Open a folder to lay out its JPEGs here, then drag them onto your zine.
			</p>
		{:else if error}
			<p class="text-sm text-destructive" role="alert">{error}</p>
		{:else if scanning}
			<p class="text-center text-sm text-muted-foreground">Scanning…</p>
		{:else if photos.length === 0}
			<p class="text-center text-sm text-muted-foreground">No .jpg or .jpeg files in this folder.</p>
		{:else}
			<p class="mb-2 text-xs text-muted-foreground">
				{photos.length} photo{photos.length === 1 ? "" : "s"} · drag onto a page, or click to add to the selected page
			</p>
			<ul class="grid grid-cols-2 gap-2">
				{#each photos as photo (photo.path)}
					<li>
						<button
							type="button"
							title={photo.failed ? `${photo.name} (could not be read)` : photo.name}
							aria-label={photo.name}
							disabled={photo.failed}
							class={cn(
								"flex aspect-square w-full touch-none items-center justify-center overflow-hidden border-2 border-border bg-muted transition select-none",
								photo.failed ? "cursor-not-allowed opacity-50" : "cursor-grab hover:shadow-md active:cursor-grabbing",
								photoDrag.path === photo.path && "opacity-40"
							)}
							onpointerdown={(e) => onPointerDown(e, photo)}
							onpointermove={onPointerMove}
							onpointerup={onPointerUp}
							onpointercancel={endDrag}
							onclick={() => onClick(photo)}
						>
							{#if photo.thumbUrl}
								<img src={photo.thumbUrl} alt="" draggable="false" class="size-full object-cover" />
							{:else}
								<span class="truncate px-1 text-xs text-muted-foreground">
									{photo.failed ? "Unreadable" : photo.name}
								</span>
							{/if}
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</div>
</aside>

<!-- Outside the transformed panel so `fixed` is relative to the viewport. -->
{#if photoDrag.path}
	<div
		class="pointer-events-none fixed z-50 w-24 -translate-x-1/2 -translate-y-1/2 border-2 border-border bg-background opacity-80 shadow-xl"
		style="left: {photoDrag.x}px; top: {photoDrag.y}px"
	>
		{#if photoDrag.thumbUrl}
			<img src={photoDrag.thumbUrl} alt="" class="block w-full" />
		{:else}
			<div class="aspect-square bg-muted"></div>
		{/if}
	</div>
{/if}
