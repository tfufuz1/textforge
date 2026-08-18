<script lang="ts">
    import { onMount } from 'svelte';
    import { Marked } from 'marked';
    import DOMPurify from 'dompurify';
    import hljs from 'highlight.js';
    import 'highlight.js/styles/github-dark.css'; // Premium dark theme for code blocks

    interface Props {
        content: string;
        sanitize?: boolean;
        highlightCode?: boolean;
        tableOfContents?: boolean;
        lineNumbers?: boolean;
        copyButton?: boolean;
        linkTarget?: '_blank' | '_self';
        previewMode?: 'editor' | 'preview' | 'split';
    }

    let {
        content = '',
        sanitize = true,
        highlightCode = true,
        tableOfContents = true,
        lineNumbers = true,
        copyButton = true,
        linkTarget = '_blank',
        previewMode = 'split'
    }: Props = $props();

    let htmlContent = $state('');
    let toc = $state<{ id: string; text: string; level: number }[]>([]);
    let containerElement = $state<HTMLDivElement | null>(null);

    // Initialisiere marked mit optionalen Highlighting-Optionen
    const marked = new Marked();

    $effect(() => {
        renderMarkdown(content);
    });

    async function renderMarkdown(raw: string) {
        if (!raw) {
            htmlContent = '';
            toc = [];
            return;
        }

        try {
            // TOC extrahieren und IDs zu den Headings hinzufügen
            const headings: { id: string; text: string; level: number }[] = [];
            
            // Custom Renderer für Heading-IDs und TOC-Generierung
            const renderer = new marked.Renderer();
            renderer.heading = ({ text, depth }) => {
                const escapedText = text.toLowerCase().replace(/[^\w]+/g, '-');
                const uniqueId = `heading-${escapedText}-${Math.random().toString(36).substr(2, 5)}`;
                
                if (tableOfContents && depth <= 3) {
                    headings.push({
                        id: uniqueId,
                        text: text.replace(/<[^>]*>/g, ''), // Strip tags for TOC text
                        level: depth
                    });
                }
                
                return `<h${depth} id="${uniqueId}" class="md-heading text-slate-100 font-bold mt-6 mb-3 border-b border-slate-800/40 pb-1">${text}</h${depth}>`;
            };

            // Custom Renderer für Links (Target-Handling)
            renderer.link = ({ href, title, text }) => {
                const titleAttr = title ? `title="${title}"` : '';
                return `<a href="${href}" target="${linkTarget}" rel="noopener noreferrer" ${titleAttr} class="text-indigo-400 hover:text-indigo-300 underline transition-all">${text}</a>`;
            };

            // Custom Code Block Renderer für Line Numbers & Copy Button Container
            renderer.code = ({ text, lang }) => {
                const language = lang || 'plaintext';
                let codeHtml = text;
                
                if (highlightCode) {
                    try {
                        if (hljs.getLanguage(language)) {
                            codeHtml = hljs.highlight(text, { language }).value;
                        } else {
                            codeHtml = hljs.highlightAuto(text).value;
                        }
                    } catch (e) {
                        console.error('Highlighting failed', e);
                    }
                }

                // Zeilennummern hinzufügen
                if (lineNumbers) {
                    const lines = codeHtml.split('\n');
                    const numberedLines = lines
                        .map((line, idx) => `<span class="line-number text-slate-600 select-none pr-3 text-right inline-block w-6">${idx + 1}</span><span class="line-content">${line}</span>`)
                        .join('\n');
                    codeHtml = numberedLines;
                }

                const uniqueId = `code-${Math.random().toString(36).substr(2, 9)}`;
                
                return `
                    <div class="code-block-wrapper relative my-4 rounded-xl border border-slate-800 bg-slate-950/90 overflow-hidden font-mono text-xs">
                        <div class="code-block-header flex items-center justify-between px-4 py-1.5 bg-slate-900 border-b border-slate-850 text-slate-400 text-[10px]">
                            <span class="font-semibold uppercase tracking-wider">${language}</span>
                            ${copyButton ? `<button data-code-id="${uniqueId}" class="copy-code-btn px-2 py-0.5 hover:bg-slate-800 hover:text-slate-200 rounded transition-all flex items-center gap-1 font-sans">📋 Kopieren</button>` : ''}
                        </div>
                        <pre id="${uniqueId}" class="p-3 overflow-x-auto leading-relaxed text-slate-300"><code>${codeHtml}</code></pre>
                    </div>
                `;
            };

            marked.use({ renderer });
            
            let rawHtml = await marked.parse(raw);

            // Sanitization gegen XSS
            if (sanitize) {
                rawHtml = DOMPurify.sanitize(rawHtml, {
                    ADD_ATTR: ['target', 'rel', 'data-code-id'],
                });
            }

            htmlContent = rawHtml;
            toc = headings;

            // Binde Event-Listener für Copy-Buttons nach dem Rendering
            setTimeout(bindCopyButtons, 50);
        } catch (err) {
            console.error('Error rendering markdown:', err);
            htmlContent = `<div class="text-rose-400 p-4 border border-rose-950 bg-rose-950/20 rounded-lg">Fehler beim Rendern des Markdowns: ${err}</div>`;
        }
    }

    function bindCopyButtons() {
        if (!containerElement) return;
        const buttons = containerElement.querySelectorAll('.copy-code-btn');
        buttons.forEach(btn => {
            btn.addEventListener('click', async (e) => {
                const target = e.currentTarget as HTMLButtonElement;
                const codeId = target.getAttribute('data-code-id');
                if (!codeId) return;

                const preElement = containerElement?.querySelector(`#${codeId}`);
                if (!preElement) return;

                // Extrahiere Text unter Ausschluss der Zeilennummern
                let textToCopy = '';
                const lineContents = preElement.querySelectorAll('.line-content');
                if (lineContents.length > 0) {
                    textToCopy = Array.from(lineContents).map(el => el.textContent).join('\n');
                } else {
                    textToCopy = preElement.textContent || '';
                }

                try {
                    await navigator.clipboard.writeText(textToCopy);
                    const originalText = target.innerText;
                    target.innerText = '✅ Kopiert!';
                    target.classList.add('text-emerald-400', 'bg-emerald-950/30');
                    setTimeout(() => {
                        target.innerText = originalText;
                        target.classList.remove('text-emerald-400', 'bg-emerald-950/30');
                    }, 2000);
                } catch (err) {
                    console.error('Failed to copy code block:', err);
                }
            });
        });
    }

    function scrollToHeading(id: string) {
        const el = document.getElementById(id);
        if (el) {
            el.scrollIntoView({ behavior: 'smooth', block: 'start' });
        }
    }
