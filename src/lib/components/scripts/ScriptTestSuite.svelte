<script lang="ts">
    import { executeScript, type ScriptExecutionResultDto } from '../../ipc/transform';
    import DiffViewer from '../diff/DiffViewer.svelte';

    interface TestCase {
        id: number;
        input: string;
        expectedOutput: string;
        actualOutput: string | null;
        status: 'pending' | 'pass' | 'fail' | 'error';
        errorMessage: string | null;
        executionTimeMs: number | null;
    }

    let {
        scriptId = '',
        jsCode = null as string | null
    } = $props();

    let testCases = $state<TestCase[]>([
        { id: 1, input: 'Hallo Welt! 123', expectedOutput: '', actualOutput: null, status: 'pending', errorMessage: null, executionTimeMs: null }
    ]);
    let nextId = $state(2);
    let running = $state(false);
    let expandedDiff = $state<number | null>(null);

    function addTestCase() {
        testCases = [...testCases, {
            id: nextId,
            input: '',
            expectedOutput: '',
            actualOutput: null,
            status: 'pending',
            errorMessage: null,
            executionTimeMs: null
        }];
        nextId++;
    }

    function removeTestCase(id: number) {
        testCases = testCases.filter(t => t.id !== id);
    }

    function resetTestCase(tc: TestCase): TestCase {
        return { ...tc, actualOutput: null, status: 'pending', errorMessage: null, executionTimeMs: null };
    }

    async function runSingle(id: number) {
        const tc = testCases.find(t => t.id === id);
        if (!tc) return;

        testCases = testCases.map(t => t.id === id ? { ...resetTestCase(t), status: 'pending' } : t);

        try {
            const res: ScriptExecutionResultDto = await executeScript({
                scriptId: scriptId || undefined,
                jsCode: jsCode || undefined,
                input: tc.input
            });

            const actualOutput = res.error ? null : res.output;
            const errorMessage = res.error || null;
            const passed = !res.error && actualOutput === tc.expectedOutput;

            testCases = testCases.map(t => t.id === id ? {
                ...t,
                actualOutput,
                status: res.error ? 'error' : (passed ? 'pass' : 'fail'),
                errorMessage,
                executionTimeMs: res.executionTimeMs
            } : t);
        } catch (e: any) {
            testCases = testCases.map(t => t.id === id ? {
                ...t,
                actualOutput: null,
                status: 'error',
                errorMessage: String(e),
                executionTimeMs: null
            } : t);
        }
    }

    async function runAll() {
        running = true;
        testCases = testCases.map(resetTestCase);
        for (const tc of testCases) {
            await runSingle(tc.id);
        }
        running = false;
    }

    let summary = $derived({
        total: testCases.length,
        pass: testCases.filter(t => t.status === 'pass').length,
        fail: testCases.filter(t => t.status === 'fail').length,
        error: testCases.filter(t => t.status === 'error').length,
    });
</script>

