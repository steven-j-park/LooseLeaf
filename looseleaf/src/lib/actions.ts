import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { zine, type Project } from "$lib/zine.svelte";

const IMAGE_EXTS = ["jpg", "jpeg", "png", "webp", "gif", "bmp", "tif", "tiff"];
const PROJECT_FILTER = { name: "LooseLeaf project", extensions: ["looseleaf"] };

export function isImagePath(path: string): boolean {
	return IMAGE_EXTS.includes(path.split(".").pop()?.toLowerCase() ?? "");
}

/** Ask for an image file with the native dialog and load it into a page slot. */
export async function chooseImage(page: number, slot: number) {
	const path = await open({ multiple: false, filters: [{ name: "Images", extensions: IMAGE_EXTS }] });
	if (typeof path !== "string") return;
	zine.selectedIndex = page;
	await zine.setImage(page, slot, path);
}

async function run(label: string, task: () => Promise<string | void>) {
	zine.status = `${label}…`;
	try {
		zine.status = (await task()) || "";
	} catch (e) {
		zine.status = String(e);
	}
}

export async function saveProject(saveAs = false) {
	let path = saveAs ? null : zine.projectPath;
	if (!path) {
		path = await save({ filters: [PROJECT_FILTER], defaultPath: "zine.looseleaf" });
		if (!path) return;
	}
	const target = path;
	await run("Saving", async () => {
		await invoke("save_project", { path: target, project: zine.toProject() });
		zine.projectPath = target;
		return "Saved";
	});
}

export async function openProject() {
	const path = await open({ multiple: false, filters: [PROJECT_FILTER] });
	if (typeof path !== "string") return;
	await run("Opening", async () => {
		const project = await invoke<Project>("load_project", { path });
		const missing = await zine.load(project);
		zine.projectPath = path;
		return missing.length ? `Opened; ${missing.length} image(s) could not be found` : "Opened";
	});
}

export async function exportZine() {
	const path = await save({
		filters: [{ name: "PDF", extensions: ["pdf"] }],
		defaultPath: "zine.pdf",
	});
	if (!path) return;
	await run("Exporting", async () => {
		await invoke("export_zine", { project: zine.toProject(), path });
		return "Exported";
	});
}
