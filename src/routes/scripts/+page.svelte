<script lang="ts">
    import { onMount } from 'svelte';
    import { 
        scriptsStore, 
        activeScriptStore, 
        executionResultStore, 
        loadScripts, 
        handleCreateScript, 
        handleDeleteScript, 
        handleTestScript,
        handleUpdateScript
    } from '../../lib/stores/scripts';
    import { saveScriptVersion } from '../../lib/ipc/scripts';
    import ScriptVersionHistory from '../../lib/components/scripts/ScriptVersionHistory.svelte';
    import ScriptTestSuite from '../../lib/components/scripts/ScriptTestSuite.svelte';
    import { pushNotification } from '../../lib/stores/notifications';
    import { Option } from '../../lib/domain/adts';

    let testInput = $state('Hallo Welt! 123');
    let newName = $state('');
    let newScriptType = $state<'js' | 'regex'>('js');
    let newJsCode = $state('return input.toUpperCase();');
    let newRegexPattern = $state('\\d+');
    let newRegexReplacement = $state('[ZAHL]');
    let showNewModal = $state(false);

    // Editable fields for active script
    let editJsCode = $state('');
    let editRegexPattern = $state('');
    let editRegexReplacement = $state('');
    let editRegexFlags = $state('g');

    // Active panel: 'test' | 'suite' | 'versions'
    let activePanel = $state<'test' | 'suite' | 'versions'>('suite');
    let versionNote = $state('');
    let showVersionModal = $state(false);

    $effect(() => {
        if ($activeScriptStore) {
            editJsCode = $activeScriptStore.jsCode ?? '';
            editRegexPattern = $activeScriptStore.regexPattern ?? '';
            editRegexReplacement = $activeScriptStore.regexReplacement ?? '';
            editRegexFlags = $activeScriptStore.regexFlags ?? 'g';
        }
    });

    onMount(async () => {
        await loadScripts();
    });

    async function runTest() {
        if ($activeScriptStore) {
            const draft = $activeScriptStore.scriptType === 'js'
                ? { jsCode: editJsCode }
                : { regexPattern: editRegexPattern, regexReplacement: editRegexReplacement, regexFlags: editRegexFlags };
            await handleTestScript(testInput, draft);
        }
    }

    async function createNew() {
        if (!newName.trim()) return;
        await handleCreateScript({
            name: newName,
            scriptType: newScriptType,
            category: 'custom',
            jsCode: newScriptType === 'js' ? newJsCode : undefined,
            regexPattern: newScriptType === 'regex' ? newRegexPattern : undefined,
            regexReplacement: newScriptType === 'regex' ? newRegexReplacement : undefined,
            regexFlags: 'g'
        });
        newName = '';
        showNewModal = false;
    }

    async function saveScript() {
        if (!$activeScriptStore) return;
        const draft: any = {};
        if ($activeScriptStore.scriptType === 'js') {
            draft.jsCode = editJsCode;
        } else {
            draft.regexPattern = editRegexPattern;
            draft.regexReplacement = editRegexReplacement;
            draft.regexFlags = editRegexFlags;
        }
        await handleUpdateScript($activeScriptStore.id, draft);
        pushNotification({ id: crypto.randomUUID(), severity: 'success', title: 'Skript gespeichert', message: Option.some(`„${$activeScriptStore.name}" wurde aktualisiert.`), duration: 1500, action: Option.none(), createdAt: Date.now() as any });
    }

    async function saveVersion() {
        if (!$activeScriptStore) return;
        try {
            // First save the current code, then snapshot version
            await saveScript();
            await saveScriptVersion($activeScriptStore.id, versionNote.trim() || 'Manuelle Sicherung');
            pushNotification({ id: crypto.randomUUID(), severity: 'success', title: 'Version gespeichert', message: Option.some(versionNote || 'Manuelle Sicherung'), duration: 2000, action: Option.none(), createdAt: Date.now() as any });
            versionNote = '';
            showVersionModal = false;
        } catch (e) {
            console.error('Failed to save version:', e);
        }
    }
</script>

