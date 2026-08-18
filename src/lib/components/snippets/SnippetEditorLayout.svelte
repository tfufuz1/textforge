<script lang="ts">
    import { onMount } from 'svelte';
    
    let { viewMode = $bindable<'editor' | 'preview' | 'split'>('editor'), children } = $props();

    function handleKeyDown(e: KeyboardEvent) {
        if (e.ctrlKey && e.shiftKey && e.key.toLowerCase() === 'm') {
            e.preventDefault();
            if (viewMode === 'editor') {
                viewMode = 'preview';
            } else if (viewMode === 'preview') {
                viewMode = 'split';
            } else {
                viewMode = 'editor';
            }
        }
    }

    onMount(() => {
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    });
</script>

<div class="flex-1 flex flex-col min-h-0">
    {@render children()}
</div>
