<script lang="ts">
	import { isCover, views, zine } from "$lib/zine.svelte";
	import MiniOptions from "./settings/MiniOptions.svelte";
	import Options from "./settings/Options.svelte";

	let { onChooseFile }: { onChooseFile: (page: number, slot: number) => void } = $props();

	const view = $derived(views[zine.viewIndex]);
</script>

<section class="flex divide-x-2 divide-border border-t-2 border-border bg-card">
	{#each view as index (index)}
		<div class="flex min-w-0 flex-1 flex-col gap-2 px-4 py-3">
			{#if isCover(index)}
				<Options {index} {onChooseFile} />
			{:else}
				<MiniOptions {index} {onChooseFile} />
			{/if}
		</div>
	{/each}
</section>
