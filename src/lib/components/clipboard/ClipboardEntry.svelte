<script lang="ts">
    import type { ClipboardEntryListItemDto } from '../../ipc/clipboard';
    import ClipboardEntryActions from './ClipboardEntryActions.svelte';

    let { entry }: { entry: ClipboardEntryListItemDto } = $props();

    function formatTime(ms: number) {
        if (!ms) return '';
        return new Date(ms).toLocaleString('de-DE', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
    }

    function formatBytes(bytes: number) {
        if (!bytes) return '0 B';
        if (bytes < 1024) return `${bytes} B`;
        return `${(bytes / 1024).toFixed(1)} KB`;
    }
</script>

<div class="p-4 rounded-2xl border transition-all duration-150 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4 group shadow-sm {entry.isPinned ? 'ring-1 ring-amber-500/40 border-amber-500/50 bg-amber-950/15 shadow-amber-950/20' : 'border-slate-800/80 bg-slate-900/50 hover:bg-slate-900/90 hover:border-slate-700/80'}">
    <div class="min-w-0 flex-1 w-full">
        <p class="font-mono text-xs text-slate-200 line-clamp-3 leading-relaxed bg-slate-950/80 p-3 rounded-xl border border-slate-800/80 select-all group-hover:border-slate-700/60 transition-colors">
            {entry.preview}
        </p>
        <div class="flex flex-wrap items-center gap-2 text-[10px] text-slate-400 mt-2.5">
            {#if entry.isPinned}
                <span class="px-2 py-0.5 bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded-md font-semibold flex items-center space-x-1">
                    <span>📌</span>
                    <span>Gepinnt</span>
                </span>
            {/if}
            <span class="px-2 py-0.5 bg-slate-800 text-slate-300 rounded-md font-mono font-medium border border-slate-700/50">{entry.contentType}</span>
            {#if entry.sourceApp}
                <span class="px-2 py-0.5 bg-slate-800/60 text-slate-400 rounded-md border border-slate-800 flex items-center space-x-1">
                    <span>📱</span>
                    <span>{entry.sourceApp}</span>
                </span>
            {/if}
            <span class="font-mono text-slate-500 bg-slate-950/50 px-2 py-0.5 rounded border border-slate-800/50">{formatBytes(entry.sizeBytes)}</span>
            <span class="font-mono text-slate-500 bg-slate-950/50 px-2 py-0.5 rounded border border-slate-800/50">{formatTime(entry.capturedAt)}</span>
        </div>
    </div>
    <div class="w-full sm:w-auto shrink-0 pt-2 sm:pt-0 border-t sm:border-t-0 border-slate-800/60 flex justify-end">
        <ClipboardEntryActions {entry} />
    </div>
</div>
