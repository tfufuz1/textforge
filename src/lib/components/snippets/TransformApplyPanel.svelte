<script lang="ts">
    import { onMount } from 'svelte';
    import { scriptsStore, loadScripts } from '../../stores/scripts';
    import { pipelinesStore, loadPipelines } from '../../stores/pipelines';
    import { invoke } from '@tauri-apps/api/core';
    import DiffViewer from '../diff/DiffViewer.svelte';
    import BuiltinPalette from './BuiltinPalette.svelte';

    let { content = $bindable(''), onApply = () => {} } = $props();

    let selectedType = $state<'builtin' | 'script' | 'pipeline'>('builtin');
    let selectedId = $state('');
    let transformResult = $state('');
    let hasTransformed = $state(false);

    onMount(async () => {
        await loadScripts();
        await loadPipelines();
    });

    async function handleTransform() {
        if (!selectedId) return;
        try {
            if (selectedType === 'script') {
                const script = $scriptsStore.find(s => s.id === selectedId);
                if ((script?.category === 'security' || (script as any)?.isSafetyCritical) && !confirm(`Achtung: "${script?.name}" ist als sicherheitskritisch/destruktiv markiert. Fortfahren?`)) {
                    return;
                }
                const res = await invoke<{ output: string }>('execute_script', {
                    req: { scriptId: selectedId, input: content }
                });
                transformResult = res.output;
            } else {
                const res = await invoke<{ finalOutput: string }>('run_pipeline', {
                    pipelineId: selectedId, input: content
                });
                transformResult = res.finalOutput;
            }
            hasTransformed = true;
        } catch (e) {
            console.error("Transformation failed:", e);
        }
    }

    function applyChange() {
        content = transformResult;
        hasTransformed = false;
        onApply();
    }

    function discardChange() {
        transformResult = '';
        hasTransformed = false;
    }
</script>

<div class="bg-slate-900/60 p-4 rounded-xl border border-slate-800/80 space-y-4">
    <div class="flex justify-between items-center">
        <h3 class="text-xs font-semibold text-slate-300 font-mono">Schnell-Transformation</h3>
        <div class="flex bg-slate-950/60 rounded-lg p-0.5 border border-slate-800">
            <button
                onclick={() => { selectedType = 'builtin'; selectedId = ''; discardChange(); }}
                class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all {selectedType === 'builtin' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
            >
                Builtins
            </button>
            <button
                onclick={() => { selectedType = 'script'; selectedId = ''; discardChange(); }}
                class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all {selectedType === 'script' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
            >
                Skript
            </button>
            <button
                onclick={() => { selectedType = 'pipeline'; selectedId = ''; discardChange(); }}
                class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all {selectedType === 'pipeline' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
            >
                Pipeline
            </button>
        </div>
    </div>

    {#if selectedType === 'builtin'}
        <BuiltinPalette bind:content={content} {onApply} />
    {:else}
        <div class="flex gap-2">
            <select
                bind:value={selectedId}
                class="flex-1 bg-slate-950 border border-slate-850 rounded-lg p-2 text-xs text-white"
            >
                <option value="">-- Wählen --</option>
                {#if selectedType === 'script'}
                    {#each $scriptsStore as s}
                        <option value={s.id}>{s.name} ({s.scriptType})</option>
                    {/each}
                {:else}
                    {#each $pipelinesStore as p}
                        <option value={p.id}>{p.name}</option>
                    {/each}
                {/if}
            </select>

            <button
                onclick={handleTransform}
                disabled={!selectedId}
                class="px-3.5 py-2 text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg transition-all"
            >
                Testen
            </button>
        </div>

        {#if hasTransformed}
            <DiffViewer original={content} modified={transformResult} />

            <div class="flex justify-end space-x-2 pt-2">
                <button
                    onclick={discardChange}
                    class="px-3.5 py-1.5 text-xs font-medium bg-slate-800 text-slate-300 rounded-lg hover:bg-slate-700"
                >
                    Verwerfen
                </button>
                <button
                    onclick={applyChange}
                    class="px-3.5 py-1.5 text-xs font-semibold bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg"
                >
                    Übernehmen
                </button>
            </div>
        {/if}
    {/if}
</div>
