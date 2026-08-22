<script lang="ts">
    import { onMount } from 'svelte';
    import { getDatabaseStats, getAllSettings, setSetting, type DatabaseStats } from '../../lib/ipc/session';
    import { exportData, importData, previewImport, type ImportPreviewDto } from '../../lib/ipc/import-export';
    import AutomationRuleManager from '$lib/components/automation/AutomationRuleManager.svelte';
    import IgnoreRulesManager from '$lib/components/search/IgnoreRulesManager.svelte';

    let stats = $state<DatabaseStats | null>(null);
    let settings = $state<Record<string, string>>({});
    let statusMsg = $state('');

    // Import Modal State
    let showImportModal = $state(false);
    let importFilePath = $state('');
    let importPreviewData = $state<ImportPreviewDto | null>(null);
    let conflictPolicy = $state<'skip' | 'overwrite' | 'rename'>('skip');
    let isImporting = $state(false);

    onMount(async () => {
        await reloadData();
    });

    async function reloadData() {
        try {
            stats = await getDatabaseStats();
            settings = await getAllSettings();
        } catch (e: any) {
            console.error('Failed to load settings data', e);
        }
    }

    async function updateSetting(key: string, value: string) {
        try {
            await setSetting(key, value);
            settings[key] = value;
            statusMsg = 'Einstellungen gespeichert.';
            setTimeout(() => statusMsg = '', 3000);
        } catch (e: any) {
            statusMsg = 'Fehler beim Speichern.';
        }
    }

    function formatBytes(bytes: number = 0) {
        if (!bytes || bytes <= 0) return '0 B';
        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
    }

    async function handleFileSelected(file: File) {
        const filePath = (file as any).path || file.name;
        importFilePath = filePath;
        try {
            importPreviewData = await previewImport(filePath);
            showImportModal = true;
        } catch (e: any) {
            // Fallback preview if archive metadata preview fails
            importPreviewData = {
                snippetCount: 0,
                scriptCount: 0,
                pipelineCount: 0,
                folderCount: 0,
                createdAt: Date.now()
            };
            showImportModal = true;
        }
    }

    async function confirmImport() {
        if (!importFilePath) return;
        isImporting = true;
        try {
            const res = await importData({ sourcePath: importFilePath, conflictPolicy });
            statusMsg = `Import erfolgreich: ${res.snippetsImported} Snippets, ${res.scriptsImported} Skripte, ${res.pipelinesImported} Pipelines.`;
            showImportModal = false;
            await reloadData();
            setTimeout(() => statusMsg = '', 5000);
        } catch (err: any) {
            statusMsg = 'Import fehlgeschlagen: ' + err;
        } finally {
            isImporting = false;
        }
    }
</script>

