<script lang="ts">
	import { ignoreRulesStore, ignoreRulesActions } from '$lib/stores/ignore-rules';
	import { onMount } from 'svelte';

	let matchType = $state('source_app');
	let pattern = $state('');

	onMount(() => {
		ignoreRulesActions.loadAll();
	});

	async function handleAdd() {
		if (!pattern.trim()) return;
		try {
			await ignoreRulesActions.create({
				matchType,
				pattern: pattern.trim(),
			});
			pattern = '';
		} catch (e) {
			console.error(e);
		}
	}
</script>

<div class="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-4">
	<div class="border-b border-slate-800 pb-2">
		<h3 class="text-xs font-semibold text-slate-200">Clipboard Ignore-Regeln</h3>
		<p class="text-[11px] text-slate-400">Verhindert das Erfassen sensibler Apps oder TOTP-Muster.</p>
	</div>

	<form onsubmit={(e) => { e.preventDefault(); handleAdd(); }} class="flex gap-2 text-xs">
		<select
			bind:value={matchType}
			class="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-white outline-none focus:border-emerald-500"
		>
			<option value="source_app">App-Name</option>
			<option value="content_regex">Inhalt Regex</option>
			<option value="content_type">Content-Typ</option>
		</select>
		<input
			type="text"
			bind:value={pattern}
			placeholder="z.B. KeePassXC oder ^[0-9]{6}$"
			class="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-white outline-none focus:border-emerald-500"
		/>
		<button
			type="submit"
			disabled={!pattern.trim()}
			class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg font-medium transition disabled:opacity-50"
		>
			Hinzufügen
		</button>
	</form>

	<div class="space-y-1.5 max-h-40 overflow-y-auto pr-1">
		{#each $ignoreRulesStore as rule (rule.id)}
			<div class="bg-slate-950 border border-slate-800/80 rounded-lg p-2.5 flex items-center justify-between text-xs">
				<div class="flex items-center gap-2">
					<span class="text-[10px] uppercase font-semibold text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded">
						{rule.matchType}
					</span>
					<span class="font-mono text-slate-300">{rule.pattern}</span>
				</div>
				<button
					type="button"
					onclick={() => ignoreRulesActions.delete(rule.id)}
					class="text-slate-500 hover:text-red-400 text-xs px-1.5"
				>
					Entfernen
				</button>
			</div>
		{:else}
			<p class="text-xs text-slate-500 italic py-2">Keine Ignore-Regeln vorhanden.</p>
		{/each}
	</div>
</div>
