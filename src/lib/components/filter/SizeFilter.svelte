<script lang="ts">
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let selectedPreset = $state<string>('all');
    let customMin = $state<number | null>(null);
    let customMax = $state<number | null>(null);

    $effect(() => {
        const storeRange = $snippetFilterStore.sizeRange;
        if (!storeRange) {
            selectedPreset = 'all';
        }
    });

    function setPreset(preset: string) {
        selectedPreset = preset;
        let min: number | null = null;
        let max: number | null = null;

        if (preset === 'tiny') {
            max = 1024;
        } else if (preset === 'small') {
            min = 1024;
            max = 10 * 1024;
        } else if (preset === 'medium') {
            min = 10 * 1024;
            max = 100 * 1024;
        } else if (preset === 'large') {
            min = 100 * 1024;
        } else if (preset === 'custom') {
            applyCustomRange();
            return;
        }

        snippetFilterStore.update(f => ({
            ...f,
            sizeRange: preset === 'all' ? null : { min, max }
        }));
        loadSnippets();
    }

    function applyCustomRange() {
        snippetFilterStore.update(f => ({
            ...f,
            sizeRange: { min: customMin, max: customMax }
        }));
        loadSnippets();
    }
</script>

<div class="space-y-3">
    <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Größe</h3>

    <div class="grid grid-cols-2 gap-1">
        <button
            onclick={() => setPreset('all')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'all' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
        >
            Alle
        </button>
        <button
            onclick={() => setPreset('tiny')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'tiny' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            title="< 1 KB"
        >
            Tiny (&lt;1KB)
        </button>
        <button
            onclick={() => setPreset('small')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'small' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            title="1KB - 10KB"
        >
            Small
        </button>
        <button
            onclick={() => setPreset('medium')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'medium' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            title="10KB - 100KB"
        >
            Medium
        </button>
    </div>

    <div class="grid grid-cols-2 gap-1">
        <button
            onclick={() => setPreset('large')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'large' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            title="> 100KB"
        >
            Large (&gt;100K)
        </button>
        <button
            onclick={() => setPreset('custom')}
            class="px-2 py-1 text-[11px] font-medium rounded-lg border text-center transition-all {selectedPreset === 'custom' ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
        >
            Custom...
        </button>
    </div>

    {#if selectedPreset === 'custom'}
        <div class="flex items-center space-x-2 pt-1">
            <input
                type="number"
                placeholder="Min (B)"
                bind:value={customMin}
                oninput={applyCustomRange}
                class="w-full px-2.5 py-1 text-[10px] bg-slate-950/80 border border-slate-850 rounded text-slate-100 outline-none"
            />
            <span class="text-slate-600 text-xs">-</span>
            <input
                type="number"
                placeholder="Max (B)"
                bind:value={customMax}
                oninput={applyCustomRange}
                class="w-full px-2.5 py-1 text-[10px] bg-slate-950/80 border border-slate-850 rounded text-slate-100 outline-none"
            />
        </div>
    {/if}
</div>
