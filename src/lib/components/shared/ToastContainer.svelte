<script lang="ts">
    import { notificationsStore, dismissNotification } from '../../stores/notifications';
    import { performUndo } from '../../stores/undo';

    function handleAction(action: { label: string, handler: string }) {
        if (action.handler === 'undo') {
            performUndo();
        }
    }
</script>

<div
    class="fixed bottom-5 right-5 z-50 flex flex-col space-y-2.5 max-w-md w-full pointer-events-none"
    role="region"
    aria-label="Benachrichtigungen"
    aria-live="polite"
>
    {#each $notificationsStore as notification (notification.id)}
        <div
            class="pointer-events-auto p-4 rounded-2xl bg-slate-900/95 border border-slate-800/90 shadow-2xl shadow-black/50 backdrop-blur-xl flex items-start justify-between gap-3 text-slate-100 transition-all duration-200 animate-in fade-in slide-in-from-bottom-2
            {notification.severity === 'success' ? 'border-l-4 border-l-emerald-500' : ''}
            {notification.severity === 'error' ? 'border-l-4 border-l-rose-500' : ''}
            {notification.severity === 'info' ? 'border-l-4 border-l-indigo-500' : ''}
            {notification.severity === 'warning' ? 'border-l-4 border-l-amber-500' : ''}"
        >
            <div class="flex items-start space-x-3 flex-1 min-w-0">
                <div class="mt-0.5 text-base shrink-0" aria-hidden="true">
                    {#if notification.severity === 'success'}
                        <span class="text-emerald-400">✓</span>
                    {:else if notification.severity === 'error'}
                        <span class="text-rose-400">⚠️</span>
                    {:else if notification.severity === 'warning'}
                        <span class="text-amber-400">⚡</span>
                    {:else}
                        <span class="text-indigo-400">ℹ️</span>
                    {/if}
                </div>
                <div class="flex-1 min-w-0">
                    <h4 class="text-xs font-bold tracking-wide text-white leading-snug">{notification.title}</h4>
                    {#if notification.message._tag === 'Some'}
                        <p class="text-xs text-slate-400 mt-0.5 break-words leading-relaxed">{notification.message.value}</p>
                    {/if}
                </div>
            </div>
            
            <div class="flex items-center space-x-2 shrink-0 self-center">
                {#if notification.action._tag === 'Some'}
                    {@const action = notification.action.value}
                    <button
                        class="px-2.5 py-1 text-xs font-bold bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg transition-all shadow-md shadow-indigo-600/20 active:scale-95 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
                        onclick={() => handleAction(action)}
                    >
                        {action.label}
                    </button>
                {/if}
                <button
                    class="p-1 rounded-lg text-slate-500 hover:text-slate-300 hover:bg-slate-800/80 transition-all text-xs focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-indigo-400"
                    aria-label="Benachrichtigung schließen"
                    title="Benachrichtigung schließen"
                    onclick={() => dismissNotification(notification.id)}
                >
                    ✕
                </button>
            </div>
        </div>
    {/each}
</div>
