<script lang="ts">
    import '../app.css';
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { goto } from '$app/navigation';
    import { listen } from '@tauri-apps/api/event';
    import { loadClipboardHistory } from '../lib/stores/clipboard';
    import ToastContainer from '../lib/components/shared/ToastContainer.svelte';
    import CommandPalette from '../lib/components/shared/CommandPalette.svelte';
    import GlobalSearchBar from '$lib/components/search/GlobalSearchBar.svelte';
    import GlobalSearchResults from '$lib/components/search/GlobalSearchResults.svelte';
    import CollectionTabBar from '$lib/components/collections/CollectionTabBar.svelte';
    import CollectionTabEditor from '$lib/components/collections/CollectionTabEditor.svelte';
    import { initSession, updateSession } from '../lib/stores/session';
    import { performUndo, performRedo } from '../lib/stores/undo';
    import type { AppView } from '../lib/domain/session';

    // SVG Icons
    import ClipboardIcon from '$lib/components/icons/ClipboardIcon.svelte';
    import SnippetIcon from '$lib/components/icons/SnippetIcon.svelte';
    import ScriptIcon from '$lib/components/icons/ScriptIcon.svelte';
    import PipelineIcon from '$lib/components/icons/PipelineIcon.svelte';
    import SequenceIcon from '$lib/components/icons/SequenceIcon.svelte';
    import SettingsIcon from '$lib/components/icons/SettingsIcon.svelte';
    import SearchIcon from '$lib/components/icons/SearchIcon.svelte';

    let { children } = $props();
    let currentPath = $derived($page.url.pathname);
    let isCommandPaletteOpen = $state(false);
    let showCreateCollectionModal = $state(false);

    onMount(async () => {
        try {
            await initSession();
        } catch (e) {
            console.warn("Failed to init workspace session:", e);
        }

        try {
            await listen('clipboard:new_entry', async (event) => {
                console.log('New clipboard entry:', event.payload);
                await loadClipboardHistory();
            });
        } catch (e) {
            console.warn("Tauri event listener failed (expected in browser preview)", e);
        }
    });

    $effect(() => {
        let view: AppView = 'clipboard';
        if (currentPath.startsWith('/snippets')) view = 'snippets';
        else if (currentPath.startsWith('/scripts')) view = 'scripts';
        else if (currentPath.startsWith('/pipelines')) view = 'pipelines';
        else if (currentPath.startsWith('/settings')) view = 'settings';
        updateSession({ activeView: view });
    });

    function handleGlobalKeyDown(e: KeyboardEvent) {
        const ctrl = e.ctrlKey || e.metaKey;

        if (ctrl && e.shiftKey && (e.key === 'P' || e.key === 'p')) {
            e.preventDefault();
            isCommandPaletteOpen = !isCommandPaletteOpen;
            return;
        }

        if (ctrl && (e.key === 'z' || e.key === 'Z')) {
            if (e.shiftKey) {
                e.preventDefault();
                performRedo();
            } else {
                e.preventDefault();
                performUndo();
            }
            return;
        }

        if (ctrl && (e.key === 'y' || e.key === 'Y')) {
            e.preventDefault();
            performRedo();
            return;
        }

        if (ctrl && (e.key === 'n' || e.key === 'N')) {
            e.preventDefault();
            goto('/snippets');
            return;
        }

        if (ctrl && e.key === ',') {
            e.preventDefault();
            goto('/settings');
            return;
        }
    }
</script>

<svelte:window onkeydown={handleGlobalKeyDown} />

