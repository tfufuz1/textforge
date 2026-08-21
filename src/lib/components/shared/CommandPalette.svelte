<script lang="ts">
    import { goto } from '$app/navigation';
    import { SHORTCUT_REGISTRY, type ShortcutCommand } from '../../shortcuts/registry';
    import { performUndo, performRedo } from '../../stores/undo';

    let { isOpen = $bindable(false) } = $props();

    let query = $state('');
    let selectedIndex = $state(0);

    const commands = [
        { id: 'nav_clipboard', title: 'Gehe zu Clipboard History', icon: '📋', category: 'Navigation', action: () => goto('/clipboard') },
        { id: 'nav_snippets', title: 'Gehe zu Snippets', icon: '📝', category: 'Navigation', action: () => goto('/snippets') },
        { id: 'nav_scripts', title: 'Gehe zu Skripten', icon: '⚡', category: 'Navigation', action: () => goto('/scripts') },
        { id: 'nav_pipelines', title: 'Gehe zu Pipelines', icon: '🔀', category: 'Navigation', action: () => goto('/pipelines') },
        { id: 'nav_settings', title: 'Gehe zu Einstellungen', icon: '⚙️', category: 'Navigation', action: () => goto('/settings') },
        { id: 'action_undo', title: 'Aktion rückgängig machen (Undo)', icon: '↩️', category: 'Aktion', action: () => performUndo() },
        { id: 'action_redo', title: 'Aktion wiederholen (Redo)', icon: '↪️', category: 'Aktion', action: () => performRedo() },
    ];

    let filtered = $derived.by(() => {
        const q = query.trim().toLowerCase();
        if (!q) return commands;
        return commands.filter(c =>
            c.title.toLowerCase().includes(q) || c.category.toLowerCase().includes(q)
        );
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
                item.action();
                isOpen = false;
            }
            e.preventDefault();
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

{#if isOpen}
    <div class="fixed inset-0 z-50 bg-black/80 backdrop-blur-sm flex items-start justify-center pt-20 p-4">
        <div class="bg-slate-900 border border-slate-800 rounded-2xl w-full max-w-xl shadow-2xl overflow-hidden flex flex-col">
            <!-- Search Input -->
            <div class="p-4 border-b border-slate-800/80 flex items-center space-x-3 bg-slate-950/60">
                <span class="text-slate-400 text-lg">🔍</span>
                <input
                    type="text"
                    bind:value={query}
                    placeholder="Befehl oder Navigation suchen (Strg+Shift+P)..."
                    class="w-full bg-transparent text-white placeholder-slate-500 border-none outline-none text-sm font-medium"
                    autofocus
                />
                <kbd class="px-2 py-1 text-[10px] font-mono bg-slate-800 text-slate-400 rounded-lg border border-slate-700">ESC</kbd>
            </div>

            <!-- Command List -->
            <div class="max-h-80 overflow-y-auto p-2 space-y-1 custom-scrollbar">
                {#if filtered.length === 0}
                    <div class="p-6 text-center text-xs text-slate-500 font-mono">Keine passenden Befehle gefunden.</div>
                {:else}
                    {#each filtered as cmd, idx}
                        <button
                            onclick={() => { cmd.action(); isOpen = false; }}
                            class="w-full text-left px-3.5 py-2.5 rounded-xl flex items-center justify-between transition-colors text-xs font-medium {idx === selectedIndex ? 'bg-indigo-600/30 text-white border border-indigo-500/40' : 'text-slate-300 hover:bg-slate-800/60'}"
                        >
                            <div class="flex items-center space-x-3">
                                <span class="text-base">{cmd.icon}</span>
                                <div>
                                    <div class="font-semibold text-slate-100">{cmd.title}</div>
                                    <div class="text-[10px] text-slate-400">{cmd.category}</div>
                                </div>
                            </div>
                            <span class="text-[10px] font-mono text-slate-500">↵ Auswählen</span>
                        </button>
                    {/each}
                {/if}
            </div>

            <!-- Shortcuts Reference Footer -->
            <div class="p-3 bg-slate-950 border-t border-slate-800/80 flex justify-between items-center text-[10px] text-slate-500 font-mono">
                <span>↑↓ Navigieren · ↵ Bestätigen · ESC Schließen</span>
                <span class="text-indigo-400 font-bold">TextForge Command Palette</span>
            </div>
        </div>
    </div>
{/if}
