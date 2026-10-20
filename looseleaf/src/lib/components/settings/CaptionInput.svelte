<script lang="ts">
	import { Input } from "$lib/components/ui/input";

	let {
		value,
		placeholder,
		label,
		oninput,
	}: { value: string; placeholder: string; label: string; oninput: (value: string) => void } = $props();
</script>

<!-- Auto-correct is off so the webview doesn't capitalize or "fix" caption text. -->
<Input
	class="min-w-40 flex-1"
	autocapitalize="off"
	autocorrect="off"
	spellcheck={false}
	{placeholder}
	aria-label={label}
	{value}
	oninput={(e) => oninput(e.currentTarget.value)}
/>
