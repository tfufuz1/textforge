<script lang="ts">
    import { onMount } from 'svelte';
    import { listen, type UnlistenFn } from '@tauri-apps/api/event';
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
        handleDuplicateSnippetsBulk,
        toggleSelectSnippetId,
        clearSnippetSelection,
        selectAllSnippets,
        handleBulkOperation,
        handleRestoreSnippet,
        handleDeleteSnippetPermanently
    } from '../../stores/snippets';

    interface BulkProgressPayload {
        completed: number;
        total: number;
        currentId: string;
    }

    let isBulkProcessing = $state(false);
    let bulkProgress = $state<BulkProgressPayload>({ completed: 0, total: 0, currentId: '' });

    onMount(() => {
        let unlisten: UnlistenFn | undefined;
        listen<BulkProgressPayload>('bulk:progress', (event) => {
            bulkProgress = event.payload;
            isBulkProcessing = true;
            if (event.payload.completed >= event.payload.total) {
                // Auto reset when done
                setTimeout(() => {
                    isBulkProcessing = false;
                }, 800);
            }
        }).then((fn) => {
            unlisten = fn;
        }).catch((err) => {
            console.error('Failed to listen to bulk:progress event:', err);
        });

        return () => {
            if (unlisten) unlisten();
        };
    });

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

    function createNewSnippet() {
        activeSnippetStore.set(null);
    }

    async function applyBulkPin(pinned: boolean) {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        try {
            await handleBulkOperation({ _type: 'bulk_pin', snippetIds, pinned });
        } finally {
            isBulkProcessing = false;
        }
    }

    async function applyBulkFavorite(favorite: boolean) {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        try {
            await handleBulkOperation({ _type: 'bulk_favorite', snippetIds, favorite });
        } finally {
            isBulkProcessing = false;
        }
    }

    async function applyBulkDuplicate() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        try {
            await handleDuplicateSnippetsBulk(snippetIds);
        } finally {
            isBulkProcessing = false;
        }
    }

    async function applyBulkDelete() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        const permanent = $snippetFilterStore.isTrashed === true;
        try {
            await handleBulkOperation({ _type: 'bulk_delete', snippetIds, permanent });
        } finally {
            isBulkProcessing = false;
        }
    }

    async function applyBulkTag() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        const tag = prompt("Tag name eingeben:");
        if (!tag || !tag.trim()) return;
        try {
            await handleBulkOperation({
                _type: 'bulk_tag',
                snippetIds,
                addTags: [tag.trim()],
                removeTags: []
            });
        } finally {
            isBulkProcessing = false;
        }
    }

    async function applyBulkTransform() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        const pipelineId = prompt("Pipeline-ID eingeben:");
        if (!pipelineId || !pipelineId.trim()) return;
        try {
            await handleBulkOperation({
                _type: 'bulk_transform',
                snippetIds,
                pipelineId: pipelineId.trim(),
                saveResults: true
            });
        } finally {
            isBulkProcessing = false;
        }
    }

    async function applyBulkExport() {
        const snippetIds = Array.from($selectedSnippetIdsStore);
        if (snippetIds.length === 0) return;
        const outputPath = prompt("Zielpfad für Export eingeben:", "bulk_export.json");
        if (!outputPath || !outputPath.trim()) return;
        try {
            await handleBulkOperation({
                _type: 'bulk_export',
                snippetIds,
                format: 'json',
                outputPath: outputPath.trim()
            });
        } finally {
            isBulkProcessing = false;
        }
    }
</script>

