<script lang="ts">
    import { pinEntry, deleteEntry, promoteToSnippet, getClipboardEntry, writeToClipboard } from '../../ipc/clipboard';
    import { loadClipboardHistory } from '../../stores/clipboard';
    import { refreshUndoState } from '../../stores/undo';
    import { pushNotification, Notifications } from '../../stores/notifications';

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
        await promoteToSnippet(entry.id, null, { _type: 'inbox', folderId: null });
        await loadClipboardHistory();
        await refreshUndoState();
        pushNotification(Notifications.snippetSaved("Clipboard-Import"));
        pushNotification(Notifications.undoAvailable("Snippet aus Zwischenablage erstellt"));
    }
</script>

<div class="actions flex items-center space-x-2 shrink-0">
    {#if entry.promotedToSnippetId}
        <span class="px-2.5 py-1 bg-emerald-950/80 text-emerald-300 border border-emerald-800/40 rounded-lg text-xs font-medium">✓ Als Snippet</span>
    {:else}
        <button class="px-2.5 py-1 bg-indigo-950 text-indigo-300 hover:bg-indigo-900 border border-indigo-700/50 rounded-lg text-xs font-medium transition-colors" onclick={handlePromote}>
            + Snippet
        </button>
    {/if}
    <button class="px-2.5 py-1 bg-slate-800 text-slate-300 hover:bg-slate-700 border border-slate-700/50 rounded-lg text-xs font-medium transition-colors" onclick={handleCopy} title="Kopieren">
        📋 Kopieren
    </button>
    <button class="px-2.5 py-1 bg-slate-800 text-slate-300 hover:bg-slate-700 border border-slate-700/50 rounded-lg text-xs font-medium transition-colors" onclick={handlePin}>
        {entry.isPinned ? '📌 Pinned' : '📍 Pin'}
    </button>
    <button class="p-1.5 bg-rose-950/40 text-slate-400 hover:text-rose-300 hover:bg-rose-950/80 border border-rose-900/40 rounded-lg text-xs transition-colors" onclick={handleDelete} title="Löschen">
        🗑️
    </button>
</div>

