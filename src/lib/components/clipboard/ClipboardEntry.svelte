<script lang="ts">
    import type { ClipboardEntryListItemDto } from '../../ipc/clipboard';
    import ClipboardEntryActions from './ClipboardEntryActions.svelte';

    let { entry }: { entry: ClipboardEntryListItemDto } = $props();

    function formatTime(ms: number) {
        if (!ms) return '';
        return new Date(ms).toLocaleString('de-DE', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
    }

    function formatBytes(bytes: number) {
        if (bytes < 1024) return `${bytes} B`;
        return `${(bytes / 1024).toFixed(1)} KB`;
    }
</script>

<div class="p-4 rounded-xl border border-slate-800 bg-slate-950/60 hover:bg-slate-900/80 transition-all flex items-start justify-between gap-4 group shadow-sm {entry.isPinned ? 'ring-1 ring-amber-500/30 border-amber-500/40 bg-amber-950/10' : ''}">
    <div class="min-w-0 flex-1">
        <p class="font-mono text-xs text-slate-200 line-clamp-3 leading-relaxed bg-slate-900/90 p-2.5 rounded-lg border border-slate-800/80 select-all">
            {entry.preview}
        </p>
        <div class="flex flex-wrap items-center gap-2 text-[10px] text-slate-400 mt-2.5">
            {#if entry.isPinned}
                <span class="px-2 py-0.5 bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded font-medium">📌 Pinned</span>
            {/if}
            <span class="px-2 py-0.5 bg-slate-800 text-slate-300 rounded font-mono font-medium">{entry.contentType}</span>
            {#if entry.sourceApp}
                <span class="px-2 py-0.5 bg-slate-800/60 text-slate-400 rounded">📱 {entry.sourceApp}</span>
            {/if}
            <span class="font-mono text-slate-500">{formatBytes(entry.sizeBytes)}</span>
            <span class="font-mono text-slate-500">{formatTime(entry.capturedAt)}</span>
        </div>
    </div>
    <ClipboardEntryActions {entry} />
</div>
