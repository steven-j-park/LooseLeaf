export type PageMode = "fit" | "fill";

export const ASPECTS = {
	auto: null,
	"1:1": 1,
	"4:3": 4 / 3,
	"3:4": 3 / 4,
	"3:2": 3 / 2,
	"2:3": 2 / 3,
	"16:9": 16 / 9,
} as const;
export type Aspect = keyof typeof ASPECTS;

/** One image position on a page, with its own framing. */
export interface Slot {
	imageUrl: string | null;
	mode: PageMode;
	/** object-position percentages (0-100) used when mode is "fill" */
	offsetX: number;
	offsetY: number;
	/** "auto" fills the slot's share of the page; otherwise a fixed frame ratio (w:h) */
	aspect: Aspect;
}

export interface Page {
	id: number;
	label: string;
	/** Up to two images, stacked top to bottom. Kept compact: slot 1 is only used if slot 0 is. */
	images: [Slot, Slot];
	caption: string;
}

const PAGE_COUNT = 8;

function labelFor(index: number): string {
	if (index === 0) return "Front cover";
	if (index === PAGE_COUNT - 1) return "Back cover";
	return `Page ${index + 1}`;
}

function blankSlot(): Slot {
	return { imageUrl: null, mode: "fit", offsetX: 50, offsetY: 50, aspect: "auto" };
}

function blankPage(index: number): Page {
	return {
		id: index,
		label: labelFor(index),
		images: [blankSlot(), blankSlot()],
		caption: "",
	};
}

/** Page indices grouped as they are shown: cover, spreads (2-3, 4-5, 6-7), back cover. */
export const views: number[][] = [[0], [1, 2], [3, 4], [5, 6], [7]];

export function viewIndexOf(pageIndex: number): number {
	return views.findIndex((v) => v.includes(pageIndex));
}

/** Covers hold a single image; interior pages hold up to two. */
export function maxImages(pageIndex: number): number {
	return pageIndex === 0 || pageIndex === PAGE_COUNT - 1 ? 1 : 2;
}

export function imageCount(page: Page): number {
	return page.images.filter((s) => s.imageUrl).length;
}

class Zine {
	pages = $state<Page[]>(Array.from({ length: PAGE_COUNT }, (_, i) => blankPage(i)));
	selectedIndex = $state(0);
	navExpanded = $state(true);

	selected = $derived(this.pages[this.selectedIndex]);
	viewIndex = $derived(viewIndexOf(this.selectedIndex));

	updatePage(index: number, patch: Partial<Pick<Page, "caption">>) {
		Object.assign(this.pages[index], patch);
	}

	updateSlot(index: number, slot: number, patch: Partial<Slot>) {
		Object.assign(this.pages[index].images[slot], patch);
	}

	setImage(index: number, slot: number, file: File) {
		if (!file.type.startsWith("image/") || slot >= maxImages(index)) return;
		this.releaseImage(index, slot);
		this.updateSlot(index, slot, {
			imageUrl: URL.createObjectURL(file),
			mode: "fit",
			offsetX: 50,
			offsetY: 50,
		});
		this.compact(index);
	}

	/** Fill the first free slot; if the page is full, replace `fallbackSlot`. */
	addImage(index: number, file: File, fallbackSlot = 0) {
		const images = this.pages[index].images;
		const max = maxImages(index);
		const slot = !images[0].imageUrl ? 0 : max > 1 && !images[1].imageUrl ? 1 : Math.min(fallbackSlot, max - 1);
		this.setImage(index, slot, file);
	}

	clearImage(index: number, slot: number) {
		this.releaseImage(index, slot);
		this.pages[index].images[slot] = blankSlot();
		this.compact(index);
	}

	swapImages(index: number) {
		const images = this.pages[index].images;
		if (!images[0].imageUrl || !images[1].imageUrl) return;
		this.pages[index].images = [images[1], images[0]];
	}

	reset() {
		this.pages.forEach((p, i) => p.images.forEach((_, s) => this.releaseImage(i, s)));
		this.pages = Array.from({ length: PAGE_COUNT }, (_, i) => blankPage(i));
		this.selectedIndex = 0;
	}

	private compact(index: number) {
		const images = this.pages[index].images;
		if (!images[0].imageUrl && images[1].imageUrl) {
			this.pages[index].images = [images[1], images[0]];
		}
	}

	private releaseImage(index: number, slot: number) {
		const url = this.pages[index].images[slot].imageUrl;
		if (url) URL.revokeObjectURL(url);
	}
}

export const zine = new Zine();
