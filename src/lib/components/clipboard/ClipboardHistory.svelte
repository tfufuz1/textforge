<script lang="ts">
    import { onMount } from 'svelte';
    import { loadClipboardHistory, filteredClipboard } from '../../stores/clipboard';
    import ClipboardEntryActions from './ClipboardEntryActions.svelte';
    import ClipboardFilter from './ClipboardFilter.svelte';

    onMount(async () => {
        await loadClipboardHistory();
    });

    function formatTime(ms: number) {
        if (!ms) return '';
        return new Date(ms).toLocaleString('de-DE', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
    }
</script>

<div class="h-full flex flex-col space-y-4 min-h-0">
    <ClipboardFilter />
    <div class="flex-1 overflow-y-auto space-y-3 pr-1 min-h-0">
        {#each $filteredClipboard as entry (entry.id)}
            <div class="p-4 rounded-xl border border-slate-800 bg-slate-950/60 hover:bg-slate-900/80 transition-all flex items-start justify-between gap-4 group shadow-sm">
                <div class="min-w-0 flex-1">
                    <p class="font-mono text-xs text-slate-200 line-clamp-3 leading-relaxed bg-slate-900/90 p-2.5 rounded-lg border border-slate-800/80">
                        {entry.preview}
                    </p>
                    <div class="flex flex-wrap items-center gap-2 text-[10px] text-slate-400 mt-2.5">
                        <span class="px-2 py-0.5 bg-slate-800 text-slate-300 rounded font-mono font-medium">{entry.contentType}</span>
                        {#if entry.sourceApp}
                            <span class="px-2 py-0.5 bg-slate-800/60 text-slate-400 rounded">📱 {entry.sourceApp}</span>
                        {/if}
                        <span class="font-mono text-slate-500">{formatTime(entry.capturedAt)}</span>
                    </div>
                </div>
                <ClipboardEntryActions {entry} />
            </div>
        {:else}
            <div class="p-12 text-center bg-slate-950/40 rounded-xl border border-slate-800/60 text-slate-500 text-sm space-y-2">
                <div class="text-3xl">📋</div>
                <p>Keine Clipboard-Einträge gefunden.</p>
            </div>
        {/each}
    </div>
</div>

