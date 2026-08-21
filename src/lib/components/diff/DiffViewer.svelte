<script lang="ts">
    import { computeDiff } from '../../ipc/diff';

    let { original = '', modified = '' } = $props();

    let diffResult = $state<any>(null);
    let viewMode = $state<'unified' | 'split' | 'inline'>('unified');

    $effect(() => {
        if (original !== undefined && modified !== undefined) {
            loadDiff();
        }
    });

    async function loadDiff() {
        try {
            diffResult = await computeDiff(original, modified);
        } catch (e) {
            console.error("Failed to compute diff:", e);
        }
    }

    // Build side-by-side lines for split view
    let splitLines = $derived.by(() => {
        if (!diffResult) return [];
        const left: any[] = [];
        const right: any[] = [];
        
        let i = 0;
        const lines = diffResult.lines;
        while (i < lines.length) {
            const line = lines[i];
            if (line.kind === 'delete') {
                if (i + 1 < lines.length && lines[i + 1].kind === 'insert') {
                    left.push(line);
                    right.push(lines[i + 1]);
                    i += 2;
                } else {
                    left.push(line);
                    right.push({ kind: 'empty', content: '' });
                    i += 1;
                }
            } else if (line.kind === 'insert') {
                left.push({ kind: 'empty', content: '' });
                right.push(line);
                i += 1;
            } else {
                left.push(line);
                right.push(line);
                i += 1;
            }
        }
        return left.map((l, idx) => ({ left: l, right: right[idx] }));
    });
</script>

