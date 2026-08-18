<script lang="ts">
    import { onMount } from 'svelte';
    import { listScriptVersions, restoreScriptVersion } from '../../ipc/scripts';
    import type { ScriptVersion } from '../../domain/script';
    import { pushNotification, Notifications } from '../../stores/notifications';
    import { Option } from '../../domain/adts';

    let {
        scriptId = '',
        onRestore = () => {}
    } = $props();

    let versions = $state<ScriptVersion[]>([]);
    let loading = $state(false);
    let expandedVersion = $state<number | null>(null);

    $effect(() => {
        if (scriptId) {
            loadVersions();
        }
    });

    async function loadVersions() {
        if (!scriptId) return;
        loading = true;
        try {
            versions = await listScriptVersions(scriptId);
        } catch (e) {
            console.error('Failed to load script versions:', e);
        } finally {
            loading = false;
        }
    }

    async function handleRestore(version: number) {
        try {
            await restoreScriptVersion(scriptId, version);
            pushNotification({ id: crypto.randomUUID(), severity: 'success', title: 'Version wiederhergestellt', message: Option.some(`Version ${version} wurde wiederhergestellt.`), duration: 2000, action: Option.none(), createdAt: Date.now() as any });
            onRestore();
        } catch (e) {
            console.error('Failed to restore version:', e);
            pushNotification({ id: crypto.randomUUID(), severity: 'error', title: 'Fehler', message: Option.some('Version konnte nicht wiederhergestellt werden.'), duration: 4000, action: Option.none(), createdAt: Date.now() as any });
        }
    }

    function formatDate(ms: number): string {
        return new Intl.DateTimeFormat('de', {
            day: '2-digit',
            month: '2-digit',
            year: '2-digit',
            hour: '2-digit',
            minute: '2-digit'
        }).format(new Date(ms));
    }

    function toggleExpand(version: number) {
        expandedVersion = expandedVersion === version ? null : version;
    }
</script>

<div class="space-y-2">
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
        <h4 class="text-xs font-bold font-mono tracking-widest text-indigo-400 uppercase">Versionshistorie</h4>
        <button
            onclick={loadVersions}
            class="text-[10px] text-slate-500 hover:text-slate-300 transition-colors px-1.5 py-0.5 rounded bg-slate-900/60 hover:bg-slate-800"
            title="Aktualisieren"
        >
            ↻
        </button>
    </div>

    {#if loading}
        <div class="flex items-center justify-center py-4">
            <span class="text-slate-500 text-xs animate-pulse">Lade Versionen...</span>
        </div>
    {:else if versions.length === 0}
        <div class="py-4 text-center">
            <p class="text-xs text-slate-500 italic">Keine gespeicherten Versionen vorhanden.</p>
            <p class="text-[10px] text-slate-600 mt-1">Speichere eine Version über den Button „Version sichern".</p>
        </div>
    {:else}
        <div class="space-y-1.5 max-h-64 overflow-y-auto pr-1">
            {#each versions as v (v.version)}
                <div class="rounded-xl border border-slate-800 bg-slate-900/50 overflow-hidden transition-all">
                    <!-- Version header -->
                    <button
                        onclick={() => toggleExpand(v.version)}
                        class="w-full flex items-center justify-between px-3 py-2.5 text-left hover:bg-slate-800/50 transition-colors"
                    >
                        <div class="flex items-center space-x-3 min-w-0">
                            <span class="text-[10px] font-bold font-mono bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 px-1.5 py-0.5 rounded shrink-0">
                                v{v.version}
                            </span>
                            <span class="text-[11px] text-slate-400 truncate">
                                {v.changeNote || 'Kein Kommentar'}
                            </span>
                        </div>
                        <div class="flex items-center space-x-2 shrink-0 ml-2">
                            <span class="text-[10px] text-slate-500 font-mono">{formatDate(v.savedAt)}</span>
                            <span class="text-[10px] text-slate-600 transition-transform duration-200 {expandedVersion === v.version ? 'rotate-180' : ''}">▼</span>
                        </div>
                    </button>

                    <!-- Expanded code preview + restore -->
                    {#if expandedVersion === v.version}
                        <div class="border-t border-slate-800 px-3 py-3 space-y-3 bg-slate-950/50">
                            {#if v.jsCode}
                                <div>
                                    <span class="text-[10px] text-slate-500 uppercase font-mono">JS Code</span>
                                    <pre class="mt-1 p-2.5 bg-slate-950 rounded-lg text-[11px] font-mono text-emerald-400 overflow-x-auto max-h-28 overflow-y-auto leading-relaxed whitespace-pre-wrap border border-slate-900">{v.jsCode}</pre>
                                </div>
                            {:else if v.regexPattern}
                                <div class="space-y-1.5">
                                    <span class="text-[10px] text-slate-500 uppercase font-mono">Regex</span>
                                    <div class="space-y-1">
                                        <div class="flex items-center space-x-2 p-2 bg-slate-950 rounded-lg border border-slate-900">
                                            <span class="text-[10px] text-slate-500 w-16 shrink-0 font-mono">Pattern:</span>
                                            <code class="text-indigo-300 text-[11px] font-mono truncate">{v.regexPattern}</code>
                                        </div>
                                        {#if v.regexReplacement !== undefined}
                                            <div class="flex items-center space-x-2 p-2 bg-slate-950 rounded-lg border border-slate-900">
                                                <span class="text-[10px] text-slate-500 w-16 shrink-0 font-mono">Replace:</span>
                                                <code class="text-emerald-300 text-[11px] font-mono truncate">{v.regexReplacement}</code>
                                            </div>
                                        {/if}
                                    </div>
                                </div>
                            {/if}

                            <div class="flex justify-end">
                                <button
                                    onclick={() => handleRestore(v.version)}
                                    class="px-3 py-1.5 text-[11px] font-semibold bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-all shadow-sm"
                                >
                                    Wiederherstellen
                                </button>
                            </div>
                        </div>
                    {/if}
                </div>
            {/each}
        </div>
    {/if}
</div>
