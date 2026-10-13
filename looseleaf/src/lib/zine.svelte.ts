export type PageMode = "fit" | "fill";

export interface Page {
	id: number;
	label: string;
	imageUrl: string | null;
	mode: PageMode;
	/** object-position percentages (0-100) used when mode is "fill" */
	offsetX: number;
	offsetY: number;
	caption: string;
}

const PAGE_COUNT = 8;

function labelFor(index: number): string {
	if (index === 0) return "Front cover";
	if (index === PAGE_COUNT - 1) return "Back cover";
	return `Page ${index + 1}`;
}

function blankPage(index: number): Page {
	return {
		id: index,
		label: labelFor(index),
		imageUrl: null,
		mode: "fit",
		offsetX: 50,
		offsetY: 50,
		caption: "",
	};
}

/** Page indices grouped as they are shown: cover, spreads (2-3, 4-5, 6-7), back cover. */
export const views: number[][] = [[0], [1, 2], [3, 4], [5, 6], [7]];

export function viewIndexOf(pageIndex: number): number {
	return views.findIndex((v) => v.includes(pageIndex));
}

class Zine {
	pages = $state<Page[]>(Array.from({ length: PAGE_COUNT }, (_, i) => blankPage(i)));
	selectedIndex = $state(0);
	navExpanded = $state(true);

	selected = $derived(this.pages[this.selectedIndex]);
	viewIndex = $derived(viewIndexOf(this.selectedIndex));

	update(index: number, patch: Partial<Page>) {
		Object.assign(this.pages[index], patch);
	}

	setImage(index: number, file: File) {
		if (!file.type.startsWith("image/")) return;
		this.releaseImage(index);
		this.update(index, {
			imageUrl: URL.createObjectURL(file),
			mode: "fit",
			offsetX: 50,
			offsetY: 50,
		});
	}

	clearImage(index: number) {
		this.releaseImage(index);
		this.update(index, { imageUrl: null, mode: "fit", offsetX: 50, offsetY: 50 });
	}

	reset() {
		this.pages.forEach((_, i) => this.releaseImage(i));
		this.pages = Array.from({ length: PAGE_COUNT }, (_, i) => blankPage(i));
		this.selectedIndex = 0;
	}

	private releaseImage(index: number) {
		const url = this.pages[index].imageUrl;
		if (url) URL.revokeObjectURL(url);
	}
}

export const zine = new Zine();
