<script lang="ts">
	import { globalSearchActions, globalSearchQueryStore } from '$lib/stores/global-search';

	let query = $state('');

	function handleInput() {
		globalSearchActions.search(query);
	}
</script>

<div class="relative w-full max-w-xl">
	<div class="relative flex items-center">
		<svg class="w-4 h-4 text-slate-400 absolute left-3 pointer-events-none" fill="none" viewBox="0 0 24 24" stroke="currentColor">
			<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
		</svg>
		<input
			type="text"
			bind:value={query}
			oninput={handleInput}
			placeholder="Globale Suche (Snippets, Clipboard, Scripts...)... [Strg+F]"
			class="w-full bg-slate-900 border border-slate-800 rounded-xl pl-9 pr-8 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-emerald-500 transition"
		/>
		{#if query}
			<button
				type="button"
				onclick={() => { query = ''; globalSearchActions.search(''); }}
				class="absolute right-2.5 text-slate-500 hover:text-slate-300 text-xs"
			>
				✕
			</button>
		{/if}
	</div>
</div>
