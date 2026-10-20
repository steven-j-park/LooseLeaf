import { convertFileSrc, invoke } from "@tauri-apps/api/core";

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

export type HAlign = "left" | "center" | "right";
export type VAlign = "top" | "middle" | "bottom";
export type CaptionSize = "sm" | "md" | "lg";

/** Overlay caption styling; only used on the covers. */
export interface CaptionStyle {
	h: HAlign;
	v: VAlign;
	size: CaptionSize;
	color: "white" | "black";
	/** When true the caption sits at x/y (percent of the page) instead of the h/v alignment. */
	custom: boolean;
	x: number;
	y: number;
}

/** One image position on a page, with its own framing. */
export interface Slot {
	/** Original file on disk (null = empty slot). This is what gets saved and exported. */
	path: string | null;
	/** Cached, downscaled copies made by the backend for display. */
	previewUrl: string | null;
	thumbUrl: string | null;
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
	/** Front cover only; styled and positioned together with the caption. */
	subCaption: string;
	author: string;
	captionStyle: CaptionStyle;
}

const PAGE_COUNT = 8;

function labelFor(index: number): string {
	if (index === 0) return "Front cover";
	if (index === PAGE_COUNT - 1) return "Back cover";
	return `Page ${index + 1}`;
}

function blankSlot(): Slot {
	return { path: null, previewUrl: null, thumbUrl: null, mode: "fit", offsetX: 50, offsetY: 50, aspect: "auto" };
}

function blankPage(index: number): Page {
	return {
		id: index,
		label: labelFor(index),
		images: [blankSlot(), blankSlot()],
		caption: "",
		subCaption: "",
		author: "",
		captionStyle: { h: "center", v: "bottom", size: "md", color: "white", custom: false, x: 50, y: 85 },
	};
}

/** Result of the backend's `prepare_image` command. */
interface ImageInfo {
	width: number;
	height: number;
	previewPath: string;
	thumbPath: string;
}

/** The project file / export payload (mirrors src-tauri/src/project.rs). */
export interface Project {
	paper: Paper;
	pages: {
		caption: string;
		subCaption: string;
		author: string;
		captionStyle: CaptionStyle;
		images: Pick<Slot, "path" | "mode" | "offsetX" | "offsetY" | "aspect">[];
	}[];
}

export type Paper = "letter" | "a4";

/**
 * One landscape sheet folds into 4 columns x 2 rows of pages, so a page is
 * (sheet width / 4) x (sheet height / 2). Ratios are width / height.
 */
export const PAPER: Record<Paper, { label: string; pageRatio: number }> = {
	letter: { label: "Letter", pageRatio: 11 / 4 / (8.5 / 2) },
	a4: { label: "A4", pageRatio: 297 / 4 / (210 / 2) },
};

/** Page indices grouped as they are shown: cover, spreads (2-3, 4-5, 6-7), back cover. */
export const views: number[][] = [[0], [1, 2], [3, 4], [5, 6], [7]];

export function viewIndexOf(pageIndex: number): number {
	return views.findIndex((v) => v.includes(pageIndex));
}

/** Covers hold a single image; interior pages hold up to two. */
export function maxImages(pageIndex: number): number {
	return pageIndex === 0 || pageIndex === PAGE_COUNT - 1 ? 1 : 2;
}

export function isCover(pageIndex: number): boolean {
	return maxImages(pageIndex) === 1;
}

export const H_POS: Record<HAlign, number> = { left: 12, center: 50, right: 88 };
export const V_POS: Record<VAlign, number> = { top: 10, middle: 50, bottom: 90 };

export function imageCount(page: Page): number {
	return page.images.filter((s) => s.path).length;
}

class Zine {
	pages = $state<Page[]>(Array.from({ length: PAGE_COUNT }, (_, i) => blankPage(i)));
	selectedIndex = $state(0);
	navExpanded = $state(true);
	paper = $state<Paper>("letter");
	/** Where the project is saved, once it has been saved or opened. */
	projectPath = $state<string | null>(null);
	/** Last message from a file operation, shown in the header. */
	status = $state("");

	pageRatio = $derived(PAPER[this.paper].pageRatio);

