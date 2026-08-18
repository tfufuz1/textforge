<script lang="ts">
    import MarkdownPreview from '../shared/MarkdownPreview.svelte';
    let { content = '' } = $props();
</script>

<div class="h-full flex-1 min-h-0">
    <MarkdownPreview {content} previewMode="preview" tableOfContents={true} />
</div>
