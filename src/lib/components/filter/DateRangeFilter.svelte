<script lang="ts">
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let dateField = $state<'createdAt' | 'updatedAt'>('updatedAt');
    let selectedPreset = $state<string>('all');
    let customFrom = $state<string>('');
    let customTo = $state<string>('');

    $effect(() => {
        const storeField = ($snippetFilterStore.dateField || 'updatedAt') as 'createdAt' | 'updatedAt';
        if (storeField !== dateField) {
            dateField = storeField;
        }
        const storeRange = $snippetFilterStore.dateRange;
        if (!storeRange) {
            selectedPreset = 'all';
        } else if (storeRange.preset) {
            selectedPreset = storeRange.preset;
        } else {
            selectedPreset = 'custom';
        }
    });

    function startOfDay() {
        const d = new Date();
        d.setHours(0, 0, 0, 0);
        return d.getTime();
    }

    function setPreset(preset: string) {
        selectedPreset = preset;
        let from: number | null = null;
        let to: number | null = null;

        if (preset === 'today') {
            from = startOfDay();
        } else if (preset === 'week') {
            from = startOfDay() - 7 * 24 * 60 * 60 * 1000;
        } else if (preset === 'month') {
            const d = new Date();
            d.setMonth(d.getMonth() - 1);
            d.setHours(0, 0, 0, 0);
            from = d.getTime();
        } else if (preset === 'custom') {
            applyCustomRange();
            return;
        }

        snippetFilterStore.update(f => ({
            ...f,
            dateField,
            dateRange: preset === 'all' ? null : { from, to, preset }
        }));
        loadSnippets();
    }

    function applyCustomRange() {
        const from = customFrom ? new Date(customFrom).getTime() : null;
        const to = customTo ? new Date(customTo).getTime() : null;
        snippetFilterStore.update(f => ({
            ...f,
            dateField,
            dateRange: { from, to, preset: 'custom' }
        }));
        loadSnippets();
    }

    function toggleField() {
        dateField = dateField === 'createdAt' ? 'updatedAt' : 'createdAt';
        snippetFilterStore.update(f => ({
            ...f,
            dateField
        }));
        loadSnippets();
    }
</script>

<div class="space-y-3">
    <div class="flex justify-between items-center">
        <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Zeitraum</h3>
        <button
            onclick={toggleField}
            class="px-2 py-0.5 text-[9px] font-bold font-mono rounded border border-slate-800 bg-slate-900 text-slate-300 hover:text-white"
        >
            {dateField === 'createdAt' ? 'Erstellt' : 'Geändert'}
        </button>
    </div>

    <div class="grid grid-cols-2 gap-1">
        <button
            onclick={() => setPreset('all')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'all' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
        >
            Immer
        </button>
        <button
            onclick={() => setPreset('today')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'today' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
        >
            Heute
        </button>
        <button
            onclick={() => setPreset('week')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'week' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
        >
            7 Tage
        </button>
        <button
            onclick={() => setPreset('month')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'month' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
        >
            Monat
        </button>
    </div>

    <button
        onclick={() => setPreset('custom')}
        class="w-full px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'custom' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
    >
        Benutzerdefiniert...
    </button>

    {#if selectedPreset === 'custom'}
        <div class="space-y-2 pt-1">
            <div class="grid grid-cols-2 gap-2">
                <div>
                    <label for="date-from" class="block text-[9px] uppercase font-mono text-slate-500 mb-0.5">Von</label>
                    <input
                        id="date-from"
                        type="date"
                        bind:value={customFrom}
                        onchange={applyCustomRange}
                        class="w-full px-2 py-1 text-[10px] bg-slate-950/80 border border-slate-850 rounded text-slate-100 outline-none"
                    />
                </div>
                <div>
                    <label for="date-to" class="block text-[9px] uppercase font-mono text-slate-500 mb-0.5">Bis</label>
                    <input
                        id="date-to"
                        type="date"
                        bind:value={customTo}
                        onchange={applyCustomRange}
                        class="w-full px-2 py-1 text-[10px] bg-slate-950/80 border border-slate-850 rounded text-slate-100 outline-none"
                    />
                </div>
            </div>
        </div>
    {/if}
</div>
