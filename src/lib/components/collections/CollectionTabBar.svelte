<script lang="ts">
	import { collectionTabsStore, activeCollectionTabId } from '$lib/stores/collections';

	interface Props {
		onOpenCreateModal?: () => void;
	}

	let { onOpenCreateModal }: Props = $props();

	function selectTab(id: string) {
		activeCollectionTabId.set(id);
	}
</script>

<div class="flex items-center gap-1.5 overflow-x-auto py-1 px-1 border-b border-slate-800 bg-slate-900/50 scrollbar-none">
	{#each $collectionTabsStore as tab (tab.id)}
		<button
			type="button"
			onclick={() => selectTab(tab.id)}
			class="flex items-center gap-2 px-3 py-1.5 text-xs font-medium rounded-lg transition-all whitespace-nowrap border
				{$activeCollectionTabId === tab.id
					? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/30 shadow-sm'
					: 'bg-slate-800/40 text-slate-400 border-slate-800 hover:bg-slate-800/80 hover:text-slate-200'}"
		>
			{#if tab.icon}
				<span>{tab.icon}</span>
			{:else}
				<div
					class="w-2 h-2 rounded-full"
					style="background-color: {tab.color || '#10b981'}"
				></div>
			{/if}
			<span>{tab.name}</span>
			{#if tab.itemCount > 0}
				<span class="text-[10px] px-1.5 py-0.2 rounded-full bg-slate-800 text-slate-400 font-mono">
					{tab.itemCount}
				</span>
			{/if}
		</button>
	{/each}

	<button
		type="button"
		onclick={onOpenCreateModal}
		title="Neuen Reiter erstellen"
		class="p-1.5 text-xs rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition border border-dashed border-slate-800 hover:border-slate-700"
	>
		<svg class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
		</svg>
	</button>
</div>
