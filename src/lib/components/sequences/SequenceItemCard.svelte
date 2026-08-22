<script lang="ts">
	import type { SequenceItemDto } from '$lib/ipc/sequences';

	interface Props {
		item: SequenceItemDto;
		index: number;
		onRemove: () => void;
		onToggle: () => void;
	}

	let { item, index, onRemove, onToggle }: Props = $props();
</script>

<div class="bg-slate-900 border border-slate-800 rounded-lg p-3 flex items-center justify-between gap-3 text-xs">
	<div class="flex items-center gap-2 flex-1 min-w-0">
		<button
			type="button"
			onclick={onToggle}
			class="w-4 h-4 rounded border flex items-center justify-center text-[10px] transition
				{item.enabled ? 'bg-emerald-600 border-emerald-500 text-white' : 'border-slate-700 text-transparent'}"
		>
			✓
		</button>
		<span class="font-mono text-slate-500">#{index + 1}</span>
		<span class="font-semibold text-emerald-400 capitalize">{item.refType}</span>
		<span class="text-slate-300 truncate">
			{item.literalText || item.refId || 'Element'}
		</span>
	</div>

	<button
		type="button"
		onclick={onRemove}
		class="text-slate-500 hover:text-red-400 p-1 transition"
		title="Entfernen"
	>
		✕
	</button>
</div>
