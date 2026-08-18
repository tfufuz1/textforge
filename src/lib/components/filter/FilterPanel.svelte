<script lang="ts">
    import LocationFilter from './LocationFilter.svelte';
    import TagFilter from './TagFilter.svelte';
    import DateRangeFilter from './DateRangeFilter.svelte';
    import SizeFilter from './SizeFilter.svelte';
    import ContentTypeFilter from './ContentTypeFilter.svelte';
    import QuickFilterBar from './QuickFilterBar.svelte';
    import FolderTree from '../snippets/FolderTree.svelte';

    let openSections = $state<Record<string, boolean>>({
        locations: true,
        quick: true,
        tags: true,
        types: false,
        date: false,
        size: false
    });

    function toggleSection(section: string) {
        openSections[section] = !openSections[section];
    }
</script>

<div class="space-y-4">
    <!-- Header/Title -->
    <div class="flex items-center justify-between pb-2 border-b border-slate-800">
        <span class="text-xs font-bold font-mono tracking-widest text-indigo-400 uppercase">Filter-Optionen</span>
        <span class="text-xs">⚙️</span>
    </div>

    <!-- Orte Section -->
    <div class="space-y-2">
        <button 
            onclick={() => toggleSection('locations')} 
            class="w-full flex justify-between items-center text-xs font-semibold text-slate-500 hover:text-slate-300 font-mono"
        >
            <span>ORTE & ORDNER</span>
            <span>{openSections.locations ? '▼' : '▶'}</span>
        </button>
        {#if openSections.locations}
            <div class="pl-1.5 space-y-2">
                <FolderTree />
                <LocationFilter />
            </div>
        {/if}
    </div>

    <!-- Schnellfilter & Sortierung Section -->
    <div class="space-y-2 pt-2 border-t border-slate-900/60">
        <button 
            onclick={() => toggleSection('quick')} 
            class="w-full flex justify-between items-center text-xs font-semibold text-slate-500 hover:text-slate-300 font-mono"
        >
            <span>SORTIERUNG & TOGGLES</span>
            <span>{openSections.quick ? '▼' : '▶'}</span>
        </button>
        {#if openSections.quick}
            <div class="pl-1.5">
                <QuickFilterBar />
            </div>
        {/if}
    </div>

    <!-- Tags Section -->
    <div class="space-y-2 pt-2 border-t border-slate-900/60">
        <button 
            onclick={() => toggleSection('tags')} 
            class="w-full flex justify-between items-center text-xs font-semibold text-slate-500 hover:text-slate-300 font-mono"
        >
            <span>SCHLAGWORTE</span>
            <span>{openSections.tags ? '▼' : '▶'}</span>
        </button>
        {#if openSections.tags}
            <div class="pl-1.5">
                <TagFilter />
            </div>
        {/if}
    </div>

    <!-- Content Types Section -->
    <div class="space-y-2 pt-2 border-t border-slate-900/60">
        <button 
            onclick={() => toggleSection('types')} 
            class="w-full flex justify-between items-center text-xs font-semibold text-slate-500 hover:text-slate-300 font-mono"
        >
            <span>INHALTSTYPEN</span>
            <span>{openSections.types ? '▼' : '▶'}</span>
        </button>
        {#if openSections.types}
            <div class="pl-1.5">
                <ContentTypeFilter />
            </div>
        {/if}
    </div>

    <!-- Zeitraum Section -->
    <div class="space-y-2 pt-2 border-t border-slate-900/60">
        <button 
            onclick={() => toggleSection('date')} 
            class="w-full flex justify-between items-center text-xs font-semibold text-slate-500 hover:text-slate-300 font-mono"
        >
            <span>ZEITRAUM</span>
            <span>{openSections.date ? '▼' : '▶'}</span>
        </button>
        {#if openSections.date}
            <div class="pl-1.5">
                <DateRangeFilter />
            </div>
        {/if}
    </div>

    <!-- Größe Section -->
    <div class="space-y-2 pt-2 border-t border-slate-900/60">
        <button 
            onclick={() => toggleSection('size')} 
            class="w-full flex justify-between items-center text-xs font-semibold text-slate-500 hover:text-slate-300 font-mono"
        >
            <span>GRÖSSE</span>
            <span>{openSections.size ? '▼' : '▶'}</span>
        </button>
        {#if openSections.size}
            <div class="pl-1.5">
                <SizeFilter />
            </div>
        {/if}
    </div>
</div>
