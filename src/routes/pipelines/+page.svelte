<script lang="ts">
    import { onMount } from 'svelte';
    import { 
        pipelinesStore, 
        activePipelineStore, 
        pipelineResultStore, 
        loadPipelines, 
        handleCreatePipeline, 
        handleDeletePipeline, 
        handleRunPipeline 
    } from '../../lib/stores/pipelines';
    import { scriptsStore, loadScripts } from '../../lib/stores/scripts';
    import { addPipelineStep, removePipelineStep, togglePipelineStep } from '../../lib/ipc/pipelines';

    let testInput = $state('Beispiel Eingabetext für die Transformation');
    let newName = $state('');
    let showNewModal = $state(false);
    let selectedScriptIdForStep = $state('');
    let stepLabel = $state('');

    onMount(async () => {
        await loadPipelines();
        await loadScripts();
    });

    async function runPipelineTest() {
        if ($activePipelineStore) {
            await handleRunPipeline($activePipelineStore.id, testInput);
        }
    }

    async function createNew() {
        if (!newName.trim()) return;
        await handleCreatePipeline({
            name: newName,
            description: 'Benutzerdefinierte Pipeline'
        });
        newName = '';
        showNewModal = false;
    }

    async function handleAddStep() {
        if (!$activePipelineStore) return;
        const script = $scriptsStore.find(s => s.id === selectedScriptIdForStep);
        const label = stepLabel.trim() || (script ? script.name : 'Pipeline Schritt');
        await addPipelineStep($activePipelineStore.id, selectedScriptIdForStep || undefined, label);
        await loadPipelines();
        stepLabel = '';
    }

    async function handleRemoveStep(stepId: string) {
        await removePipelineStep(stepId);
        await loadPipelines();
    }

    async function handleToggleStep(stepId: string, currentEnabled: boolean) {
        await togglePipelineStep(stepId, !currentEnabled);
        await loadPipelines();
    }
</script>