	selected = $derived(this.pages[this.selectedIndex]);
	viewIndex = $derived(viewIndexOf(this.selectedIndex));

	updatePage(index: number, patch: Partial<Pick<Page, "caption" | "subCaption" | "author">>) {
		Object.assign(this.pages[index], patch);
	}

	updateCaptionStyle(index: number, patch: Partial<CaptionStyle>) {
		Object.assign(this.pages[index].captionStyle, patch);
	}

	updateSlot(index: number, slot: number, patch: Partial<Slot>) {
		Object.assign(this.pages[index].images[slot], patch);
	}

	/** Import an image from disk into a slot; the backend builds the preview and thumbnail. */
	async setImage(index: number, slot: number, path: string) {
		if (slot >= maxImages(index)) return;
		try {
			const info = await invoke<ImageInfo>("prepare_image", { path });
			this.updateSlot(index, slot, {
				path,
				previewUrl: convertFileSrc(info.previewPath),
				thumbUrl: convertFileSrc(info.thumbPath),
				mode: "fit",
				offsetX: 50,
				offsetY: 50,
			});
			this.compact(index);
		} catch (e) {
			this.status = String(e);
		}
	}

	/** Fill the first free slot; if the page is full, replace `fallbackSlot`. */
	async addImage(index: number, path: string, fallbackSlot = 0) {
		const images = this.pages[index].images;
		const max = maxImages(index);
		const slot = !images[0].path ? 0 : max > 1 && !images[1].path ? 1 : Math.min(fallbackSlot, max - 1);
		await this.setImage(index, slot, path);
	}

	clearImage(index: number, slot: number) {
		this.pages[index].images[slot] = blankSlot();
		this.compact(index);
	}

	swapImages(index: number) {
		const images = this.pages[index].images;
		if (!images[0].path || !images[1].path) return;
		this.pages[index].images = [images[1], images[0]];
	}

	reset() {
		this.pages = Array.from({ length: PAGE_COUNT }, (_, i) => blankPage(i));
		this.selectedIndex = 0;
		this.paper = "letter";
		this.projectPath = null;
	}

	/** The persisted subset of the state, in the shape the backend saves and exports. */
	toProject(): Project {
		return {
			paper: this.paper,
			pages: this.pages.map((p) => ({
				caption: p.caption,
				subCaption: p.subCaption,
				author: p.author,
				captionStyle: { ...p.captionStyle },
				images: p.images.map((s) => ({
					path: s.path,
					mode: s.mode,
					offsetX: s.offsetX,
					offsetY: s.offsetY,
					aspect: s.aspect,
				})),
			})) as Project["pages"],
		};
	}

	/** Replace the state with a loaded project, rebuilding previews; returns paths that failed. */
	async load(project: Project): Promise<string[]> {
		const pages = project.pages.map((data, i) => {
			const page = blankPage(i);
			page.caption = data.caption;
			page.subCaption = data.subCaption ?? "";
			page.author = data.author ?? "";
			page.captionStyle = { ...page.captionStyle, ...data.captionStyle };
			page.images = data.images.map((s) => ({ ...blankSlot(), ...s })) as [Slot, Slot];
			return page;
		});
		const loaded = pages.map((p) => p.images.map((s) => s.path));
		pages.forEach((p) => p.images.forEach((s) => (s.path = null)));
		this.pages = pages;
		this.paper = project.paper;
		this.selectedIndex = 0;

		const failed: string[] = [];
		await Promise.all(
			pages.flatMap((p, i) =>
				p.images.map(async (_, s) => {
					const path = loaded[i][s];
					if (!path) return;
					try {
						const info = await invoke<ImageInfo>("prepare_image", { path });
						this.updateSlot(i, s, {
							path,
							previewUrl: convertFileSrc(info.previewPath),
							thumbUrl: convertFileSrc(info.thumbPath),
						});
					} catch {
						failed.push(path);
					}
				})
			)
		);
		this.pages.forEach((_, i) => this.compact(i));
		return failed;
	}

	private compact(index: number) {
		const images = this.pages[index].images;
		if (!images[0].path && images[1].path) {
			this.pages[index].images = [images[1], images[0]];
		}
	}
}

export const zine = new Zine();
