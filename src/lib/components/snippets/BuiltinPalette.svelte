<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';

    let { content = $bindable(''), onApply = () => {} } = $props();

    let search = $state('');
    let selectedCategory = $state<string>('all');
    let lastResult = $state('');
    let hasResult = $state(false);

    const categories = [
        { id: 'all', label: 'Alle' },
        { id: 'text', label: 'Text' },
        { id: 'case', label: 'Groß/Klein' },
        { id: 'lines', label: 'Zeilen' },
        { id: 'code', label: 'Code & Data' },
        { id: 'encode', label: 'Kodierung' },
        { id: 'extract', label: 'Extraktion' },
        { id: 'security', label: 'Sicherheit' },
    ];

    const builtins = [
        { id: 'trim', label: 'Trim Whitespace', category: 'text', description: 'Führende & folgende Leerzeichen entfernen' },
        { id: 'remove_empty_lines', label: 'Leerzeilen löschen', category: 'lines', description: 'Entfernt leere Zeilen' },
        { id: 'collapse_whitespace', label: 'Whitespace kollabieren', category: 'text', description: 'Mehrfache Leerzeichen durch einzelnes ersetzen' },
        { id: 'uppercase', label: 'UPPERCASE', category: 'case', description: 'In Großbuchstaben umwandeln' },
        { id: 'lowercase', label: 'lowercase', category: 'case', description: 'In Kleinbuchstaben umwandeln' },
        { id: 'title_case', label: 'Title Case', category: 'case', description: 'Jedes Wort großschreiben' },
        { id: 'sentence_case', label: 'Sentence case', category: 'case', description: 'Erstes Wort pro Satz groß' },
        { id: 'alternating_case', label: 'aLtErNaTiNg CaSe', category: 'case', description: 'Wechselnde Groß/Kleinschreibung' },
        { id: 'rot13', label: 'ROT13', category: 'case', description: 'ROT13 Chiffre anwenden' },
        { id: 'sort_lines', label: 'Zeilen sortieren (A-Z)', category: 'lines', description: 'Alphabetisch aufsteigend sortieren' },
        { id: 'sort_lines_desc', label: 'Zeilen sortieren (Z-A)', category: 'lines', description: 'Alphabetisch absteigend sortieren' },
        { id: 'sort_lines_by_length', label: 'Zeilen nach Länge sortieren', category: 'lines', description: 'Kurze Zeilen zuerst' },
        { id: 'reverse_lines', label: 'Zeilen umkehren', category: 'lines', description: 'Reihenfolge der Zeilen umkehren' },
        { id: 'unique_lines', label: 'Doppelte Zeilen löschen', category: 'lines', description: 'Duplikate entfernen' },
        { id: 'shuffle_lines', label: 'Zeilen mischen', category: 'lines', description: 'Zufällige Zeilenreihenfolge' },
        { id: 'add_line_numbers', label: 'Zeilennummern hinzufügen', category: 'lines', description: '1: 2: 3: voranstellen' },
        { id: 'remove_line_numbers', label: 'Zeilennummern entfernen', category: 'lines', description: 'Führende Nummern entfernen' },
        { id: 'pretty_json', label: 'JSON Formatieren', category: 'code', description: 'Formatiert JSON lesbar' },
        { id: 'minify_json', label: 'JSON Minifizieren', category: 'code', description: 'Kompakt ohne Whitespace' },
        { id: 'strip_markdown', label: 'Markdown entfernen', category: 'code', description: 'Reiner Text ohne Auszeichnung' },
        { id: 'markdown_to_html', label: 'Markdown → HTML', category: 'code', description: 'In HTML Tags umwandeln' },
        { id: 'strip_html_tags', label: 'HTML Tags entfernen', category: 'code', description: 'Tags löschen, Text behalten' },
        { id: 'extract_code_blocks', label: 'Codeblocks extrahieren', category: 'extract', description: 'Alle ``` Code-Blöcke sammeln' },
        { id: 'extract_emails', label: 'E-Mails extrahieren', category: 'extract', description: 'Alle E-Mail-Adressen finden' },
        { id: 'extract_urls', label: 'URLs extrahieren', category: 'extract', description: 'Alle HTTP/HTTPS-Links finden' },
        { id: 'extract_numbers', label: 'Zahlen extrahieren', category: 'extract', description: 'Alle Zahlenwerte auflisten' },
        { id: 'base64_encode', label: 'Base64 Encoden', category: 'encode', description: 'In Base64 umwandeln' },
        { id: 'base64_decode', label: 'Base64 Decoden', category: 'encode', description: 'Aus Base64 dekodieren' },
        { id: 'url_encode', label: 'URL Encoden', category: 'encode', description: 'URL-Prozentkodierung' },
        { id: 'url_decode', label: 'URL Decoden', category: 'encode', description: 'URL-Kodierung auflösen' },
        { id: 'html_entity_encode', label: 'HTML Entities Encoden', category: 'encode', description: '& < > " \' umwandeln' },
        { id: 'hash_sha256', label: 'SHA-256 Hash', category: 'encode', description: 'SHA-256 Hex-Hash generieren' },
        { id: 'camel_to_snake', label: 'camelToSnake', category: 'code', description: 'camelCase → snake_case' },
        { id: 'snake_to_camel', label: 'snakeToCamel', category: 'code', description: 'snake_case → camelCase' },
        { id: 'to_slug', label: 'URL Slug', category: 'code', description: 'In clean-url-slug umwandeln' },
        { id: 'redact_sensitive', label: 'Sensitive Daten maskieren', category: 'security', description: 'IPs & API-Keys unkenntlich machen' },
        { id: 'strip_pii', label: 'PII entfernen', category: 'security', description: 'E-Mails & Telefonnummern löschen' },
    ];

    let filteredBuiltins = $derived(
        builtins.filter(b => {
            const matchesCat = selectedCategory === 'all' || b.category === selectedCategory;
            const matchesSearch = !search.trim() || 
                b.label.toLowerCase().includes(search.toLowerCase()) || 
                b.description.toLowerCase().includes(search.toLowerCase());
            return matchesCat && matchesSearch;
        })
    );

    async function applyBuiltin(id: string) {
        if ((id === 'redact_sensitive' || id === 'strip_pii') && !confirm("Achtung: Dies ist eine sicherheitskritische/destruktive Operation. Möchten Sie wirklich fortfahren?")) {
            return;
        }
        try {
            const res = await invoke<string>('execute_builtin', {
                id,
                input: content,
                params: {}
            });
            lastResult = res;
            hasResult = true;
        } catch (e) {
            console.error("Builtin failed:", e);
        }
    }

    function confirmApply() {
        content = lastResult;
        hasResult = false;
        onApply();
    }

    function discardResult() {
        lastResult = '';
        hasResult = false;
    }