<div class="h-full flex flex-col p-6 space-y-5 bg-slate-950 text-slate-100 overflow-hidden">
    <div class="flex justify-between items-center">
        <div>
            <h1 class="text-2xl font-extrabold tracking-tight text-white flex items-center space-x-2">
                <span>⚡</span>
                <span>Skripte &amp; Transformationen</span>
            </h1>
            <p class="text-xs text-slate-400 mt-1">QuickJS Sandbox &amp; Regex-Transformationen</p>
        </div>
        <button 
            onclick={() => showNewModal = true}
            class="px-4 py-2 text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl transition-all shadow-md flex items-center space-x-1.5"
        >
            <span>+</span>
            <span>Neues Skript</span>
        </button>
    </div>

    <div class="flex-1 grid grid-cols-3 gap-6 overflow-hidden min-h-0">
        <!-- Sidebar List -->
        <div class="col-span-1 bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col overflow-hidden backdrop-blur-md">
            <h2 class="font-semibold text-xs text-slate-400 uppercase tracking-wider mb-3">Verfügbare Skripte</h2>
            <div class="flex-1 overflow-y-auto space-y-2 pr-1">
                {#each $scriptsStore as script (script.id)}
                    <div 
                        class="p-3 border rounded-xl cursor-pointer transition-all flex justify-between items-start {$activeScriptStore?.id === script.id ? 'bg-indigo-950/70 border-indigo-500/80 text-white' : 'bg-slate-900/40 hover:bg-slate-800/60 border-slate-800 text-slate-300'}"
                        onclick={() => activeScriptStore.set(script)}
                        role="button"
                        tabindex="0"
                        onkeydown={(e) => e.key === 'Enter' && activeScriptStore.set(script)}
                    >
                        <div class="min-w-0 pr-2">
                            <div class="font-semibold text-sm truncate">{script.name}</div>
                            <div class="text-xs text-slate-400 mt-1 truncate">{script.description || 'Keine Beschreibung'}</div>
                            <div class="flex items-center space-x-2 mt-2">
                                <span class="inline-block text-[10px] px-2 py-0.5 bg-slate-800 text-slate-300 rounded-md font-mono border border-slate-700">
                                    {script.scriptType}
                                </span>
                                <span class="text-[10px] text-slate-500 font-mono">{script.usageCount}× genutzt</span>
                            </div>
                        </div>
                        <button 
                            class="text-xs p-1.5 text-slate-400 hover:text-rose-400 transition-colors shrink-0" 
                            title="Löschen"
                            onclick={(e) => { e.stopPropagation(); handleDeleteScript(script.id); }}
                        >
                            🗑️
                        </button>
                    </div>
                {:else}
                    <div class="p-6 text-center text-slate-500 text-sm">
                        Keine Skripte vorhanden.
                    </div>
                {/each}
            </div>
        </div>

        <!-- Editor & Panels -->
        <div class="col-span-2 flex flex-col overflow-hidden gap-4 min-h-0">
            {#if $activeScriptStore}
                <!-- Script editor card -->
                <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 flex flex-col overflow-hidden backdrop-blur-md min-h-0">
                    <div class="flex justify-between items-center pb-3 border-b border-slate-800 mb-4 shrink-0">
                        <div>
                            <h2 class="font-bold text-lg text-white flex items-center space-x-2">
                                <span>{$activeScriptStore.name}</span>
                                <span class="text-xs px-2 py-0.5 rounded bg-indigo-950 text-indigo-300 border border-indigo-800 font-mono">{$activeScriptStore.scriptType}</span>
                            </h2>
                            <span class="text-xs text-slate-400">Kategorie: {$activeScriptStore.category}</span>
                        </div>
                        <div class="flex items-center space-x-2">
                            <button
                                onclick={() => showVersionModal = true}
                                class="px-3 py-1.5 text-xs font-medium bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-xl transition-all flex items-center space-x-1"
                                title="Version speichern"
                            >
                                <span>🏷️</span>
                                <span>Version sichern</span>
                            </button>
                            <button 
                                onclick={saveScript}
                                class="px-4 py-1.5 bg-gradient-to-r from-blue-600 to-indigo-600 hover:from-blue-500 hover:to-indigo-500 text-white rounded-xl text-xs font-semibold shadow-md transition-all"
                            >
                                Speichern
                            </button>
                        </div>
                    </div>

                    <!-- Code editor -->
                    <div class="flex-1 flex flex-col min-h-0 space-y-3">
                        {#if $activeScriptStore.scriptType === 'js'}
                            <div class="flex-1 flex flex-col min-h-0">
                                <label for="js-code" class="text-xs font-semibold text-slate-400 mb-1 shrink-0">
                                    JavaScript Code <span class="text-slate-600 font-normal">(QuickJS Sandbox)</span>
                                </label>
                                <textarea 
                                    id="js-code"
                                    bind:value={editJsCode}
                                    class="flex-1 w-full p-3 font-mono text-sm border border-slate-800 rounded-xl resize-none bg-slate-950 text-emerald-400 outline-none focus:ring-1 focus:ring-indigo-500 leading-relaxed"
                                ></textarea>
                            </div>
                        {:else}
                            <div class="space-y-3 flex-1">
                                <div>
                                    <label for="regex-pat" class="text-xs text-slate-400 block mb-1">Regex Pattern</label>
                                    <input id="regex-pat" type="text" bind:value={editRegexPattern} class="w-full bg-slate-900 border border-slate-800 rounded-lg p-2.5 font-mono text-xs text-indigo-300 outline-none focus:ring-1 focus:ring-indigo-500 transition-all" />
                                </div>
                                <div>
                                    <label for="regex-rep" class="text-xs text-slate-400 block mb-1">Replacement</label>
                                    <input id="regex-rep" type="text" bind:value={editRegexReplacement} class="w-full bg-slate-900 border border-slate-800 rounded-lg p-2.5 font-mono text-xs text-emerald-300 outline-none focus:ring-1 focus:ring-indigo-500 transition-all" />
                                </div>
                                <div>
                                    <label for="regex-flags" class="text-xs text-slate-400 block mb-1">Flags</label>
                                    <input id="regex-flags" type="text" bind:value={editRegexFlags} class="w-32 bg-slate-900 border border-slate-800 rounded-lg p-2.5 font-mono text-xs text-amber-300 outline-none focus:ring-1 focus:ring-indigo-500 transition-all" placeholder="gi" />
                                </div>
                            </div>
                        {/if}

                        <!-- Quick test row -->
                        <div class="shrink-0 grid grid-cols-2 gap-3 border-t border-slate-800 pt-3">
                            <div class="flex flex-col">
                                <div class="flex items-center justify-between mb-1">
                                    <label for="test-input-quick" class="text-xs font-semibold text-slate-400">Schnell-Test Eingabe</label>
                                    <button
                                        onclick={runTest}
                                        class="px-2.5 py-0.5 bg-emerald-700 hover:bg-emerald-600 text-white rounded-lg text-[11px] font-semibold transition-all flex items-center space-x-1"
                                    >
                                        <span>▶</span>
                                        <span>Testen</span>
                                    </button>
                                </div>
                                <textarea 
                                    id="test-input-quick"
                                    bind:value={testInput}
                                    rows="2"
                                    class="w-full p-2.5 font-mono text-xs border border-slate-800 bg-slate-950 text-slate-200 rounded-xl resize-none outline-none focus:ring-1 focus:ring-indigo-500"
                                ></textarea>
                            </div>
                            <div class="flex flex-col">
                                <span class="text-xs font-semibold text-slate-400 mb-1">Ausgabe</span>
                                <div class="flex-1 p-2.5 font-mono text-xs border border-slate-800 bg-slate-950 rounded-xl overflow-auto text-slate-200 min-h-[3rem]">
                                    {#if $executionResultStore}
                                        {#if $executionResultStore.error}
                                            <span class="text-rose-400">{$executionResultStore.error}</span>
                                        {:else}
                                            <pre class="whitespace-pre-wrap text-emerald-300">{$executionResultStore.output}</pre>
                                            <span class="text-[10px] text-slate-600 block mt-1">{$executionResultStore.executionTimeMs}ms</span>
                                        {/if}
                                    {:else}
                                        <span class="text-slate-600 italic">—</span>
                                    {/if}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <!-- Bottom panel: Test Suite or Version History -->
                <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 backdrop-blur-md overflow-y-auto shrink-0 max-h-72">
                    <!-- Panel tabs -->
                    <div class="flex items-center space-x-1 mb-4 bg-slate-950/60 rounded-lg p-0.5 border border-slate-800 w-fit">
                        <button
                            onclick={() => activePanel = 'suite'}
                            class="px-3 py-1 text-[11px] font-semibold rounded-md transition-all {activePanel === 'suite' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            🧪 Test Suite
                        </button>
                        <button
                            onclick={() => activePanel = 'versions'}
                            class="px-3 py-1 text-[11px] font-semibold rounded-md transition-all {activePanel === 'versions' ? 'bg-indigo-600 text-white' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            🏷️ Versionen
                        </button>
                    </div>

                    {#if activePanel === 'suite'}
                        <ScriptTestSuite
                            scriptId={$activeScriptStore.id}
                            jsCode={$activeScriptStore.scriptType === 'js' ? editJsCode : null}
                        />
                    {:else}
                        <ScriptVersionHistory
                            scriptId={$activeScriptStore.id}
                            onRestore={async () => { await loadScripts(); }}
                        />
                    {/if}
                </div>

            {:else}
                <div class="flex-1 bg-slate-900/60 border border-slate-800 rounded-2xl flex items-center justify-center text-slate-500 text-sm backdrop-blur-md">
                    <div class="text-center space-y-2">
                        <div class="text-4xl">⚡</div>
                        <p>Wähle ein Skript aus oder erstelle ein neues.</p>
                    </div>
                </div>
            {/if}
        </div>
    </div>
</div>

<!-- New Script Modal -->
{#if showNewModal}
    <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 backdrop-blur-sm">
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 w-96 space-y-4 shadow-2xl text-slate-100">
            <h3 class="font-bold text-lg text-white">Neues Skript erstellen</h3>
            <div>
                <label for="script-name-modal" class="block text-xs font-semibold text-slate-400 mb-1">Name</label>
                <input 
                    id="script-name-modal"
                    type="text" 
                    bind:value={newName} 
                    placeholder="z.B. Trim and Upper"
                    class="w-full px-3 py-2 text-sm bg-slate-950 border border-slate-800 rounded-xl outline-none focus:ring-1 focus:ring-indigo-500 text-white"
                />
            </div>
            <div>
                <label for="script-type-modal" class="block text-xs font-semibold text-slate-400 mb-1">Typ</label>
                <select id="script-type-modal" bind:value={newScriptType} class="w-full bg-slate-950 border border-slate-800 rounded-xl p-2 text-xs text-white">
                    <option value="js">JavaScript (QuickJS)</option>
                    <option value="regex">Regex Substitution</option>
                </select>
            </div>
            {#if newScriptType === 'js'}
                <div>
                    <label for="script-code-modal" class="block text-xs font-semibold text-slate-400 mb-1">Code</label>
                    <textarea 
                        id="script-code-modal"
                        bind:value={newJsCode} 
                        class="w-full h-28 p-2.5 font-mono text-xs bg-slate-950 border border-slate-800 rounded-xl resize-none outline-none text-emerald-400"
                    ></textarea>
                </div>
            {:else}
                <div class="space-y-2">
                    <div>
                        <label for="regex-pat-modal" class="block text-xs text-slate-400 mb-1">Pattern</label>
                        <input id="regex-pat-modal" type="text" bind:value={newRegexPattern} class="w-full p-2 font-mono text-xs bg-slate-950 border border-slate-800 rounded-xl text-indigo-300" />
                    </div>
                    <div>
                        <label for="regex-rep-modal" class="block text-xs text-slate-400 mb-1">Replacement</label>
                        <input id="regex-rep-modal" type="text" bind:value={newRegexReplacement} class="w-full p-2 font-mono text-xs bg-slate-950 border border-slate-800 rounded-xl text-emerald-300" />
                    </div>
                </div>
            {/if}
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

<!-- Version Note Modal -->
{#if showVersionModal}
    <div class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 backdrop-blur-sm">
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 w-80 space-y-4 shadow-2xl text-slate-100">
            <h3 class="font-bold text-base text-white flex items-center space-x-2">
                <span>🏷️</span>
                <span>Version sichern</span>
            </h3>
            <p class="text-xs text-slate-400">Speichert den aktuellen Stand als neue Version mit einem optionalen Kommentar.</p>
            <div>
                <label for="version-note" class="block text-xs font-semibold text-slate-400 mb-1">Kommentar (optional)</label>
                <input
                    id="version-note"
                    type="text"
                    bind:value={versionNote}
                    placeholder="z.B. Neues Fehlerhandling hinzugefügt"
                    class="w-full px-3 py-2 text-sm bg-slate-950 border border-slate-800 rounded-xl outline-none focus:ring-1 focus:ring-indigo-500 text-white"
                    onkeydown={(e) => e.key === 'Enter' && saveVersion()}
                />
            </div>
            <div class="flex justify-end space-x-2">
                <button
                    onclick={() => showVersionModal = false}
                    class="px-4 py-2 text-xs bg-slate-800 text-slate-300 rounded-xl hover:bg-slate-700"
                >
                    Abbrechen
                </button>
                <button
                    onclick={saveVersion}
                    class="px-4 py-2 text-xs bg-indigo-600 text-white rounded-xl hover:bg-indigo-500 font-semibold"
                >
                    Version sichern
                </button>
            </div>
        </div>
    </div>
{/if}
