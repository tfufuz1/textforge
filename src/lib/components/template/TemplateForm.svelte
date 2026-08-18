<script lang="ts">
    import { onMount } from 'svelte';
    import { parseTemplate, renderTemplate, writeToClipboard, type ParsedTemplateDto } from '../../ipc/snippets';
    import { pushNotification, Notifications } from '../../stores/notifications';
    import TemplatePreview from './TemplatePreview.svelte';

    let { content = '', snippetId = null as string | null, renderedOutput = $bindable('') } = $props();

    let parsed = $state<ParsedTemplateDto | null>(null);
    let values = $state<Record<string, string>>({});
    let unresolvedVars = $state<string[]>([]);
    let warnings = $state<string[]>([]);

    $effect(() => {
        if (content) {
            analyzeTemplate();
        }
    });

    async function analyzeTemplate() {
        try {
            parsed = await parseTemplate(content);
            // Initialize default values
            const newValues: Record<string, string> = {};
            parsed.variables.forEach(v => {
                newValues[v.name] = values[v.name] || v.defaultVal || '';
            });
            values = newValues;
            await render();
        } catch (e) {
            console.error("Failed to parse template:", e);
        }
    }

    async function render() {
        try {
            const res = await renderTemplate(content, values, false);
            renderedOutput = res.output;
            unresolvedVars = res.unresolvedVars;
            warnings = res.warnings;
        } catch (e) {
            console.error("Failed to render template:", e);
        }
    }

    function handleInputChange(name: string, val: string) {
        values[name] = val;
        render();
    }

    async function copyToClipboard() {
        await writeToClipboard(renderedOutput, snippetId);
        pushNotification(Notifications.snippetCopied());
    }
</script>

<div class="space-y-4 bg-slate-900/60 p-4 rounded-xl border border-slate-800/80">
    <div class="flex justify-between items-center">
        <h3 class="text-sm font-semibold text-slate-300 font-mono">Template-Variablen ausfüllen</h3>
        {#if parsed}
            <div class="flex space-x-1">
                {#if parsed.hasConditionals}
                    <span class="px-1.5 py-0.5 text-[9px] font-semibold bg-blue-500/20 text-blue-300 border border-blue-500/30 rounded" title="Bedingte Blöcke vorhanden">
                        Conditional (if/unless)
                    </span>
                {/if}
                {#if parsed.hasLoops}
                    <span class="px-1.5 py-0.5 text-[9px] font-semibold bg-purple-500/20 text-purple-300 border border-purple-500/30 rounded" title="Schleifen vorhanden">
                        Loop (each)
                    </span>
                {/if}
            </div>
        {/if}
    </div>
    
    {#if parsed && parsed.variables.filter(v => !v.isSpecial).length > 0}
        <div class="grid grid-cols-2 gap-3 max-h-48 overflow-y-auto pr-1">
            {#each parsed.variables as variable}
                {#if !variable.isSpecial}
                    <div>
                        <label for="var-{variable.name}" class="flex items-center justify-between text-[11px] font-semibold text-slate-400 mb-1 truncate">
                            <span class="truncate">
                                {variable.name}
                                {#if variable.isRequired}
                                    <span class="text-rose-400" title="Pflichtfeld">*</span>
                                {/if}
                            </span>
                            {#if variable.filter}
                                <span class="ml-1 px-1 py-0.2 text-[8px] bg-slate-800 text-slate-300 rounded font-mono border border-slate-700">
                                    {variable.filter}
                                </span>
                            {/if}
                        </label>
                        <input
                            id="var-{variable.name}"
                            type="text"
                            value={values[variable.name] || ''}
                            oninput={(e) => handleInputChange(variable.name, (e.target as HTMLInputElement).value)}
                            placeholder={variable.defaultVal ? `Standard: ${variable.defaultVal}` : 'Wert eingeben...'}
                            class="w-full px-2.5 py-1.5 text-xs bg-slate-950/80 border border-slate-850 rounded-lg text-slate-100 focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 outline-none transition-all"
                        />
                    </div>
                {/if}
            {/each}
        </div>
    {:else if parsed && parsed.variables.length > 0}
        <p class="text-xs text-slate-400 italic">Es werden nur Spezial-Variablen verwendet (werden automatisch gefüllt).</p>
    {:else}
        <p class="text-xs text-slate-500 italic">Keine Variablen in diesem Template gefunden.</p>
    {/if}

    {#if parsed && parsed.variables.some(v => v.isSpecial)}
        <div class="pt-2 border-t border-slate-850/60 text-[10px] text-slate-400 space-y-1">
            <span class="font-semibold text-slate-500">Auto-Variablen:</span>
            <div class="flex flex-wrap gap-1">
                {#each parsed.variables.filter(v => v.isSpecial) as specialVar}
                    <span class="px-1.5 py-0.5 bg-slate-950/80 rounded border border-slate-850 font-mono text-[9px] text-indigo-400" title="Automatisch befüllt">
                        {specialVar.name}{specialVar.filter ? `|${specialVar.filter}` : ''}
                    </span>
                {/each}
            </div>
        </div>
    {/if}

    <TemplatePreview 
        {renderedOutput} 
        {unresolvedVars} 
        {warnings} 
        onCopy={copyToClipboard} 
    />
</div>
