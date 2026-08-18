<script lang="ts">
    import { snippetsStore, activeSnippetStore, selectSnippet, handleTrashSnippet, togglePinSnippet, handleDuplicateSnippet } from '../../stores/snippets';

    function formatTime(ms: number) {
        if (!ms) return '';
        const d = new Date(ms);
        return d.toLocaleDateString('de-DE', { day: '2-digit', month: '2-digit', hour: '2-digit', minute: '2-digit' });
    }
</script>

<div class="space-y-2.5 overflow-y-auto max-h-full pr-1.5">
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
                    <button 
                        class="text-xs p-1 rounded hover:bg-slate-800 transition-colors opacity-75 hover:opacity-100"
                        title={item.isPinned ? 'Fixierung aufheben' : 'Anheften'}
                        onclick={(e) => { e.stopPropagation(); togglePinSnippet(item); }}
                    >
                        {item.isPinned ? '📌' : '📍'}
                    </button>
                    {#if item.color}
                        <span class="w-2 h-2 rounded-full shrink-0" style="background-color: {item.color}"></span>
                    {/if}
                    <h3 class="font-semibold text-sm truncate text-slate-100 group-hover:text-indigo-300 transition-colors">{item.title}</h3>
                    {#if item.isFavorite}
                        <span class="text-xs text-amber-400 shrink-0">⭐</span>
                    {/if}
                </div>

                <div class="flex items-center space-x-1 opacity-0 group-hover:opacity-100 transition-opacity">
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

