<script lang="ts">
	/**
	 * Notes as a nav section: a "Notes" row styled like the top-level nav
	 * items (click = new note, Ctrl+\), and each note beneath it styled
	 * like a nav child. Same tokens, weights and padding as the layout's
	 * nav rules — this is the sidebar's selector look, not a separate
	 * widget. Click a note to edit; × asks once before deleting; the copy
	 * icon exports everything as Markdown. Stored in the bundle's database
	 * (lib/notes.svelte.ts writes through); the row turns alert-colored when
	 * a write failed.
	 */
	import {
		notes,
		openQuickNote,
		deleteNote,
		noteTitle,
		notesAsMarkdown,
	} from "$lib/notes.svelte";
	import { formatTimestamp } from "$lib/format";
	import { copyText } from "$lib/clipboard";
	import Icon from "$lib/components/Icon.svelte";

	const sorted = $derived(notes.list.toSorted((a, b) => b.updatedAt - a.updatedAt));

	/** id awaiting a second click to delete */
	let confirming = $state<number | null>(null);
	let copied = $state(false);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		month: "short",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
		hourCycle: "h23",
	});

	function remove(id: number) {
		if (confirming !== id) {
			confirming = id;
			setTimeout(() => (confirming = confirming === id ? null : confirming), 2500);
			return;
		}
		deleteNote(id).catch((e) => (notes.error = String(e)));
		confirming = null;
	}

	async function copyAll() {
		const md = notesAsMarkdown((ms) => formatTimestamp(timeFormat, ms));
		if (await copyText(md)) {
			copied = true;
			setTimeout(() => (copied = false), 1200);
		}
	}
</script>

<div class="section">
	<div class="head" title={notes.error ?? undefined}>
		<button class="row parent" class:error={notes.error !== null} onclick={() => openQuickNote(null)} title="New note (Ctrl+\)">
			<span class="icon"><Icon name="note" /></span>
			Notes
			{#if notes.list.length > 0}
				<span class="count mono">{notes.list.length}</span>
			{/if}
		</button>
		{#if notes.list.length > 0}
			<button class="tool" onclick={copyAll} title="Copy all notes as Markdown" aria-label="Copy all notes"
				><Icon name={copied ? "check" : "copy"} size={12} /></button
			>
		{/if}
	</div>
	<div class="list">
		{#each sorted as n (n.createdAt)}
			<div class="entry" class:confirming={confirming === n.createdAt}>
				<button
					class="row child"
					onclick={() => openQuickNote(n.createdAt)}
					title="{formatTimestamp(timeFormat, n.updatedAt)} · {n.route}&#10;&#10;{n.text}"
				>
					<span class="title">{noteTitle(n)}</span>
				</button>
				<button
					class="del"
					onclick={() => remove(n.createdAt)}
					title={confirming === n.createdAt ? "Click again to delete" : "Delete note"}
					aria-label="Delete note">{confirming === n.createdAt ? "sure?" : "✕"}</button
				>
			</div>
		{/each}
	</div>
</div>

<style>
	.section {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 0;
	}
	.head,
	.entry {
		display: flex;
		align-items: stretch;
		border-radius: var(--radius);
	}
	.head:hover,
	.entry:hover {
		background: var(--bg-hover);
	}
	.entry.confirming {
		background: color-mix(in srgb, var(--alert) 12%, transparent);
	}

	/* mirrors `nav a` / `nav a.child` in the layout — same face, weights,
	   padding and colors, so the section reads as part of the selector */
	.row {
		flex: 1;
		min-width: 0;
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 10px;
		border-radius: var(--radius);
		color: var(--fg-muted);
		text-align: left;
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.07em;
	}
	.row:hover {
		color: var(--fg);
	}
	/* the last database write failed: the notes are only in memory */
	.row.error {
		color: var(--alert);
	}
	.row.child {
		margin-left: 18px;
		padding: 5px 10px;
		font-weight: 300;
		/* note titles are the user's own words: keep their case */
		text-transform: none;
		letter-spacing: 0;
	}
	.icon {
		display: grid;
		place-items: center;
		color: var(--accent);
	}
	.count {
		margin-left: auto;
		font-size: 10.5px;
		font-weight: 400;
		letter-spacing: 0;
	}
	.title {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.mono {
		font-family: var(--font-mono);
	}

	.tool,
	.del {
		padding: 0 8px;
		color: var(--fg-muted);
		border-radius: var(--radius);
		font-size: 10.5px;
		opacity: 0;
	}
	.head:hover .tool,
	.entry:hover .del,
	.entry.confirming .del {
		opacity: 1;
	}
	.tool:hover {
		color: var(--fg);
	}
	.del:hover {
		color: var(--alert);
	}

	.list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 0;
		max-height: 32vh;
		overflow: auto;
	}
</style>
