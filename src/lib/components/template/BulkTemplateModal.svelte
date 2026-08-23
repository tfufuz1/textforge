<script lang="ts">
    import { onMount } from 'svelte';
    import {
        previewTemplateVariablesForSelection,
        getSnippet,
        renderTemplate,
        writeToClipboard,
        type AggregatedTemplateVariableDto,
        type SnippetDto
    } from '../../ipc/snippets';
    import { pushNotification, Notifications } from '../../stores/notifications';

    interface Props {
        snippetIds: string[];
        onClose: () => void;
    }

    let { snippetIds, onClose }: Props = $props();

    let loading = $state(true);
    let variables = $state<AggregatedTemplateVariableDto[]>([]);
    let totalSnippets = $state(0);
    let snippets = $state<SnippetDto[]>([]);
    let values = $state<Record<string, string>>({});
    let renderedOutputs = $state<Record<string, string>>({});
    let activeTab = $state<'form' | 'previews'>('form');

    onMount(() => {
        init();
    });

    async function init() {
        loading = true;
        try {
            const previewData = await previewTemplateVariablesForSelection(snippetIds);
            variables = previewData.variables;
            totalSnippets = previewData.totalSnippets;

            // Initialize default values for inputs
            const initialValues: Record<string, string> = {};
            variables.forEach((v) => {
                if (!v.isSpecial) {
                    initialValues[v.name] = v.defaultVal || '';
                }
            });
            values = initialValues;

            // Fetch actual snippets to render outputs
            const fetched = await Promise.all(snippetIds.map((id) => getSnippet(id).catch(() => null)));
            snippets = fetched.filter((s): s is SnippetDto => s !== null);

            await renderAll();
        } catch (err) {
            console.error('Failed to initialize bulk template view:', err);
        } finally {
            loading = false;
        }
    }

    async function renderAll() {
        const outputs: Record<string, string> = {};
        for (const snip of snippets) {
            try {
                const res = await renderTemplate(snip.content, values, false);
                outputs[snip.id] = res.output;
            } catch (e) {
                outputs[snip.id] = snip.content;
            }
        }
        renderedOutputs = outputs;
    }

    function handleInputChange(name: string, val: string) {
        values[name] = val;
        renderAll();
    }

    async function copySingle(id: string) {
        const text = renderedOutputs[id] || '';
        await writeToClipboard(text, id);
        pushNotification(Notifications.snippetCopied());
    }

    async function copyAll() {
        const combined = snippets
            .map((s) => `--- ${s.title} ---\n${renderedOutputs[s.id] || s.content}`)
            .join('\n\n');
        await writeToClipboard(combined, null);
        pushNotification(Notifications.snippetCopied());
    }
</script>

