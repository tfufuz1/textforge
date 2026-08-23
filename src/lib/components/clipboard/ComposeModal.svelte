<script lang="ts">
    import { onMount } from 'svelte';
    import { getClipboardEntry, composeClipboardEntriesToSnippet } from '../../ipc/clipboard';
    import { loadClipboardHistory } from '../../stores/clipboard';
    import { loadSnippets } from '../../stores/snippets';
    import { refreshUndoState } from '../../stores/undo';
    import { pushNotification, Notifications } from '../../stores/notifications';

    import PlusIcon from '$lib/components/icons/PlusIcon.svelte';
    import TrashIcon from '$lib/components/icons/TrashIcon.svelte';

    interface Props {
        entryIds: string[];
        onClose: () => void;
        onComposed?: () => void;
    }

    let { entryIds, onClose, onComposed }: Props = $props();

    interface CompositionItem {
        id: string;
        content: string;
        preview: string;
        contentType: string;
    }

    let items = $state<CompositionItem[]>([]);
    let isLoading = $state(true);
    let title = $state('');
    let separatorPreset = $state<'double_newline' | 'newline' | 'dash_divider' | 'comma' | 'space' | 'custom'>('double_newline');
    let customSeparator = $state('\n\n');
    let isSubmitting = $state(false);
    let draggedIndex = $state<number | null>(null);

    onMount(async () => {
        isLoading = true;
        const loaded: CompositionItem[] = [];
        for (const id of entryIds) {
            try {
                const detail = await getClipboardEntry(id);
                loaded.push({
                    id: detail.id,
                    content: detail.content,
                    preview: detail.preview,
                    contentType: detail.contentType
                });
            } catch (e) {
                console.error(`Error loading clipboard entry ${id}:`, e);
            }
        }
        items = loaded;
        isLoading = false;
    });

    let effectiveSeparator = $derived.by(() => {
        switch (separatorPreset) {
            case 'double_newline': return '\n\n';
            case 'newline': return '\n';
            case 'dash_divider': return '\n\n---\n\n';
            case 'comma': return ', ';
            case 'space': return ' ';
            case 'custom': return customSeparator;
            default: return '\n\n';
        }
    });

    let composedPreview = $derived(items.map(it => it.content).join(effectiveSeparator));

    let titlePlaceholder = $derived.by(() => {
        const first = items.find(it => it.content.trim().length > 0);
        if (!first) return 'Clipboard-Import';
        const trimmed = first.content.trim();
        return trimmed.length > 60 ? trimmed.substring(0, 60) + '...' : trimmed;
    });

    function moveItem(index: number, direction: 'up' | 'down') {
        const targetIndex = direction === 'up' ? index - 1 : index + 1;
        if (targetIndex < 0 || targetIndex >= items.length) return;
        const newItems = [...items];
        const [moved] = newItems.splice(index, 1);
        newItems.splice(targetIndex, 0, moved);
        items = newItems;
    }

    function removeItem(index: number) {
        items = items.filter((_, i) => i !== index);
    }

    function handleDragStart(e: DragEvent, index: number) {
        draggedIndex = index;
        if (e.dataTransfer) {
            e.dataTransfer.effectAllowed = 'move';
            e.dataTransfer.setData('text/plain', String(index));
        }
    }

    function handleDragOver(e: DragEvent) {
        e.preventDefault();
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = 'move';
        }
    }

    function handleDrop(e: DragEvent, dropIndex: number) {
        e.preventDefault();
        if (draggedIndex === null || draggedIndex === dropIndex) return;
        const newItems = [...items];
        const [moved] = newItems.splice(draggedIndex, 1);
        newItems.splice(dropIndex, 0, moved);
        items = newItems;
        draggedIndex = null;
    }

    async function handleCompose() {
        if (items.length === 0 || isSubmitting) return;
        isSubmitting = true;
        try {
            const finalTitle = title.trim() || titlePlaceholder;
            const snippetId = await composeClipboardEntriesToSnippet(
                items.map(i => i.id),
                effectiveSeparator,
                finalTitle,
                { _type: 'inbox', folderId: null }
            );
            await loadClipboardHistory();
            await loadSnippets();
            await refreshUndoState();
            pushNotification(Notifications.snippetSaved(finalTitle));
            pushNotification(Notifications.undoAvailable("Snippet aus Zwischenablage zusammengestellt"));
            onComposed?.();
            onClose();
        } catch (e) {
            console.error("Failed to compose snippet:", e);
            pushNotification({
                id: crypto.randomUUID(),
                severity: 'error',
                title: 'Fehler beim Zusammenfügen',
                message: { _tag: 'Some', value: String(e) },
                duration: 4000,
                action: { _tag: 'None' },
                createdAt: Date.now() as any
            });
        } finally {
            isSubmitting = false;
        }
    }
