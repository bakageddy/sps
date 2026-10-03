<script lang="ts">
	/**
	 * Floating note editor (Ctrl+\): a wide panel over the current page,
	 * like an editor's terminal drawer. Plain textarea — paste works as
	 * paste does. Nothing is saved until you say so: Ctrl+S (or the Save
	 * button) writes the note; Esc closes, asking once if you'd lose edits.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		noteEditor,
		closeQuickNote,
		getNote,
		saveNote,
		deleteNote,
	} from "$lib/notes.svelte";
	import { formatTimestamp } from "$lib/format";

	let text = $state("");
	let route = $state("");
	let id = $state<number | null>(null);
	/** the text as last saved (or "" for a new note) — dirty = differs */
	let savedText = $state("");
	let textarea = $state<HTMLTextAreaElement>();
	let status = $state<{ kind: "saved" | "deleted" | "error"; text: string } | null>(null);
	let confirmDiscard = $state(false);
	let saving = $state(false);

	const dirty = $derived(text !== savedText);

	// (re)load whenever the editor opens or is pointed at another note
	$effect(() => {
		if (!noteEditor.open) return;
		const target = noteEditor.id;
		const existing = target === null ? undefined : getNote(target);
		// NOTE: this effect must not READ `text` — doing so would make every
		// keystroke re-run it and reset the editor to the saved copy
		const initial = existing?.text ?? "";
		id = existing?.createdAt ?? null;
		text = initial;
		savedText = initial;
		route = existing?.route ?? page.url.pathname + page.url.search;
		status = null;
		confirmDiscard = false;
		requestAnimationFrame(() => {
			textarea?.focus();
			textarea?.setSelectionRange(initial.length, initial.length);
		});
	});

	async function save() {
		if (saving) return;
		saving = true;
		try {
			if (text.trim() === "") {
				// saving an empty note is deleting it
				if (id !== null) {
					await deleteNote(id);
					id = null;
					status = { kind: "deleted", text: "deleted" };
				}
				savedText = "";
			} else {
				id = await saveNote(id, text, route);
				noteEditor.id = id;
				savedText = text;
				status = { kind: "saved", text: "saved" };
			}
			confirmDiscard = false;
			setTimeout(() => (status = status?.kind === "error" ? status : null), 1500);
		} catch (e) {
			status = { kind: "error", text: `not saved: ${String(e)}` };
		} finally {
			saving = false;
		}
	}

	function close() {
		if (dirty && !confirmDiscard) {
			confirmDiscard = true;
			return;
		}
		closeQuickNote();
	}

	function onkeydown(e: KeyboardEvent) {
		if ((e.ctrlKey || e.metaKey) && e.key === "s") {
			e.preventDefault();
			e.stopPropagation();
			void save();
		} else if (e.key === "Escape") {
			e.preventDefault();
			close();
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
				<span class="muted">new</span>
			{/if}
			<button
				class="route mono"
				onclick={() => {
					closeQuickNote();
					goto(route);
				}}
				title="Open the page this note was taken on">{route}</button
			>
			{#if dirty}
				<span class="dirty" title="Unsaved changes">● unsaved</span>
			{/if}
			{#if status}
				<span class="status {status.kind}">{status.text}</span>
			{/if}
			<span class="grow"></span>
			<button class="act save" onclick={save} disabled={!dirty || saving} title="Ctrl+S"
				>{saving ? "saving…" : "Save"}</button
			>
			<span class="muted hint">Ctrl+S saves · Esc closes · Ctrl+\ toggles</span>
			<button class="act" onclick={close} aria-label="Close">✕</button>
		</header>
		{#if confirmDiscard}
			<div class="confirm" role="alert">
				Unsaved changes — <button class="act" onclick={save}>save</button>
				<button class="act danger" onclick={closeQuickNote}>discard</button>
				<span class="muted">(Esc again discards)</span>
			</div>
		{/if}
		<textarea
			bind:this={textarea}
			bind:value={text}
			oninput={() => (confirmDiscard = false)}
			placeholder="What did you notice? Timestamps, tids, statements — paste freely. Ctrl+S to save."
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
	.dirty {
		color: var(--alert);
		font-size: 11px;
	}
	.status {
		font-size: 11px;
	}
	.status.saved {
		color: var(--green);
	}
	.status.deleted {
		color: var(--fg-muted);
	}
	.status.error {
		color: var(--red);
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
	.act:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	.act.save {
		background: var(--accent);
		color: var(--bg-hard);
		font-weight: 600;
	}
	.act.save:disabled {
		background: var(--bg-soft);
		color: var(--fg-muted);
		opacity: 0.7;
	}
	.act.danger {
		color: var(--red);
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
	.confirm {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		background: color-mix(in srgb, var(--alert) 12%, transparent);
		color: var(--alert);
		font-size: 11.5px;
		flex-shrink: 0;
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
