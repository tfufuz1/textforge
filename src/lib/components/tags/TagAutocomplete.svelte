<script lang="ts">
	import { suggestTags, type TagInfoDto } from '$lib/ipc/tags';

	interface Props {
		selectedTags: string[];
		onAddTag: (tag: string) => void;
		onRemoveTag: (tag: string) => void;
	}

	let { selectedTags, onAddTag, onRemoveTag }: Props = $props();

	let inputQuery = $state('');
	let suggestions = $state<TagInfoDto[]>([]);
	let showDropdown = $state(false);

	async function handleInput() {
		if (!inputQuery.trim()) {
			suggestions = [];
			showDropdown = false;
			return;
		}
		try {
			const res = await suggestTags(inputQuery, 5);
			suggestions = res.filter(t => !selectedTags.includes(t.name));
			showDropdown = suggestions.length > 0;
		} catch (e) {
			console.error(e);
		}
	}

	function handleAdd(tagName: string) {
		onAddTag(tagName);
		inputQuery = '';
		suggestions = [];
		showDropdown = false;
	}

	function handleKeyDown(e: KeyboardEvent) {
		if (e.key === 'Enter' && inputQuery.trim()) {
			e.preventDefault();
			handleAdd(inputQuery.trim());
		}
	}
</script>

<div class="space-y-2">
	<div class="flex flex-wrap gap-1.5 items-center bg-slate-950 border border-slate-800 rounded-lg p-2 min-h-[38px] relative">
		{#each selectedTags as tag}
			<span class="inline-flex items-center gap-1 bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 px-2 py-0.5 rounded text-xs">
				#{tag}
				<button type="button" onclick={() => onRemoveTag(tag)} class="hover:text-red-400 ml-0.5">✕</button>
			</span>
		{/each}

		<input
			type="text"
			bind:value={inputQuery}
			oninput={handleInput}
			onkeydown={handleKeyDown}
			placeholder={selectedTags.length === 0 ? 'Tags hinzufügen...' : ''}
			class="bg-transparent text-xs text-white outline-none flex-1 min-w-[80px]"
		/>

		{#if showDropdown}
			<div class="absolute left-0 right-0 top-full mt-1 bg-slate-900 border border-slate-800 rounded-lg shadow-xl z-50 overflow-hidden py-1">
				{#each suggestions as sug}
					<button
						type="button"
						onclick={() => handleAdd(sug.name)}
						class="w-full text-left px-3 py-1.5 text-xs text-slate-200 hover:bg-slate-800 flex items-center justify-between"
					>
						<span>#{sug.name}</span>
						<span class="text-[10px] text-slate-500 font-mono">{sug.usageCount}</span>
					</button>
				{/each}
			</div>
		{/if}
	</div>
</div>
