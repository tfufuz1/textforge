<script lang="ts">
    import { onMount } from 'svelte';
    import SnippetList from '../../lib/components/snippets/SnippetList.svelte';
    import SnippetEditor from '../../lib/components/snippets/SnippetEditor.svelte';
    import FilterPanel from '../../lib/components/filter/FilterPanel.svelte';
    import { loadSnippets, snippetFilterStore, selectedTagStore, tagCloud, activeSnippetStore } from '../../lib/stores/snippets';

    let searchQuery = $state('');

    onMount(async () => {
        await loadSnippets();
    });

    function handleSearch() {
        snippetFilterStore.update(f => ({ ...f, searchQuery }));
        loadSnippets();
    }

    function selectTag(tag: string | null) {
        selectedTagStore.set(tag);
        loadSnippets();
    }

    function createNewSnippet() {
        activeSnippetStore.set(null);
    }
</script>

<div class="h-full flex flex-col p-6 space-y-5 bg-slate-950 text-slate-100 overflow-hidden">
    <!-- Header -->
    <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
        <div>
            <h1 class="text-2xl font-extrabold tracking-tight text-white flex items-center space-x-2">
                <span>📝</span>
                <span>Snippets & Bausteine</span>
            </h1>
            <p class="text-xs text-slate-400 mt-1">Verwalte, erstelle und nutze deine wiederverwendbaren Textbausteine</p>
        </div>

        <div class="flex items-center space-x-3 w-full sm:w-auto">
            <div class="relative w-full sm:w-72">
                <input 
                    type="text" 
                    placeholder="Snippets durchsuchen..." 
                    bind:value={searchQuery}
                    oninput={handleSearch}
                    class="w-full pl-9 pr-4 py-2 text-sm bg-slate-900 border border-slate-800 rounded-xl text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all shadow-inner"
                />
                <span class="absolute left-3 top-2.5 text-xs text-slate-500">🔍</span>
            </div>
            <button
                onclick={() => {
                    const current = $snippetFilterStore.isTrashed;
                    snippetFilterStore.update(f => ({ ...f, isTrashed: !current }));
                    loadSnippets();
                }}
                class="px-3 py-2 text-xs font-semibold rounded-xl transition-all border {$snippetFilterStore.isTrashed ? 'bg-rose-950/60 text-rose-300 border-rose-800' : 'bg-slate-900 text-slate-400 border-slate-800 hover:text-slate-200'}"
                title={$snippetFilterStore.isTrashed ? 'Zurück zu aktiven Snippets' : 'Papierkorb anzeigen'}
            >
                🗑️ {$snippetFilterStore.isTrashed ? 'Papierkorb aktiv' : 'Papierkorb'}
            </button>
            <button 
                onclick={createNewSnippet}
                class="px-4 py-2 text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl shadow-lg shadow-indigo-600/20 transition-all shrink-0 flex items-center space-x-1.5"
            >
                <span>+</span>
                <span>Neues Snippet</span>
            </button>
        </div>
    </div>

    <!-- Tag Cloud Bar -->
    {#if $tagCloud.length > 0}
        <div class="flex items-center space-x-2 overflow-x-auto pb-1 scrollbar-none text-xs">
            <span class="text-slate-500 text-[11px] font-medium shrink-0">Tags:</span>
            <button 
                onclick={() => selectTag(null)}
                class="px-2.5 py-1 rounded-lg border font-medium transition-all {$selectedTagStore === null ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200 hover:bg-slate-800'}"
            >
                Alle ({$tagCloud.reduce((a, b) => a + b.count, 0)})
            </button>
            {#each $tagCloud as item}
                <button 
                    onclick={() => selectTag(item.tag)}
                    class="px-2.5 py-1 rounded-lg border font-medium transition-all flex items-center space-x-1 {$selectedTagStore === item.tag ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200 hover:bg-slate-800'}"
                >
                    <span>#{item.tag}</span>
                    <span class="text-[10px] px-1 bg-slate-800/80 rounded-full text-slate-400">{item.count}</span>
                </button>
            {/each}
        </div>
    {/if}

    <!-- Content Grid -->
    <div class="flex-1 grid grid-cols-12 gap-6 overflow-hidden min-h-0">
        <!-- Sidebar Filter (Schritt 1) -->
        <div class="col-span-2 h-full overflow-y-auto bg-slate-900/45 p-4 rounded-2xl border border-slate-900">
            <FilterPanel />
        </div>
        <div class="col-span-3 h-full overflow-hidden flex flex-col">
            <SnippetList />
        </div>
        <div class="col-span-7 h-full overflow-hidden">
            <SnippetEditor />
        </div>
    </div>
</div>

