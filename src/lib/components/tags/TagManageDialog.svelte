<script lang="ts">
	import { tagsActions } from '$lib/stores/tags';

	interface Props {
		tagName: string;
		onClose: () => void;
	}

	let { tagName, onClose }: Props = $props();

	let newName = $state('');
	let color = $state('#10b981');

	$effect(() => {
		newName = tagName;
	});

	async function handleRename() {
		if (!newName.trim() || newName === tagName) return;
		try {
			await tagsActions.rename(tagName, newName.trim());
			onClose();
		} catch (e) {
			console.error(e);
		}
	}

	async function handleSetColor() {
		try {
			await tagsActions.setColor(tagName, color);
			onClose();
		} catch (e) {
			console.error(e);
		}
	}
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
	<div class="bg-slate-900 border border-slate-800 rounded-xl p-5 w-full max-w-sm shadow-2xl space-y-4">
		<div class="flex items-center justify-between border-b border-slate-800 pb-3">
			<h3 class="text-sm font-semibold text-slate-200">Tag bearbeiten: #{tagName}</h3>
			<button type="button" onclick={onClose} class="text-slate-400 hover:text-white">✕</button>
		</div>

		<div class="space-y-3 text-xs">
			<div>
				<label for="tag-new-name" class="block text-slate-400 mb-1">Tag umbenennen</label>
				<div class="flex gap-2">
					<input
						id="tag-new-name"
						type="text"
						bind:value={newName}
						class="flex-1 bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-white outline-none focus:border-emerald-500"
					/>
					<button
						type="button"
						onclick={handleRename}
						class="px-3 py-1.5 bg-emerald-600 hover:bg-emerald-500 text-white rounded-lg font-medium transition"
					>
						Speichern
					</button>
				</div>
			</div>

			<div>
				<label for="tag-color-picker" class="block text-slate-400 mb-1">Tag-Farbe zuweisen</label>
				<div class="flex gap-2 items-center">
					<input
						id="tag-color-picker"
						type="color"
						bind:value={color}
						class="bg-slate-950 border border-slate-800 rounded h-8 w-12 p-1 cursor-pointer"
					/>
					<button
						type="button"
						onclick={handleSetColor}
						class="px-3 py-1.5 bg-slate-800 hover:bg-slate-700 text-slate-200 rounded-lg font-medium transition"
					>
						Farbe anwenden
					</button>
				</div>
			</div>
		</div>
	</div>
</div>