<div class="space-y-4">
    <!-- Header row -->
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
        <div class="flex items-center space-x-3">
            <h4 class="text-xs font-bold font-mono tracking-widest text-indigo-400 uppercase">Test Suite</h4>
            {#if summary.total > 0}
                <div class="flex items-center space-x-1.5 text-[10px] font-mono">
                    {#if summary.pass > 0}<span class="px-1.5 py-0.5 rounded bg-emerald-900/40 text-emerald-400 border border-emerald-700/40">✓ {summary.pass}</span>{/if}
                    {#if summary.fail > 0}<span class="px-1.5 py-0.5 rounded bg-rose-900/40 text-rose-400 border border-rose-700/40">✗ {summary.fail}</span>{/if}
                    {#if summary.error > 0}<span class="px-1.5 py-0.5 rounded bg-amber-900/40 text-amber-400 border border-amber-700/40">⚠ {summary.error}</span>{/if}
                </div>
            {/if}
        </div>
        <div class="flex space-x-2">
            <button
                onclick={addTestCase}
                class="px-2.5 py-1 text-[11px] font-semibold bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-lg transition-colors"
            >
                + Test hinzufügen
            </button>
            <button
                onclick={runAll}
                disabled={running || (!scriptId && !jsCode)}
                class="px-3 py-1 text-[11px] font-semibold bg-emerald-700 hover:bg-emerald-600 disabled:opacity-50 text-white rounded-lg transition-all flex items-center space-x-1"
            >
                {#if running}
                    <span class="animate-spin text-xs">⟳</span>
                {:else}
                    <span>▶</span>
                {/if}
                <span>Alle testen</span>
            </button>
        </div>
    </div>

    <!-- Test cases list -->
    <div class="space-y-3 max-h-96 overflow-y-auto pr-1">
        {#each testCases as tc (tc.id)}
            <div class="rounded-xl border bg-slate-950/60 overflow-hidden transition-all {
                tc.status === 'pass' ? 'border-emerald-700/40' :
                tc.status === 'fail' ? 'border-rose-700/40' :
                tc.status === 'error' ? 'border-amber-700/40' :
                'border-slate-800'
            }">
                <!-- Test case header -->
                <div class="flex items-center justify-between px-3 py-2 bg-slate-900/40 border-b border-slate-800/60">
                    <div class="flex items-center space-x-2">
                        {#if tc.status === 'pass'}
                            <span class="text-emerald-400 text-sm font-bold">✓</span>
                        {:else if tc.status === 'fail'}
                            <span class="text-rose-400 text-sm font-bold">✗</span>
                        {:else if tc.status === 'error'}
                            <span class="text-amber-400 text-sm font-bold">⚠</span>
                        {:else}
                            <span class="text-slate-600 text-sm">○</span>
                        {/if}
                        <span class="text-[11px] font-mono text-slate-400">Test #{tc.id}</span>
                        {#if tc.executionTimeMs !== null}
                            <span class="text-[10px] text-slate-600 font-mono">{tc.executionTimeMs}ms</span>
                        {/if}
                    </div>
                    <div class="flex items-center space-x-1.5">
                        <button
                            onclick={() => runSingle(tc.id)}
                            disabled={!scriptId && !jsCode}
                            class="text-[10px] px-2 py-0.5 bg-indigo-700/50 hover:bg-indigo-600 text-indigo-200 rounded disabled:opacity-40 transition-colors"
                        >
                            ▶ Run
                        </button>
                        {#if tc.status === 'fail' && tc.actualOutput !== null}
                            <button
                                onclick={() => expandedDiff = expandedDiff === tc.id ? null : tc.id}
                                class="text-[10px] px-2 py-0.5 bg-slate-800 hover:bg-slate-700 text-slate-300 rounded transition-colors"
                            >
                                Diff {expandedDiff === tc.id ? '▲' : '▼'}
                            </button>
                        {/if}
                        {#if testCases.length > 1}
                            <button
                                onclick={() => removeTestCase(tc.id)}
                                class="text-[10px] px-1.5 py-0.5 text-slate-600 hover:text-rose-400 transition-colors"
                                title="Löschen"
                            >
                                ✕
                            </button>
                        {/if}
                    </div>
                </div>

                <!-- Input / Expected -->
                <div class="grid grid-cols-2 gap-3 p-3">
                    <div>
                        <label for="tc-input-{tc.id}" class="block text-[10px] font-semibold text-slate-500 uppercase mb-1 font-mono">Eingabe</label>
                        <textarea
                            id="tc-input-{tc.id}"
                            rows="3"
                            value={tc.input}
                            oninput={(e) => {
                                const val = (e.target as HTMLTextAreaElement).value;
                                testCases = testCases.map(t => t.id === tc.id ? { ...t, input: val, status: 'pending' } : t);
                            }}
                            class="w-full p-2 font-mono text-[11px] bg-slate-950 border border-slate-800 rounded-lg text-slate-200 resize-none focus:border-indigo-500 outline-none transition-all leading-relaxed"
                            placeholder="Eingabetext..."
                        ></textarea>
                    </div>
                    <div>
                        <label for="tc-expected-{tc.id}" class="block text-[10px] font-semibold text-slate-500 uppercase mb-1 font-mono">Erwartete Ausgabe</label>
                        <textarea
                            id="tc-expected-{tc.id}"
                            rows="3"
                            value={tc.expectedOutput}
                            oninput={(e) => {
                                const val = (e.target as HTMLTextAreaElement).value;
                                testCases = testCases.map(t => t.id === tc.id ? { ...t, expectedOutput: val, status: 'pending' } : t);
                            }}
                            class="w-full p-2 font-mono text-[11px] bg-slate-950 border border-slate-800 rounded-lg text-emerald-300 resize-none focus:border-indigo-500 outline-none transition-all leading-relaxed"
                            placeholder="Erwarteter Output..."
                        ></textarea>
                    </div>
                </div>

                <!-- Actual output or error -->
                {#if tc.status !== 'pending'}
                    <div class="px-3 pb-3">
                        {#if tc.status === 'error'}
                            <div class="p-2 bg-amber-950/20 border border-amber-700/30 rounded-lg">
                                <span class="text-[10px] font-mono text-amber-400 font-bold uppercase">Fehler:</span>
                                <pre class="mt-1 text-[11px] text-amber-300 font-mono whitespace-pre-wrap">{tc.errorMessage}</pre>
                            </div>
                        {:else if tc.actualOutput !== null}
                            <div>
                                <span class="text-[10px] font-semibold text-slate-500 uppercase font-mono">Tatsächliche Ausgabe:</span>
                                <pre class="mt-1 p-2 bg-slate-950 rounded-lg text-[11px] font-mono {tc.status === 'pass' ? 'text-emerald-400' : 'text-rose-300'} whitespace-pre-wrap border border-slate-900 leading-relaxed">{tc.actualOutput}</pre>
                            </div>
                        {/if}
                    </div>
                {/if}

                <!-- Diff expanded view for failures -->
                {#if expandedDiff === tc.id && tc.actualOutput !== null}
                    <div class="px-3 pb-3 border-t border-slate-800/60 pt-3">
                        <DiffViewer original={tc.expectedOutput} modified={tc.actualOutput} />
                    </div>
                {/if}
            </div>
        {/each}
    </div>
</div>