<div class="h-full flex flex-col p-6 space-y-6 bg-slate-950 text-slate-100 overflow-y-auto custom-scrollbar">
    <div class="flex justify-between items-center">
        <div>
            <h1 class="text-2xl font-extrabold tracking-tight text-white flex items-center space-x-2">
                <span>⚙️</span>
                <span>Einstellungen & Statistik</span>
            </h1>
            <p class="text-xs text-slate-400 mt-1">System-Konfiguration & Datenbank-Übersicht</p>
        </div>
        {#if statusMsg}
            <span class="text-xs text-emerald-400 font-mono bg-emerald-950/80 px-3 py-1.5 rounded-xl border border-emerald-800">
                {statusMsg}
            </span>
        {/if}
    </div>

    <!-- Database Stats Grid -->
    <div class="grid grid-cols-5 gap-4">
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Text Snippets</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.totalSnippets ?? stats?.snippetsCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Zwischenablage</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.totalClipboardEntries ?? stats?.clipboardEntriesCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Skripte</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.totalScripts ?? stats?.scriptsCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Pipelines</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.totalPipelines ?? stats?.pipelinesCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">DB-Größe</span>
            <span class="text-3xl font-extrabold text-emerald-400 mt-2 font-mono">{formatBytes(stats?.dbSizeBytes ?? stats?.databaseSizeBytes ?? 0)}</span>
        </div>
    </div>

    <!-- Settings Forms -->
    <div class="grid grid-cols-2 gap-6">
        <!-- Clipboard Settings -->
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 space-y-4 backdrop-blur-md">
            <h3 class="font-bold text-sm text-white flex items-center space-x-2">
                <span>📋</span>
                <span>Zwischenablage Monitor</span>
            </h3>
            
            <div class="space-y-3 text-xs">
                <div>
                    <label for="set-clip-max" class="text-slate-400 block mb-1">Max. Historie Einträge</label>
                    <input 
                        id="set-clip-max"
                        type="number" 
                        value={settings['clipboard.max_entries'] ?? '500'}
                        onchange={(e) => updateSetting('clipboard.max_entries', (e.target as HTMLInputElement).value)}
                        class="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white font-mono"
                    />
                </div>
                <div>
                    <label for="set-clip-min" class="text-slate-400 block mb-1">Mindestlänge (Zeichen)</label>
                    <input 
                        id="set-clip-min"
                        type="number" 
                        value={settings['clipboard.min_length'] ?? '1'}
                        onchange={(e) => updateSetting('clipboard.min_length', (e.target as HTMLInputElement).value)}
                        class="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white font-mono"
                    />
                </div>
            </div>
        </div>

        <!-- UI & Editor Settings -->
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-5 space-y-4 backdrop-blur-md">
            <h3 class="font-bold text-sm text-white flex items-center space-x-2">
                <span>🎨</span>
                <span>Benutzeroberfläche & Editor</span>
            </h3>
            
            <div class="space-y-3 text-xs">
                <div>
                    <label for="set-ui-theme" class="text-slate-400 block mb-1">Farbschema</label>
                    <select 
                        id="set-ui-theme"
                        value={settings['ui.theme'] ?? 'dark'}
                        onchange={(e) => updateSetting('ui.theme', (e.target as HTMLSelectElement).value)}
                        class="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white"
                    >
                        <option value="dark">Dark Theme (Standard)</option>
                        <option value="light">Light Theme</option>
                    </select>
                </div>
                <div>
                    <label for="set-ui-diff" class="text-slate-400 block mb-1">Diff Anzeige Modus</label>
                    <select 
                        id="set-ui-diff"
                        value={settings['ui.diff_mode'] ?? 'split'}
                        onchange={(e) => updateSetting('ui.diff_mode', (e.target as HTMLSelectElement).value)}
                        class="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white"
                    >
                        <option value="split">Nebeneinander (Split)</option>
                        <option value="unified">Kombiniert (Unified)</option>
                    </select>
                </div>
            </div>
        </div>

        <!-- Ignore Rules Section -->
        <div class="col-span-2">
            <IgnoreRulesManager />
        </div>

        <!-- Automation Script Engine Section -->
        <div class="col-span-2">
            <AutomationRuleManager />
        </div>

        <!-- Import / Export Section -->
        <div class="col-span-2 bg-slate-900/60 border border-slate-800 rounded-2xl p-5 space-y-4 backdrop-blur-md">
            <h3 class="font-bold text-sm text-white flex items-center space-x-2">
                <span>📦</span>
                <span>Sicherung & Daten-Export / Import</span>
            </h3>
            <p class="text-xs text-slate-400">Exportiere deine Snippets, Skripte & Pipelines als Backup-Bundle oder stelle Daten aus einer Sicherung wieder her.</p>

            <div class="flex items-center space-x-4 pt-2">
                <button
                    onclick={async () => {
                        try {
                            const fileName = prompt("Ziel-Dateiname für Export:", "textforge_backup.tfbundle");
                            if (!fileName) return;
                            const res = await exportData({ exportType: 'full', format: 'tfbundle', targetPath: fileName });
                            statusMsg = `Export erfolgreich: ${res.exportedCount} Objekte nach ${res.filePath}`;
                            setTimeout(() => statusMsg = '', 5000);
                        } catch (e: any) {
                            statusMsg = 'Export fehlgeschlagen: ' + e;
                        }
                    }}
                    class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs rounded-xl transition-all flex items-center space-x-2 shadow-lg shadow-indigo-600/20"
                >
                    <span>📥</span>
                    <span>Alles Exportieren (.tfbundle)</span>
                </button>

                <label class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold text-xs rounded-xl transition-all cursor-pointer flex items-center space-x-2 border border-slate-700">
                    <span>📤</span>
                    <span>Backup Importieren...</span>
                    <input 
                        type="file" 
                        accept=".tfbundle,.json" 
                        class="hidden"
                        onchange={(e) => {
                            const file = (e.target as HTMLInputElement).files?.[0];
                            if (file) handleFileSelected(file);
                        }}
                    />
                </label>
            </div>
        </div>
    </div>
</div>

<!-- Import Preview & Conflict Strategy Modal -->
{#if showImportModal}
    <div class="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm flex items-center justify-center p-4">
        <div class="bg-slate-900 border border-slate-800 rounded-2xl p-6 max-w-md w-full space-y-5 shadow-2xl">
            <div class="flex justify-between items-center">
                <h3 class="font-bold text-lg text-white flex items-center space-x-2">
                    <span>📦</span>
                    <span>Backup Import Vorschau</span>
                </h3>
                <button
                    onclick={() => showImportModal = false}
                    class="text-slate-400 hover:text-white text-sm"
                >
                    ✕
                </button>
            </div>

            <div class="space-y-3 text-xs bg-slate-950/60 p-3.5 rounded-xl border border-slate-800/80">
                <div class="flex justify-between text-slate-400">
                    <span>Datei:</span>
                    <span class="font-mono text-slate-200 truncate max-w-[200px]">{importFilePath}</span>
                </div>
                {#if importPreviewData}
                    <div class="grid grid-cols-2 gap-2 pt-2 border-t border-slate-800/60 text-slate-300">
                        <div>📝 Snippets: <span class="font-mono font-bold text-indigo-400">{importPreviewData.snippetCount}</span></div>
                        <div>⚡ Skripte: <span class="font-mono font-bold text-indigo-400">{importPreviewData.scriptCount}</span></div>
                        <div>🔄 Pipelines: <span class="font-mono font-bold text-indigo-400">{importPreviewData.pipelineCount}</span></div>
                        <div>📁 Ordner: <span class="font-mono font-bold text-indigo-400">{importPreviewData.folderCount}</span></div>
                    </div>
                {/if}
            </div>

            <div class="space-y-2 text-xs">
                <label for="conflict-policy-select" class="block font-semibold text-slate-300">Konflikt-Strategie bei Duplikaten:</label>
                <select
                    id="conflict-policy-select"
                    bind:value={conflictPolicy}
                    class="w-full bg-slate-950 border border-slate-800 rounded-xl p-2.5 text-white"
                >
                    <option value="skip">Duplikate überspringen (Empfohlen)</option>
                    <option value="overwrite">Existierende Einträge überschreiben</option>
                    <option value="rename">Mit neuem Namen/ID duplizieren</option>
                </select>
            </div>

            <div class="flex justify-end space-x-3 pt-2">
                <button
                    onclick={() => showImportModal = false}
                    class="px-4 py-2 text-xs bg-slate-800 hover:bg-slate-700 text-slate-300 rounded-xl font-medium transition-colors"
                >
                    Abbrechen
                </button>
                <button
                    onclick={confirmImport}
                    disabled={isImporting}
                    class="px-4 py-2 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-xl font-semibold transition-colors shadow-lg shadow-indigo-600/20 flex items-center space-x-1.5"
                >
                    {#if isImporting}
                        <span>Wird importiert...</span>
                    {:else}
                        <span>Importieren</span>
                    {/if}
                </button>
            </div>
        </div>
    </div>
{/if}
