<script lang="ts">
	import Header from "$lib/components/Header.svelte";
	import PageNav from "$lib/components/PageNav.svelte";
	import PageViewer from "$lib/components/PageViewer.svelte";
	import SettingsPane from "$lib/components/SettingsPane.svelte";

	let viewer: PageViewer;
</script>

<div class="grid h-screen grid-cols-[auto_1fr] grid-rows-[auto_1fr_auto]">
	<div class="col-span-2"><Header /></div>
	<PageNav />
	<PageViewer bind:this={viewer} />
	<div class="col-span-2"><SettingsPane onChooseFile={() => viewer.chooseFile()} /></div>
</div>
