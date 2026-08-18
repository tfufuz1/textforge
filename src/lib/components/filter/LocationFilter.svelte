<script lang="ts">
    import { onMount } from 'svelte';
    import { listFolders, type FolderDto } from '../../ipc/snippets';
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let folders = $state<FolderDto[]>([]);
    let selectedLocation = $state('all');
    let selectedFolderId = $state<string | null>(null);

    onMount(async () => {
        try {
            folders = await listFolders();
        } catch (e) {
            console.error("Failed to load folders for location filter:", e);
        }
    });

    $effect(() => {
        const storeLocation = $snippetFilterStore.locationType || 'all';
        const storeFolderId = $snippetFilterStore.folderId || null;
        if (storeLocation !== selectedLocation || storeFolderId !== selectedFolderId) {
            selectedLocation = storeLocation;
            selectedFolderId = storeFolderId;
        }
    });

    function setLocation(loc: string, folderId: string | null = null) {
        selectedLocation = loc;
        selectedFolderId = folderId;
        
        snippetFilterStore.update(f => ({
            ...f,
            locationType: loc === 'all' ? null : loc,
            folderId: folderId,
            isTrashed: loc === 'trash' ? true : false
        }));
        loadSnippets();
    }
</script>

<div class="space-y-4">
    <!-- Orte -->
    <div class="space-y-1.5">
        <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Orte</h3>
        <div class="space-y-1">
            <button
                onclick={() => setLocation('all')}
                class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors flex items-center space-x-2 {selectedLocation === 'all' ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/20' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'}"
            >
                <span>🗂️</span>
                <span>Alle Snippets</span>
            </button>
            <button
                onclick={() => setLocation('inbox')}
                class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors flex items-center space-x-2 {selectedLocation === 'inbox' ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/20' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'}"
            >
                <span>📥</span>
                <span>Inbox</span>
            </button>
            <button
                onclick={() => setLocation('archive')}
                class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors flex items-center space-x-2 {selectedLocation === 'archive' ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/20' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'}"
            >
                <span>📦</span>
                <span>Archiv</span>
            </button>
            <button
                onclick={() => setLocation('trash')}
                class="w-full text-left px-3 py-2 rounded-xl text-sm font-medium transition-colors flex items-center space-x-2 {selectedLocation === 'trash' ? 'bg-rose-950/30 text-rose-300 border border-rose-800/20' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/60'}"
            >
                <span>🗑️</span>
                <span>Papierkorb</span>
            </button>
        </div>
    </div>

    <!-- Ordner -->
    {#if folders.length > 0}
        <div class="space-y-1.5">
            <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Ordner</h3>
            <div class="space-y-1 max-h-40 overflow-y-auto pr-1">
                {#each folders as folder}
                    <button
                        onclick={() => setLocation('folder', folder.id)}
                        class="w-full text-left px-3 py-1.5 rounded-lg text-xs font-medium transition-colors flex items-center space-x-2 {selectedLocation === 'folder' && selectedFolderId === folder.id ? 'bg-indigo-600/20 text-indigo-300' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-900/40'}"
                    >
                        <span>{folder.icon || '📁'}</span>
                        <span class="truncate">{folder.name}</span>
                    </button>
                {/each}
            </div>
        </div>
    {/if}
</div>
