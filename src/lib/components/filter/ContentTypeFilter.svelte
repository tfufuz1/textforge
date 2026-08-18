<script lang="ts">
    import { snippetFilterStore, loadSnippets } from '../../stores/snippets';

    let selectedTypes = $state<string[]>([]);

    const contentTypes = [
        { value: 'plain_text', label: 'Plain Text' },
        { value: 'markdown', label: 'Markdown' },
        { value: 'json', label: 'JSON' },
        { value: 'javascript', label: 'JavaScript' },
        { value: 'typescript', label: 'TypeScript' },
        { value: 'python', label: 'Python' },
        { value: 'sql', label: 'SQL' },
        { value: 'url', label: 'URL' }
    ];

    $effect(() => {
        const storeTypes = $snippetFilterStore.contentTypes || [];
        if (JSON.stringify(storeTypes) !== JSON.stringify(selectedTypes)) {
            selectedTypes = [...storeTypes];
        }
    });

    function toggleType(type: string) {
        if (selectedTypes.includes(type)) {
            selectedTypes = selectedTypes.filter(t => t !== type);
        } else {
            selectedTypes = [...selectedTypes, type];
        }
        snippetFilterStore.update(f => ({
            ...f,
            contentTypes: selectedTypes
        }));
        loadSnippets();
    }
</script>

<div class="space-y-3">
    <h3 class="text-xs font-semibold text-slate-400 uppercase tracking-wider font-mono">Typen</h3>
    <div class="grid grid-cols-2 gap-1.5">
        {#each contentTypes as type}
            <button
                onclick={() => toggleType(type.value)}
                class="px-2 py-1 text-[10px] font-medium rounded-lg border text-left transition-all truncate {selectedTypes.includes(type.value) ? 'bg-indigo-600/30 border-indigo-500 text-indigo-200' : 'bg-slate-900/60 border-slate-800 text-slate-400 hover:text-slate-200 hover:bg-slate-800'}"
            >
                {type.label}
            </button>
        {/each}
    </div>
</div>
