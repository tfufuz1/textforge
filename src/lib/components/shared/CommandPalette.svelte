<script lang="ts">
    import { goto } from '$app/navigation';
    import { performUndo, performRedo } from '../../stores/undo';

    let {
        isOpen = $bindable(false),
        onOpenQuickCapture = () => {},
        onFocusSearch = () => {},
        onDuplicateFocusedSnippet = () => {},
        onCopyTransformedResult = () => {},
    } = $props<{
        isOpen?: boolean;
        onOpenQuickCapture?: () => void;
        onFocusSearch?: () => void;
        onDuplicateFocusedSnippet?: () => void;
        onCopyTransformedResult?: () => void;
    }>();

    let query = $state('');
    let selectedIndex = $state(0);

    const commands = $derived([
        { id: 'quick_capture', title: 'Neues Snippet aus Zwischenablage (Quick Capture)', icon: '⚡', category: 'Snippet-Aktionen', shortcut: 'Ctrl+Alt+V', action: () => onOpenQuickCapture() },
        { id: 'duplicate_snippet', title: 'Aktuell fokussiertes Snippet duplizieren', icon: '👯', category: 'Snippet-Aktionen', shortcut: 'Ctrl+D', action: () => onDuplicateFocusedSnippet() },
        { id: 'copy_transformed', title: 'Fokussiertes Snippet-Ergebnis kopieren', icon: '📋', category: 'Snippet-Aktionen', shortcut: 'Ctrl+Shift+C', action: () => onCopyTransformedResult() },
        { id: 'quick_search', title: 'Schnellsuche über Snippet-Bibliothek', icon: '🔍', category: 'Suche', shortcut: 'Ctrl+K', action: () => onFocusSearch() },

        { id: 'nav_clipboard', title: 'Gehe zu Zwischenablage (Clipboard)', icon: '📥', category: 'Navigation', shortcut: 'Alt+1', action: () => goto('/clipboard') },
        { id: 'nav_snippets', title: 'Gehe zu Snippets', icon: '📝', category: 'Navigation', shortcut: 'Ctrl+N', action: () => goto('/snippets') },
        { id: 'nav_scripts', title: 'Gehe zu Skripten', icon: '⚙️', category: 'Navigation', shortcut: 'Alt+3', action: () => goto('/scripts') },
        { id: 'nav_pipelines', title: 'Gehe zu Pipelines', icon: '🔀', category: 'Navigation', shortcut: 'Alt+4', action: () => goto('/pipelines') },
        { id: 'nav_settings', title: 'Gehe zu Einstellungen', icon: '🔧', category: 'Navigation', shortcut: 'Ctrl+,', action: () => goto('/settings') },

        { id: 'action_undo', title: 'Aktion rückgängig machen (Undo)', icon: '↩️', category: 'Aktionen', shortcut: 'Ctrl+Z', action: () => performUndo() },
        { id: 'action_redo', title: 'Aktion wiederholen (Redo)', icon: '↪️', category: 'Aktionen', shortcut: 'Ctrl+Shift+Z', action: () => performRedo() },
    ]);

    let filtered = $derived.by(() => {
        const q = query.trim().toLowerCase();
        if (!q) return commands;
        return commands.filter(c =>
            c.title.toLowerCase().includes(q) || c.category.toLowerCase().includes(q)
        );
    });

    $effect(() => {
        // Reset selection index when search query changes
        query;
        selectedIndex = 0;
    });

    function handleKeyDown(e: KeyboardEvent) {
        if (!isOpen) return;

        if (e.key === 'Escape') {
            isOpen = false;
            e.preventDefault();
        } else if (e.key === 'ArrowDown') {
            selectedIndex = (selectedIndex + 1) % (filtered.length || 1);
            e.preventDefault();
        } else if (e.key === 'ArrowUp') {
            selectedIndex = (selectedIndex - 1 + filtered.length) % (filtered.length || 1);
            e.preventDefault();
        } else if (e.key === 'Enter') {
            const item = filtered[selectedIndex];
            if (item) {
                isOpen = false;
                item.action();
            }
            e.preventDefault();
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
    <!-- Backdrop -->
    <div
        class="fixed inset-0 z-50 bg-slate-950/80 backdrop-blur-md flex items-start justify-center pt-24 p-4 animate-in fade-in duration-150"
        onclick={(e) => { if (e.target === e.currentTarget) isOpen = false; }}
        role="presentation"
    >
        <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-xl shadow-2xl shadow-indigo-950/30 overflow-hidden flex flex-col border-indigo-500/20">
            <!-- Search Bar -->
            <div class="p-4 border-b border-slate-800 flex items-center space-x-3 bg-slate-950/80">
                <span class="text-indigo-400 text-lg">🔍</span>
                <input
                    type="text"
                    bind:value={query}
                    placeholder="Befehl, Aktion oder Navigation suchen..."
                    class="w-full bg-transparent text-white placeholder-slate-500 border-none outline-none text-sm font-medium focus:ring-0"
                    autofocus
                />
                <div class="flex items-center space-x-1">
                    <kbd class="px-2 py-1 text-[10px] font-mono bg-slate-800/90 text-slate-400 rounded-lg border border-slate-700 shadow-sm">ESC</kbd>
                </div>
            </div>

            <!-- Command List -->
            <div class="max-h-88 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                {#if filtered.length === 0}
                    <div class="py-10 px-4 text-center">
                        <div class="text-2xl mb-2">🔎</div>
                        <p class="text-xs font-semibold text-slate-400">Keine Befehle für "{query}" gefunden</p>
                        <p class="text-[11px] text-slate-500 mt-1">Versuche andere Suchbegriffe wie 'Snippet', 'Quick Capture' oder 'Undo'</p>
                    </div>
                {:else}
                    {#each filtered as cmd, idx}
                        <button
                            onclick={() => { isOpen = false; cmd.action(); }}
                            class="w-full text-left px-3.5 py-3 rounded-xl flex items-center justify-between transition-all text-xs font-medium group {idx === selectedIndex ? 'bg-indigo-600/25 text-white border border-indigo-500/40 shadow-inner' : 'text-slate-300 hover:bg-slate-800/60 border border-transparent'}"
                        >
                            <div class="flex items-center space-x-3 min-w-0">
                                <span class="text-lg p-1.5 rounded-lg bg-slate-800/80 border border-slate-700/50 group-hover:scale-105 transition-transform">{cmd.icon}</span>
                                <div class="truncate">
                                    <div class="font-semibold text-slate-100 truncate">{cmd.title}</div>
                                    <div class="text-[10px] text-slate-400 font-mono mt-0.5">{cmd.category}</div>
                                </div>
                            </div>

                            <div class="flex items-center space-x-2 shrink-0 ml-3">
                                {#if cmd.shortcut}
                                    <kbd class="px-2 py-0.5 text-[10px] font-mono bg-slate-800 text-indigo-300 rounded-md border border-slate-700/80">{cmd.shortcut}</kbd>
                                {/if}
                                <span class="text-[10px] font-mono text-slate-500 group-hover:text-slate-300 transition-colors">↵</span>
                            </div>
                        </button>
                    {/each}
                {/if}
            </div>

            <!-- Footer -->
            <div class="px-4 py-2.5 bg-slate-950 border-t border-slate-800/80 flex justify-between items-center text-[10px] text-slate-500 font-mono">
                <div class="flex items-center space-x-3">
                    <span><kbd class="text-slate-400">↑↓</kbd> Navigieren</span>
                    <span><kbd class="text-slate-400">↵</kbd> Auswählen</span>
                </div>
                <span class="text-indigo-400 font-semibold flex items-center space-x-1">
                    <span>⚡</span>
                    <span>TextForge Palette</span>
                </span>
            </div>
        </div>
    </div>
{/if}
