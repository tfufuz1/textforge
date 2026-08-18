<script lang="ts">
    import type { ScriptParameter } from '../../domain/script';

    let {
        parameters = [] as ScriptParameter[],
        values = $bindable<Record<string, string | number | boolean>>({}),
        onChange = (_vals: Record<string, string | number | boolean>) => {}
    } = $props();

    function handleChange(key: string, val: string | number | boolean) {
        values = { ...values, [key]: val };
        onChange(values);
    }
</script>

{#if parameters.length > 0}
    <div class="space-y-3">
        <h4 class="text-xs font-bold font-mono tracking-widest text-indigo-400 uppercase">Parameter</h4>
        <div class="space-y-3">
            {#each parameters as param}
                <div>
                    <label
                        for="param-{param.key}"
                        class="block text-xs font-semibold text-slate-400 mb-1.5"
                    >
                        {param.label || param.key}
                        {#if param._type === 'text' && param.required}
                            <span class="text-rose-400 ml-0.5" title="Pflichtfeld">*</span>
                        {/if}
                    </label>

                    {#if param._type === 'boolean'}
                        <label class="flex items-center space-x-2 cursor-pointer group w-fit">
                            <div class="relative">
                                <input
                                    id="param-{param.key}"
                                    type="checkbox"
                                    checked={values[param.key] === true || (values[param.key] === undefined && param.default)}
                                    onchange={(e) => handleChange(param.key, (e.target as HTMLInputElement).checked)}
                                    class="sr-only peer"
                                />
                                <div class="w-9 h-5 bg-slate-800 border border-slate-700 rounded-full peer-checked:bg-indigo-600 peer-checked:border-indigo-500 transition-all"></div>
                                <div class="absolute top-0.5 left-0.5 w-4 h-4 bg-slate-400 rounded-full transition-all peer-checked:translate-x-4 peer-checked:bg-white"></div>
                            </div>
                            <span class="text-xs text-slate-300 group-hover:text-slate-100 transition-colors">
                                {(values[param.key] ?? param.default) ? 'Aktiviert' : 'Deaktiviert'}
                            </span>
                        </label>
                        {#if param.description._tag === 'Some'}
                            <p class="mt-1 text-[10px] text-slate-500">{param.description.value}</p>
                        {/if}

                    {:else if param._type === 'select'}
                        <select
                            id="param-{param.key}"
                            value={values[param.key] ?? param.default ?? ''}
                            onchange={(e) => handleChange(param.key, (e.target as HTMLSelectElement).value)}
                            class="w-full px-3 py-2 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all"
                        >
                            {#each param.options as opt}
                                <option value={opt.value}>{opt.label}</option>
                            {/each}
                        </select>

                    {:else if param._type === 'number'}
                        <div class="relative">
                            <input
                                id="param-{param.key}"
                                type="number"
                                value={values[param.key] ?? param.default ?? ''}
                                oninput={(e) => handleChange(param.key, parseFloat((e.target as HTMLInputElement).value))}
                                min={param.min._tag === 'Some' ? param.min.value : undefined}
                                max={param.max._tag === 'Some' ? param.max.value : undefined}
                                step={param.step}
                                placeholder={`Standard: ${param.default}`}
                                class="w-full px-3 py-2 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all font-mono {param.unit._tag === 'Some' ? 'pr-8' : ''}"
                            />
                            {#if param.unit._tag === 'Some'}
                                <span class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-slate-500 pointer-events-none">
                                    {param.unit.value}
                                </span>
                            {/if}
                        </div>

                    {:else if param._type === 'textarea'}
                        <textarea
                            id="param-{param.key}"
                            value={values[param.key] ?? param.default ?? ''}
                            oninput={(e) => handleChange(param.key, (e.target as HTMLTextAreaElement).value)}
                            placeholder={param.placeholder || `Standard: ${param.default}`}
                            rows={param.rows}
                            class="w-full px-3 py-2 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all resize-y"
                        ></textarea>

                    {:else if param._type === 'regex'}
                        <input
                            id="param-{param.key}"
                            type="text"
                            value={values[param.key] ?? param.default ?? ''}
                            oninput={(e) => handleChange(param.key, (e.target as HTMLInputElement).value)}
                            placeholder={`Standard: ${param.default}`}
                            class="w-full px-3 py-2 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-indigo-300 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all font-mono"
                        />

                    {:else}
                        <!-- text -->
                        <input
                            id="param-{param.key}"
                            type="text"
                            value={values[param.key] ?? param.default ?? ''}
                            oninput={(e) => handleChange(param.key, (e.target as HTMLInputElement).value)}
                            placeholder={param.placeholder || `Standard: ${param.default}`}
                            maxLength={param.maxLength._tag === 'Some' ? param.maxLength.value : undefined}
                            class="w-full px-3 py-2 text-xs bg-slate-950/80 border border-slate-800 rounded-xl text-slate-100 outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all"
                        />
                    {/if}
                </div>
            {/each}
        </div>
    </div>
{/if}
