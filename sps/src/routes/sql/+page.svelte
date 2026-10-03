<script lang="ts">
	/**
	 * SQL console — ad-hoc read-only queries against the open DuckDB.
	 *
	 * Left: editor over results. Right: the schema as a tree (tables →
	 * columns; click a name to insert it at the caret).
	 * Run with the button or Ctrl/⌘+Enter. Results are paged: the backend
	 * caps every page at `limit` rows and returns None past the last one
	 * (Option<SqlResult>, no probe row to drop); the console infers "maybe
	 * another page" from a page coming back exactly `limit` long, and
	 * walks pages with prev/next. "export CSV" writes the WHOLE result
	 * through DuckDB COPY, so nothing large ever crosses IPC.
	 *
	 * The SELECT-only rule lives in the backend (api/sql.ts) — the console
	 * just shows whatever error comes back, verbatim, under the editor.
	 */
	import { save } from "@tauri-apps/plugin-dialog";
	import {
		sqlQuery,
		sqlSchema,
		sqlExportCsv,
		type SqlResult,
		type SqlTable,
	} from "$lib/api/sql";
	import SqlResultGrid from "$lib/components/SqlResultGrid.svelte";
	import SchemaBrowser from "$lib/components/SchemaBrowser.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { cached } from "$lib/query-cache";
	import { persisted } from "$lib/persisted.svelte";
	import { formatDuration } from "$lib/format";

	// the draft survives navigation and restarts; history keeps the last
	// 20 statements that RAN (successfully or not — a failed one is the one
	// you most want back to fix)
	const draft = persisted("sql-draft", "SELECT * FROM connectiondump LIMIT 100");
	const history = persisted<string[]>("sql-history", []);
	const limit = persisted("sql-limit", 1000);
	const LIMITS = [100, 1000, 10000];

	let tables = $state<SqlTable[]>([]);
	/** the current page's data; null both before the first run AND when a
	 *  run came back empty (Option<SqlResult> collapses those — same "No
	 *  rows" render either way) — `ranOnce` is what tells them apart */
	let result = $state<SqlResult | null>(null);
	let ranOnce = $state(false);
	let offset = $state(0);
	let running = $state(false);
	let error = $state<string | null>(null);
	/** last export outcome, shown in the status line */
	let exportNote = $state<string | null>(null);
	let editor = $state<HTMLTextAreaElement>();

	const ready = $derived(db.state.status === "open");
	const statement = $derived(draft.value.trim());

	async function loadSchema() {
		try {
			tables = await cached("sql_schema", sqlSchema);
		} catch (e) {
			error = String(e);
		}
	}

	async function run(page = 0) {
		if (!ready || statement === "" || running) return;
		running = true;
		error = null;
		exportNote = null;
		const sql = statement;
		const off = page * limit.value;
		try {
			const r = await sqlQuery(sql, limit.value, off);
			result = r;
			offset = off;
			ranOnce = true;
			if (page === 0) remember(sql);
		} catch (e) {
			error = String(e);
			result = null;
		} finally {
			running = false;
		}
	}

	function remember(sql: string) {
		history.value = [sql, ...history.value.filter((h) => h !== sql)].slice(
			0,
			20,
		);
	}

	async function exportCsv() {
		if (!ready || statement === "") return;
		error = null;
		try {
			const path = await save({
				defaultPath: "query.csv",
				filters: [{ name: "CSV", extensions: ["csv"] }],
				title: "Export query result as CSV",
			});
			if (path === null) return; // cancelled — a normal outcome
			const n = await sqlExportCsv(statement, path);
			exportNote = `wrote ${n.toLocaleString()} rows → ${path}`;
		} catch (e) {
			error = String(e);
		}
	}

	/** drop text at the caret, keeping focus in the editor */
	function insert(text: string) {
		const el = editor;
		if (!el) {
			draft.value += text;
			return;
		}
		const start = el.selectionStart;
		const end = el.selectionEnd;
		const value = draft.value;
		draft.value = value.slice(0, start) + text + value.slice(end);
		requestAnimationFrame(() => {
			el.focus();
			el.selectionStart = el.selectionEnd = start + text.length;
		});
	}

	function onkeydown(e: KeyboardEvent) {
		if (e.key === "Enter" && (e.ctrlKey || e.metaKey)) {
			e.preventDefault();
			run();
		}
	}

	$effect(() => {
		if (db.state.status === "open") {
			void db.epoch;
			loadSchema();
		} else {
			tables = [];
			result = null;
		}
	});
	$effect(() => {
		if (ingest.generation === 0) return;
		loadSchema();
	});

	const page = $derived(Math.floor(offset / limit.value));
	// a short page is DEFINITELY the last one; a full page MIGHT not be —
	// the only way to know is to ask for the next page and see
	const canGoNext = $derived(result !== null && result.rows.length === limit.value);
</script>

