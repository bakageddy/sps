<script lang="ts">
	/**
	 * Ctrl/⌘+K palette: type to filter, ↑↓ to move, Enter to run, Esc to
	 * close. Items are whatever the layout hands in — pages and a few
	 * app-level actions. Matching is per word: every typed word must occur
	 * somewhere in the label (order-free, case-insensitive).
	 */
	export interface Command {
		label: string;
		/** small grey text after the label (a path, a current value) */
		hint?: string;
		run: () => void;
	}

	interface Props {
		open: boolean;
		items: Command[];
		/** commands built FROM the query itself (e.g. "jump to 14:32"),
		 *  appended after the filtered items */
		dynamic?: (query: string) => Command[];
		onclose: () => void;
	}

	let { open, items, dynamic, onclose }: Props = $props();

	let query = $state("");
	let cursor = $state(0);
	let input = $state<HTMLInputElement>();

	const matches = $derived.by(() => {
		const words = query.toLowerCase().split(/\s+/).filter(Boolean);
		const extra = dynamic?.(query) ?? [];
		if (words.length === 0) return [...items, ...extra];
		const filtered = items.filter((it) => {
			const hay = `${it.label} ${it.hint ?? ""}`.toLowerCase();
			return words.every((w) => hay.includes(w));
		});
		// a typed time is more likely the intent than a page whose name
		// happens to contain those digits — dynamic first
		return [...extra, ...filtered];
	});

	$effect(() => {
		if (open) {
			query = "";
			cursor = 0;
			requestAnimationFrame(() => input?.focus());
		}
	});
	$effect(() => {
		void matches;
		cursor = 0;
	});

	function run(cmd: Command) {
		onclose();
		cmd.run();
	}

	function onkeydown(e: KeyboardEvent) {
		switch (e.key) {
			case "Escape":
				e.preventDefault();
				onclose();
				break;
			case "ArrowDown":
				e.preventDefault();
				cursor = Math.min(matches.length - 1, cursor + 1);
				break;
			case "ArrowUp":
				e.preventDefault();
				cursor = Math.max(0, cursor - 1);
				break;
			case "Enter":
				e.preventDefault();
				if (matches[cursor]) run(matches[cursor]);
				break;
		}
	}
</script>

{#if open}
	<!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
	<div class="backdrop" onclick={onclose}>
		<div
			class="palette"
			role="dialog"
			aria-label="Command palette"
			tabindex="-1"
			onclick={(e) => e.stopPropagation()}
		>
			<input
				bind:this={input}
				bind:value={query}
				{onkeydown}
				placeholder="Page, command, or a time (14:32 · 2026-09-19 14:32:05)…"
				spellcheck="false"
				aria-label="Command"
			/>
			<ul role="listbox">
				{#each matches as cmd, i (cmd.label)}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<li
						role="option"
						aria-selected={i === cursor}
						class:active={i === cursor}
						onmouseenter={() => (cursor = i)}
						onclick={() => run(cmd)}
					>
						<span class="label">{cmd.label}</span>
						{#if cmd.hint}<span class="hint mono">{cmd.hint}</span>{/if}
					</li>
				{:else}
					<li class="empty">No match.</li>
				{/each}
			</ul>
		</div>
	</div>
{/if}

<style>
	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 50;
		background: color-mix(in srgb, var(--bg-hard) 55%, transparent);
		display: flex;
		justify-content: center;
		align-items: flex-start;
		padding-top: 12vh;
	}
	.palette {
		width: min(560px, 90vw);
		background: var(--bg-soft);
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		box-shadow: 0 12px 40px color-mix(in srgb, var(--bg-hard) 70%, transparent);
		overflow: hidden;
	}
	input {
		width: 100%;
		box-sizing: border-box;
		padding: 10px 14px;
		background: var(--bg-hard);
		border: none;
		outline: none;
		color: var(--fg);
		font-family: var(--font-ui);
		font-size: 13px;
	}
	ul {
		list-style: none;
		margin: 0;
		padding: 4px 0;
		max-height: 50vh;
		overflow: auto;
	}
	li {
		display: flex;
		align-items: baseline;
		gap: 12px;
		padding: 6px 14px;
		font-size: 12.5px;
		cursor: pointer;
	}
	li.active {
		background: color-mix(in srgb, var(--accent) 14%, transparent);
	}
	.label {
		color: var(--fg);
	}
	.hint {
		margin-left: auto;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.mono {
		font-family: var(--font-mono);
	}
	.empty {
		color: var(--fg-muted);
		cursor: default;
	}
</style>
