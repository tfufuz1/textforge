<script lang="ts">
    import { notificationsStore, dismissNotification } from '../../stores/notifications';
    import { performUndo } from '../../stores/undo';

    function handleAction(action: { label: string, handler: string }) {
        if (action.handler === 'undo') {
            performUndo();
        }
    }
</script>

<div class="fixed bottom-4 right-4 z-50 flex flex-col space-y-2">
    {#each $notificationsStore as notification (notification.id)}
        <div class="px-4 py-3 rounded shadow-lg flex items-center justify-between text-sm text-white max-w-sm
            {notification.severity === 'success' ? 'bg-green-600' : ''}
            {notification.severity === 'error' ? 'bg-red-600' : ''}
            {notification.severity === 'info' ? 'bg-blue-600' : ''}
            {notification.severity === 'warning' ? 'bg-yellow-600' : ''}">
            
            <div>
                <strong class="block">{notification.title}</strong>
                {#if notification.message._tag === 'Some'}
                    <span class="opacity-90">{notification.message.value}</span>
                {/if}
            </div>
            
            <div class="ml-4 flex items-center space-x-2">
                {#if notification.action._tag === 'Some'}
                    {@const action = notification.action.value}
                    <button class="underline hover:no-underline font-bold" onclick={() => handleAction(action)}>
                        {action.label}
                    </button>
                {/if}
                <button class="text-white opacity-70 hover:opacity-100" onclick={() => dismissNotification(notification.id)}>
                    ✕
                </button>
            </div>
        </div>
    {/each}
</div>