<div class="h-full flex flex-col p-6 space-y-5 bg-slate-950 text-slate-100 overflow-hidden">
    <div class="flex justify-between items-center">
        <div>
            <h1 class="text-2xl font-extrabold tracking-tight text-white flex items-center space-x-2">
                <span>🔗</span>
                <span>Transformations-Pipelines</span>
            </h1>
            <p class="text-xs text-slate-400 mt-1">Verkette mehrere Skripte und Vorlagen zu einer automatisierten Pipeline</p>
        </div>
        <button 
            onclick={() => showNewModal = true}
            class="px-4 py-2 text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl transition-all shadow-md flex items-center space-x-1.5"
        >
            <span>+</span>
            <span>Neue Pipeline</span>
        </button>
    </div>

    <div class="flex-1 grid grid-cols-3 gap-6 overflow-hidden min-h-0">
        <!-- Sidebar List -->
        <div class="col-span-1 bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col overflow-hidden backdrop-blur-md">
            <h2 class="font-semibold text-xs text-slate-400 uppercase tracking-wider mb-3">Verfügbare Pipelines</h2>
            <div class="flex-1 overflow-y-auto space-y-2 pr-1 custom-scrollbar">
                {#each $pipelinesStore as pipeline (pipeline.id)}
                    <div 
                        class="p-3 border rounded-xl cursor-pointer transition-all flex justify-between items-start {$activePipelineStore?.id === pipeline.id ? 'bg-indigo-950/70 border-indigo-500/80 text-white' : 'bg-slate-900/40 hover:bg-slate-800/60 border-slate-800 text-slate-300'}"
                        onclick={() => activePipelineStore.set(pipeline)}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && activePipelineStore.set(pipeline)}
                    >
                        <div class="min-w-0 pr-2">
                            <div class="font-semibold text-sm truncate">{pipeline.name}</div>
                            <div class="text-xs text-slate-400 mt-1 truncate">{pipeline.description || 'Keine Beschreibung'}</div>
                            <span class="inline-block text-[10px] px-2 py-0.5 bg-slate-800 text-slate-300 rounded-md mt-2 font-mono border border-slate-700">
                                {pipeline.steps.length} Schritte
                            </span>
                        </div>
                        <button 
                            class="text-xs p-1.5 text-slate-400 hover:text-rose-400 transition-colors" 
                            title="Löschen"
                            onclick={(e) => { e.stopPropagation(); handleDeletePipeline(pipeline.id); }}
                        >
                            🗑️
                        </button>
                    </div>
                {:else}
                    <div class="p-6 text-center text-slate-500 text-sm">
                        Keine Pipelines vorhanden.
                    </div>
                {/each}
            </div>
        </div>

        <!-- Pipeline Steps Pane -->
        <div class="col-span-2 bg-slate-900/60 border border-slate-800 rounded-2xl p-5 flex flex-col overflow-hidden backdrop-blur-md">
            {#if $activePipelineStore}
                <div class="flex justify-between items-center pb-3 border-b border-slate-800 mb-4">
                    <div>
                        <h2 class="font-bold text-lg text-white font-mono">{$activePipelineStore.name}</h2>
                        <span class="text-xs text-slate-400">{$activePipelineStore.description}</span>
                    </div>
                    <button 
                        onclick={runPipelineTest}
                        class="px-4 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-xl text-xs font-semibold shadow-md transition-all flex items-center space-x-1.5"
                    >
                        <span>▶</span>
                        <span>Pipeline Ausführen</span>
                    </button>
                </div>

                <div class="flex-1 flex flex-col space-y-4 overflow-hidden">
                    <div class="flex-1 border border-slate-800 bg-slate-950/50 rounded-xl p-3 overflow-y-auto space-y-2">
                        <div class="flex justify-between items-center mb-2">
                            <h3 class="text-xs font-semibold text-slate-400">Pipeline Schritte</h3>
                        </div>

                        <!-- Add Step Controls -->
                        <div class="flex items-center space-x-2 mb-3 bg-slate-900 p-2.5 rounded-xl border border-slate-800">
                            <select bind:value={selectedScriptIdForStep} class="flex-1 bg-slate-950 text-white border border-slate-800 rounded-lg p-1.5 text-xs">
                                <option value="">-- Skript wählen --</option>
                                {#each $scriptsStore as script}
                                    <option value={script.id}>{script.name} ({script.scriptType})</option>
                                {/each}
                            </select>
                            <input type="text" bind:value={stepLabel} placeholder="Bezeichnung (optional)" class="flex-1 bg-slate-950 text-white border border-slate-800 rounded-lg p-1.5 text-xs" />
                            <button onclick={handleAddStep} class="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg text-xs font-semibold">
                                + Schritt
                            </button>
                        </div>

                        {#each $activePipelineStore.steps as step, idx}
                            <div class="p-3 border rounded-xl bg-slate-900/80 border-slate-800 flex items-center justify-between text-sm">
                                <div class="flex items-center space-x-3">
                                    <span class="w-6 h-6 rounded-full bg-indigo-950 border border-indigo-700 text-indigo-300 font-bold text-xs flex items-center justify-center font-mono">
                                        {idx + 1}
                                    </span>
                                    <span class="font-medium text-slate-200">{step.label}</span>
                                    <button 
                                        onclick={() => handleToggleStep(step.id, step.enabled)} 
                                        class="text-[10px] px-2 py-0.5 rounded font-mono border {step.enabled ? 'bg-emerald-950 text-emerald-300 border-emerald-800' : 'bg-slate-800 text-slate-500 border-slate-700'}"
                                    >
                                        {step.enabled ? 'Aktiv' : 'Deaktiviert'}
                                    </button>
                                </div>
                                <div class="flex items-center space-x-2">
                                    <span class="text-[10px] text-slate-500 font-mono">ID: {step.scriptId || 'Builtin'}</span>
                                    <button onclick={() => handleRemoveStep(step.id)} class="text-slate-400 hover:text-rose-400 text-xs p-1">🗑️</button>
                                </div>
                            </div>
                        {:else}
                            <div class="p-8 text-center text-slate-500 text-xs italic">
                                Diese Pipeline hat noch keine Schritte.
                            </div>
                        {/each}
                    </div>

                    <div class="grid grid-cols-2 gap-4 h-48">
                        <div class="flex flex-col">
                            <label for="pipeline-input" class="text-xs font-semibold text-slate-400 mb-1">Eingabe Text</label>
                            <textarea 
                                id="pipeline-input"
                                bind:value={testInput} 
                                class="flex-1 w-full p-3 font-mono text-xs border border-slate-800 bg-slate-950 text-slate-200 rounded-xl resize-none outline-none focus:ring-1 focus:ring-indigo-500"
                            ></textarea>
                        </div>
                        <div class="flex flex-col">
                            <span class="text-xs font-semibold text-slate-400 mb-1">Endergebnis</span>
                            <div class="flex-1 p-3 font-mono text-xs border border-slate-800 bg-slate-950 rounded-xl overflow-auto text-slate-200">
                                {#if $pipelineResultStore}
                                    <pre class="whitespace-pre-wrap font-mono text-xs text-emerald-300">{$pipelineResultStore.finalOutput}</pre>
                                    <div class="text-[10px] text-slate-500 mt-2">Dauer: {$pipelineResultStore.totalTimeMs}ms</div>
                                {:else}
                                    <span class="text-slate-500 italic">Kein Ergebnis vorliegend.</span>
                                {/if}
                            </div>
                        </div>
                    </div>
                </div>
            {:else}
                <div class="h-full flex items-center justify-center text-slate-500 text-sm">
                    Wähle eine Pipeline aus oder erstelle eine neue.
                </div>
            {/if}
        </div>
    </div>
</div>

{#if showNewModal}
    <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 backdrop-blur-sm">
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 w-96 space-y-4 shadow-2xl text-slate-100">
            <h3 class="font-bold text-lg text-white">Neue Pipeline erstellen</h3>
            <div>
                <label for="pipeline-name-modal" class="block text-xs font-semibold text-slate-400 mb-1">Name</label>
                <input 
                    id="pipeline-name-modal"
                    type="text" 
                    bind:value={newName} 
                    placeholder="z.B. Clean HTML & Format"
                    class="w-full px-3 py-2 text-sm bg-slate-950 border border-slate-800 rounded-xl outline-none focus:ring-1 focus:ring-indigo-500 text-white"
                />
            </div>
            <div class="flex justify-end space-x-2 pt-2">
                <button 
                    onclick={() => showNewModal = false}
                    class="px-4 py-2 text-xs bg-slate-800 text-slate-300 rounded-xl hover:bg-slate-700"
                >
                    Abbrechen
                </button>
                <button 
                    onclick={createNew}
                    class="px-4 py-2 text-xs bg-indigo-600 text-white rounded-xl hover:bg-indigo-500 font-semibold"
                >
                    Erstellen
                </button>
            </div>
        </div>
    </div>
{/if}
