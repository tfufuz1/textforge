<script lang="ts">
    import { activeSnippetStore, handleUpdateSnippet, handleCreateSnippet, handleDuplicateSnippet } from '../../stores/snippets';
    import { writeToClipboard } from '../../ipc/snippets';
    import { pushNotification, Notifications } from '../../stores/notifications';
    import SnippetPreview from './SnippetPreview.svelte';
    import TemplateForm from '../template/TemplateForm.svelte';
    import SnippetStats from './SnippetStats.svelte';
    import TransformApplyPanel from './TransformApplyPanel.svelte';
    import SnippetEditorLayout from './SnippetEditorLayout.svelte';

    let title = $state('');
    let content = $state('');
    let contentType = $state('plain_text');
    let tagsInput = $state('');
    let isEditing = $state(false);
    let viewMode = $state<'editor' | 'preview' | 'split'>('editor');
    let lastRenderedOutput = $state('');

    $effect(() => {
        if ($activeSnippetStore) {
            title = $activeSnippetStore.title;
            content = $activeSnippetStore.content;
            contentType = $activeSnippetStore.contentType;
            tagsInput = $activeSnippetStore.tags.join(', ');
            isEditing = true;
        } else {
            title = '';
            content = '';
            contentType = 'plain_text';
            tagsInput = '';
            isEditing = false;
        }
    });

    let isTemplate = $derived(content.includes('{{') && content.includes('}}'));

    async function save() {
        const tags = tagsInput.split(',').map(t => t.trim()).filter(Boolean);
        if (isEditing && $activeSnippetStore) {
            await handleUpdateSnippet($activeSnippetStore.id, {
                title,
                content,
                contentType,
                tags
            });
            pushNotification(Notifications.snippetSaved(title));
        } else {
            if (!title.trim() || !content.trim()) return;
            const created = await handleCreateSnippet({
                title,
                content,
                contentType,
                tags
            });
            if (created) {
                pushNotification(Notifications.snippetSaved(title));
            }
        }
    }

    async function copy() {
        const textToCopy = isTemplate ? lastRenderedOutput : content;
        if (!textToCopy) return;
        await writeToClipboard(textToCopy, $activeSnippetStore?.id ?? null);
        pushNotification(Notifications.snippetCopied());
    }

    async function duplicate() {
        if (!isEditing || !$activeSnippetStore) return;
        await handleDuplicateSnippet($activeSnippetStore.id);
        pushNotification(Notifications.snippetSaved(`Kopie von ${title}`));
    }

    function createNew() {
        activeSnippetStore.set(null);
    }
</script>