</script>

<div class="space-y-3 bg-slate-900/60 p-4 rounded-xl border border-slate-800/80">
    <div class="flex items-center justify-between gap-2">
        <input 
            type="text" 
            placeholder="Builtin-Transformation suchen..." 
            bind:value={search}
            class="flex-1 px-3 py-1.5 text-xs bg-slate-950/80 border border-slate-800 rounded-lg text-slate-200 outline-none focus:border-indigo-500 transition-all"
        />
    </div>

    <!-- Category Pills -->
    <div class="flex items-center gap-1 overflow-x-auto pb-1 scrollbar-none">
        {#each categories as cat}
            <button
                onclick={() => selectedCategory = cat.id}
                class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all shrink-0 {selectedCategory === cat.id ? 'bg-indigo-600 text-white' : 'bg-slate-950/60 text-slate-400 hover:text-slate-200 border border-slate-800'}"
            >
                {cat.label}
            </button>
        {/each}
    </div>

    <!-- Builtins Grid -->
    <div class="grid grid-cols-2 gap-2 max-h-48 overflow-y-auto pr-1">
        {#each filteredBuiltins as b}
            <button
                onclick={() => applyBuiltin(b.id)}
                class="text-left p-2.5 rounded-lg bg-slate-950/70 hover:bg-slate-850 border border-slate-800/80 hover:border-indigo-500/50 transition-all group"
            >
                <div class="font-semibold text-xs text-slate-200 group-hover:text-indigo-300 transition-colors">{b.label}</div>
                <div class="text-[10px] text-slate-400 truncate mt-0.5">{b.description}</div>
            </button>
        {:else}
            <div class="col-span-2 text-center py-4 text-xs text-slate-500">Keine Transformationen gefunden.</div>
        {/each}
    </div>

    {#if hasResult}
        <div class="pt-3 border-t border-slate-800 space-y-2">
            <div class="text-xs font-semibold text-slate-400 font-mono">Ergebnis-Vorschau</div>
            <pre class="p-3 bg-slate-950 rounded-lg border border-slate-800 text-xs font-mono text-slate-200 max-h-32 overflow-y-auto whitespace-pre-wrap">{lastResult}</pre>
            <div class="flex justify-end space-x-2">
                <button 
                    onclick={discardResult}
                    class="px-3 py-1.5 text-xs bg-slate-800 text-slate-300 rounded-lg hover:bg-slate-700"
                >
                    Verwerfen
                </button>
                <button 
                    onclick={confirmApply}
                    class="px-3 py-1.5 text-xs bg-emerald-600 text-white font-semibold rounded-lg hover:bg-emerald-500"
                >
                    Übernehmen
                </button>
            </div>
        </div>
    {/if}
</div>