<div class="flex h-screen bg-slate-950 text-slate-100 overflow-hidden font-sans select-none">
    <!-- Compact Sidebar -->
    <aside class="w-60 bg-slate-900/90 border-r border-slate-800/80 flex flex-col backdrop-blur-md shrink-0">
        <!-- App Header Brand -->
        <div class="p-3.5 border-b border-slate-800/80 flex items-center justify-between">
            <div class="flex items-center space-x-2.5">
                <div class="w-8 h-8 rounded-xl bg-gradient-to-tr from-blue-600 via-indigo-600 to-violet-500 flex items-center justify-center font-black text-white shadow-lg shadow-indigo-500/20 border border-indigo-400/30 text-xs">
                    TF
                </div>
                <div>
                    <h1 class="font-black text-sm tracking-tight bg-gradient-to-r from-white via-slate-100 to-indigo-200 bg-clip-text text-transparent">TextForge</h1>
                    <span class="text-[9px] text-indigo-400 font-mono tracking-wider uppercase font-semibold">v2.1 Pro</span>
                </div>
            </div>
            <div class="flex items-center space-x-1 bg-emerald-500/10 border border-emerald-500/20 px-1.5 py-0.5 rounded-full" title="Wayland Clipboard Watcher Active">
                <span class="w-1.5 h-1.5 rounded-full bg-emerald-400 animate-pulse"></span>
                <span class="text-[9px] font-mono text-emerald-300 font-semibold">Live</span>
            </div>
        </div>

        <!-- Command Palette Trigger -->
        <div class="px-2.5 pt-2.5">
            <button
                onclick={() => isCommandPaletteOpen = true}
                class="w-full px-2.5 py-1.5 bg-slate-950/80 hover:bg-slate-800/80 border border-slate-800 rounded-xl text-xs text-slate-400 flex items-center justify-between transition-all group shadow-inner"
            >
                <span class="flex items-center space-x-2">
                    <SearchIcon class="w-3.5 h-3.5 text-indigo-400" />
                    <span class="group-hover:text-slate-200 transition-colors text-[11px]">Befehl suchen...</span>
                </span>
                <kbd class="px-1 py-0.5 text-[9px] font-mono bg-slate-800 text-indigo-300 rounded border border-slate-700">Ctrl+Shift+P</kbd>
            </button>
        </div>

        <!-- Main Navigation Links -->
        <nav class="flex-1 p-2.5 space-y-1 overflow-y-auto">
            <a 
                href="/clipboard" 
                class="flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-150 {currentPath.startsWith('/clipboard') ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border border-transparent'}"
            >
                <div class="flex items-center space-x-2.5">
                    <ClipboardIcon class="w-4 h-4 text-indigo-400" />
                    <span>Zwischenablage</span>
                </div>
                <span class="text-[9px] text-slate-600 font-mono">Alt+1</span>
            </a>
            <a 
                href="/snippets" 
                class="flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-150 {currentPath.startsWith('/snippets') ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border border-transparent'}"
            >
                <div class="flex items-center space-x-2.5">
                    <SnippetIcon class="w-4 h-4 text-indigo-400" />
                    <span>Snippets</span>
                </div>
                <span class="text-[9px] text-slate-600 font-mono">Ctrl+N</span>
            </a>
            <a 
                href="/scripts" 
                class="flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-150 {currentPath.startsWith('/scripts') ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border border-transparent'}"
            >
                <div class="flex items-center space-x-2.5">
                    <ScriptIcon class="w-4 h-4 text-indigo-400" />
                    <span>Skripte</span>
                </div>
                <span class="text-[9px] text-slate-600 font-mono">Alt+3</span>
            </a>
            <a 
                href="/pipelines" 
                class="flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-150 {currentPath.startsWith('/pipelines') ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border border-transparent'}"
            >
                <div class="flex items-center space-x-2.5">
                    <PipelineIcon class="w-4 h-4 text-indigo-400" />
                    <span>Pipelines</span>
                </div>
                <span class="text-[9px] text-slate-600 font-mono">Alt+4</span>
            </a>
            <a
                href="/sequences"
                class="flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-150 {currentPath.startsWith('/sequences') ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border border-transparent'}"
            >
                <div class="flex items-center space-x-2.5">
                    <SequenceIcon class="w-4 h-4 text-indigo-400" />
                    <span>Sequenzen</span>
                </div>
                <span class="text-[9px] text-slate-600 font-mono">Alt+5</span>
            </a>

            <!-- Custom Tabs in Sidebar -->
            <div class="pt-3 border-t border-slate-800/60 mt-3">
                <CollectionTabBar variant="vertical" onOpenCreateModal={() => showCreateCollectionModal = true} />
            </div>

            <a 
                href="/settings" 
                class="flex items-center justify-between px-3 py-2 rounded-xl text-xs font-semibold transition-all duration-150 mt-4 {currentPath.startsWith('/settings') ? 'bg-indigo-600/20 text-indigo-200 border border-indigo-500/40 shadow-sm' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/50 border border-transparent'}"
            >
                <div class="flex items-center space-x-2.5">
                    <SettingsIcon class="w-4 h-4 text-indigo-400" />
                    <span>Einstellungen</span>
                </div>
                <span class="text-[9px] text-slate-600 font-mono">Ctrl+,</span>
            </a>
        </nav>

        <!-- Sidebar Footer Status -->
        <div class="p-2.5 border-t border-slate-800/80 text-xs text-slate-500 flex items-center justify-between bg-slate-950/40">
            <span class="text-[10px] font-medium text-slate-400">Wayland Desktop</span>
            <span class="px-1.5 py-0.5 rounded bg-indigo-950/60 border border-indigo-800/50 font-mono text-[9px] text-indigo-300 font-semibold">Active</span>
        </div>
    </aside>

    <main class="flex-1 overflow-auto relative bg-slate-950 select-text flex flex-col">
        <!-- Top Search Header & Collection Tab Bar -->
        <div class="p-2.5 border-b border-slate-800/80 bg-slate-900/60 backdrop-blur-md space-y-2">
            <GlobalSearchBar />
            <CollectionTabBar variant="horizontal" onOpenCreateModal={() => showCreateCollectionModal = true} />
        </div>

        <div class="flex-1 overflow-auto p-2">
            <GlobalSearchResults />
            {@render children()}
        </div>

        {#if showCreateCollectionModal}
            <CollectionTabEditor onClose={() => showCreateCollectionModal = false} />
        {/if}

        <ToastContainer />
        <CommandPalette bind:isOpen={isCommandPaletteOpen} />
    </main>
</div>
