<script lang="ts">
    import { onMount } from 'svelte';
    import { listAllTags } from '../../ipc/snippets';
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let allTags = $state<string[]>([]);
    let selectedTags = $state<string[]>([]);
    let tagsMode = $state<'all' | 'any'>('all');

    onMount(async () => {
        try {
            allTags = await listAllTags();
        } catch (e) {
            console.error("Failed to load tags for filter:", e);
        }
    });

    $effect(() => {
        const storeTags = $snippetFilterStore.tags || [];
        if (JSON.stringify(storeTags) !== JSON.stringify(selectedTags)) {
            selectedTags = [...storeTags];
        }
        const storeMode = ($snippetFilterStore.tagsMode as 'all' | 'any') || 'all';
        if (storeMode !== tagsMode) {
            tagsMode = storeMode;
        }
    });

    function toggleTag(tag: string) {
        if (selectedTags.includes(tag)) {
            selectedTags = selectedTags.filter(t => t !== tag);
        } else {
            selectedTags = [...selectedTags, tag];
        }
        updateStore();
    }

    function toggleMode() {
        tagsMode = tagsMode === 'all' ? 'any' : 'all';
        updateStore();
    }

    function updateStore() {
        snippetFilterStore.update(f => ({
            ...f,
            tags: selectedTags,
            tagsMode
        }));
        loadSnippets();
    }

    function clearTags() {
        selectedTags = [];
        updateStore();
    }
</script>

<div class="space-y-3">
    <div class="flex justify-between items-center">
        <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Tags</h3>
        <div class="flex items-center space-x-2">
            {#if selectedTags.length > 0}
                <button 
                    onclick={clearTags} 
                    class="text-[10px] text-slate-500 hover:text-slate-300 transition-colors"
                >
                    Clear
                </button>
            {/if}
            <button
                onclick={toggleMode}
                class="px-2 py-0.5 text-[9px] font-bold font-mono rounded border transition-colors {tagsMode === 'all' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-300' : 'bg-amber-600/30 border-amber-500 text-amber-300'}"
                title={tagsMode === 'all' ? 'Snippets müssen ALLE ausgewählten Tags haben' : 'Snippets können EINIGE der ausgewählten Tags haben'}
            >
                {tagsMode.toUpperCase()}
            </button>
        </div>
    </div>

    {#if allTags.length > 0}
        <div class="flex flex-wrap gap-1.5 max-h-36 overflow-y-auto pr-1">
            {#each allTags as tag}
                <button
                    onclick={() => toggleTag(tag)}
                    class="px-2 py-0.5 text-[11px] font-medium rounded-lg border transition-all {selectedTags.includes(tag) ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200 hover:bg-slate-800'}"
                >
                    #{tag}
                </button>
            {/each}
        </div>
    {:else}
        <p class="text-[11px] text-slate-500 italic">Keine Tags vorhanden.</p>
    {/if}
</div>
