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
        if (confirm("Möchten Sie den nicht gepinnten Verlauf wirklich leeren? Gepinnte Einträge bleiben erhalten.")) {
            await clearHistory(true);
            await loadClipboardHistory(0);
        }
    }

    async function changePage(newPage: number) {
        await loadClipboardHistory(newPage);
    }
</script>

<div class="h-full flex flex-col space-y-4 min-h-0">
    <div class="flex items-center justify-between gap-4 bg-slate-900/60 p-3 rounded-2xl border border-slate-800/80">
        <div class="flex-1">
            <ClipboardFilter />
        </div>
        <button
            onclick={handleClearHistory}
            class="px-3.5 py-2 text-xs font-semibold bg-rose-950/40 hover:bg-rose-900/60 text-rose-300 border border-rose-800/60 hover:border-rose-700 rounded-xl transition-all shrink-0 flex items-center space-x-1.5 shadow-sm"
            title="Nicht gepinnte Verlaufseinträge leeren"
        >
            <span>🗑️</span>
            <span>Verlauf leeren</span>
        </button>
    </div>

    <div class="flex-1 overflow-y-auto space-y-3 pr-1 min-h-0 custom-scrollbar">
        {#each $filteredClipboard as entry (entry.id)}
            <ClipboardEntry {entry} />
        {:else}
            <div class="py-16 px-6 text-center bg-slate-900/40 rounded-2xl border border-dashed border-slate-800/80 text-slate-400 space-y-3 my-auto">
                <div class="w-14 h-14 mx-auto rounded-2xl bg-indigo-950/50 border border-indigo-800/40 flex items-center justify-center text-2xl text-indigo-400 shadow-lg shadow-indigo-950/20">
                    📋
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

    <div class="pt-3 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-400 bg-slate-950/40 px-1">
        <span class="font-medium text-slate-400">Gesamt: <strong class="text-slate-200 font-mono">{$clipboardTotalStore}</strong> Einträge</span>
        <div class="flex items-center space-x-2">
            <button
                disabled={!$clipboardHasPrevStore}
                onclick={() => changePage($clipboardPageStore - 1)}
                class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 disabled:opacity-30 disabled:hover:bg-slate-900 text-slate-200 rounded-xl transition-all text-xs font-semibold"
            >
                ← Zurück
            </button>
            <span class="font-mono text-[11px] px-2 py-1 rounded bg-slate-900 border border-slate-800/80 text-slate-400">
                Seite {$clipboardPageStore + 1}
            </span>
            <button
                disabled={!$clipboardHasNextStore}
                onclick={() => changePage($clipboardPageStore + 1)}
                class="px-3 py-1.5 bg-slate-900 hover:bg-slate-800 border border-slate-800 disabled:opacity-30 disabled:hover:bg-slate-900 text-slate-200 rounded-xl transition-all text-xs font-semibold"
            >
                Weiter →
            </button>
        </div>
    </div>
</div>
