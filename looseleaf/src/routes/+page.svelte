<script lang="ts">
	import Header from "#lib/components/Header.svelte";
	import PageNav from "#lib/components/PageNav.svelte";
	import PageViewer from "#lib/components/PageViewer.svelte";
	import PhotoTable from "#lib/components/PhotoTable.svelte";
	import SettingsPane from "#lib/components/SettingsPane.svelte";

	import { invoke } from "@tauri-apps/api/core";
	import { zine } from "#lib/zine.svelte.js";

	let viewer: PageViewer;

	// Once the project has a file, keep it saved shortly after each change.
	$effect(() => {
		const path = zine.projectPath;
		if (!path) return;
		const project = zine.toProject(); // reads every persisted field, so edits re-run this
		const timer = setTimeout(() => {
			invoke("save_project", { path, project }).catch((e) => (zine.status = String(e)));
		}, 1500);
		return () => clearTimeout(timer);
	});
</script>

<div class="grid h-screen grid-cols-[auto_1fr] grid-rows-[auto_1fr_auto]">
	<div class="col-span-2"><Header /></div>
	<PageNav />
	<div class="relative grid min-h-0 min-w-0 grid-cols-[minmax(0,1fr)] grid-rows-[minmax(0,1fr)] overflow-hidden">
		<PageViewer bind:this={viewer} />
		<PhotoTable />
	</div>
	<div class="col-span-2"><SettingsPane onChooseFile={(page, slot) => viewer.chooseFile(page, slot)} /></div>
</div>
