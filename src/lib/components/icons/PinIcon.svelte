<script lang="ts">
	interface Props {
		class?: string;
		size?: number;
		filled?: boolean;
	}
	let { class: className = 'w-4 h-4', size, filled = false }: Props = $props();
</script>

<svg xmlns="http://www.w3.org/2000/svg" width={size} height={size} viewBox="0 0 24 24" fill={filled ? "currentColor" : "none"} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class={className}>
	<line x1="12" y1="17" x2="12" y2="22" />
	<path d="M5 17h14l-1.7-7A3 3 0 0 0 14.36 7.8L12 2 9.64 7.8A3 3 0 0 0 6.7 10L5 17z" />
</svg>
