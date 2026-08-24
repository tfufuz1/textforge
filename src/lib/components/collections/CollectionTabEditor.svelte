<script lang="ts">
	import { collectionsActions } from '$lib/stores/collections';

	interface Props {
		onClose: () => void;
	}

	let { onClose }: Props = $props();

	let name = $state('');
	let icon = $state('');
	let color = $state('#10b981');
	let kind = $state('manual');
	let isSubmitting = $state(false);

	async function handleSubmit() {
		if (!name.trim() || isSubmitting) return;
		isSubmitting = true;
		try {
			await collectionsActions.create({
				name: name.trim(),
				icon: icon.trim() || null,
				color: color || null,
				kind,
			});
			onClose();
		} catch (e) {
			console.error(e);
		} finally {
			isSubmitting = false;
		}
	}
</script>

<div class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
	<div class="bg-slate-900 border border-slate-800 rounded-xl p-5 w-full max-w-md shadow-2xl space-y-4">
		<div class="flex items-center justify-between border-b border-slate-800 pb-3">
			<h3 class="text-sm font-semibold text-slate-200">Neuen Reiter erstellen</h3>
			<button type="button" onclick={onClose} aria-label="Schließen" class="text-slate-400 hover:text-white">✕</button>
		</div>

		<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-3 text-xs">
			<div>
				<label for="tab-name" class="block text-slate-400 mb-1">Name</label>
				<input
					id="tab-name"
					type="text"
					bind:value={name}
					placeholder="z.B. Arbeit, Favoriten, Screenshots"
					required
					class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
				/>
			</div>

			<div class="grid grid-cols-2 gap-3">
				<div>
					<label for="tab-icon" class="block text-slate-400 mb-1">Icon (Emoji)</label>
					<input
						id="tab-icon"
						type="text"
						bind:value={icon}
						placeholder="📁"
						class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
					/>
				</div>
				<div>
					<label for="tab-color" class="block text-slate-400 mb-1">Farbe</label>
					<input
						id="tab-color"
						type="color"
						bind:value={color}
						class="w-full bg-slate-950 border border-slate-800 rounded-lg h-9 p-1 text-white cursor-pointer"
					/>
				</div>
			</div>

			<div>
				<label for="tab-kind" class="block text-slate-400 mb-1">Typ</label>
				<select
					id="tab-kind"
					bind:value={kind}
					class="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-2 text-white focus:outline-none focus:border-emerald-500"
				>
					<option value="manual">Manuelle Sammlung</option>
					<option value="smart">Smart Tab (Filter-basiert)</option>
					<option value="clipboard_capture">Clipboard-Erfassung</option>
				</select>
			</div>

			<div class="flex items-center justify-end gap-2 pt-3">
				<button
					type="button"
					onclick={onClose}
					class="px-3 py-1.5 rounded-lg text-slate-400 hover:text-white hover:bg-slate-800 transition"
				>
					Abbrechen
				</button>
				<button
					type="submit"
					disabled={!name.trim() || isSubmitting}
					class="px-4 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white font-medium transition disabled:opacity-50"
				>
					Erstellen
				</button>
			</div>
		</form>
	</div>
</div>
