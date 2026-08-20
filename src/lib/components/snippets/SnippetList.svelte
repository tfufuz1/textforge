<script lang="ts">
    import {
        snippetsStore,
        activeSnippetStore,
        snippetFilterStore,
        selectedSnippetIdsStore,
        selectSnippet,
        handleTrashSnippet,
        togglePinSnippet,
        toggleFavoriteSnippet,
        handleDuplicateSnippet,
        toggleSelectSnippetId,
        clearSnippetSelection,
        selectAllSnippets,
        handleBulkOperation,
        handleRestoreSnippet,
        handleDeleteSnippetPermanently
    } from '../../stores/snippets';

    function formatTime(ms: number) {
        if (!ms) return '';
        const d = new Date(ms);
        return d.toLocaleDateString('de-DE', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
    }

    let isAllSelected = $derived(
        $snippetsStore.length > 0 && $snippetsStore.every(s => $selectedSnippetIdsStore.has(s.id))
    );

    function toggleSelectAll() {
        if (isAllSelected) {
            clearSnippetSelection();
        } else {
            selectAllSnippets($snippetsStore.map(s => s.id));
        }
    }

    async function applyBulkPin(pinned: boolean) {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        await handleBulkOperation({ _type: 'bulk_pin', snippetIds, pinned });
    }

    async function applyBulkFavorite(favorite: boolean) {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        await handleBulkOperation({ _type: 'bulk_favorite', snippetIds, favorite });
    }

    async function applyBulkDelete() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        const permanent = $snippetFilterStore.isTrashed === true;
        await handleBulkOperation({ _type: 'bulk_delete', snippetIds, permanent });
    }

    async function applyBulkTag() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        const tag = prompt("Tag name eingeben:");
        if (!tag || !tag.trim()) return;
        await handleBulkOperation({
            _type: 'bulk_tag',
            snippetIds,
            addTags: [tag.trim()],
            removeTags: []
        });
    }
</script>

