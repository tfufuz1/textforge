<script lang="ts">
	import { activeCollectionTabId, collectionTabsStore } from '$lib/stores/collections';

	let activeTab = $derived.by(() => {
		return $collectionTabsStore.find(t => t.id === $activeCollectionTabId) ?? null;
	});
</script>

{#if activeTab}
	<div class="p-3 bg-slate-900/30 rounded-lg border border-slate-800/50 flex items-center justify-between">
		<div class="flex items-center gap-2">
			{#if activeTab.icon}
				<span>{activeTab.icon}</span>
			{:else}
				<div class="w-2.5 h-2.5 rounded-full" style="background-color: {activeTab.color || '#10b981'}"></div>
			{/if}
			<span class="text-xs font-semibold text-slate-200">{activeTab.name}</span>
			<span class="text-[10px] text-slate-400">({activeTab.kind})</span>
		</div>
		<span class="text-xs text-slate-400 font-mono">{activeTab.itemCount} Elemente</span>
	</div>
{/if}
