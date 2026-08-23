<script lang="ts">
    import { onMount } from 'svelte';
    import { readClipboardNow } from '$lib/ipc/clipboard';
    import { createSnippet, listAllTags, listFolders, type FolderDto } from '$lib/ipc/snippets';
    import { pushNotification } from '$lib/stores/notifications';
    import { Option, type UnixMs } from '$lib/domain/adts';

    let { isOpen = $bindable(false), onSnippetCreated = () => {} } = $props<{
        isOpen?: boolean;
        onSnippetCreated?: () => void;
    }>();

    let clipboardText = $state('');
    let snippetTitle = $state('');
    let selectedFolderId = $state<string | null>(null);
    let selectedTags = $state<string[]>([]);
    let newTagInput = $state('');

    let isLoading = $state(false);
    let isSaving = $state(false);
    let availableFolders = $state<FolderDto[]>([]);
    let availableTags = $state<string[]>([]);

    $effect(() => {
        if (isOpen) {
            initData();
        }
    });

    async function initData() {
        isLoading = true;
        snippetTitle = '';
        selectedFolderId = null;
        selectedTags = [];
        newTagInput = '';

        try {
            const content = await readClipboardNow();
            clipboardText = content || '';
            if (clipboardText) {
                // Auto generate a concise title candidate from the first line
                const firstLine = clipboardText.trim().split('\n')[0].slice(0, 60).trim();
                snippetTitle = firstLine || 'Zwischenablage-Snippet';
            }
        } catch (e) {
            console.error('Failed to read clipboard:', e);
            clipboardText = '';
        }

        try {
            const [folders, tags] = await Promise.all([
                listFolders(),
                listAllTags()
            ]);
            availableFolders = folders || [];
            availableTags = tags || [];
        } catch (e) {
            console.error('Failed to load metadata:', e);
        } finally {
            isLoading = false;
        }
    }

    function toggleTag(tag: string) {
        if (selectedTags.includes(tag)) {
            selectedTags = selectedTags.filter(t => t !== tag);
        } else {
            selectedTags = [...selectedTags, tag];
        }
    }

    function handleAddTag() {
        const t = newTagInput.trim().toLowerCase();
        if (t && !selectedTags.includes(t)) {
            selectedTags = [...selectedTags, t];
        }
        newTagInput = '';
    }

    async function handleSave() {
        if (!clipboardText.trim()) {
            pushNotification({
                id: crypto.randomUUID(),
                severity: 'warning',
                title: 'Sicherheitsfehler',
                message: Option.some('Zwischenablage ist leer.'),
                duration: 3000,
                action: Option.none(),
                createdAt: Date.now() as UnixMs
            });
            return;
        }

        isSaving = true;
        try {
            const finalTitle = snippetTitle.trim() || 'Zwischenablage-Snippet';
            await createSnippet({
                title: finalTitle,
                content: clipboardText,
                folderId: selectedFolderId,
                tags: selectedTags,
            });

            pushNotification({
                id: crypto.randomUUID(),
                severity: 'success',
                title: 'Erfolg',
                message: Option.some(`Snippet "${finalTitle}" aus Zwischenablage erstellt!`),
                duration: 2500,
                action: Option.none(),
                createdAt: Date.now() as UnixMs
            });
            isOpen = false;
            onSnippetCreated();
        } catch (e: any) {
            console.error('Failed to save snippet from Quick Capture:', e);
            pushNotification({
                id: crypto.randomUUID(),
                severity: 'error',
                title: 'Fehler',
                message: Option.some(`Speichern fehlgeschlagen: ${e?.message || e}`),
                duration: 4000,
                action: Option.none(),
                createdAt: Date.now() as UnixMs
            });
        } finally {
            isSaving = false;
        }
    }

    function handleKeyDown(e: KeyboardEvent) {
        if (!isOpen) return;

        if (e.key === 'Escape') {
            isOpen = false;
            e.preventDefault();
        } else if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
            handleSave();
            e.preventDefault();
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
    <!-- Backdrop -->
    <div
        class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4 animate-in fade-in duration-150"
        onclick={(e) => { if (e.target === e.currentTarget) isOpen = false; }}
        role="presentation"
    >
        <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-xl shadow-2xl shadow-indigo-950/40 overflow-hidden flex flex-col border-indigo-500/30">
            <!-- Header -->
            <div class="p-4 border-b border-slate-800/80 flex items-center justify-between bg-slate-950/80">
                <div class="flex items-center space-x-2.5">
                    <span class="text-xl p-1.5 rounded-lg bg-indigo-600/20 text-indigo-400 border border-indigo-500/30">⚡</span>
                    <div>
                        <h2 class="font-bold text-sm text-slate-100">Quick Capture</h2>
                        <p class="text-[11px] text-slate-400">Neues Snippet direkt aus der Zwischenablage anlegen</p>
                    </div>
                </div>
                <button
                    onclick={() => isOpen = false}
                    class="p-1.5 text-slate-400 hover:text-white rounded-lg hover:bg-slate-800 transition-colors text-xs font-mono"
                >
                    ✕
                </button>
            </div>

            <!-- Content Area -->
            <div class="p-4 space-y-4 max-h-[75vh] overflow-y-auto custom-scrollbar">
                {#if isLoading}
                    <div class="py-12 text-center text-slate-400 space-y-2">
                        <div class="animate-spin w-6 h-6 border-2 border-indigo-500 border-t-transparent rounded-full mx-auto"></div>
                        <p class="text-xs">Lade Zwischenablage-Inhalt...</p>
                    </div>
                {:else}
                    <!-- Title Input -->
                    <div>
                        <label for="snippet-title" class="block text-xs font-semibold text-slate-300 mb-1">Titel</label>
                        <input
                            id="snippet-title"
                            type="text"
                            bind:value={snippetTitle}
                            placeholder="Snippet-Titel eingeben..."
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500/70"
                            autofocus
                        />
                    </div>

                    <!-- Clipboard Content Preview -->
                    <div>
                        <div class="flex items-center justify-between mb-1">
                            <label for="snippet-content" class="block text-xs font-semibold text-slate-300">Zwischenablage-Inhalt (Vorschau)</label>
                            <span class="text-[10px] text-slate-500 font-mono">{clipboardText.length} Zeichen</span>
                        </div>
                        {#if !clipboardText}
                            <div class="p-4 bg-slate-950/60 border border-amber-500/30 rounded-xl text-center text-amber-400 text-xs font-medium">
                                ⚠️ Keine Daten in der Zwischenablage vorhanden.
                            </div>
                        {:else}
                            <textarea
                                id="snippet-content"
                                bind:value={clipboardText}
                                rows="5"
                                class="w-full bg-slate-950 border border-slate-800 rounded-xl p-3 text-xs font-mono text-slate-200 placeholder-slate-600 focus:outline-none focus:border-indigo-500/70 resize-y"
                            ></textarea>
                        {/if}
                    </div>

                    <!-- Target Folder Selection -->
                    <div>
                        <label for="folder-select" class="block text-xs font-semibold text-slate-300 mb-1">Ziel-Ordner (optional)</label>
                        <select
                            id="folder-select"
                            bind:value={selectedFolderId}
                            class="w-full bg-slate-950 border border-slate-800 rounded-xl px-3 py-2 text-xs text-white focus:outline-none focus:border-indigo-500/70"
                        >
                            <option value={null}>Inbox (Kein Ordner)</option>
                            {#each availableFolders as folder}
                                <option value={folder.id}>{folder.name}</option>
                            {/each}
                        </select>
                    </div>

                    <!-- Tags Selection & Input -->
                    <div>
                        <span class="block text-xs font-semibold text-slate-300 mb-1">Tags zuweisen</span>
                        <div class="flex flex-wrap gap-1.5 mb-2">
                            {#each selectedTags as tag}
                                <span class="px-2 py-0.5 rounded-lg bg-indigo-600/30 border border-indigo-500/40 text-indigo-200 text-[11px] font-medium flex items-center space-x-1">
                                    <span>#{tag}</span>
                                    <button onclick={() => toggleTag(tag)} class="hover:text-red-400 ml-1">✕</button>
                                </span>
                            {/each}
                        </div>

                        <div class="flex space-x-2">
                            <input
                                type="text"
                                bind:value={newTagInput}
                                onkeydown={(e) => { if (e.key === 'Enter') { e.preventDefault(); handleAddTag(); } }}
                                placeholder="Tag hinzufügen..."
                                class="flex-1 bg-slate-950 border border-slate-800 rounded-xl px-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-indigo-500/70"
                            />
                            <button
                                onclick={handleAddTag}
                                class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 text-xs font-medium rounded-xl border border-slate-700 transition-colors"
                            >
                                + Tag
                            </button>
                        </div>

                        {#if availableTags.length > 0}
                            <div class="mt-2 text-[11px] text-slate-400">
                                <span>Vorhandene Tags: </span>
                                <div class="flex flex-wrap gap-1 mt-1">
                                    {#each availableTags as tag}
                                        {#if !selectedTags.includes(tag)}
                                            <button
                                                onclick={() => toggleTag(tag)}
                                                class="px-2 py-0.5 rounded-md bg-slate-800/80 hover:bg-indigo-900/40 text-slate-400 hover:text-indigo-200 text-[10px] border border-slate-700/60 transition-colors"
                                            >
                                                +{tag}
                                            </button>
                                        {/if}
                                    {/each}
                                </div>
                            </div>
                        {/if}
                    </div>
                {/if}
            </div>

            <!-- Footer -->
            <div class="px-4 py-3 bg-slate-950 border-t border-slate-800/80 flex justify-between items-center text-xs">
                <span class="text-slate-500 font-mono text-[10px]">
                    <kbd class="text-slate-400">Ctrl+Enter</kbd> zum Speichern
                </span>
                <div class="flex items-center space-x-2">
                    <button
                        onclick={() => isOpen = false}
                        class="px-3 py-1.5 rounded-xl border border-slate-800 text-slate-400 hover:text-white hover:bg-slate-800 font-medium transition-colors"
                    >
                        Abbrechen
                    </button>
                    <button
                        onclick={handleSave}
                        disabled={isSaving || !clipboardText.trim()}
                        class="px-4 py-1.5 rounded-xl bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white font-semibold shadow-lg shadow-indigo-600/30 transition-all flex items-center space-x-1.5"
                    >
                        {#if isSaving}
                            <span class="animate-spin w-3 h-3 border-2 border-white border-t-transparent rounded-full"></span>
                        {/if}
                        <span>Als Snippet speichern</span>
                    </button>
                </div>
            </div>
        </div>
    </div>
{/if}
