<script lang="ts">
    import '../app.css';
    import { onMount } from 'svelte';
    import { page } from '$app/stores';
    import { listen } from '@tauri-apps/api/event';
    import { loadClipboardHistory } from '../lib/stores/clipboard';
    import ToastContainer from '../lib/components/shared/ToastContainer.svelte';

    let { children } = $props();
    let currentPath = $derived($page.url.pathname);

    onMount(async () => {
        // STATUS: Implemented (Phase 1 - Tauri clipboard event listener)
        try {
            await listen('clipboard:new_entry', async (event) => {
                console.log('New clipboard entry:', event.payload);
                await loadClipboardHistory();
            });
        } catch (e) {
            console.warn("Tauri event listener failed (expected in browser preview)", e);
        }
    });
</script>

<div class="flex h-screen bg-slate-950 text-slate-100 overflow-hidden font-sans">
    <aside class="w-64 bg-slate-900/90 border-r border-slate-800 flex flex-col backdrop-blur-md">
        <div class="p-5 border-b border-slate-800 flex items-center justify-between">
            <div class="flex items-center space-x-3">
                <div class="w-8 h-8 rounded-lg bg-gradient-to-tr from-blue-600 to-indigo-500 flex items-center justify-center font-bold text-white shadow-lg shadow-blue-500/20">
                    TF
                </div>
                <div>
                    <h1 class="font-extrabold text-base tracking-tight bg-gradient-to-r from-white to-slate-400 bg-clip-text text-transparent">TextForge</h1>
                    <span class="text-[10px] text-indigo-400 font-mono tracking-wide uppercase">v2.1 Pro</span>
                </div>
            </div>
            <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" title="Clipboard Watcher Active"></span>
        </div>

        <nav class="flex-1 p-4 space-y-1.5 overflow-y-auto">
            <a 
                href="/clipboard" 
                class="flex items-center space-x-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {currentPath.startsWith('/clipboard') ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-inner' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
            >
                <span class="text-lg">📋</span>
                <span>Clipboard History</span>
            </a>
            <a 
                href="/snippets" 
                class="flex items-center space-x-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {currentPath.startsWith('/snippets') ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-inner' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
            >
                <span class="text-lg">📝</span>
                <span>Snippets</span>
            </a>
            <a 
                href="/scripts" 
                class="flex items-center space-x-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {currentPath.startsWith('/scripts') ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-inner' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
            >
                <span class="text-lg">⚡</span>
                <span>Skripte</span>
            </a>
            <a 
                href="/pipelines" 
                class="flex items-center space-x-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {currentPath.startsWith('/pipelines') ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-inner' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
            >
                <span class="text-lg">🔀</span>
                <span>Pipelines</span>
            </a>
            <a 
                href="/settings" 
                class="flex items-center space-x-3 px-3.5 py-2.5 rounded-xl text-sm font-medium transition-all duration-150 {currentPath.startsWith('/settings') ? 'bg-indigo-600/20 text-indigo-300 border border-indigo-500/30 shadow-inner' : 'text-slate-400 hover:text-slate-200 hover:bg-slate-800/60'}"
            >
                <span class="text-lg">⚙️</span>
                <span>Einstellungen</span>
            </a>
        </nav>

        <div class="p-4 border-t border-slate-800 text-xs text-slate-500 flex items-center justify-between">
            <span>Desktop Engine</span>
            <span class="px-2 py-0.5 rounded bg-slate-800 font-mono text-[10px] text-slate-400">Wayland OK</span>
        </div>
    </aside>

    <main class="flex-1 overflow-auto relative bg-slate-950">
        {@render children()}
        <ToastContainer />
    </main>
</div>

