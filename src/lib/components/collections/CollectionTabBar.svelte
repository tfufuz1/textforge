<script lang="ts">
	import { collectionTabsStore, activeCollectionTabId } from '$lib/stores/collections';
	import PlusIcon from '$lib/components/icons/PlusIcon.svelte';
	import TagIcon from '$lib/components/icons/TagIcon.svelte';

	interface Props {
		onOpenCreateModal?: () => void;
		variant?: 'horizontal' | 'vertical';
	}

	let { onOpenCreateModal, variant = 'horizontal' }: Props = $props();

	function selectTab(id: string) {
		activeCollectionTabId.set(id);
	}
</script>

{#if variant === 'horizontal'}
	<div class="flex items-center gap-1.5 overflow-x-auto py-1 px-1 border-b border-slate-800/80 bg-slate-900/40 scrollbar-none">
		{#each $collectionTabsStore as tab (tab.id)}
			<button
				type="button"
				onclick={() => selectTab(tab.id)}
				class="flex items-center gap-2 px-3 py-1.5 text-xs font-semibold rounded-xl transition-all whitespace-nowrap border
					{$activeCollectionTabId === tab.id
						? 'bg-indigo-600/20 text-indigo-300 border-indigo-500/40 shadow-sm'
						: 'bg-slate-900/60 text-slate-400 border-slate-800 hover:bg-slate-800/60 hover:text-slate-200'}"
			>
				{#if tab.icon}
					<span class="text-xs">{tab.icon}</span>
				{:else}
					<div
						class="w-2 h-2 rounded-full shrink-0"
						style="background-color: {tab.color || '#6366f1'}"
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
			class="p-1.5 text-xs rounded-xl text-slate-400 hover:text-white hover:bg-slate-800 transition border border-dashed border-slate-800 hover:border-slate-700 flex items-center justify-center"
		>
			<PlusIcon class="w-3.5 h-3.5" />
		</button>
	</div>
{:else}
	<div class="space-y-1">
		<div class="flex items-center justify-between px-3 py-1.5 text-[11px] font-semibold text-slate-400 uppercase tracking-wider">
			<span class="flex items-center space-x-1.5">
				<TagIcon class="w-3.5 h-3.5 text-indigo-400" />
				<span>Eigene Tabs</span>
			</span>
			<button
				type="button"
				onclick={onOpenCreateModal}
				title="Neuen Tab erstellen"
				class="p-1 rounded-md text-slate-400 hover:text-indigo-300 hover:bg-slate-800/80 transition"
			>
				<PlusIcon class="w-3.5 h-3.5" />
			</button>
		</div>

		{#each $collectionTabsStore as tab (tab.id)}
			<button
				type="button"
				onclick={() => selectTab(tab.id)}
				class="w-full flex items-center justify-between px-3 py-2 rounded-xl text-xs font-medium transition-all duration-150 border
					{$activeCollectionTabId === tab.id
						? 'bg-indigo-600/20 text-indigo-200 border-indigo-500/40 shadow-sm font-semibold'
						: 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border-transparent'}"
			>
				<div class="flex items-center space-x-2.5 min-w-0">
					{#if tab.icon}
						<span class="text-sm shrink-0">{tab.icon}</span>
					{:else}
						<div
							class="w-2.5 h-2.5 rounded-full shrink-0"
							style="background-color: {tab.color || '#6366f1'}"
						></div>
					{/if}
					<span class="truncate">{tab.name}</span>
				</div>
				{#if tab.itemCount > 0}
					<span class="text-[10px] px-1.5 py-0.5 rounded-md bg-slate-800/80 text-slate-400 font-mono shrink-0">
						{tab.itemCount}
					</span>
				{/if}
			</button>
		{/each}
	</div>
{/if}
