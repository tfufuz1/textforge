<script lang="ts">
    import { clipboardFilterStore, loadClipboardHistory } from '../../stores/clipboard';
    import { Option } from '../../domain/adts';

    let searchQuery = $state('');

    async function handleSearch() {
        clipboardFilterStore.update(f => ({
            ...f,
            searchQuery: searchQuery ? Option.some(searchQuery) : Option.none()
        }));
        await loadClipboardHistory();
    }
</script>

<div class="relative w-full">
    <input
        type="text"
        class="w-full pl-9 pr-4 py-2 text-sm bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 placeholder-slate-500 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all shadow-inner"
        placeholder="Zwischenablage durchsuchen..."
        bind:value={searchQuery}
        onkeyup={handleSearch}
    />
    <span class="absolute left-3 top-2.5 text-xs text-slate-500">🔍</span>
</div>

