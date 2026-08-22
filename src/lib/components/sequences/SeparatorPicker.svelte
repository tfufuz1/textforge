<script lang="ts">
	interface Props {
		separatorType: string;
		customText: string;
		onChangeType: (type: string) => void;
		onChangeCustomText: (text: string) => void;
	}

	let { separatorType, customText, onChangeType, onChangeCustomText }: Props = $props();
</script>

<div class="space-y-2 text-xs">
	<label for="sep-type" class="block font-medium text-slate-400">Trennzeichen</label>
	<select
		id="sep-type"
		value={separatorType}
		onchange={(e) => onChangeType((e.target as HTMLSelectElement).value)}
		class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-white outline-none focus:border-emerald-500"
	>
		<option value="newline">Leerzeile (1x)</option>
		<option value="double_newline">Doppelte Leerzeile (2x)</option>
		<option value="numbered">Nummerierte Liste (1., 2., ...)</option>
		<option value="markdown">Markdown Abschnitte (## ...)</option>
		<option value="custom">Benutzerdefiniert</option>
		<option value="none">Kein Trennzeichen</option>
	</select>

	{#if separatorType === 'custom'}
		<input
			type="text"
			value={customText}
			oninput={(e) => onChangeCustomText((e.target as HTMLInputElement).value)}
			placeholder="z.B. \n---\n"
			class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-white outline-none focus:border-emerald-500"
		/>
	{/if}
</div>
