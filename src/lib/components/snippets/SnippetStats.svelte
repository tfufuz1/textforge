<script lang="ts">
    import { computeTextStats, type TextStatsDto } from '../../ipc/snippets';

    let { content = '' } = $props();

    let stats = $state<TextStatsDto | null>(null);

    $effect(() => {
        if (content) {
            updateStats();
        } else {
            stats = null;
        }
    });

    async function updateStats() {
        try {
            stats = await computeTextStats(content);
        } catch (e) {
            console.error("Failed to compute stats:", e);
        }
    }

    function formatReadingTime(ms: number) {
        const sec = Math.round(ms / 1000);
        if (sec < 1) return '< 1 Sek';
        if (sec < 60) return `${sec} Sek`;
        return `${Math.round(sec / 60)} Min`;
    }
</script>

{#if stats}
    <div class="bg-slate-950/60 p-4 rounded-xl border border-slate-800/80 grid grid-cols-2 sm:grid-cols-4 gap-4 text-xs font-mono">
        <div class="flex flex-col">
            <span class="text-slate-500 text-[10px] uppercase">Zeilen / Absätze</span>
            <span class="text-indigo-400 font-bold text-sm mt-0.5">{stats.lineCount} / {stats.paragraphCount}</span>
        </div>
        <div class="flex flex-col">
            <span class="text-slate-500 text-[10px] uppercase">Wörter / Sätze</span>
            <span class="text-indigo-400 font-bold text-sm mt-0.5">{stats.wordCount} / {stats.sentenceCount}</span>
        </div>
        <div class="flex flex-col">
            <span class="text-slate-500 text-[10px] uppercase">Zeichen</span>
            <span class="text-indigo-400 font-bold text-sm mt-0.5">{stats.charCount}</span>
        </div>
        <div class="flex flex-col">
            <span class="text-slate-500 text-[10px] uppercase">Est. Tokens / Lesezeit</span>
            <span class="text-emerald-400 font-bold text-sm mt-0.5">{stats.estimatedTokens} / {formatReadingTime(stats.readingTimeMs)}</span>
        </div>
    </div>
{/if}