<div class="page">
	<SplitPane direction="row" initial={0.78}>
		{#snippet a()}
			<SplitPane direction="column" initial={0.3}>
				{#snippet a()}
					<div class="editor">
						<textarea
							bind:this={editor}
							bind:value={draft.value}
							{onkeydown}
							spellcheck="false"
							placeholder="SELECT … (one statement; Ctrl/⌘+Enter runs)"
							disabled={!ready}
						></textarea>
						<div class="toolbar">
							<button
								class="run"
								onclick={() => run()}
								disabled={!ready || running || statement === ""}
								title="Ctrl/⌘+Enter">{running ? "running…" : "Run"}</button
							>
							<label class="limit">
								rows / page
								<select bind:value={limit.value}>
									{#each LIMITS as n (n)}
										<option value={n}>{n.toLocaleString()}</option>
									{/each}
								</select>
							</label>
							<select
								class="history"
								disabled={history.value.length === 0}
								onchange={(e) => {
									const v = (e.currentTarget as HTMLSelectElement).value;
									if (v !== "") draft.value = v;
									(e.currentTarget as HTMLSelectElement).value = "";
								}}
							>
								<option value="">history…</option>
								{#each history.value as h, i (i)}
									<option value={h}>{h.replace(/\s+/g, " ").slice(0, 80)}</option>
								{/each}
							</select>
							<span class="grow"></span>
							<button
								class="export"
								onclick={exportCsv}
								disabled={!ready || statement === ""}
								title="Write the full result to a CSV file via DuckDB COPY"
								>export CSV</button
							>
						</div>
						{#if error}
							<pre class="error" role="alert">{error}</pre>
						{/if}
					</div>
				{/snippet}
				{#snippet b()}
					<div class="results">
						{#if !ready}
							<p class="empty">Open a database to run queries.</p>
						{:else if !ranOnce}
							<p class="empty">
								{running ? "Running…" : "Run a query to see results here."}
							</p>
						{:else if result === null}
							<!-- Ok(None): genuinely empty, or paged past the end — the
							     same render either way; "prev" still works, "next" would
							     only find more of the same -->
							<div class="status">
								<span class="mono">
									No rows{offset > 0 ? ` · page ${page + 1}` : ""}
								</span>
								<span class="grow"></span>
								{#if offset > 0}
									<button disabled={running} onclick={() => run(page - 1)}
										>‹ prev</button
									>
								{/if}
							</div>
							<p class="empty">
								{offset > 0 ? "No rows on this page." : "No rows."}
							</p>
						{:else}
							<div class="status">
								<span class="mono">
									{result.rows.length.toLocaleString()} rows
									{#if offset > 0 || canGoNext}
										· page {page + 1}
									{/if}
									· {formatDuration(result.elapsedMs)}
								</span>
								{#if exportNote}
									<span class="note mono">{exportNote}</span>
								{/if}
								<span class="grow"></span>
								<button
									disabled={page === 0 || running}
									onclick={() => run(page - 1)}>‹ prev</button
								>
								<button
									disabled={!canGoNext || running}
									onclick={() => run(page + 1)}>next ›</button
								>
							</div>
							<div class="grid">
								<SqlResultGrid {result} {offset} />
							</div>
						{/if}
					</div>
				{/snippet}
			</SplitPane>
		{/snippet}
		{#snippet b()}
			<SchemaBrowser {tables} oninsert={insert} />
		{/snippet}
	</SplitPane>
</div>

<style>
	.page {
		height: 100%;
	}

	.editor {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	textarea {
		flex: 1;
		min-height: 0;
		resize: none;
		margin: 0;
		padding: 10px 12px;
		border: none;
		outline: none;
		background: var(--bg-hard);
		color: var(--fg);
		font-family: var(--font-mono);
		font-size: 12.5px;
		line-height: 1.5;
		tab-size: 2;
	}
	textarea:disabled {
		opacity: 0.6;
	}

	.toolbar,
	.status {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 10px;
		border-top: 1px solid var(--hairline);
		flex-shrink: 0;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.status {
		border-top: none;
		border-bottom: 1px solid var(--hairline);
	}
	.grow {
		flex: 1;
	}

	.run {
		padding: 2px 14px;
		border-radius: 999px;
		background: var(--accent);
		color: var(--bg-hard);
		font-weight: 600;
		font-size: 11.5px;
	}
	.run:disabled {
		opacity: 0.5;
	}
	.export,
	.status button {
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.export:hover:not(:disabled),
	.status button:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	.export:disabled,
	.status button:disabled {
		color: var(--fg-muted);
		opacity: 0.6;
	}

	.limit {
		display: flex;
		align-items: center;
		gap: 6px;
	}
	select {
		padding: 2px 6px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		color: var(--fg);
		font-family: var(--font-mono);
		font-size: 11.5px;
	}
	.history {
		max-width: 260px;
	}

	.error {
		margin: 0;
		padding: 8px 12px;
		border-top: 1px solid var(--hairline);
		background: color-mix(in srgb, var(--red) 12%, transparent);
		color: var(--red);
		font-family: var(--font-mono);
		font-size: 11.5px;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		max-height: 40%;
		overflow: auto;
		flex-shrink: 0;
	}

	.results {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.grid {
		flex: 1;
		min-height: 0;
	}
	.note {
		color: var(--green);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		max-width: 50%;
	}
	.mono {
		font-family: var(--font-mono);
	}
	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
</style>