</script>

<div class="flex flex-col h-full bg-slate-950 border border-slate-900 rounded-xl overflow-hidden shadow-2xl">
    <!-- Header / Control bar -->
    <div class="flex items-center justify-between px-4 py-2 bg-slate-900 border-b border-slate-850">
        <div class="flex items-center space-x-2">
            <span class="text-xs font-semibold text-slate-400 font-mono">Vorschau Modus:</span>
            <div class="flex p-0.5 bg-slate-950 rounded-lg border border-slate-850">
                <button
                    onclick={() => previewMode = 'editor'}
                    class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all {previewMode === 'editor' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
                >
                    📝 Code
                </button>
                <button
                    onclick={() => previewMode = 'split'}
                    class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all {previewMode === 'split' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
                >
                    🪟 Split
                </button>
                <button
                    onclick={() => previewMode = 'preview'}
                    class="px-2.5 py-1 text-[10px] font-semibold rounded-md transition-all {previewMode === 'preview' ? 'bg-indigo-600 text-white shadow' : 'text-slate-400 hover:text-slate-200'}"
                >
                    👁️ Vorschau
                </button>
            </div>
        </div>
        {#if tableOfContents && toc.length > 0 && previewMode !== 'editor'}
            <div class="text-[10px] text-slate-400 font-mono">
                {toc.length} Überschriften
            </div>
        {/if}
    </div>

    <!-- Main Workspace -->
    <div class="flex flex-1 overflow-hidden min-h-0">
        <!-- Editor Mode View -->
        {#if previewMode === 'editor' || previewMode === 'split'}
            <div class="flex-1 overflow-auto p-4 border-r border-slate-900 bg-slate-950/60 font-mono text-xs text-slate-300 leading-relaxed whitespace-pre-wrap select-text">
                {content || 'Kein Inhalt...'}
            </div>
        {/if}

        <!-- Rendered HTML Preview View -->
        {#if previewMode === 'preview' || previewMode === 'split'}
            <div 
                bind:this={containerElement} 
                class="flex-1 overflow-auto p-5 prose prose-invert prose-slate max-w-none bg-slate-900/10 text-slate-300 leading-relaxed text-sm select-text"
            >
                {#if htmlContent}
                    {@html htmlContent}
                {:else}
                    <p class="text-slate-500 italic text-xs font-mono text-center my-8">Kein gerendertes Ergebnis vorhanden.</p>
                {/if}
            </div>

            <!-- Table of Contents Sidebar -->
            {#if tableOfContents && toc.length > 0}
                <div class="w-48 bg-slate-900/40 border-l border-slate-900 p-3 overflow-y-auto hidden md:block">
                    <h4 class="text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-2.5 font-mono">Inhalt</h4>
                    <nav class="space-y-1.5">
                        {#each toc as item}
                            <button
                                onclick={() => scrollToHeading(item.id)}
                                class="w-full text-left truncate text-[10px] text-slate-400 hover:text-indigo-400 transition-all block font-mono"
                                style="padding-left: {(item.level - 1) * 8}px"
                            >
                                • {item.text}
                            </button>
                        {/each}
                    </nav>
                </div>
            {/if}
        {/if}
    </div>
</div>

<style>
    /* Styling für Custom Markdown-Tags */
    :global(.prose h1, .prose h2, .prose h3) {
        font-family: ui-sans-serif, system-ui, sans-serif;
    }
    :global(.prose h1) {
        font-size: 1.5rem;
        border-bottom: 2px solid var(--color-slate-800);
        padding-bottom: 0.25rem;
    }
    :global(.prose h2) {
        font-size: 1.25rem;
    }
    :global(.prose h3) {
        font-size: 1.1rem;
    }
    :global(.prose p) {
        margin-top: 0.75rem;
        margin-bottom: 0.75rem;
    }
    :global(.prose ul) {
        list-style-type: disc;
        padding-left: 1.25rem;
        margin-top: 0.5rem;
        margin-bottom: 0.5rem;
    }
    :global(.prose ol) {
        list-style-type: decimal;
        padding-left: 1.25rem;
        margin-top: 0.5rem;
        margin-bottom: 0.5rem;
    }
    :global(.prose li) {
        margin-top: 0.25rem;
        margin-bottom: 0.25rem;
    }
    :global(.prose blockquote) {
        border-left: 4px solid var(--color-indigo-500);
        background-color: rgba(99, 102, 241, 0.05);
        padding: 0.5rem 1rem;
        margin: 1rem 0;
        border-radius: 0 0.5rem 0.5rem 0;
        font-style: italic;
        color: var(--color-slate-400);
    }
</style>
