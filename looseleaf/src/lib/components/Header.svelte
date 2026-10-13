<script lang="ts">
	import { Button } from "$lib/components/ui/button";
	import { Text } from "$lib/components/ui/text";
	import { zine } from "$lib/zine.svelte";
</script>

<header class="flex items-center justify-between border-b-2 border-border bg-primary px-4 py-2">
	<Text as="h1" class="text-2xl">LooseLeaf</Text>
	<div class="flex gap-2">
		<Button variant="outline" size="sm" class="bg-background" onclick={() => zine.reset()}>New</Button>
		<Button variant="secondary" size="sm" disabled class="opacity-50">Export</Button>
	</div>
</header>
