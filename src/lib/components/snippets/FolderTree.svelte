<script lang="ts">
    import { onMount } from 'svelte';
    import { listFolders, createFolder, renameFolder, deleteFolder, type FolderDto } from '../../ipc/snippets';
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let folders = $state<FolderDto[]>([]);
    let selectedFolderId = $state<string | null>(null);
    let isCreating = $state(false);
    let newFolderName = $state('');

    onMount(async () => {
        await reloadFolders();
    });

    async function reloadFolders() {
        try {
            folders = await listFolders();
        } catch (e) {
            console.error("Failed to load folders:", e);
        }
    }

    function selectFolder(id: string | null) {
        selectedFolderId = id;
        snippetFilterStore.update(f => ({
            ...f,
            locationType: id ? 'folder' : 'all',
            folderId: id
        }));
        loadSnippets();
    }

    async function handleCreateFolder() {
        if (!newFolderName.trim()) return;
        try {
            await createFolder(newFolderName.trim(), null, '📁');
            newFolderName = '';
            isCreating = false;
            await reloadFolders();
        } catch (e) {
            console.error("Failed to create folder:", e);
        }
    }

    async function handleDelete(id: string) {
        try {
            await deleteFolder(id);
            if (selectedFolderId === id) selectFolder(null);
            await reloadFolders();
        } catch (e) {
            console.error("Failed to delete folder:", e);
        }
    }
</script>

<div class="space-y-3 bg-slate-900/60 p-4 rounded-xl border border-slate-800/80">
    <div class="flex justify-between items-center">
        <h3 class="text-xs font-bold text-slate-300 uppercase tracking-wider font-mono">Ordner</h3>
        <button
            onclick={() => isCreating = !isCreating}
            class="text-xs text-indigo-400 hover:text-indigo-300 font-semibold"
        >
            {isCreating ? 'Abbrechen' : '+ Neu'}
        </button>
    </div>

    {#if isCreating}
        <div class="flex gap-2">
            <input
                type="text"
                placeholder="Ordnername..."
                bind:value={newFolderName}
                onkeydown={(e) => e.key === 'Enter' && handleCreateFolder()}
                class="flex-1 px-2.5 py-1 text-xs bg-slate-950 border border-slate-800 rounded-lg text-slate-100 outline-none focus:border-indigo-500"
            />
            <button
                onclick={handleCreateFolder}
                class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white font-semibold rounded-lg"
            >
                OK
            </button>
        </div>
    {/if}

    <div class="space-y-1 max-h-40 overflow-y-auto pr-1">
        <button
            onclick={() => selectFolder(null)}
            class="w-full text-left px-2.5 py-1.5 rounded-lg text-xs font-medium transition-all flex items-center justify-between {selectedFolderId === null ? 'bg-indigo-600/30 text-indigo-200 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
        >
            <span class="flex items-center space-x-2">
                <span>📁</span>
                <span>Alle Ordner</span>
            </span>
        </button>

        {#each folders as folder}
            <div class="group flex items-center justify-between rounded-lg transition-all {selectedFolderId === folder.id ? 'bg-indigo-600/30 text-indigo-200 border border-indigo-500/40' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}">
                <button
                    onclick={() => selectFolder(folder.id)}
                    class="flex-1 text-left px-2.5 py-1.5 text-xs font-medium truncate flex items-center space-x-2"
                >
                    <span>{folder.icon || '📁'}</span>
                    <span class="truncate">{folder.name}</span>
                </button>
                <button
                    onclick={() => handleDelete(folder.id)}
                    class="opacity-0 group-hover:opacity-100 p-1 text-[10px] text-slate-500 hover:text-rose-400 rounded mr-1"
                    title="Ordner löschen"
                >
                    🗑️
                </button>
            </div>
        {:else}
            <div class="text-[11px] text-slate-500 italic p-1">Keine Ordner angelegt.</div>
        {/each}
    </div>
</div>
