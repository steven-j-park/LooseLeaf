<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Text } from "$lib/components/ui/text";
	import { PAPER, zine, type Paper } from "$lib/zine.svelte";

	const papers = Object.keys(PAPER) as Paper[];
</script>

<header class="flex items-center justify-between border-b-2 border-border bg-primary px-4 py-2">
	<Text as="h1" class="text-2xl">LooseLeaf</Text>
	<div class="flex items-center gap-2">
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
		<Button variant="outline" size="sm" class="bg-background" onclick={() => zine.reset()}>New</Button>
		<Button variant="secondary" size="sm" disabled class="opacity-50">Export</Button>
	</div>
</header>
