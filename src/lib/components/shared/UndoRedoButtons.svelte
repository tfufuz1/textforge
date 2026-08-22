<script lang="ts">
  import { onMount } from 'svelte';
  import { undoStateStore, performUndo, performRedo, refreshUndoState } from '../../stores/undo';

  onMount(() => {
    refreshUndoState();
  });
</script>

<div class="undo-redo-buttons flex items-center gap-1">
  <button
    class="btn-icon p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700/50 disabled:opacity-30 disabled:hover:bg-transparent transition-colors"
    disabled={!$undoStateStore.canUndo}
    onclick={performUndo}
    aria-label={$undoStateStore.topUndoDescription ? `Rückgängig: ${$undoStateStore.topUndoDescription}` : 'Rückgängig (Ctrl+Z)'}
    title={$undoStateStore.topUndoDescription ? `Rückgängig: ${$undoStateStore.topUndoDescription}` : 'Rückgängig (Ctrl+Z)'}
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M3 7v6h6" />
      <path d="M21 17a9 9 0 0 0-9-9 9 9 0 0 0-6 2.3L3 13" />
    </svg>
  </button>

  <button
    class="btn-icon p-1.5 rounded text-gray-400 hover:text-white hover:bg-gray-700/50 disabled:opacity-30 disabled:hover:bg-transparent transition-colors"
    disabled={!$undoStateStore.canRedo}
    onclick={performRedo}
    aria-label="Wiederherstellen (Ctrl+Y)"
    title="Wiederherstellen (Ctrl+Y)"
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
      <path d="M21 7v6h-6" />
      <path d="M3 17a9 9 0 0 1 9-9 9 9 0 0 1 6 2.3l3 2.7" />
    </svg>
  </button>
</div>
