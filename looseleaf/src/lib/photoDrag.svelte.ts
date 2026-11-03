/** A page / image slot that something can be dropped onto. */
export interface DropTarget {
	page: number;
	slot: number;
}

/** Page / image slot under a viewport position (CSS px), found via the viewer's data attributes. */
export function dropTargetAt(x: number, y: number): DropTarget | null {
	const el = document.elementFromPoint(x, y);
	const pageEl = el?.closest<HTMLElement>("[data-page]");
	if (!pageEl) return null;
	const slotEl = el?.closest<HTMLElement>("[data-slot]");
	return { page: Number(pageEl.dataset.page), slot: Number(slotEl?.dataset.slot ?? 0) };
}

/**
 * An in-app drag from the photo table to the zine. Uses pointer events rather than HTML5
 * drag-and-drop, which conflicts with Tauri's native file-drop handling on some platforms.
 */
export const photoDrag = $state<{
	path: string | null;
	thumbUrl: string | null;
	x: number;
	y: number;
	target: DropTarget | null;
}>({ path: null, thumbUrl: null, x: 0, y: 0, target: null });