<div class="flex flex-col h-full space-y-2.5 overflow-hidden">
    <!-- Progress Bar Banner -->
    {#if isBulkProcessing && bulkProgress.total > 0}
        <div class="bg-indigo-950/90 border border-indigo-700/70 p-2.5 rounded-xl shadow-md text-xs space-y-1.5 animate-pulse">
            <div class="flex items-center justify-between font-mono text-[11px] text-indigo-200">
                <span class="font-semibold">Bulk-Operation läuft...</span>
                <span class="font-bold text-indigo-300">{bulkProgress.completed} von {bulkProgress.total} verarbeitet</span>
            </div>
            <div class="w-full bg-slate-950 rounded-full h-2 overflow-hidden border border-indigo-900/50">
                <div
                    class="bg-gradient-to-r from-indigo-500 to-indigo-400 h-full transition-all duration-200"
                    style="width: {Math.min(100, (bulkProgress.completed / Math.max(1, bulkProgress.total)) * 100)}%"
                ></div>
            </div>
        </div>
    {/if}

    <!-- Header with Select-All & Bulk Toolbar -->
    {#if $snippetsStore.length > 0}
        <div class="flex items-center justify-between px-3 py-2 bg-slate-900/90 rounded-2xl border border-slate-800 text-xs shadow-sm">
            <label class="flex items-center space-x-2 cursor-pointer select-none text-slate-400 hover:text-slate-200">
                <input
                    type="checkbox"
                    checked={isAllSelected}
                    onchange={toggleSelectAll}
                    class="rounded border-slate-700 bg-slate-950 text-indigo-600 focus:ring-indigo-500 focus:ring-offset-0"
                />
                <span class="font-medium text-[11px]">Alle auswählen ({$snippetsStore.length})</span>
            </label>

            {#if $selectedSnippetIdsStore.size > 0}
                <div class="flex items-center space-x-1.5">
                    <span class="px-2 py-0.5 text-[10px] bg-indigo-950 text-indigo-300 rounded-lg border border-indigo-800/50 font-mono font-bold">
                        {$selectedSnippetIdsStore.size} aktiv
                    </span>
                    <button 
                        onclick={() => applyBulkPin(true)}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors text-xs"
                        title="Ausgewählte anheften"
                    >
                        📌
                    </button>
                    <button 
                        onclick={() => applyBulkFavorite(true)}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors text-xs"
                        title="Ausgewählte favorisieren"
                    >
                        ⭐
                    </button>
                    <button
                        onclick={applyBulkDuplicate}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors text-xs"
                        title="Ausgewählte duplizieren"
                    >
                        📄
                    </button>
                    <button 
                        onclick={applyBulkTag}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors text-xs"
                        title="Tag hinzufügen"
                    >
                        🏷️
                    </button>
                    <button
                        onclick={applyBulkTransform}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors text-xs"
                        title="Pipeline anwenden (Bulk Transform)"
                    >
                        ⚡
                    </button>
                    <button
                        onclick={applyBulkExport}
                        class="p-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors text-xs"
                        title="Ausgewählte exportieren (Bulk Export)"
                    >
                        📥
                    </button>
                    <button
                        onclick={applyBulkDelete}
                        class="p-1.5 bg-rose-950/80 hover:bg-rose-900 text-rose-300 rounded-lg transition-colors text-xs"
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
    <div class="space-y-2.5 overflow-y-auto flex-1 pr-1.5 custom-scrollbar">
        {#each $snippetsStore as item (item.id)}
            <div
                class="p-3.5 rounded-2xl border transition-all duration-150 flex flex-col justify-between cursor-pointer group shadow-sm {$activeSnippetStore?.id === item.id ? 'bg-indigo-950/70 border-indigo-500/70 ring-1 ring-indigo-500/30 shadow-indigo-950/40' : 'bg-slate-900/50 hover:bg-slate-900/90 border-slate-800/80 hover:border-slate-700'}"
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
                            class="text-xs p-1 rounded-md hover:bg-slate-800 transition-colors opacity-75 hover:opacity-100 shrink-0"
                            title={item.isPinned ? 'Fixierung aufheben' : 'Anheften'}
                            onclick={(e) => { e.stopPropagation(); togglePinSnippet(item); }}
                        >
                            {item.isPinned ? '📌' : '📍'}
                        </button>
                        {#if item.color}
                            <span class="w-2.5 h-2.5 rounded-full shrink-0 shadow-sm" style="background-color: {item.color}"></span>
                        {/if}
                        <h3 class="font-bold text-xs truncate text-slate-100 group-hover:text-indigo-300 transition-colors">{item.title}</h3>
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

                <p class="text-xs text-slate-400 line-clamp-2 mt-2 font-mono leading-relaxed bg-slate-950/60 p-2.5 rounded-xl border border-slate-800/60">
                    {item.preview}
                </p>

                <div class="flex items-center justify-between mt-3 text-[10px] text-slate-500">
                    <div class="flex flex-wrap gap-1">
                        <span class="px-1.5 py-0.5 bg-slate-800/80 text-slate-300 rounded-md font-mono font-medium border border-slate-700/50">{item.contentType}</span>
                        {#each item.tags as tag}
                            <span class="px-1.5 py-0.5 bg-indigo-950/80 text-indigo-300 rounded-md border border-indigo-800/40 font-semibold">#{tag}</span>
                        {/each}
                    </div>
                    <span class="font-mono text-slate-500 shrink-0 ml-2">{formatTime(item.updatedAt)}</span>
                </div>
            </div>
        {:else}
            <div class="py-12 px-4 text-center bg-slate-900/40 rounded-2xl border border-dashed border-slate-800/80 text-slate-400 space-y-3">
                <div class="w-12 h-12 mx-auto rounded-2xl bg-indigo-950/50 border border-indigo-800/40 flex items-center justify-center text-xl text-indigo-400 shadow-md">
                    📝
                </div>
                <div>
                    <h3 class="text-xs font-bold text-slate-200">Keine Snippets vorhanden</h3>
                    <p class="text-[11px] text-slate-500 mt-0.5">Erstelle dein erstes wiederverwendbares Text-Snippet.</p>
                </div>
                <button
                    onclick={createNewSnippet}
                    class="px-3.5 py-1.5 text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl shadow-lg shadow-indigo-600/20 transition-all inline-flex items-center space-x-1"
                >
                    <span>+ Neues Snippet</span>
                </button>
            </div>
        {/each}
    </div>
</div>
