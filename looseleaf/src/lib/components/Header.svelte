<script lang="ts">
	import { exportZine, openProject, saveProject } from "$lib/actions";
	import { Download, FilePlus, FolderOpen, Save } from "@lucide/svelte";
	import { Button } from "$lib/components/ui/button";
	import { Text } from "$lib/components/ui/text";
	import { PAPER, zine, type Paper } from "$lib/zine.svelte";

	const papers = Object.keys(PAPER) as Paper[];
</script>

<header class="flex items-center justify-between border-b-2 border-border bg-primary px-4 py-2">
	<div class="flex items-center gap-4">
		<Text as="h1" class="text-2xl">LooseLeaf</Text>
		<div class="flex gap-1" role="group" aria-label="Paper size">
			{#each papers as p (p)}
				<Button
					size="sm"
					variant={zine.paper === p ? "default" : "outline"}
					class={zine.paper === p ? "" : "bg-background"}
					onclick={() => (zine.paper = p)}>{PAPER[p].label}</Button
				>
			{/each}
		</div>
	</div>
	<div class="flex items-center gap-2">
		{#if zine.status}
			<span class="max-w-72 truncate text-sm" role="status" title={zine.status}>{zine.status}</span>
		{/if}
		<Button variant="outline" size="sm" class="bg-background" onclick={() => zine.reset()}><FilePlus /> New</Button>
		<Button
			variant="outline"
			size="sm"
			class="bg-background"
			title="Open project"
			aria-label="Open project"
			onclick={() => openProject()}><FolderOpen /> Open</Button
		>
		<Button
			variant="outline"
			size="sm"
			class="bg-background"
			title="Save project"
			aria-label="Save project"
			onclick={() => saveProject()}><Save /> Save</Button
		>
		<Button variant="outline" size="sm" class="bg-chart-5" onclick={() => exportZine()}><Download /> Export as PDF</Button>
	</div>
</header>
