<script lang="ts">
	import { globalSearchResultsStore, globalSearchQueryStore } from '$lib/stores/global-search';
</script>

{#if $globalSearchQueryStore.trim().length > 0}
	<div class="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-3 mb-4">
		<div class="flex items-center justify-between border-b border-slate-800 pb-2">
			<h3 class="text-xs font-semibold text-slate-300">
				Suchergebnisse für "{$globalSearchQueryStore}"
			</h3>
			<span class="text-[10px] text-slate-500 font-mono">{$globalSearchResultsStore.length} Treffer</span>
		</div>

		<div class="space-y-2 max-h-80 overflow-y-auto pr-1">
			{#each $globalSearchResultsStore as item (item.id)}
				<div class="bg-slate-950 border border-slate-800/80 rounded-lg p-3 hover:border-slate-700 transition space-y-1">
					<div class="flex items-center justify-between text-xs">
						<span class="font-medium text-slate-200">{item.title}</span>
						<span class="text-[10px] font-semibold px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-400 uppercase">
							{item.itemKind}
						</span>
					</div>
					<div class="text-[11px] text-slate-400 font-mono line-clamp-2">
						<!-- eslint-disable-next-line svelte/no-at-html-tags -->
						{@html item.highlightedPreview}
					</div>
				</div>
			{:else}
				<p class="text-xs text-slate-500 italic py-4 text-center">Keine Treffer gefunden.</p>
			{/each}
		</div>
	</div>
{/if}
