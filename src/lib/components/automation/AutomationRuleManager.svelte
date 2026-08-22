<script lang="ts">
	import { automationRulesStore, automationActions } from '$lib/stores/automation';
	import { onMount } from 'svelte';

	let name = $state('');
	let triggerType = $state('on_clipboard_change');
	let scriptId = $state('');

	onMount(() => {
		automationActions.loadAll();
	});

	async function handleAdd() {
		if (!name.trim() || !scriptId.trim()) return;
		try {
			await automationActions.create({
				name: name.trim(),
				trigger: JSON.stringify({ _type: triggerType }),
				scriptId: scriptId.trim(),
			});
			name = '';
			scriptId = '';
		} catch (e) {
			console.error(e);
		}
	}
</script>

<div class="bg-slate-900 border border-slate-800 rounded-xl p-4 space-y-4">
	<div class="border-b border-slate-800 pb-2">
		<h3 class="text-xs font-semibold text-slate-200">Ereignis-Skript Automatisierung (§ 31)</h3>
		<p class="text-[11px] text-slate-400">Automatische Ausführung bei Clipboard-Änderungen oder Snippet-Aktionen.</p>
	</div>

	<form onsubmit={(e) => { e.preventDefault(); handleAdd(); }} class="space-y-3 text-xs">
		<div class="grid grid-cols-2 gap-2">
			<input
				type="text"
				bind:value={name}
				placeholder="Regel-Name"
				class="bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-white outline-none focus:border-emerald-500"
			/>
			<select
				bind:value={triggerType}
				class="bg-slate-950 border border-slate-800 rounded-lg px-2.5 py-1.5 text-white outline-none focus:border-emerald-500"
			>
				<option value="on_clipboard_change">Bei Clipboard-Änderung</option>
				<option value="on_snippet_insert">Bei Snippet-Einfügen</option>
				<option value="on_app_focus">Bei App-Fokus</option>
			</select>
		</div>

		<div class="flex gap-2">
			<input
				type="text"
				bind:value={scriptId}
				placeholder="Skript ID (QuickJS)"
				class="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-white outline-none focus:border-emerald-500"
			/>
			<button
				type="submit"
				disabled={!name.trim() || !scriptId.trim()}
				class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg font-medium transition disabled:opacity-50"
			>
				Regel Hinzufügen
			</button>
		</div>
	</form>

	<div class="space-y-1.5 max-h-48 overflow-y-auto pr-1">
		{#each $automationRulesStore as rule (rule.id)}
			<div class="bg-slate-950 border border-slate-800/80 rounded-lg p-2.5 flex items-center justify-between text-xs">
				<div>
					<span class="font-medium text-slate-200">{rule.name}</span>
					<span class="text-[10px] text-emerald-400 font-mono ml-2">[{rule.scriptId}]</span>
				</div>
				<button
					type="button"
					onclick={() => automationActions.delete(rule.id)}
					class="text-slate-500 hover:text-red-400 text-xs px-1.5"
				>
					Löschen
				</button>
			</div>
		{:else}
			<p class="text-xs text-slate-500 italic py-2">Keine Automatisierungsregeln vorhanden.</p>
		{/each}
	</div>
</div>
