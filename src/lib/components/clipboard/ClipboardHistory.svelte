<script lang="ts">
    import { onMount } from 'svelte';
    import {
        loadClipboardHistory,
        filteredClipboard,
        clipboardPageStore,
        clipboardTotalStore,
        clipboardHasNextStore,
        clipboardHasPrevStore
    } from '../../stores/clipboard';
    import { clearHistory, deleteEntry, pinEntry, promoteToSnippet, getClipboardEntry, writeToClipboard } from '../../ipc/clipboard';
    import ClipboardEntry from './ClipboardEntry.svelte';
    import ClipboardFilter from './ClipboardFilter.svelte';

    // SVG Icons
    import TrashIcon from '$lib/components/icons/TrashIcon.svelte';
    import PinIcon from '$lib/components/icons/PinIcon.svelte';
    import PlusIcon from '$lib/components/icons/PlusIcon.svelte';
    import CopyIcon from '$lib/components/icons/CopyIcon.svelte';
    import ClipboardIcon from '$lib/components/icons/ClipboardIcon.svelte';
    import CheckIcon from '$lib/components/icons/CheckIcon.svelte';
    import { pushNotification, Notifications } from '../../stores/notifications';

    let selectedIds = $state<Set<string>>(new Set());

    onMount(async () => {
        await loadClipboardHistory(0);
    });

    function toggleSelect(id: string) {
        const next = new Set(selectedIds);
        if (next.has(id)) {
            next.delete(id);
        } else {
            next.add(id);
        }
        selectedIds = next;
    }

    function toggleSelectAll() {
        if (selectedIds.size === $filteredClipboard.length) {
            selectedIds = new Set();
        } else {
            selectedIds = new Set($filteredClipboard.map(e => e.id));
        }
    }

    async function handleClearHistory() {
        if (confirm("Möchten Sie den nicht gepinnten Verlauf wirklich leeren? Gepinnte Einträge bleiben erhalten.")) {
            await clearHistory(true);
            selectedIds = new Set();
            await loadClipboardHistory(0);
        }
    }

    async function handleBulkDelete() {
        if (selectedIds.size === 0) return;
        if (confirm(`Möchten Sie ${selectedIds.size} ausgewählte Einträge löschen?`)) {
            for (const id of selectedIds) {
                await deleteEntry(id);
            }
            selectedIds = new Set();
            await loadClipboardHistory();
        }
    }

    async function handleBulkPin(pinState: boolean) {
        if (selectedIds.size === 0) return;
        for (const id of selectedIds) {
            await pinEntry(id, pinState);
        }
        selectedIds = new Set();
        await loadClipboardHistory();
    }

    async function handleBulkPromote() {
        if (selectedIds.size === 0) return;
        let count = 0;
        for (const id of selectedIds) {
            try {
                await promoteToSnippet(id, null, { _type: 'inbox', folderId: null });
                count++;
            } catch (e) {
                console.error("Failed to promote entry:", id, e);
            }
        }
        selectedIds = new Set();
        await loadClipboardHistory();
        pushNotification(Notifications.snippetSaved(`${count} Snippets importiert`));
    }

    async function handleBulkCopyCombined() {
        if (selectedIds.size === 0) return;
        const contents: string[] = [];
        for (const id of selectedIds) {
            try {
                const detail = await getClipboardEntry(id);
                if (detail.content) contents.push(detail.content);
            } catch (e) {
                console.error(e);
            }
        }
        if (contents.length > 0) {
            await writeToClipboard(contents.join('\n\n---\n\n'));
            pushNotification(Notifications.snippetCopied());
        }
    }

    async function changePage(newPage: number) {
        selectedIds = new Set();
        await loadClipboardHistory(newPage);
    }

    let allSelected = $derived($filteredClipboard.length > 0 && selectedIds.size === $filteredClipboard.length);
</script>

