<script lang="ts">
    import { onMount } from 'svelte';
    import { getDatabaseStats, getAllSettings, setSetting, type DatabaseStats } from '../../lib/ipc/session';

    let stats = $state<DatabaseStats | null>(null);
    let settings = $state<Record<string, string>>({});
    let statusMsg = $state('');

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
    <div class="grid grid-cols-4 gap-4">
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Text Snippets</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.snippetsCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Zwischenablage</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.clipboardEntriesCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Skripte</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.scriptsCount ?? 0}</span>
        </div>
        <div class="bg-slate-900/60 border border-slate-800 rounded-2xl p-4 flex flex-col justify-between backdrop-blur-md">
            <span class="text-xs text-slate-400 font-medium">Pipelines</span>
            <span class="text-3xl font-extrabold text-indigo-400 mt-2 font-mono">{stats?.pipelinesCount ?? 0}</span>
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

        <!-- Import / Export Section -->
        <div class="col-span-2 bg-slate-900/60 border border-slate-800 rounded-2xl p-5 space-y-4 backdrop-blur-md">
            <h3 class="font-bold text-sm text-white flex items-center space-x-2">
                <span>📦</span>
                <span>Sicherung & Daten-Export / Import</span>
            </h3>
            <p class="text-xs text-slate-400">Exportiere deine Snippets, Skripte & Pipelines als Backup-Bundle oder stelle Daten wieder her.</p>

            <div class="flex items-center space-x-4 pt-2">
                <button
                    onclick={async () => {
                        try {
                            const { exportData } = await import('../../lib/ipc/import-export');
                            const res = await exportData({ exportType: 'full', format: 'tfbundle', targetPath: 'textforge_export.tfbundle' });
                            statusMsg = `Export erfolgreich: ${res.exportedCount} Objekte nach ${res.filePath}`;
                            setTimeout(() => statusMsg = '', 5000);
                        } catch (e: any) {
                            statusMsg = 'Export fehlgeschlagen: ' + e;
                        }
                    }}
                    class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 text-white font-semibold text-xs rounded-xl transition-all flex items-center space-x-2"
                >
                    <span>📥</span>
                    <span>Alles Exportieren (.tfbundle)</span>
                </button>

                <label class="px-4 py-2 bg-slate-800 hover:bg-slate-700 text-slate-200 font-semibold text-xs rounded-xl transition-all cursor-pointer flex items-center space-x-2">
                    <span>📤</span>
                    <span>Backup Importieren...</span>
                    <input 
                        type="file" 
                        accept=".tfbundle,.json" 
                        class="hidden"
                        onchange={async (e) => {
                            const file = (e.target as HTMLInputElement).files?.[0];
                            if (!file) return;
                            try {
                                const { importData } = await import('../../lib/ipc/import-export');
                                const res = await importData({ sourcePath: (file as any).path || file.name, conflictPolicy: 'skip' });
                                statusMsg = `Import erfolgreich: ${res.snippetsImported} Snippets, ${res.scriptsImported} Skripte.`;
                                await reloadData();
                                setTimeout(() => statusMsg = '', 5000);
                            } catch (err: any) {
                                statusMsg = 'Import fehlgeschlagen.';
                            }
                        }}
                    />
                </label>
            </div>
        </div>
    </div>
</div>
