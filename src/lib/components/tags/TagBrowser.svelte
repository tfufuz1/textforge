<script lang="ts">
	import { tagRegistryStore, tagsActions } from '$lib/stores/tags';
	import { onMount } from 'svelte';

	interface Props {
		onSelectTag?: (tagName: string) => void;
	}

	let { onSelectTag }: Props = $props();

	onMount(() => {
		tagsActions.loadSuggestions('');
	});
</script>

<div class="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3">
	<div class="flex items-center justify-between border-b border-slate-800 pb-2">
		<h3 class="text-xs font-semibold text-slate-300 flex items-center gap-2">
			<svg class="w-3.5 h-3.5 text-emerald-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
				<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
			</svg>
			Tag Browser
		</h3>
		<span class="text-[10px] text-slate-500 font-mono">{$tagRegistryStore.length} Tags</span>
	</div>

	<div class="flex flex-wrap gap-1.5 max-h-48 overflow-y-auto pr-1">
		{#each $tagRegistryStore as tag (tag.name)}
			<button
				type="button"
				onclick={() => onSelectTag?.(tag.name)}
				class="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium transition hover:scale-105 border border-slate-800 bg-slate-800/60 text-slate-200 hover:bg-slate-800"
			>
				{#if tag.color}
					<span class="w-2 h-2 rounded-full" style="background-color: {tag.color}"></span>
				{:else}
					<span class="text-emerald-400">#</span>
				{/if}
				<span>{tag.name}</span>
				<span class="text-[10px] text-slate-400 ml-0.5">({tag.usageCount})</span>
			</button>
		{:else}
			<p class="text-xs text-slate-500 italic py-2">Keine Tags vorhanden.</p>
		{/each}
	</div>
</div>
