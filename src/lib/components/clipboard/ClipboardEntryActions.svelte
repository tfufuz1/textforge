<script lang="ts">
    import { goto } from '$app/navigation';
    import { pinEntry, deleteEntry, promoteToSnippet, getClipboardEntry, writeToClipboard } from '../../ipc/clipboard';
    import { loadClipboardHistory } from '../../stores/clipboard';
    import { loadSnippets, selectSnippet } from '../../stores/snippets';
    import { refreshUndoState } from '../../stores/undo';
    import { pushNotification, Notifications } from '../../stores/notifications';

    import CopyIcon from '$lib/components/icons/CopyIcon.svelte';
    import PinIcon from '$lib/components/icons/PinIcon.svelte';
    import TrashIcon from '$lib/components/icons/TrashIcon.svelte';
    import PlusIcon from '$lib/components/icons/PlusIcon.svelte';
    import CheckIcon from '$lib/components/icons/CheckIcon.svelte';

    let { entry } = $props();

    async function handleCopy() {
        try {
            const detail = await getClipboardEntry(entry.id);
            await writeToClipboard(detail.content);
            pushNotification(Notifications.snippetCopied());
        } catch (e) {
            console.error("Failed to copy clipboard entry:", e);
        }
    }

    async function handlePin() {
        await pinEntry(entry.id, !entry.isPinned);
        await loadClipboardHistory();
    }

    async function handleDelete() {
        await deleteEntry(entry.id);
        await loadClipboardHistory();
    }

    async function handlePromote() {
        try {
            const snippetId = await promoteToSnippet(entry.id, null, { _type: 'inbox', folderId: null });
            await loadClipboardHistory();
            await loadSnippets();
            await refreshUndoState();
            pushNotification(Notifications.snippetSaved("Clipboard-Import"));
            pushNotification(Notifications.undoAvailable("Snippet aus Zwischenablage erstellt"));
        } catch (e) {
            console.error("Failed to promote clipboard entry:", e);
        }
    }

    async function handleViewSnippet() {
        if (entry.promotedToSnippetId) {
            await selectSnippet(entry.promotedToSnippetId);
            await goto('/snippets');
        }
    }
</script>

<div class="actions flex items-center space-x-1.5 shrink-0">
    {#if entry.promotedToSnippetId}
        <button
            class="px-2.5 py-1 bg-emerald-950/80 hover:bg-emerald-900 text-emerald-300 border border-emerald-800/40 rounded-xl text-xs font-semibold flex items-center space-x-1 transition-all cursor-pointer shadow-sm"
            onclick={handleViewSnippet}
            title="Snippet anzeigen und bearbeiten"
        >
            <CheckIcon class="w-3.5 h-3.5 text-emerald-400" />
            <span>Snippet</span>
        </button>
    {:else}
        <button
            class="px-2.5 py-1 bg-indigo-950 hover:bg-indigo-900 text-indigo-300 border border-indigo-700/50 rounded-xl text-xs font-semibold transition-all flex items-center space-x-1 shadow-sm"
            onclick={handlePromote}
            title="Als Snippet speichern"
        >
            <PlusIcon class="w-3.5 h-3.5 text-indigo-400" />
            <span>Snippet</span>
        </button>
    {/if}

    <button
        class="p-1.5 bg-slate-800/90 text-slate-300 hover:text-white hover:bg-slate-700 border border-slate-700/60 rounded-xl transition-all flex items-center justify-center"
        onclick={handleCopy}
        title="In Zwischenablage kopieren"
    >
        <CopyIcon class="w-3.5 h-3.5" />
    </button>

    <button
        class="p-1.5 border rounded-xl transition-all flex items-center justify-center {entry.isPinned ? 'bg-amber-500/20 text-amber-300 border-amber-500/40' : 'bg-slate-800/90 text-slate-300 hover:text-white hover:bg-slate-700 border-slate-700/60'}"
        onclick={handlePin}
        title={entry.isPinned ? 'Entpinnen' : 'Pinnen'}
    >
        <PinIcon class="w-3.5 h-3.5" filled={entry.isPinned} />
    </button>

    <button
        class="p-1.5 bg-rose-950/40 text-rose-400 hover:text-rose-200 hover:bg-rose-900/80 border border-rose-900/40 rounded-xl transition-all flex items-center justify-center"
        onclick={handleDelete}
        title="Löschen"
    >
        <TrashIcon class="w-3.5 h-3.5" />
    </button>
</div>
