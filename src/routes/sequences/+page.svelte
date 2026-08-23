<script lang="ts">
	import { onMount } from 'svelte';
	import { sequencesStore, sequencesActions } from '$lib/stores/sequences';
	import SequenceBuilder from '$lib/components/sequences/SequenceBuilder.svelte';
	import { renderSequence } from '$lib/ipc/sequences';
	import { writeToClipboard } from '$lib/ipc/clipboard';

	let showBuilder = $state(false);
	let copyingId = $state<string | null>(null);
	let copiedId = $state<string | null>(null);

	onMount(() => {
		sequencesActions.loadAll();
	});

	async function handleRenderAndCopy(id: string) {
		copyingId = id;
		try {
			const res = await renderSequence(id);
			await writeToClipboard(res.finalOutput);
			copiedId = id;
			setTimeout(() => {
				if (copiedId === id) copiedId = null;
			}, 2000);
		} catch (e) {
			console.error('Failed to render/copy sequence:', e);
		} finally {
			copyingId = null;
		}
	}
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
				<div class="bg-slate-900 border border-slate-800 rounded-2xl p-4 space-y-3 hover:border-slate-700 transition shadow-sm flex flex-col justify-between">
					<div class="space-y-1">
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

					<div class="pt-2 border-t border-slate-800/80 flex justify-end">
						<button
							type="button"
							onclick={() => handleRenderAndCopy(seq.id)}
							disabled={copyingId === seq.id}
							class="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-semibold text-xs rounded-lg transition flex items-center gap-1.5 shadow-sm"
						>
							{#if copyingId === seq.id}
								<span>Rendere...</span>
							{:else if copiedId === seq.id}
								<span class="text-emerald-300">✓ In Zwischenablage!</span>
							{:else}
								<span>📋 Rendern & Kopieren</span>
							{/if}
						</button>
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
