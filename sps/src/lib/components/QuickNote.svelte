<script lang="ts">
	/**
	 * Floating note editor (Ctrl+\): a wide panel over the current page,
	 * like an editor's terminal drawer. Plain textarea — paste works as
	 * paste does; the "paste" button is for when the clipboard has the
	 * text and your hands are on the mouse. Every keystroke saves.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		noteEditor,
		closeQuickNote,
		getNote,
		upsertNote,
		deleteNote,
	} from "$lib/notes.svelte";
	import { formatTimestamp } from "$lib/format";

	let text = $state("");
	let route = $state("");
	let id = $state<number | null>(null);
	let textarea = $state<HTMLTextAreaElement>();
	let pasteNote = $state<string | null>(null);

	// (re)load whenever the editor opens or is pointed at another note
	$effect(() => {
		if (!noteEditor.open) return;
		const target = noteEditor.id;
		const existing = target === null ? undefined : getNote(target);
		id = existing?.createdAt ?? null;
		text = existing?.text ?? "";
		route = existing?.route ?? page.url.pathname + page.url.search;
		pasteNote = null;
		requestAnimationFrame(() => {
			textarea?.focus();
			textarea?.setSelectionRange(text.length, text.length);
		});
	});

	function save() {
		if (text.trim() === "") {
			// emptied out: an empty note is a deleted note
			if (id !== null) deleteNote(id);
			id = null;
			return;
		}
		id = upsertNote(id, text, route);
		noteEditor.id = id;
	}

	async function pasteFromClipboard() {
		try {
			const clip = await navigator.clipboard.readText();
			if (clip === "") return;
			const el = textarea;
			if (el) {
				const s = el.selectionStart;
				const e = el.selectionEnd;
				text = text.slice(0, s) + clip + text.slice(e);
				requestAnimationFrame(() => {
					el.focus();
					el.selectionStart = el.selectionEnd = s + clip.length;
				});
			} else text += clip;
			save();
		} catch {
			pasteNote = "clipboard read not permitted — Ctrl+V works";
		}
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === "Escape") {
			e.preventDefault();
			closeQuickNote();
		}
	}

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "short",
		hourCycle: "h23",
	});
	const current = $derived(id === null ? undefined : getNote(id));
</script>

{#if noteEditor.open}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div class="panel" role="dialog" aria-label="Quick note" tabindex="-1" {onkeydown}>
		<header>
			<span class="title">note</span>
			{#if current}
				<span class="muted mono">{formatTimestamp(timeFormat, current.createdAt)}</span>
			{:else}
				<span class="muted">new — saves as you type</span>
			{/if}
			<button
				class="route mono"
				onclick={() => {
					closeQuickNote();
					goto(route);
				}}
				title="Open the page this note was taken on">{route}</button
			>
			<span class="grow"></span>
			<button class="act" onclick={pasteFromClipboard} title="Insert the clipboard at the caret">paste</button>
			{#if pasteNote}<span class="muted">{pasteNote}</span>{/if}
			<span class="muted hint">Esc closes · Ctrl+\ toggles</span>
			<button class="act" onclick={closeQuickNote} aria-label="Close">✕</button>
		</header>
		<textarea
			bind:this={textarea}
			bind:value={text}
			oninput={save}
			placeholder="What did you notice? Timestamps, tids, statements — paste freely."
			spellcheck="false"
		></textarea>
	</div>
{/if}

<style>
	.panel {
		position: fixed;
		inset: 10vh 4vw 5vh 4vw;
		z-index: 40;
		display: flex;
		flex-direction: column;
		background: color-mix(in srgb, var(--bg-hard) 96%, transparent);
		border: 1px solid var(--hairline);
		border-radius: var(--radius);
		box-shadow: 0 16px 48px color-mix(in srgb, var(--bg-hard) 80%, transparent);
		outline: none;
	}
	header {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		font-size: 11.5px;
		flex-shrink: 0;
		flex-wrap: wrap;
	}
	.title {
		text-transform: uppercase;
		letter-spacing: 0.06em;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.route {
		padding: 0 8px;
		border-radius: 999px;
		background: var(--bg-soft);
		color: var(--accent);
		font-size: 11px;
		max-width: 40vw;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.route:hover {
		background: var(--bg-hover);
	}
	.grow {
		flex: 1;
	}
	.act {
		padding: 1px 10px;
		border-radius: 999px;
		background: var(--bg-soft);
		color: var(--fg);
		font-size: 11px;
	}
	.act:hover {
		background: var(--bg-hover);
	}
	.hint {
		font-size: 10.5px;
	}
	.muted {
		color: var(--fg-muted);
	}
	.mono {
		font-family: var(--font-mono);
	}
	textarea {
		flex: 1;
		min-height: 0;
		margin: 0;
		padding: 12px 14px;
		border: none;
		outline: none;
		resize: none;
		background: transparent;
		color: var(--fg);
		font-family: var(--font-mono);
		font-size: 12.5px;
		line-height: 1.55;
		tab-size: 2;
	}
</style>
