<script lang="ts">
	import { quickCombine } from '$lib/ipc/sequences';
	import { writeToClipboard } from '$lib/ipc/snippets';

	interface Props {
		selectedTexts: string[];
		onClearSelection?: () => void;
	}

	let { selectedTexts, onClearSelection }: Props = $props();

	let combinedResult = $state('');
	let isCopied = $state(false);

	async function handleCombine() {
		if (selectedTexts.length === 0) return;
		try {
			combinedResult = await quickCombine(selectedTexts, '\n\n');
			await writeToClipboard(combinedResult);
			isCopied = true;
			setTimeout(() => {
				isCopied = false;
			}, 2000);
		} catch (e) {
			console.error(e);
		}
	}
</script>

{#if selectedTexts.length > 0}
	<div class="fixed bottom-4 left-1/2 -translate-x-1/2 bg-slate-900 border border-slate-700 text-white px-4 py-2.5 rounded-xl shadow-2xl z-50 flex items-center gap-4 text-xs">
		<span class="font-medium text-slate-300">
			{selectedTexts.length} Elemente ausgewählt
		</span>

		<div class="flex items-center gap-2">
			<button
				type="button"
				onclick={handleCombine}
				class="px-3 py-1 bg-emerald-600 hover:bg-emerald-500 font-medium rounded-lg transition shadow-sm flex items-center gap-1.5"
			>
				{#if isCopied}
					<span>✓ Kopiert!</span>
				{:else}
					<span>Kombinieren & Kopieren</span>
				{/if}
			</button>

			{#if onClearSelection}
				<button
					type="button"
					onclick={onClearSelection}
					class="px-2 py-1 text-slate-400 hover:text-white transition"
				>
					Auswahl aufheben
				</button>
			{/if}
		</div>
	</div>
{/if}