<div class="flex flex-col h-full space-y-2 overflow-hidden">
    <!-- Header with Select-All & Bulk Toolbar -->
    {#if $snippetsStore.length > 0}
        <div class="flex items-center justify-between px-2 py-1.5 bg-slate-900/80 rounded-xl border border-slate-800 text-xs">
            <label class="flex items-center space-x-2 cursor-pointer select-none text-slate-400 hover:text-slate-200">
                <input
                    type="checkbox"
                    checked={isAllSelected}
                    onchange={toggleSelectAll}
                    class="rounded border-slate-700 bg-slate-950 text-indigo-600 focus:ring-indigo-500 focus:ring-offset-0"
                />
                <span>Alle auswählen</span>
            </label>

            {#if $selectedSnippetIdsStore.size > 0}
                <div class="flex items-center space-x-1.5">
                    <span class="px-2 py-0.5 text-[10px] bg-indigo-950 text-indigo-300 rounded-lg border border-indigo-800/50 font-mono font-medium">
                        {$selectedSnippetIdsStore.size} ausgewählt
                    </span>
                    <button 
                        onclick={() => applyBulkPin(true)}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors"
                        title="Ausgewählte anheften"
                    >
                        📌
                    </button>
                    <button 
                        onclick={() => applyBulkFavorite(true)}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors"
                        title="Ausgewählte favorisieren"
                    >
                        ⭐
                    </button>
                    <button 
                        onclick={applyBulkTag}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors"
                        title="Tag hinzufügen"
                    >
                        🏷️
                    </button>
                    <button
                        onclick={applyBulkDelete}
                        class="p-1.5 bg-rose-950/80 hover:bg-rose-900 text-rose-300 rounded-lg transition-colors"
                        title="Ausgewählte löschen"
                    >
                        🗑️
                    </button>
                    <button
                        onclick={clearSnippetSelection}
                        class="p-1.5 text-slate-400 hover:text-slate-200 text-xs"
                        title="Auswahl aufheben"
                    >
                        ✕
                    </button>
                </div>
            {/if}
        </div>
    {/if}

    <!-- Snippet List Items -->
    <div class="space-y-2.5 overflow-y-auto flex-1 pr-1.5">
        {#each $snippetsStore as item (item.id)}
            <div
                class="p-3.5 rounded-xl border transition-all duration-150 flex flex-col justify-between cursor-pointer group shadow-sm {$activeSnippetStore?.id === item.id ? 'bg-indigo-950/60 border-indigo-500/60 ring-1 ring-indigo-500/30' : 'bg-slate-900/60 hover:bg-slate-800/80 border-slate-800 hover:border-slate-700'}"
                onclick={() => selectSnippet(item.id)}
                role="button"
                tabindex="0"
                onkeydown={(e) => e.key === 'Enter' && selectSnippet(item.id)}
            >
                <div class="flex items-start justify-between gap-2">
                    <div class="flex items-center space-x-2 min-w-0">
                        <input
                            type="checkbox"
                            checked={$selectedSnippetIdsStore.has(item.id)}
                            onclick={(e) => e.stopPropagation()}
                            onchange={() => toggleSelectSnippetId(item.id)}
                            class="rounded border-slate-700 bg-slate-950 text-indigo-600 focus:ring-indigo-500 focus:ring-offset-0 shrink-0"
                        />
                        <button
                            class="text-xs p-1 rounded hover:bg-slate-800 transition-colors opacity-75 hover:opacity-100 shrink-0"
                            title={item.isPinned ? 'Fixierung aufheben' : 'Anheften'}
                            onclick={(e) => { e.stopPropagation(); togglePinSnippet(item); }}
                        >
                            {item.isPinned ? '📌' : '📍'}
                        </button>
                        {#if item.color}
                            <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {item.color}"></span>
                        {/if}
                        <h3 class="font-semibold text-sm truncate text-slate-100 group-hover:text-indigo-300 transition-colors">{item.title}</h3>
                        <button
                            class="p-0.5 rounded text-xs shrink-0 hover:bg-slate-800"
                            title={item.isFavorite ? 'Favorit entfernen' : 'Als Favorit markieren'}
                            onclick={(e) => { e.stopPropagation(); toggleFavoriteSnippet(item); }}
                        >
                            <span class={item.isFavorite ? 'text-amber-400' : 'text-slate-600 hover:text-amber-400'}>⭐</span>
                        </button>
                    </div>

                    <div class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                        {#if $snippetFilterStore.isTrashed}
                            <button
                                class="p-1 hover:bg-emerald-950/80 text-emerald-400 rounded text-xs"
                                title="Wiederherstellen"
                                onclick={(e) => { e.stopPropagation(); handleRestoreSnippet(item.id); }}
                            >
                                🔄
                            </button>
                            <button
                                class="p-1 hover:bg-rose-950/80 text-rose-400 rounded text-xs"
                                title="Endgültig löschen"
                                onclick={(e) => { e.stopPropagation(); handleDeleteSnippetPermanently(item.id); }}
                            >
                                ❌
                            </button>
                        {:else}
                            <button
                                class="p-1 hover:bg-slate-800 text-slate-400 hover:text-slate-200 rounded text-xs"
                                title="Duplizieren"
                                onclick={(e) => { e.stopPropagation(); handleDuplicateSnippet(item.id); }}
                            >
                                📄
                            </button>
                            <button
                                class="p-1 hover:bg-rose-950/80 text-slate-400 hover:text-rose-400 rounded text-xs"
                                title="In Papierkorb"
                                onclick={(e) => { e.stopPropagation(); handleTrashSnippet(item.id); }}
                            >
                                🗑️
                            </button>
                        {/if}
                    </div>
                </div>

                <p class="text-xs text-slate-400 line-clamp-2 mt-2 font-mono leading-relaxed bg-slate-950/40 p-2 rounded-lg border border-slate-800/50">
                    {item.preview}
                </p>

                <div class="flex items-center justify-between mt-3 text-[10px] text-slate-500">
                    <div class="flex flex-wrap gap-1">
                        <span class="px-1.5 py-0.5 bg-slate-800 text-slate-300 rounded font-mono font-medium">{item.contentType}</span>
                        {#each item.tags as tag}
                            <span class="px-1.5 py-0.5 bg-indigo-950/80 text-indigo-300 rounded border border-indigo-800/40 font-medium">#{tag}</span>
                        {/each}
                    </div>
                    <span class="font-mono text-slate-500 shrink-0 ml-2">{formatTime(item.updatedAt)}</span>
                </div>
            </div>
        {:else}
            <div class="p-8 text-center bg-slate-900/30 rounded-xl border border-slate-800/60 text-slate-500 text-sm space-y-2">
                <div class="text-2xl">📝</div>
                <p>Keine Snippets gefunden.</p>
            </div>
        {/each}
    </div>
</div>
