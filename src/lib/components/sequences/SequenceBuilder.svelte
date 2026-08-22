<script lang="ts">
	import { sequencesActions } from '$lib/stores/sequences';
	import type { SequenceItemDto } from '$lib/ipc/sequences';
	import SequenceItemCard from './SequenceItemCard.svelte';
	import SeparatorPicker from './SeparatorPicker.svelte';

	interface Props {
		onClose?: () => void;
	}

	let { onClose }: Props = $props();

	let name = $state('');
	let separatorType = $state('newline');
	let customSeparator = $state('');
	let items = $state<SequenceItemDto[]>([]);
	let isSubmitting = $state(false);

	function addLiteralItem() {
		items = [
			...items,
			{
				id: crypto.randomUUID(),
				orderIndex: items.length,
				refType: 'literal',
				refId: null,
				literalText: 'Freitext Baustein',
				pipelineId: null,
				prefixOverride: null,
				suffixOverride: null,
				enabled: true,
			},
		];
	}

	function removeItem(idx: number) {
		items = items.filter((_, i) => i !== idx);
	}

	function toggleItem(idx: number) {
		items = items.map((it, i) => (i === idx ? { ...it, enabled: !it.enabled } : it));
	}

	async function handleSave() {
		if (!name.trim() || items.length === 0 || isSubmitting) return;
		isSubmitting = true;
		try {
			await sequencesActions.create({
				name: name.trim(),
				separator: JSON.stringify({ _type: separatorType, text: customSeparator }),
				items,
			});
			onClose?.();
		} catch (e) {
			console.error(e);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="bg-slate-900 border border-slate-800 rounded-xl p-5 space-y-4 shadow-2xl">
	<div class="flex items-center justify-between border-b border-slate-800 pb-3">
		<h3 class="text-sm font-semibold text-slate-200">Neue Sequenz erstellen</h3>
		{#if onClose}
			<button type="button" onclick={onClose} class="text-slate-400 hover:text-white">✕</button>
		{/if}
	</div>

	<div class="space-y-3">
		<div>
			<label for="seq-name-input" class="block text-xs font-medium text-slate-400 mb-1">Sequenz Name</label>
			<input
				id="seq-name-input"
				type="text"
				bind:value={name}
				placeholder="z.B. E-Mail Bausteine, Release Notes"
				class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-xs text-white outline-none focus:border-emerald-500"
			/>
		</div>

		<SeparatorPicker
			{separatorType}
			customText={customSeparator}
			onChangeType={(t) => (separatorType = t)}
			onChangeCustomText={(t) => (customSeparator = t)}
		/>

		<div class="space-y-2 pt-2">
			<div class="flex items-center justify-between">
				<span class="text-xs font-medium text-slate-400">Elemente ({items.length})</span>
				<button
					type="button"
					onclick={addLiteralItem}
					class="text-xs text-emerald-400 hover:underline font-medium"
				>
					+ Freitext-Baustein
				</button>
			</div>

			<div class="space-y-1.5 max-h-48 overflow-y-auto pr-1">
				{#each items as item, idx (item.id)}
					<SequenceItemCard
						{item}
						index={idx}
						onRemove={() => removeItem(idx)}
						onToggle={() => toggleItem(idx)}
					/>
				{:else}
					<p class="text-xs text-slate-500 italic py-3 text-center border border-dashed border-slate-800 rounded-lg">
						Noch keine Elemente hinzugefügt.
					</p>
				{/each}
			</div>
		</div>
	</div>

	<div class="flex items-center justify-end gap-2 pt-2 border-t border-slate-800">
		<button
			type="button"
			onclick={handleSave}
			disabled={!name.trim() || items.length === 0 || isSubmitting}
			class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg text-xs font-medium transition disabled:opacity-50"
		>
			Sequenz Speichern
		</button>
	</div>
</div>
