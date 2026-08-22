<script lang="ts">
	import { onMount } from 'svelte';
	import { sequencesStore, sequencesActions } from '$lib/stores/sequences';
	import SequenceBuilder from '$lib/components/sequences/SequenceBuilder.svelte';

	let showBuilder = $state(false);

	onMount(() => {
		sequencesActions.loadAll();
	});
</script>

<div class="h-full flex flex-col p-6 space-y-6 bg-slate-950 text-slate-100 overflow-y-auto">
	<div class="flex justify-between items-center">
		<div>
			<h1 class="text-2xl font-extrabold tracking-tight text-white flex items-center space-x-2">
				<span>🔗</span>
				<span>Sequenzen Engine (§ 24)</span>
			</h1>
			<p class="text-xs text-slate-400 mt-1">
				Kombiniere Snippets, Clipboard-Einträge und Freitext in fester Reihenfolge.
			</p>
		</div>
		<button
			type="button"
			onclick={() => showBuilder = true}
			class="px-4 py-2 bg-emerald-600 hover:bg-emerald-500 text-white font-semibold text-xs rounded-xl transition shadow-lg shadow-emerald-600/20"
		>
			+ Neue Sequenz
		</button>
	</div>

	{#if showBuilder}
		<SequenceBuilder onClose={() => showBuilder = false} />
	{/if}

	<div class="space-y-3">
		<h2 class="text-xs font-semibold text-slate-400 uppercase tracking-wider">
			Gespeicherte Sequenzen ({$sequencesStore.length})
		</h2>

		<div class="grid grid-cols-1 md:grid-cols-2 gap-4">
			{#each $sequencesStore as seq (seq.id)}
				<div class="bg-slate-900 border border-slate-800 rounded-2xl p-4 space-y-2 hover:border-slate-700 transition shadow-sm">
					<div class="flex items-center justify-between">
						<span class="font-bold text-sm text-slate-200">{seq.name}</span>
						<button
							type="button"
							onclick={() => sequencesActions.delete(seq.id)}
							class="text-slate-500 hover:text-red-400 text-xs transition"
						>
							Löschen
						</button>
					</div>
					<div class="text-xs text-slate-400 font-mono">
						{seq.items.length} Elemente
					</div>
				</div>
			{:else}
				<div class="col-span-2 p-8 text-center border border-dashed border-slate-800 rounded-2xl text-slate-500 text-xs italic">
					Noch keine Sequenzen erstellt. Klicke oben auf "+ Neue Sequenz".
				</div>
			{/each}
		</div>
	</div>
</div>