<div class="h-full flex flex-col space-y-3 min-h-0">
    <div class="flex items-center justify-between gap-4 bg-slate-900/60 p-3 rounded-2xl border border-slate-800/80">
        <div class="flex-1">
            <ClipboardFilter />
        </div>
        <button
            onclick={handleClearHistory}
            class="px-3.5 py-2 text-xs font-semibold bg-rose-950/40 hover:bg-rose-900/60 text-rose-300 border border-rose-800/60 hover:border-rose-700 rounded-xl transition-all shrink-0 flex items-center space-x-1.5 shadow-sm"
            title="Nicht gepinnte Verlaufseinträge leeren"
        >
            <TrashIcon class="w-3.5 h-3.5 text-rose-400" />
            <span>Verlauf leeren</span>
        </button>
    </div>

    <!-- Bulk Action Toolbar -->
    {#if $filteredClipboard.length > 0}
        <div class="flex flex-wrap items-center justify-between gap-2 px-3 py-2 bg-slate-900/40 rounded-xl border border-slate-800/60 text-xs">
            <div class="flex items-center space-x-2">
                <input
                    type="checkbox"
                    checked={allSelected}
                    onchange={toggleSelectAll}
                    class="w-4 h-4 rounded bg-slate-950 border-slate-700 text-indigo-600 focus:ring-indigo-500 cursor-pointer"
                />
                <span class="text-slate-400 font-medium">
                    {#if selectedIds.size > 0}
                        <strong class="text-indigo-300 font-mono">{selectedIds.size}</strong> von {$filteredClipboard.length} ausgewählt
                    {:else}
                        Alle auswählen ({$filteredClipboard.length})
                    {/if}
                </span>
            </div>

            {#if selectedIds.size > 0}
                <div class="flex items-center gap-1.5">
                    <button
                        onclick={handleBulkCopyCombined}
                        class="px-2.5 py-1 bg-slate-800 hover:bg-slate-700 text-slate-200 border border-slate-700 rounded-lg font-semibold flex items-center space-x-1 transition-all"
                        title="Ausgewählte kombiniert kopieren"
                    >
                        <CopyIcon class="w-3.5 h-3.5" />
                        <span>Kombiniert kopieren</span>
                    </button>
                    <button
                        onclick={handleBulkPromote}
                        class="px-2.5 py-1 bg-indigo-950 hover:bg-indigo-900 text-indigo-300 border border-indigo-700/50 rounded-lg font-semibold flex items-center space-x-1 transition-all"
                    >
                        <PlusIcon class="w-3.5 h-3.5 text-indigo-400" />
                        <span>Zu Snippets ({selectedIds.size})</span>
                    </button>
                    <button
                        onclick={() => handleBulkPin(true)}
                        class="px-2.5 py-1 bg-amber-500/20 hover:bg-amber-500/30 text-amber-300 border border-amber-500/40 rounded-lg font-semibold flex items-center space-x-1 transition-all"
                    >
                        <PinIcon class="w-3.5 h-3.5 text-amber-400" filled={true} />
                        <span>Pinnen</span>
                    </button>
                    <button
                        onclick={handleBulkDelete}
                        class="px-2.5 py-1 bg-rose-950/40 hover:bg-rose-900/60 text-rose-300 border border-rose-800/50 rounded-lg font-semibold flex items-center space-x-1 transition-all"
                    >
                        <TrashIcon class="w-3.5 h-3.5 text-rose-400" />
                        <span>Löschen ({selectedIds.size})</span>
                    </button>
                </div>
            {/if}
        </div>
    {/if}

    <div class="flex-1 overflow-y-auto space-y-2.5 pr-1 min-h-0 custom-scrollbar">
        {#each $filteredClipboard as entry (entry.id)}
            <ClipboardEntry
                {entry}
                isSelected={selectedIds.has(entry.id)}
                onToggleSelect={toggleSelect}
            />
        {:else}
            <div class="py-16 px-6 text-center bg-slate-900/40 rounded-2xl border border-dashed border-slate-800/80 text-slate-400 space-y-3 my-auto">
                <div class="w-14 h-14 mx-auto rounded-2xl bg-indigo-950/50 border border-indigo-800/40 flex items-center justify-center text-indigo-400 shadow-lg shadow-indigo-950/20">
                    <ClipboardIcon class="w-7 h-7" />
                </div>
                <div>
                    <h3 class="text-sm font-bold text-slate-200">Keine Zwischenablagen-Einträge</h3>
                    <p class="text-xs text-slate-500 mt-1 max-w-sm mx-auto">Kopiere Text in einer beliebigen Anwendung — TextForge erfasst ihn automatisch auf Wayland & X11.</p>
                </div>
                <div class="inline-flex items-center space-x-2 px-3 py-1.5 rounded-lg bg-slate-900 text-[11px] font-mono text-indigo-300 border border-slate-800">
                    <span>💡 Tipp: Drücke Strg+C zum Kopieren</span>
                </div>
            </div>
        {/each}
    </div>

    <div class="pt-2.5 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-400 bg-slate-950/40 px-1">
        <span class="font-medium text-slate-400">Gesamt: <strong class="text-slate-200 font-mono">{$clipboardTotalStore}</strong> Einträge</span>
        <div class="flex items-center space-x-2">
            <button
                disabled={!$clipboardHasPrevStore}
                onclick={() => changePage($clipboardPageStore - 1)}
                class="px-3 py-1 bg-slate-900 hover:bg-slate-800 border border-slate-800 disabled:opacity-30 disabled:hover:bg-slate-900 text-slate-200 rounded-xl transition-all text-xs font-semibold"
            >
                ← Zurück
            </button>
            <span class="font-mono text-[11px] px-2 py-0.5 rounded bg-slate-900 border border-slate-800/80 text-slate-400">
                Seite {$clipboardPageStore + 1}
            </span>
            <button
                disabled={!$clipboardHasNextStore}
                onclick={() => changePage($clipboardPageStore + 1)}
                class="px-3 py-1 bg-slate-900 hover:bg-slate-800 border border-slate-800 disabled:opacity-30 disabled:hover:bg-slate-900 text-slate-200 rounded-xl transition-all text-xs font-semibold"
            >
                Weiter →
            </button>
        </div>
    </div>
</div>