<div class="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-sm p-4">
    <div class="w-full max-w-3xl bg-slate-900 border border-slate-800 rounded-2xl shadow-2xl flex flex-col max-h-[85vh] overflow-hidden">
        <!-- Modal Header -->
        <div class="flex items-center justify-between px-5 py-4 border-b border-slate-800 bg-slate-900/90">
            <div>
                <h2 class="text-base font-bold text-slate-100 flex items-center space-x-2">
                    <span>⚡ Bulk-Template-Variablen</span>
                    <span class="px-2 py-0.5 text-xs bg-indigo-950 text-indigo-300 rounded-lg border border-indigo-800/50 font-mono">
                        {totalSnippets} Templates ausgewählt
                    </span>
                </h2>
                <p class="text-xs text-slate-400 mt-0.5">
                    Konsolidierte Variablen aller ausgewählten Templates auf einmal ausfüllen.
                </p>
            </div>
            <button
                onclick={onClose}
                class="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800 rounded-xl transition-colors"
                title="Schließen"
            >
                ✕
            </button>
        </div>

        {#if loading}
            <div class="p-12 text-center text-slate-400 space-y-2">
                <span class="animate-spin inline-block text-2xl text-indigo-400">⏳</span>
                <p class="text-xs">Analysiere Template-Variablen für {snippetIds.length} Snippets...</p>
            </div>
        {:else}
            <!-- View Tabs -->
            <div class="flex border-b border-slate-800 px-5 bg-slate-950/40 text-xs">
                <button
                    class="py-2.5 px-4 font-semibold border-b-2 transition-colors {activeTab === 'form' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
                    onclick={() => activeTab = 'form'}
                >
                    Variablen-Formular ({variables.filter(v => !v.isSpecial).length})
                </button>
                <button
                    class="py-2.5 px-4 font-semibold border-b-2 transition-colors {activeTab === 'previews' ? 'border-indigo-500 text-indigo-400' : 'border-transparent text-slate-400 hover:text-slate-200'}"
                    onclick={() => activeTab = 'previews'}
                >
                    Vorschau & Rendern ({snippets.length})
                </button>
            </div>

            <div class="p-5 flex-1 overflow-y-auto space-y-4 custom-scrollbar">
                {#if activeTab === 'form'}
                    {#if variables.filter(v => !v.isSpecial).length > 0}
                        <div class="grid grid-cols-1 md:grid-cols-2 gap-3.5">
                            {#each variables as variable}
                                {#if !variable.isSpecial}
                                    <div class="p-3 bg-slate-950/60 rounded-xl border border-slate-800/80 space-y-1.5">
                                        <div class="flex items-center justify-between text-xs">
                                            <label for="bulk-var-{variable.name}" class="font-bold text-slate-200 flex items-center space-x-1">
                                                <span>{variable.name}</span>
                                                {#if variable.isRequired}
                                                    <span class="text-rose-400" title="Pflichtfeld in mind. 1 Template">*</span>
                                                {/if}
                                            </label>
                                            <span class="text-[10px] text-slate-400 bg-slate-800 px-1.5 py-0.5 rounded border border-slate-700/60 font-mono">
                                                In {variable.snippetCount}/{totalSnippets} Snippets
                                            </span>
                                        </div>

                                        <input
                                            id="bulk-var-{variable.name}"
                                            type="text"
                                            value={values[variable.name] || ''}
                                            oninput={(e) => handleInputChange(variable.name, (e.target as HTMLInputElement).value)}
                                            placeholder={variable.defaultVal ? `Standard: ${variable.defaultVal}` : 'Wert eingeben...'}
                                            class="w-full px-3 py-1.5 text-xs bg-slate-900 border border-slate-700/80 rounded-lg text-slate-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all"
                                        />

                                        {#if variable.filter}
                                            <div class="text-[10px] text-slate-500 font-mono">
                                                Filter: <span class="text-indigo-400">{variable.filter}</span>
                                            </div>
                                        {/if}
                                    </div>
                                {/if}
                            {/each}
                        </div>
                    {:else}
                        <div class="p-8 text-center text-slate-400 text-xs">
                            Keine ausfüllbaren Variablen in den ausgewählten Templates gefunden.
                        </div>
                    {/if}

                    {#if variables.some(v => v.isSpecial)}
                        <div class="pt-3 border-t border-slate-800 text-[11px] text-slate-400 space-y-1">
                            <span class="font-semibold text-slate-500">Automatische Spezial-Variablen:</span>
                            <div class="flex flex-wrap gap-1.5">
                                {#each variables.filter(v => v.isSpecial) as v}
                                    <span class="px-2 py-0.5 bg-slate-950/80 text-indigo-400 rounded-md border border-slate-800 font-mono text-[10px]">
                                        {v.name}
                                    </span>
                                {/each}
                            </div>
                        </div>
                    {/if}
                {:else}
                    <!-- Previews tab -->
                    <div class="space-y-3">
                        {#each snippets as snip (snip.id)}
                            <div class="p-3.5 bg-slate-950/60 border border-slate-800 rounded-xl space-y-2">
                                <div class="flex items-center justify-between text-xs">
                                    <span class="font-bold text-slate-200">{snip.title}</span>
                                    <button
                                        onclick={() => copySingle(snip.id)}
                                        class="px-2.5 py-1 text-[11px] font-medium bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg transition-colors flex items-center space-x-1"
                                    >
                                        <span>📋 Kopieren</span>
                                    </button>
                                </div>
                                <pre class="p-3 bg-slate-900 rounded-lg text-xs font-mono text-slate-300 whitespace-pre-wrap border border-slate-800 max-h-40 overflow-y-auto">{renderedOutputs[snip.id] || snip.content}</pre>
                            </div>
                        {/each}
                    </div>
                {/if}
            </div>

            <!-- Modal Footer -->
            <div class="flex items-center justify-between px-5 py-3.5 border-t border-slate-800 bg-slate-900/90">
                <button
                    onclick={copyAll}
                    class="px-4 py-2 text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl shadow-md shadow-indigo-600/20 transition-all flex items-center space-x-1.5"
                >
                    <span>📋 Alle gerenderten Templates kopieren</span>
                </button>

                <button
                    onclick={onClose}
                    class="px-4 py-2 text-xs font-semibold bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-xl transition-colors"
                >
                    Fertig / Schließen
                </button>
            </div>
        {/if}
    </div>
</div>
