<script lang="ts">
    import { onMount } from 'svelte';
    import { listScripts, listPipelines, type ScriptDto, type PipelineDto } from '../../ipc/transform';
    import { transformClipboardEntry, writeToClipboard, getClipboardEntry, promoteToSnippet, type TransformClipboardResultDto } from '../../ipc/clipboard';
    import { createSnippet } from '../../ipc/snippets';
    import { loadClipboardHistory } from '../../stores/clipboard';
    import { loadSnippets } from '../../stores/snippets';
    import { refreshUndoState } from '../../stores/undo';
    import { pushNotification, Notifications } from '../../stores/notifications';

    import SparklesIcon from '$lib/components/icons/SparklesIcon.svelte';
    import ScriptIcon from '$lib/components/icons/ScriptIcon.svelte';
    import PipelineIcon from '$lib/components/icons/PipelineIcon.svelte';
    import StarIcon from '$lib/components/icons/StarIcon.svelte';
    import CopyIcon from '$lib/components/icons/CopyIcon.svelte';
    import PlusIcon from '$lib/components/icons/PlusIcon.svelte';

    interface Props {
        entryId: string;
        onClose: () => void;
    }

    let { entryId, onClose }: Props = $props();

    let scripts = $state<ScriptDto[]>([]);
    let pipelines = $state<PipelineDto[]>([]);
    let isLoadingList = $state(true);
    let isTransforming = $state(false);
    let originalContent = $state('');
    let transformResult = $state<TransformClipboardResultDto | null>(null);

    let activeTab = $state<'script' | 'pipeline'>('script');
    let selectedScriptId = $state<string | null>(null);
    let selectedPipelineId = $state<string | null>(null);

    onMount(async () => {
        isLoadingList = true;
        try {
            const entry = await getClipboardEntry(entryId);
            originalContent = entry.content;

            const [sList, pList] = await Promise.all([
                listScripts(),
                listPipelines()
            ]);
            scripts = sList;
            pipelines = pList;

            // Auto-select favorited script/pipeline if present
            const favScript = scripts.find(s => s.isFavorite);
            if (favScript) {
                selectedScriptId = favScript.id;
                await runTransformForScript(favScript.id);
            } else if (scripts.length > 0) {
                selectedScriptId = scripts[0].id;
                await runTransformForScript(scripts[0].id);
            } else if (pipelines.length > 0) {
                activeTab = 'pipeline';
                selectedPipelineId = pipelines[0].id;
                await runTransformForPipeline(pipelines[0].id);
            }
        } catch (e) {
            console.error("Error initializing TransformModal:", e);
        } finally {
            isLoadingList = false;
        }
    });

    async function runTransformForScript(scriptId: string) {
        selectedScriptId = scriptId;
        selectedPipelineId = null;
        activeTab = 'script';
        isTransforming = true;
        try {
            transformResult = await transformClipboardEntry(entryId, scriptId, null);
        } catch (e) {
            transformResult = {
                originalContent,
                transformedContent: originalContent,
                executionTimeMs: 0,
                error: String(e)
            };
        } finally {
            isTransforming = false;
        }
    }

    async function runTransformForPipeline(pipelineId: string) {
        selectedPipelineId = pipelineId;
        selectedScriptId = null;
        activeTab = 'pipeline';
        isTransforming = true;
        try {
            transformResult = await transformClipboardEntry(entryId, null, pipelineId);
        } catch (e) {
            transformResult = {
                originalContent,
                transformedContent: originalContent,
                executionTimeMs: 0,
                error: String(e)
            };
        } finally {
            isTransforming = false;
        }
    }

    async function handleWriteToClipboard() {
        if (!transformResult || isTransforming) return;
        try {
            await writeToClipboard(transformResult.transformedContent);
            pushNotification(Notifications.snippetCopied());
            onClose();
        } catch (e) {
            console.error("Failed to write transformed content to clipboard:", e);
        }
    }

    async function handleSaveAsSnippet() {
        if (!transformResult || isTransforming) return;
        try {
            const titleStr = transformResult.transformedContent.trim().slice(0, 60) || "Transformierter Eintrag";
            await createSnippet({
                title: titleStr,
                content: transformResult.transformedContent,
                contentType: 'plain_text',
                tags: ['transformed']
            });
            await loadClipboardHistory();
            await loadSnippets();
            await refreshUndoState();
            pushNotification(Notifications.snippetSaved(titleStr));
            pushNotification(Notifications.undoAvailable("Snippet aus Transformation erstellt"));
            onClose();
        } catch (e) {
            console.error("Failed to save as snippet:", e);
        }
    }
</script>

