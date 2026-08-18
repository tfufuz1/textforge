<script lang="ts">
    import type { MouseEventHandler } from 'svelte/elements';
    let { renderedOutput = '', unresolvedVars = [] as string[], warnings = [] as string[], onCopy = undefined as MouseEventHandler<HTMLButtonElement> | undefined } = $props();
</script>

<div class="pt-3 border-t border-slate-850 space-y-2">
    <div class="flex justify-between items-center">
        <div class="flex items-center space-x-2">
            <span class="text-[11px] font-semibold text-slate-400 font-mono">Template Live-Vorschau</span>
            {#if unresolvedVars.length > 0}
                <span class="px-1.5 py-0.5 text-[9px] font-mono font-bold rounded bg-amber-500/20 text-amber-300 border border-amber-500/30" title="Unaufgelöste Variablen">
                    {unresolvedVars.length} offen
                </span>
            {/if}
        </div>
        <button
            onclick={onCopy}
            disabled={!renderedOutput}
            class="px-2.5 py-1 text-[10px] font-semibold bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg transition-all"
        >
            📋 Kopieren
        </button>
    </div>

    <pre class="w-full p-3 font-mono text-xs bg-slate-950 rounded-lg text-slate-200 border border-slate-850 min-h-[4rem] max-h-36 overflow-auto whitespace-pre-wrap leading-relaxed">{renderedOutput}</pre>

    {#if unresolvedVars.length > 0}
        <div class="p-2 rounded-lg bg-slate-950/40 border border-slate-850/80 text-[10px] text-slate-400 font-mono flex items-center gap-1.5 flex-wrap">
            <span class="text-amber-500 font-semibold">Fehlend:</span>
            {#each unresolvedVars as uv}
                <span class="px-1.5 py-0.5 bg-amber-500/10 text-amber-300 border border-amber-500/20 rounded font-mono text-[9px]">
                    {uv}
                </span>
            {/each}
        </div>
    {/if}

    {#if warnings.length > 0}
        <div class="p-2.5 rounded-lg bg-amber-950/20 border border-amber-900/40 text-[10px] text-amber-300 font-mono space-y-1">
            <div class="font-bold">Warnungen:</div>
            {#each warnings as warning}
                <div>• {warning}</div>
            {/each}
        </div>
    {/if}
</div>