{#if diffResult}
    <div class="space-y-3 bg-slate-950/85 p-4 rounded-xl border border-slate-800/80">
        <!-- Stats and Toggle Header -->
        <div class="flex justify-between items-center text-xs font-mono text-slate-400">
            <div class="flex items-center space-x-3">
                <span class="font-bold text-slate-300">Diff-Vergleich</span>
                <span class="text-rose-400">-{diffResult.deletedLines} Zeilen</span>
                <span class="text-emerald-400">+{diffResult.addedLines} Zeilen</span>
                {#if diffResult.similarity !== undefined}
                    <span class="text-indigo-400">Ähnlichkeit: {(diffResult.similarity * 100).toFixed(0)}%</span>
                {/if}
            </div>
            
            <div class="flex bg-slate-900 rounded-lg p-0.5 border border-slate-800 space-x-0.5">
                <button
                    onclick={() => viewMode = 'unified'}
                    class="px-2 py-0.5 text-[10px] font-semibold rounded-md transition-all {viewMode === 'unified' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
                >
                    Unified
                </button>
                <button
                    onclick={() => viewMode = 'split'}
                    class="px-2 py-0.5 text-[10px] font-semibold rounded-md transition-all {viewMode === 'split' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
                >
                    Split
                </button>
                <button
                    onclick={() => viewMode = 'inline'}
                    class="px-2 py-0.5 text-[10px] font-semibold rounded-md transition-all {viewMode === 'inline' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
                >
                    Inline
                </button>
            </div>
        </div>

        <!-- Unified Diff View -->
        {#if viewMode === 'unified'}
            <div class="font-mono text-xs overflow-auto max-h-72 rounded-lg border border-slate-900 leading-relaxed bg-slate-950 p-3 space-y-0.5 select-text">
                {#each diffResult.lines as line}
                    {#if line.kind === 'insert'}
                        <div class="bg-emerald-950/20 text-emerald-300 px-2 py-0.5 rounded border-l-2 border-emerald-500 flex">
                            <span class="w-8 select-none opacity-40 text-right pr-2 font-mono">{line.newLineNum || ''}</span>
                            <span class="w-4 select-none opacity-50 text-left pr-1">+</span>
                            <pre class="whitespace-pre-wrap flex-1 font-mono">{line.content.replace('\n', '')}</pre>
                        </div>
                    {:else if line.kind === 'delete'}
                        <div class="bg-rose-950/20 text-rose-300 px-2 py-0.5 rounded border-l-2 border-rose-500 flex">
                            <span class="w-8 select-none opacity-40 text-right pr-2 font-mono">{line.oldLineNum || ''}</span>
                            <span class="w-4 select-none opacity-50 text-left pr-1">-</span>
                            <pre class="whitespace-pre-wrap flex-1 font-mono">{line.content.replace('\n', '')}</pre>
                        </div>
                    {:else}
                        <div class="text-slate-400 px-2 py-0.5 flex">
                            <span class="w-8 select-none opacity-15 text-right pr-2 font-mono">{line.oldLineNum || ''}</span>
                            <span class="w-4 select-none opacity-15 text-left pr-1"> </span>
                            <pre class="whitespace-pre-wrap flex-1 font-mono">{line.content.replace('\n', '')}</pre>
                        </div>
                    {/if}
                {/each}
            </div>
        {:else if viewMode === 'split'}
            <!-- Split Diff View -->
            <div class="font-mono text-xs overflow-auto max-h-72 rounded-lg border border-slate-900 leading-relaxed bg-slate-950 grid grid-cols-2 divide-x divide-slate-900 select-text">
                <!-- Left Pane: Old/Original -->
                <div class="p-2 space-y-0.5 overflow-hidden">
                    {#each splitLines as { left }}
                        {#if left.kind === 'delete'}
                            <div class="bg-rose-950/20 text-rose-300 px-2 py-0.5 rounded border-l-2 border-rose-500 flex">
                                <span class="w-6 select-none opacity-40 text-right pr-1.5 font-mono">{left.oldLineNum || ''}</span>
                                <pre class="whitespace-pre-wrap flex-1 font-mono">{left.content.replace('\n', '')}</pre>
                            </div>
                        {:else if left.kind === 'empty'}
                            <div class="opacity-0 px-2 py-0.5 select-none font-mono">~</div>
                        {:else}
                            <div class="text-slate-450 px-2 py-0.5 flex">
                                <span class="w-6 select-none opacity-15 text-right pr-1.5 font-mono">{left.oldLineNum || ''}</span>
                                <pre class="whitespace-pre-wrap flex-1 font-mono">{left.content.replace('\n', '')}</pre>
                            </div>
                        {/if}
                    {/each}
                </div>

                <!-- Right Pane: New/Modified -->
                <div class="p-2 space-y-0.5 overflow-hidden">
                    {#each splitLines as { right }}
                        {#if right.kind === 'insert'}
                            <div class="bg-emerald-950/20 text-emerald-300 px-2 py-0.5 rounded border-l-2 border-emerald-500 flex">
                                <span class="w-6 select-none opacity-40 text-right pr-1.5 font-mono">{right.newLineNum || ''}</span>
                                <pre class="whitespace-pre-wrap flex-1 font-mono">{right.content.replace('\n', '')}</pre>
                            </div>
                        {:else if right.kind === 'empty'}
                            <div class="opacity-0 px-2 py-0.5 select-none font-mono">~</div>
                        {:else}
                            <div class="text-slate-450 px-2 py-0.5 flex">
                                <span class="w-6 select-none opacity-15 text-right pr-1.5 font-mono">{right.newLineNum || ''}</span>
                                <pre class="whitespace-pre-wrap flex-1 font-mono">{right.content.replace('\n', '')}</pre>
                            </div>
                        {/if}
                    {/each}
                </div>
            </div>
        {:else}
            <!-- Inline Word Diff View -->
            <div class="font-mono text-xs overflow-auto max-h-72 rounded-lg border border-slate-900 leading-relaxed bg-slate-950 p-3 flex flex-wrap gap-1 select-text">
                {#each diffResult.lines as line}
                    {#if line.kind === 'insert'}
                        <span class="bg-emerald-900/40 text-emerald-300 px-1.5 py-0.5 rounded border border-emerald-700/50 underline">{line.content.trim()}</span>
                    {:else if line.kind === 'delete'}
                        <span class="bg-rose-900/40 text-rose-300 px-1.5 py-0.5 rounded border border-rose-700/50 line-through opacity-80">{line.content.trim()}</span>
                    {:else}
                        <span class="text-slate-300 px-1 py-0.5">{line.content.trim()}</span>
                    {/if}
                {/each}
            </div>
        {/if}
    </div>
{/if}