</script>

<div class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-center justify-center p-4 overflow-y-auto">
    <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-3xl shadow-2xl flex flex-col max-h-[90vh] overflow-hidden">
        <!-- Header -->
        <div class="flex items-center justify-between px-6 py-4 border-b border-slate-800 bg-slate-900/80 shrink-0">
            <div class="flex items-center space-x-3">
                <div class="w-9 h-9 rounded-xl bg-indigo-950/80 border border-indigo-700/50 flex items-center justify-center text-indigo-400">
                    <PlusIcon class="w-5 h-5" />
                </div>
                <div>
                    <h2 class="text-base font-bold text-white">Prompt aus Zwischenablagen zusammenfügen</h2>
                    <p class="text-xs text-slate-400">Ordne die Textbausteine und wähle den Trenner für das finale Snippet</p>
                </div>
            </div>
            <button
                type="button"
                onclick={onClose}
                class="p-2 text-slate-400 hover:text-white hover:bg-slate-800 rounded-xl transition-all"
            >
                ✕
            </button>
        </div>

        {#if isLoading}
            <div class="p-12 text-center text-slate-400 space-y-2">
                <div class="inline-block animate-spin rounded-full h-8 w-8 border-2 border-indigo-500 border-t-transparent"></div>
                <p class="text-xs font-mono">Lade Zwischenablage-Einträge...</p>
            </div>
        {:else}
            <div class="p-6 overflow-y-auto space-y-5 flex-1 custom-scrollbar">
                <!-- Title Input -->
                <div>
                    <label for="compose-title" class="block text-xs font-semibold text-slate-300 mb-1.5">
                        Titel (optional)
                    </label>
                    <input
                        id="compose-title"
                        type="text"
                        bind:value={title}
                        placeholder={titlePlaceholder}
                        class="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-3.5 py-2 text-xs text-white placeholder-slate-500 outline-none transition-all font-medium"
                    />
                </div>

                <!-- Separator Picker -->
                <div>
                    <span class="block text-xs font-semibold text-slate-300 mb-2">Trennzeichen / Separator</span>
                    <div class="flex flex-wrap items-center gap-2">
                        <button
                            type="button"
                            onclick={() => separatorPreset = 'double_newline'}
                            class="px-3 py-1.5 text-xs rounded-xl border transition-all font-medium {separatorPreset === 'double_newline' ? 'bg-indigo-950 border-indigo-500 text-indigo-200' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-200'}"
                        >
                            Absatz (2x \n)
                        </button>
                        <button
                            type="button"
                            onclick={() => separatorPreset = 'newline'}
                            class="px-3 py-1.5 text-xs rounded-xl border transition-all font-medium {separatorPreset === 'newline' ? 'bg-indigo-950 border-indigo-500 text-indigo-200' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-200'}"
                        >
                            Zeilenumbruch (\n)
                        </button>
                        <button
                            type="button"
                            onclick={() => separatorPreset = 'dash_divider'}
                            class="px-3 py-1.5 text-xs rounded-xl border transition-all font-medium {separatorPreset === 'dash_divider' ? 'bg-indigo-950 border-indigo-500 text-indigo-200' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-200'}"
                        >
                            Trennlinie (---)
                        </button>
                        <button
                            type="button"
                            onclick={() => separatorPreset = 'comma'}
                            class="px-3 py-1.5 text-xs rounded-xl border transition-all font-medium {separatorPreset === 'comma' ? 'bg-indigo-950 border-indigo-500 text-indigo-200' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-200'}"
                        >
                            Komma (,)
                        </button>
                        <button
                            type="button"
                            onclick={() => separatorPreset = 'space'}
                            class="px-3 py-1.5 text-xs rounded-xl border transition-all font-medium {separatorPreset === 'space' ? 'bg-indigo-950 border-indigo-500 text-indigo-200' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-200'}"
                        >
                            Leerzeichen
                        </button>
                        <button
                            type="button"
                            onclick={() => separatorPreset = 'custom'}
                            class="px-3 py-1.5 text-xs rounded-xl border transition-all font-medium {separatorPreset === 'custom' ? 'bg-indigo-950 border-indigo-500 text-indigo-200' : 'bg-slate-950 border-slate-800 text-slate-400 hover:text-slate-200'}"
                        >
                            Eigene...
                        </button>
                    </div>

                    {#if separatorPreset === 'custom'}
                        <div class="mt-2">
                            <input
                                type="text"
                                bind:value={customSeparator}
                                placeholder="Benutzerdefiniertes Trennzeichen eingeben..."
                                class="w-full bg-slate-950 border border-slate-800 focus:border-indigo-500 rounded-xl px-3 py-1.5 text-xs text-white placeholder-slate-500 font-mono outline-none"
                            />
                        </div>
                    {/if}
                </div>

                <!-- Items Reordering List -->
                <div class="space-y-2">
                    <div class="flex items-center justify-between">
                        <span class="text-xs font-semibold text-slate-300">
                            Reihenfolge der Bausteine ({items.length})
                        </span>
                        <span class="text-[11px] text-slate-500">Per Drag-and-Drop oder Pfeile verschieben</span>
                    </div>

                    <div class="space-y-2 max-h-56 overflow-y-auto pr-1 custom-scrollbar">
                        {#each items as item, idx (item.id)}
                            <div
                                role="listitem"
                                draggable="true"
                                ondragstart={(e) => handleDragStart(e, idx)}
                                ondragover={handleDragOver}
                                ondrop={(e) => handleDrop(e, idx)}
                                class="p-3 bg-slate-950 border border-slate-800 rounded-xl flex items-center justify-between gap-3 hover:border-slate-700 transition-all cursor-grab active:cursor-grabbing group"
                            >
                                <div class="flex items-center space-x-2.5 min-w-0 flex-1">
                                    <span class="w-5 h-5 rounded-lg bg-slate-800 text-slate-400 text-[10px] font-bold flex items-center justify-center shrink-0">
                                        {idx + 1}
                                    </span>
                                    <p class="font-mono text-xs text-slate-300 truncate leading-relaxed">
                                        {item.content}
                                    </p>
                                </div>

                                <div class="flex items-center space-x-1 shrink-0">
                                    <button
                                        type="button"
                                        disabled={idx === 0}
                                        onclick={() => moveItem(idx, 'up')}
                                        class="p-1 text-slate-400 hover:text-white disabled:opacity-20 rounded"
                                        title="Nach oben"
                                    >
                                        ▲
                                    </button>
                                    <button
                                        type="button"
                                        disabled={idx === items.length - 1}
                                        onclick={() => moveItem(idx, 'down')}
                                        class="p-1 text-slate-400 hover:text-white disabled:opacity-20 rounded"
                                        title="Nach unten"
                                    >
                                        ▼
                                    </button>
                                    <button
                                        type="button"
                                        onclick={() => removeItem(idx)}
                                        class="p-1 text-rose-400 hover:text-rose-200 rounded ml-1"
                                        title="Entfernen"
                                    >
                                        <TrashIcon class="w-3.5 h-3.5" />
                                    </button>
                                </div>
                            </div>
                        {:else}
                            <p class="text-xs text-slate-500 italic py-4 text-center border border-dashed border-slate-800 rounded-xl">
                                Alle Elemente wurden entfernt.
                            </p>
                        {/each}
                    </div>
                </div>

                <!-- Live Preview -->
                <div class="space-y-1.5">
                    <span class="block text-xs font-semibold text-slate-300">Vorschau des erzeugten Snippets</span>
                    <div class="p-3 bg-slate-950 border border-slate-800 rounded-xl max-h-40 overflow-y-auto custom-scrollbar">
                        <pre class="font-mono text-xs text-indigo-200 whitespace-pre-wrap break-words leading-relaxed">{composedPreview || '(Leer)'}</pre>
                    </div>
                </div>
            </div>

            <!-- Footer -->
            <div class="flex items-center justify-between px-6 py-4 border-t border-slate-800 bg-slate-900/80 shrink-0">
                <span class="text-xs text-slate-400 font-mono">
                    {composedPreview.length} Zeichen · {items.length} Bausteine
                </span>
                <div class="flex items-center space-x-2">
                    <button
                        type="button"
                        onclick={onClose}
                        class="px-4 py-2 text-xs font-semibold text-slate-300 hover:text-white bg-slate-800 hover:bg-slate-700 rounded-xl transition-all"
                    >
                        Abbrechen
                    </button>
                    <button
                        type="button"
                        onclick={handleCompose}
                        disabled={items.length === 0 || isSubmitting}
                        class="px-4 py-2 text-xs font-semibold text-white bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 rounded-xl transition-all flex items-center space-x-1.5 shadow-lg shadow-indigo-950/50"
                    >
                        <PlusIcon class="w-4 h-4" />
                        <span>Snippet zusammenfügen</span>
                    </button>
                </div>
            </div>
        {/if}
    </div>
</div>
