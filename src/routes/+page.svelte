<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => {
		goto('/clipboard');
	});
</script>

<div class="p-8 text-slate-400 text-xs font-mono">Lade TextForge...</div>
