<script lang="ts">
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let isPinned = $state<boolean | null>(null);
    let isFavorite = $state<boolean | null>(null);
    let isTemplate = $state<boolean | null>(null);

    let sortBy = $state('updatedAt');
    let sortDir = $state<'asc' | 'desc'>('desc');

    $effect(() => {
        isPinned = $snippetFilterStore.isPinned ?? null;
        isFavorite = $snippetFilterStore.isFavorite ?? null;
        isTemplate = $snippetFilterStore.isTemplate ?? null;
        sortBy = $snippetFilterStore.sortBy || 'updatedAt';
        sortDir = ($snippetFilterStore.sortDir as 'asc' | 'desc') || 'desc';
    });

    function togglePinned() {
        const next = isPinned === true ? null : true;
        snippetFilterStore.update(f => ({ ...f, isPinned: next }));
        loadSnippets();
    }

    function toggleFavorite() {
        const next = isFavorite === true ? null : true;
        snippetFilterStore.update(f => ({ ...f, isFavorite: next }));
        loadSnippets();
    }

    function toggleTemplate() {
        const next = isTemplate === true ? null : true;
        snippetFilterStore.update(f => ({ ...f, isTemplate: next }));
        loadSnippets();
    }

    function handleSortChange(e: Event) {
        const target = e.target as HTMLSelectElement;
        sortBy = target.value;
        snippetFilterStore.update(f => ({ ...f, sortBy }));
        loadSnippets();
    }

    function toggleSortDir() {
        sortDir = sortDir === 'asc' ? 'desc' : 'asc';
        snippetFilterStore.update(f => ({ ...f, sortDir }));
        loadSnippets();
    }
</script>

<div class="space-y-4">
    <!-- Schnellfilter -->
    <div class="space-y-1.5">
        <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Schnellfilter</h3>
        <div class="flex flex-wrap gap-1.5">
            <button
                onclick={togglePinned}
                class="px-2.5 py-1 text-[11px] font-medium rounded-lg border transition-all flex items-center space-x-1 {isPinned === true ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200 shadow-inner' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            >
                <span>📌</span>
                <span>Gepinnt</span>
            </button>
            <button
                onclick={toggleFavorite}
                class="px-2.5 py-1 text-[11px] font-medium rounded-lg border transition-all flex items-center space-x-1 {isFavorite === true ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200 shadow-inner' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            >
                <span>⭐</span>
                <span>Favoriten</span>
            </button>
            <button
                onclick={toggleTemplate}
                class="px-2.5 py-1 text-[11px] font-medium rounded-lg border transition-all flex items-center space-x-1 {isTemplate === true ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200 shadow-inner' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200'}"
            >
                <span>⚡</span>
                <span>Templates</span>
            </button>
        </div>
    </div>

    <!-- Sortierung -->
    <div class="space-y-1.5">
        <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Sortierung</h3>
        <div class="flex space-x-1.5">
            <select
                value={sortBy}
                onchange={handleSortChange}
                class="flex-1 px-2.5 py-1.5 text-xs bg-slate-950/80 border border-slate-850 rounded-lg text-slate-100 outline-none focus:border-indigo-500 transition-all"
            >
                <option value="updatedAt">Geändert</option>
                <option value="createdAt">Erstellt</option>
                <option value="title">Titel</option>
                <option value="size">Größe</option>
                <option value="usageCount">Häufigkeit</option>
            </select>
            <button
                onclick={toggleSortDir}
                class="px-2.5 py-1.5 text-xs bg-slate-950/80 border border-slate-850 rounded-lg text-slate-100 hover:bg-slate-900 transition-all font-mono"
                title={sortDir === 'asc' ? 'Aufsteigend' : 'Absteigend'}
            >
                {sortDir === 'asc' ? '▲' : '▼'}
            </button>
        </div>
    </div>
</div>