<div class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4 overflow-y-auto">
    <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-4xl shadow-2xl flex flex-col max-h-[90vh] overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-900/80 shrink-0">
            <div class="flex items-center space-x-3">
                <div class="w-9 h-9 rounded-xl bg-indigo-950/80 border border-indigo-700/50 flex items-center justify-center text-indigo-400">
                    <SparklesIcon class="w-5 h-5" />
                </div>
                <div>
                    <h2 class="text-base font-bold text-white">Zwischenablage direkt transformieren</h2>
                    <p class="text-xs text-slate-400">Transformiere den Text mit einem Regex-Skript oder einer Pipeline und passe das Ergebnis an</p>
                </div>
            </div>
            <button
                type="button"
                onclick={onClose}
                class="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-xl transition-all"
            >
                ✕
            </button>
        </div>

        {#if isLoadingList}
            <div class="p-12 text-center text-slate-400 space-y-2">
                <div class="inline-block animate-spin rounded-full h-8 w-8 border-2 border-indigo-500 border-t-transparent"></div>
                <p class="text-xs font-mono">Lade Skripte & Pipelines...</p>
            </div>
        {:else}
            <div class="grid grid-cols-1 md:grid-cols-3 flex-1 overflow-hidden min-h-0 divide-y md:divide-y-0 md:divide-x divide-slate-800">
                <!-- Selection Sidebar -->
                <div class="p-4 bg-slate-950/60 space-y-3 flex flex-col min-h-0 overflow-y-auto custom-scrollbar">
                    <div class="flex items-center bg-slate-900 p-1 rounded-xl border border-slate-800 shrink-0">
                        <button
                            type="button"
                            onclick={() => activeTab = 'script'}
                            class="flex-1 py-1.5 text-xs font-semibold rounded-lg transition-all flex items-center justify-center space-x-1.5 {activeTab === 'script' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            <ScriptIcon class="w-3.5 h-3.5" />
                            <span>Skripte ({scripts.length})</span>
                        </button>
                        <button
                            type="button"
                            onclick={() => activeTab = 'pipeline'}
                            class="flex-1 py-1.5 text-xs font-semibold rounded-lg transition-all flex items-center justify-center space-x-1.5 {activeTab === 'pipeline' ? 'bg-indigo-600 text-white shadow-md' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            <PipelineIcon class="w-3.5 h-3.5" />
                            <span>Pipelines ({pipelines.length})</span>
                        </button>
                    </div>

                    <div class="space-y-1.5 flex-1 overflow-y-auto custom-scrollbar pr-1">
                        {#if activeTab === 'script'}
                            {#each scripts as script (script.id)}
                                <button
                                    type="button"
                                    onclick={() => runTransformForScript(script.id)}
                                    class="w-full text-left p-2.5 rounded-xl border transition-all flex items-center justify-between gap-2 {selectedScriptId === script.id ? 'bg-indigo-950/60 border-indigo-500/80 text-white ring-1 ring-indigo-500/40' : 'bg-slate-900/50 border-slate-800/80 hover:bg-slate-900 text-slate-300 hover:border-slate-700'}"
                                >
                                    <div class="min-w-0 flex-1">
                                        <div class="flex items-center space-x-1.5">
                                            <span class="text-xs font-bold truncate">{script.name}</span>
                                            {#if script.isFavorite}
                                                <StarIcon class="w-3 h-3 text-amber-400 shrink-0" />
                                            {/if}
                                        </div>
                                        <p class="text-[10px] text-slate-400 truncate mt-0.5">{script.description || script.scriptType}</p>
                                    </div>
                                    <span class="text-[9px] px-1.5 py-0.5 bg-slate-950 rounded border border-slate-800/80 uppercase font-mono text-indigo-300 shrink-0">{script.scriptType}</span>
                                </button>
                            {:else}
                                <p class="text-xs text-slate-500 italic py-4 text-center">Keine Skripte vorhanden</p>
                            {/each}
                        {:else}
                            {#each pipelines as pipeline (pipeline.id)}
                                <button
                                    type="button"
                                    onclick={() => runTransformForPipeline(pipeline.id)}
                                    class="w-full text-left p-2.5 rounded-xl border transition-all flex items-center justify-between gap-2 {selectedPipelineId === pipeline.id ? 'bg-indigo-950/60 border-indigo-500/80 text-white ring-1 ring-indigo-500/40' : 'bg-slate-900/50 border-slate-800/80 hover:bg-slate-900 text-slate-300 hover:border-slate-700'}"
                                >
                                    <div class="min-w-0 flex-1">
                                        <div class="flex items-center space-x-1.5">
                                            <span class="text-xs font-bold truncate">{pipeline.name}</span>
                                            {#if pipeline.isFavorite}
                                                <StarIcon class="w-3 h-3 text-amber-400 shrink-0" />
                                            {/if}
                                        </div>
                                        <p class="text-[10px] text-slate-400 truncate mt-0.5">{pipeline.description || `${pipeline.steps.length} Schritte`}</p>
                                    </div>
                                    <span class="text-[9px] px-1.5 py-0.5 bg-slate-950 rounded border border-slate-800/80 uppercase font-mono text-emerald-300 shrink-0">{pipeline.steps.length} Steps</span>
                                </button>
                            {:else}
                                <p class="text-xs text-slate-500 italic py-4 text-center">Keine Pipelines vorhanden</p>
                            {/each}
                        {/if}
                    </div>
                </div>

                <!-- Preview Area -->
                <div class="md:col-span-2 p-5 space-y-4 flex flex-col min-h-0 overflow-y-auto custom-scrollbar">
                    {#if isTransforming}
                        <div class="py-12 text-center text-slate-400 space-y-2 my-auto">
                            <div class="inline-block animate-spin rounded-full h-8 w-8 border-2 border-indigo-500 border-t-transparent"></div>
                            <p class="text-xs font-mono">Führe Transformation aus...</p>
                        </div>
                    {:else if transformResult}
                        {#if transformResult.error}
                            <div class="p-3.5 bg-rose-950/50 border border-rose-800/60 rounded-xl text-rose-300 text-xs space-y-1">
                                <span class="font-bold block">Transformationsfehler:</span>
                                <p class="font-mono text-[11px] whitespace-pre-wrap">{transformResult.error}</p>
                            </div>
                        {/if}

                        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3 flex-1 min-h-0">
                            <!-- Original -->
                            <div class="flex flex-col space-y-1.5 min-h-0">
                                <div class="flex items-center justify-between text-xs font-semibold text-slate-400">
                                    <span>Originaler Text</span>
                                    <span class="font-mono text-[10px] text-slate-500">{transformResult.originalContent.length} Zeichen</span>
                                </div>
                                <div class="flex-1 bg-slate-950 border border-slate-800 rounded-xl p-3 max-h-64 sm:max-h-80 overflow-y-auto custom-scrollbar">
                                    <pre class="font-mono text-xs text-slate-300 whitespace-pre-wrap break-words">{transformResult.originalContent}</pre>
                                </div>
                            </div>

                            <!-- Transformed -->
                            <div class="flex flex-col space-y-1.5 min-h-0">
                                <div class="flex items-center justify-between text-xs font-semibold text-indigo-300">
                                    <span>Transformiertes Ergebnis</span>
                                    <span class="font-mono text-[10px] text-indigo-400/80">{transformResult.transformedContent.length} Zeichen · {transformResult.executionTimeMs} ms</span>
                                </div>
                                <div class="flex-1 bg-slate-950 border border-indigo-900/50 rounded-xl p-3 max-h-64 sm:max-h-80 overflow-y-auto custom-scrollbar ring-1 ring-indigo-500/20">
                                    <pre class="font-mono text-xs text-indigo-100 whitespace-pre-wrap break-words">{transformResult.transformedContent}</pre>
                                </div>
                            </div>
                        </div>
                    {:else}
                        <div class="py-12 text-center text-slate-500 text-xs my-auto">
                            Wähle ein Skript oder eine Pipeline aus der linken Spalte aus, um eine Vorschau anzuzeigen.
                        </div>
                    {/if}
                </div>
            </div>

            <!-- Footer Actions -->
            <div class="flex flex-wrap items-center justify-between gap-3 px-6 py-4 border-t border-slate-800 bg-slate-900/80 shrink-0">
                <span class="text-xs text-slate-500 font-mono">
                    {#if transformResult}
                        Dauer: <strong class="text-slate-300">{transformResult.executionTimeMs} ms</strong>
                    {/if}
                </span>

                <div class="flex items-center space-x-2">
                    <button
                        type="button"
                        onclick={onClose}
                        class="px-4 py-2 text-xs font-semibold text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-xl transition-all"
                    >
                        Verwerfen
                    </button>
                    <button
                        type="button"
                        onclick={handleSaveAsSnippet}
                        disabled={!transformResult || isTransforming || !!transformResult.error}
                        class="px-4 py-2 text-xs font-semibold text-indigo-300 bg-indigo-950 hover:bg-indigo-900 border border-indigo-700/50 disabled:opacity-40 rounded-xl transition-all flex items-center space-x-1.5"
                    >
                        <PlusIcon class="w-4 h-4 text-indigo-400" />
                        <span>Als neues Snippet speichern</span>
                    </button>
                    <button
                        type="button"
                        onclick={handleWriteToClipboard}
                        disabled={!transformResult || isTransforming || !!transformResult.error}
                        class="px-4 py-2 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 rounded-xl transition-all flex items-center space-x-1.5 shadow-lg shadow-indigo-950/50"
                    >
                        <CopyIcon class="w-4 h-4" />
                        <span>In Zwischenablage zurückschreiben</span>
                    </button>
                </div>
            </div>
        {/if}
    </div>
</div>
