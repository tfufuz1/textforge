<script lang="ts">
    import type { ClipboardEntryListItemDto } from '../../ipc/clipboard';
    import ClipboardEntryActions from './ClipboardEntryActions.svelte';
    import PinIcon from '$lib/components/icons/PinIcon.svelte';
    import ChevronDownIcon from '$lib/components/icons/ChevronDownIcon.svelte';
    import ChevronRightIcon from '$lib/components/icons/ChevronRightIcon.svelte';

    interface Props {
        entry: ClipboardEntryListItemDto;
        isSelected?: boolean;
        onToggleSelect?: (id: string) => void;
    }

    let { entry, isSelected = false, onToggleSelect }: Props = $props();

    let isExpanded = $state(false);

    function formatTime(ms: number) {
        if (!ms) return '';
        return new Date(ms).toLocaleString('de-DE', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
    }

    function formatBytes(bytes: number) {
        if (!bytes) return '0 B';
        if (bytes < 1024) return `${bytes} B`;
        return `${(bytes / 1024).toFixed(1)} KB`;
    }

    let isLongContent = $derived(entry.preview && entry.preview.length > 120);
</script>

<div class="p-3.5 rounded-2xl border transition-all duration-150 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 group shadow-sm {entry.isPinned ? 'ring-1 ring-amber-500/40 border-amber-500/50 bg-amber-950/15 shadow-amber-950/20' : 'border-slate-800/80 bg-slate-900/50 hover:bg-slate-900/90 hover:border-slate-700/80'} {isSelected ? 'bg-indigo-950/30 border-indigo-500/60 ring-1 ring-indigo-500/40' : ''}">

    {#if onToggleSelect}
        <div class="pt-1 sm:pt-0 shrink-0">
            <input
                type="checkbox"
                checked={isSelected}
                onchange={() => onToggleSelect && onToggleSelect(entry.id)}
                class="w-4 h-4 rounded bg-slate-950 border-slate-700 text-indigo-600 focus:ring-indigo-500 focus:ring-offset-slate-900 cursor-pointer"
            />
        </div>
    {/if}

    <div class="min-w-0 flex-1 w-full space-y-2">
        {#if entry.contentType === 'image/png' && entry.preview.startsWith('data:image')}
            <div class="bg-slate-950/80 p-2 rounded-xl border border-slate-800/80 flex items-center justify-center max-h-36 overflow-hidden">
                <img src={entry.preview} alt="Clipboard Thumbnail" class="max-h-32 object-contain rounded" />
            </div>
        {:else}
            <div class="relative group/content">
                <p class="font-mono text-xs text-slate-200 leading-relaxed bg-slate-950/80 p-2.5 rounded-xl border border-slate-800/80 select-all group-hover/content:border-slate-700/60 transition-colors whitespace-pre-wrap break-words {isExpanded ? '' : 'line-clamp-2'}">
                    {entry.preview}
                </p>
                {#if isLongContent}
                    <button
                        onclick={() => isExpanded = !isExpanded}
                        class="mt-1 text-[11px] text-indigo-400 hover:text-indigo-300 font-semibold flex items-center space-x-1 transition-colors"
                    >
                        {#if isExpanded}
                            <ChevronDownIcon class="w-3.5 h-3.5" />
                            <span>Weniger anzeigen</span>
                        {:else}
                            <ChevronRightIcon class="w-3.5 h-3.5" />
                            <span>Mehr anzeigen</span>
                        {/if}
                    </button>
                {/if}
            </div>
        {/if}

        <div class="flex flex-wrap items-center gap-2 text-[10px] text-slate-400">
            {#if entry.isPinned}
                <span class="px-2 py-0.5 bg-amber-500/20 text-amber-300 border border-amber-500/30 rounded-md font-semibold flex items-center space-x-1">
                    <PinIcon class="w-3 h-3 text-amber-400" filled={true} />
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
