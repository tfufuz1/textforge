<script lang="ts">
    import { clipboardFilterStore, loadClipboardHistory } from '../../stores/clipboard';
    import { Option } from '../../domain/adts';
    import SearchIcon from '$lib/components/icons/SearchIcon.svelte';
    import PinIcon from '$lib/components/icons/PinIcon.svelte';
    import FilterIcon from '$lib/components/icons/FilterIcon.svelte';

    let searchQuery = $state('');
    let selectedContentType = $state('');
    let isPinnedOnly = $state(false);

    const contentTypes = [
        { value: '', label: 'Alle Typen' },
        { value: 'plain_text', label: 'Plain Text' },
        { value: 'markdown', label: 'Markdown' },
        { value: 'json', label: 'JSON' },
        { value: 'url', label: 'URL' },
        { value: 'html', label: 'HTML' },
        { value: 'xml', label: 'XML' }
    ];

    async function applyFilters() {
        clipboardFilterStore.update(f => ({
            ...f,
            searchQuery: searchQuery.trim() ? Option.some(searchQuery.trim()) : Option.none(),
            contentTypes: selectedContentType ? [selectedContentType] : [],
            isPinned: isPinnedOnly ? Option.some(true) : Option.none()
        }));
        await loadClipboardHistory(0);
    }

    async function setQuickType(type: string) {
        if (selectedContentType === type) {
            selectedContentType = '';
        } else {
            selectedContentType = type;
        }
        await applyFilters();
    }

    async function resetFilters() {
        searchQuery = '';
        selectedContentType = '';
        isPinnedOnly = false;
        clipboardFilterStore.set({
            searchQuery: Option.none(),
            contentTypes: [],
            sourceApps: [],
            isPinned: Option.none()
        });
        await loadClipboardHistory(0);
    }
</script>

<div class="flex flex-col gap-2.5 w-full">
    <div class="flex flex-col sm:flex-row items-center gap-2.5 w-full">
        <div class="relative flex-1 w-full">
            <input
                type="text"
                class="w-full pl-9 pr-4 py-1.5 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all shadow-inner"
                placeholder="Zwischenablage durchsuchen..."
                bind:value={searchQuery}
                onkeyup={applyFilters}
            />
            <span class="absolute left-3 top-2 text-slate-500 flex items-center justify-center">
                <SearchIcon class="w-3.5 h-3.5" />
            </span>
        </div>

        <select
            class="py-1.5 px-3 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-slate-200 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all cursor-pointer w-full sm:w-auto"
            bind:value={selectedContentType}
            onchange={applyFilters}
        >
            {#each contentTypes as ct}
                <option value={ct.value} class="bg-slate-900 text-slate-200">{ct.label}</option>
            {/each}
        </select>

        <button
            onclick={() => { isPinnedOnly = !isPinnedOnly; applyFilters(); }}
            class="px-3 py-1.5 text-xs font-semibold rounded-xl border transition-all flex items-center gap-1.5 shrink-0 w-full sm:w-auto justify-center {isPinnedOnly ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-950/80 text-slate-400 border-slate-800 hover:text-slate-200'}"
        >
            <PinIcon class="w-3.5 h-3.5" filled={isPinnedOnly} />
            <span>Gepinnt</span>
        </button>

        {#if searchQuery || selectedContentType || isPinnedOnly}
            <button
                onclick={resetFilters}
                class="px-3 py-1.5 text-xs font-medium bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/60 rounded-xl transition-all shrink-0"
            >
                Zurücksetzen
            </button>
        {/if}
    </div>

    <!-- Quick filter chips -->
    <div class="flex items-center gap-1.5 overflow-x-auto pb-0.5 scrollbar-none text-[11px]">
        <span class="text-slate-500 text-[10px] uppercase font-mono tracking-wider font-semibold mr-1 flex items-center gap-1">
            <FilterIcon class="w-3 h-3 text-indigo-400" /> Filter:
        </span>
        <button
            onclick={() => setQuickType('plain_text')}
            class="px-2.5 py-0.5 rounded-lg border font-mono transition-all {selectedContentType === 'plain_text' ? 'bg-indigo-600/30 text-indigo-300 border-indigo-500/50 font-bold' : 'bg-slate-950/50 text-slate-400 border-slate-800/80 hover:text-slate-200'}"
        >
            Text
        </button>
        <button
            onclick={() => setQuickType('json')}
            class="px-2.5 py-0.5 rounded-lg border font-mono transition-all {selectedContentType === 'json' ? 'bg-indigo-600/30 text-indigo-300 border-indigo-500/50 font-bold' : 'bg-slate-950/50 text-slate-400 border-slate-800/80 hover:text-slate-200'}"
        >
            JSON
        </button>
        <button
            onclick={() => setQuickType('markdown')}
            class="px-2.5 py-0.5 rounded-lg border font-mono transition-all {selectedContentType === 'markdown' ? 'bg-indigo-600/30 text-indigo-300 border-indigo-500/50 font-bold' : 'bg-slate-950/50 text-slate-400 border-slate-800/80 hover:text-slate-200'}"
        >
            Markdown
        </button>
        <button
            onclick={() => setQuickType('url')}
            class="px-2.5 py-0.5 rounded-lg border font-mono transition-all {selectedContentType === 'url' ? 'bg-indigo-600/30 text-indigo-300 border-indigo-500/50 font-bold' : 'bg-slate-950/50 text-slate-400 border-slate-800/80 hover:text-slate-200'}"
        >
            URL
        </button>
    </div>
</div>
