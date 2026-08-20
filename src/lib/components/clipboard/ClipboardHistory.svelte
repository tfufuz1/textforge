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
    import { clearHistory } from '../../ipc/clipboard';
    import ClipboardEntry from './ClipboardEntry.svelte';
    import ClipboardFilter from './ClipboardFilter.svelte';

    onMount(async () => {
        await loadClipboardHistory(0);
    });

    async function handleClearHistory() {
        if (confirm("Möchten Sie den nicht gepinnten Verlauf wirklich leeren?")) {
            await clearHistory(true);
            await loadClipboardHistory(0);
        }
    }

    async function changePage(newPage: number) {
        await loadClipboardHistory(newPage);
    }
</script>

<div class="h-full flex flex-col space-y-4 min-h-0">
    <div class="flex items-center justify-between gap-4">
        <ClipboardFilter />
        <button
            onclick={handleClearHistory}
            class="px-3 py-2 text-xs font-medium bg-rose-950/60 hover:bg-rose-900/80 text-rose-300 border border-rose-800/50 rounded-xl transition-all shrink-0 flex items-center gap-1.5"
            title="Verlauf leeren (gepinnte Einträge bleiben erhalten)"
        >
            <span>🗑️</span>
            <span>Verlauf leeren</span>
        </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-3 pr-1 min-h-0">
        {#each $filteredClipboard as entry (entry.id)}
            <ClipboardEntry {entry} />
        {:else}
            <div class="p-12 text-center bg-slate-950/40 rounded-xl border border-slate-800/60 text-slate-500 text-sm space-y-2">
                <div class="text-3xl">📋</div>
                <p>Keine Clipboard-Einträge gefunden.</p>
            </div>
        {/each}
    </div>

    <div class="pt-2 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-400">
        <span>Gesamt: {$clipboardTotalStore} Einträge</span>
        <div class="flex items-center space-x-2">
            <button
                disabled={!$clipboardHasPrevStore}
                onclick={() => changePage($clipboardPageStore - 1)}
                class="px-3 py-1 bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:hover:bg-slate-800 text-slate-200 rounded-lg transition-all"
            >
                ← Zurück
            </button>
            <span class="font-mono text-slate-500">Seite {$clipboardPageStore + 1}</span>
            <button
                disabled={!$clipboardHasNextStore}
                onclick={() => changePage($clipboardPageStore + 1)}
                class="px-3 py-1 bg-slate-800 hover:bg-slate-700 disabled:opacity-40 disabled:hover:bg-slate-800 text-slate-200 rounded-lg transition-all"
            >
                Weiter →
            </button>
        </div>
    </div>
</div>
