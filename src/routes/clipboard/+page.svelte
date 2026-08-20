<script lang="ts">
    import ClipboardHistory from '../../lib/components/clipboard/ClipboardHistory.svelte';
</script>

<div class="h-full flex flex-col p-6 space-y-5 bg-slate-950 text-slate-100 overflow-hidden">
    <div>
        <h1 class="text-2xl font-extrabold tracking-tight text-white flex items-center space-x-2">
            <span>📋</span>
            <span>Clipboard Historie</span>
        </h1>
        <p class="text-xs text-slate-400 mt-1">Automatisch erfasste Zwischenablage mit Wayland Integration</p>
    </div>

    <div class="flex-1 overflow-hidden min-h-0 bg-slate-900/60 rounded-2xl border border-slate-800 p-5 shadow-xl backdrop-blur-md flex flex-col">
        <ClipboardHistory />
    </div>
</div>
