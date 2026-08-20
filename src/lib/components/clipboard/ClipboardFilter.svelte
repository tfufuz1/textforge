<script lang="ts">
    import { clipboardFilterStore, loadClipboardHistory } from '../../stores/clipboard';
    import { Option } from '../../domain/adts';

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

<div class="flex flex-col sm:flex-row gap-3 w-full">
    <div class="relative flex-1">
        <input
            type="text"
            class="w-full pl-9 pr-4 py-2 text-sm bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all shadow-inner"
            placeholder="Zwischenablage durchsuchen..."
            bind:value={searchQuery}
            onkeyup={applyFilters}
        />
        <span class="absolute left-3 top-2.5 text-xs text-slate-500">🔍</span>
    </div>

    <select
        class="py-2 px-3 text-sm bg-slate-950/80 border border-slate-800 rounded-xl text-slate-200 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all cursor-pointer"
        bind:value={selectedContentType}
        onchange={applyFilters}
    >
        {#each contentTypes as ct}
            <option value={ct.value} class="bg-slate-900 text-slate-200">{ct.label}</option>
        {/each}
    </select>

    <button
        onclick={() => { isPinnedOnly = !isPinnedOnly; applyFilters(); }}
        class="px-3 py-2 text-xs font-medium rounded-xl border transition-all flex items-center gap-1.5 {isPinnedOnly ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-950/80 text-slate-400 border-slate-800 hover:text-slate-200'}"
    >
        <span>📌</span>
        <span>Nur Pinned</span>
    </button>

    {#if searchQuery || selectedContentType || isPinnedOnly}
        <button
            onclick={resetFilters}
            class="px-3 py-2 text-xs font-medium bg-slate-800/80 hover:bg-slate-700 text-slate-300 border border-slate-700/60 rounded-xl transition-all"
        >
            Filter zurücksetzen
        </button>
    {/if}
</div>