<SnippetEditorLayout bind:viewMode={viewMode}>
    <div class="h-full flex flex-col bg-slate-900/90 rounded-2xl border border-slate-800/80 p-5 shadow-xl backdrop-blur-md">
        <div class="flex justify-between items-center pb-4 mb-4 border-b border-slate-800/80">
            <div class="flex items-center space-x-3">
                <h2 class="font-black text-base text-slate-100 flex items-center space-x-2">
                    <span class="text-indigo-400">{isEditing ? '✏️' : '✨'}</span>
                    <span>{isEditing ? 'Snippet bearbeiten' : 'Neues Snippet'}</span>
                </h2>
                {#if isTemplate}
                    <span class="px-2.5 py-0.5 text-[10px] uppercase font-mono font-bold tracking-wider rounded-md bg-amber-500/20 text-amber-300 border border-amber-500/40">
                        ⚡ Template Mode
                    </span>
                {/if}
                {#if contentType === 'markdown'}
                    <div class="flex items-center bg-slate-950/80 rounded-xl p-1 border border-slate-800/80 ml-2 shadow-inner">
                        <button 
                            onclick={() => viewMode = 'editor'}
                            class="px-2.5 py-1 text-[10px] font-bold rounded-lg transition-all {viewMode === 'editor' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            Editor
                        </button>
                        <button 
                            onclick={() => viewMode = 'preview'}
                            class="px-2.5 py-1 text-[10px] font-bold rounded-lg transition-all {viewMode === 'preview' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            Vorschau
                        </button>
                        <button 
                            onclick={() => viewMode = 'split'}
                            class="px-2.5 py-1 text-[10px] font-bold rounded-lg transition-all {viewMode === 'split' ? 'bg-indigo-600 text-white shadow-sm' : 'text-slate-400 hover:text-slate-200'}"
                        >
                            Split
                        </button>
                    </div>
                {/if}
            </div>
            <div class="flex space-x-2 items-center">
                {#if isEditing}
                    <!-- Color Picker -->
                    <div class="flex items-center space-x-1.5 bg-slate-950/80 p-1.5 rounded-xl border border-slate-800/80 shadow-inner">
                        {#each ['#6366f1', '#ec4899', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6'] as c}
                            <button
                                onclick={async () => {
                                    if ($activeSnippetStore) {
                                        await handleUpdateSnippet($activeSnippetStore.id, { color: c });
                                    }
                                }}
                                class="w-3.5 h-3.5 rounded-full border border-white/20 transition-transform hover:scale-125"
                                style="background-color: {c}"
                                title="Farbe setzen"
                            ></button>
                        {/each}
                    </div>

                    <!-- Favorite Button -->
                    <button
                        onclick={async () => {
                            if ($activeSnippetStore) {
                                await handleUpdateSnippet($activeSnippetStore.id, { isFavorite: !$activeSnippetStore.isFavorite });
                            }
                        }}
                        class="px-3 py-1.5 text-xs font-semibold rounded-xl transition-all border {$activeSnippetStore?.isFavorite ? 'bg-amber-500/20 text-amber-300 border-amber-500/40 shadow-sm' : 'bg-slate-800/80 text-slate-400 border-slate-700/80 hover:text-amber-300'}"
                        title="Favorit"
                    >
                        {$activeSnippetStore?.isFavorite ? '⭐ Favorit' : '☆ Favorit'}
                    </button>

                    <button 
                        onclick={createNew} 
                        class="px-3.5 py-1.5 text-xs font-semibold bg-slate-800/80 text-slate-300 border border-slate-700/80 rounded-xl hover:bg-slate-700/80 transition-all"
                    >
                        + Neu
                    </button>
                    <button 
                        onclick={duplicate} 
                        class="px-3.5 py-1.5 text-xs font-semibold bg-slate-800/80 text-slate-300 border border-slate-700/80 rounded-xl hover:bg-slate-700/80 transition-all flex items-center space-x-1"
                        title="Duplizieren & Bearbeiten"
                    >
                        <span>📄</span>
                        <span>Duplizieren</span>
                    </button>
                    <button 
                        onclick={copy} 
                        class="px-3.5 py-1.5 text-xs font-semibold bg-indigo-950/80 text-indigo-300 border border-indigo-700/60 rounded-xl hover:bg-indigo-900 transition-all flex items-center space-x-1 shadow-sm"
                    >
                        <span>📋</span>
                        <span>Kopieren</span>
                    </button>
                {/if}
                <button 
                    onclick={save} 
                    class="px-4 py-1.5 text-xs font-bold bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-xl hover:from-blue-500 hover:to-indigo-500 shadow-lg shadow-indigo-600/25 transition-all"
                >
                    Speichern
                </button>
            </div>
        </div>

        <div class="space-y-4 flex-1 flex flex-col min-h-0">
            <div class="grid grid-cols-3 gap-4">
                <div class="col-span-2">
                    <label for="snippet-title" class="block text-xs font-bold text-slate-400 mb-1.5">Titel</label>
                    <input 
                        id="snippet-title"
                        type="text" 
                        bind:value={title} 
                        placeholder="Snippet Titel..." 
                        class="w-full px-3.5 py-2 text-sm bg-slate-950/80 border border-slate-800/90 rounded-xl text-slate-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all shadow-inner font-medium"
                    />
                </div>
                <div>
                    <label for="snippet-content-type" class="block text-xs font-bold text-slate-400 mb-1.5">Typ</label>
                    <select 
                        id="snippet-content-type"
                        bind:value={contentType} 
                        class="w-full px-3.5 py-2 text-sm bg-slate-950/80 border border-slate-800/90 rounded-xl text-slate-100 outline-none focus:border-indigo-500 transition-all shadow-inner font-medium"
                    >
                        <option value="plain_text">Plain Text</option>
                        <option value="markdown">Markdown</option>
                        <option value="json">JSON</option>
                        <option value="javascript">JavaScript</option>
                        <option value="typescript">TypeScript</option>
                        <option value="python">Python</option>
                        <option value="sql">SQL</option>
                        <option value="url">URL</option>
                    </select>
                </div>
            </div>

            <div>
                <label for="snippet-tags" class="block text-xs font-bold text-slate-400 mb-1.5">Tags (kommagetrennt)</label>
                <input 
                    id="snippet-tags"
                    type="text" 
                    bind:value={tagsInput} 
                    placeholder="z.B. code, helper, api" 
                    class="w-full px-3.5 py-2 text-sm bg-slate-950/80 border border-slate-800/90 rounded-xl text-slate-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all shadow-inner font-medium"
                />
            </div>

            <div class="flex-1 flex min-h-0 gap-4">
                {#if viewMode === 'editor' || viewMode === 'split' || contentType !== 'markdown'}
                    <div class="flex-1 flex flex-col min-h-0 justify-between gap-4">
                        <div class="flex-1 flex flex-col min-h-0">
                            <label for="snippet-content" class="block text-xs font-bold text-slate-400 mb-1.5">Inhalt</label>
                            <textarea 
                                id="snippet-content"
                                bind:value={content} 
                                placeholder="Snippet Inhalt hier eingeben..."
                                class="flex-1 w-full p-4 font-mono text-sm bg-slate-950/90 border border-slate-800/90 rounded-2xl text-slate-200 resize-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all leading-relaxed shadow-inner"
                            ></textarea>
                        </div>

                        {#if isTemplate && isEditing}
                            <TemplateForm {content} snippetId={$activeSnippetStore?.id} bind:renderedOutput={lastRenderedOutput} />
                        {/if}

                        {#if isEditing}
                            <TransformApplyPanel bind:content={content} />
                        {/if}
                    </div>
                {/if}

                {#if (viewMode === 'preview' || viewMode === 'split') && contentType === 'markdown'}
                    <div class="flex-1 flex flex-col min-h-0">
                        <span class="block text-xs font-bold text-slate-400 mb-1.5">Vorschau (Markdown)</span>
                        <SnippetPreview {content} />
                    </div>
                {/if}
            </div>

            <SnippetStats {content} />

            <div class="pt-3 border-t border-slate-800/80 flex items-center justify-between text-xs text-slate-500 font-mono">
                <div>
                    {#if isEditing && $activeSnippetStore}
                        <span class="px-2 py-0.5 rounded bg-slate-950 border border-slate-800 text-slate-400">ID: {$activeSnippetStore.id.substring(0, 8)}...</span>
                    {/if}
                </div>
                <span class="text-indigo-400 font-semibold">TextForge Stats Engine</span>
            </div>
        </div>
    </div>
</SnippetEditorLayout>
