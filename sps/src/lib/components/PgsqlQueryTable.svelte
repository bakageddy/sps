<script lang="ts">
	/**
	 * "Currently Running Queries" (PGSQL flavor) for one snapshot.
	 * Same anatomy as the MSSQL table: sortable, rows expand in place.
	 * 'idle in transaction' is flagged loudly — it holds locks and blocks
	 * vacuum while doing nothing, the classic Postgres incident.
	 */
	import type { PgsqlQuery } from "$lib/api/stuckquery";
	import { formatDuration } from "$lib/format";
	import { slowThreshold } from "$lib/stuckquery-settings.svelte";
	import { copyText } from "$lib/clipboard";
	import Icon from "$lib/components/Icon.svelte";

	interface Props {
		queries: PgsqlQuery[];
	}

	let { queries }: Props = $props();

	type SortKey = "pid" | "state" | "waiting" | "db" | "queryTime" | "txnTime";
	let sortKey = $state<SortKey>("queryTime");
	let sortDescending = $state(true);
	let expanded = $state<number | null>(null);

	function sortBy(key: SortKey) {
		if (sortKey === key) sortDescending = !sortDescending;
		else {
			sortKey = key;
			sortDescending =
				key === "queryTime" || key === "txnTime" || key === "waiting";
		}
	}

	function sortValue(q: PgsqlQuery): string | number {
		switch (sortKey) {
			case "pid":
				return q.pid;
			case "state":
				return q.state;
			case "waiting":
				return q.waiting ? 1 : 0;
			case "db":
				return q.dbName;
			case "queryTime":
				return q.queryTime ?? -1;
			case "txnTime":
				return q.txnTime ?? -1;
		}
	}

	const sorted = $derived(
		queries.toSorted((a, b) => {
			const va = sortValue(a);
			const vb = sortValue(b);
			const order = va < vb ? -1 : va > vb ? 1 : 0;
			return sortDescending ? -order : order;
		}),
	);

	let copied = $state<number | null>(null);
	async function copyQuery(q: PgsqlQuery) {
		const lines = [
			`PID: ${q.pid} (${q.state})  DB: ${q.dbName}  Waiting: ${q.waiting ? "yes" : "no"}`,
			`Query time: ${q.queryTime === null ? "—" : formatDuration(q.queryTime)}  Txn time: ${q.txnTime === null ? "—" : formatDuration(q.txnTime)}`,
			`Application: ${q.applicationName ?? "—"}  Client: ${q.clientHost ?? "—"}${q.clientPort !== null ? `:${q.clientPort}` : ""}`,
			"",
			q.query,
		];
		if (await copyText(lines.join("\n"))) {
			copied = q.pid;
			setTimeout(() => (copied = null), 1500);
		}
	}

	const columns: { key: SortKey; label: string; class: string }[] = [
		{ key: "pid", label: "PID", class: "col-num" },
		{ key: "state", label: "State", class: "col-state" },
		{ key: "waiting", label: "Waiting", class: "col-waiting" },
		{ key: "db", label: "DB", class: "col-db" },
		{ key: "queryTime", label: "Query time", class: "col-num" },
		{ key: "txnTime", label: "Txn time", class: "col-num" },
	];
</script>

<div class="table">
	<div class="head">
		{#each columns as col (col.key)}
			<button class="sort {col.class}" onclick={() => sortBy(col.key)}>
				{col.label}
				{#if sortKey === col.key}<span class="arrow"
						>{sortDescending ? "▼" : "▲"}</span
					>{/if}
			</button>
		{/each}
		<span class="sort col-query">Query</span>
	</div>

	<div class="rows">
		{#each sorted as q}
			<button
				class="row"
				class:slow={(q.queryTime ?? 0) > slowThreshold.value * 1000}
				onclick={() => (expanded = expanded === q.pid ? null : q.pid)}
			>
				<span class="col-num mono">{q.pid}</span>
				<span class="col-state">
					<span class="badge" class:idle={q.state !== "active"}
						>{q.state}</span
					>
				</span>
				<span class="col-waiting mono">{q.waiting ? "yes" : "—"}</span>
				<span class="col-db mono">{q.dbName}</span>
				<span class="col-num mono"
					>{q.queryTime === null
						? "—"
						: formatDuration(q.queryTime)}</span
				>
				<span class="col-num mono"
					>{q.txnTime === null
						? "—"
						: formatDuration(q.txnTime)}</span
				>
				<span class="col-query mono">{q.query}</span>
			</button>
			{#if expanded === q.pid}
				<div class="expand">
					<div class="expand-actions">
						<button
							class="copy"
							onclick={() => copyQuery(q)}
							title="Copy query as text"
							aria-label="Copy query as text"
							><Icon
								name={copied === q.pid ? "check" : "copy"}
								size={12}
							/></button
						>
					</div>
					<dl>
						<dt>Application</dt>
						<dd class="mono">{q.applicationName ?? "—"}</dd>
						<dt>Client</dt>
						<dd class="mono">
							{q.clientHost ?? "—"}{q.clientPort !== null
								? `:${q.clientPort}`
								: ""}
						</dd>
					</dl>
					<pre>{q.query}</pre>
				</div>
			{/if}
		{:else}
			<p class="empty">No queries in this snapshot.</p>
		{/each}
	</div>
</div>

<style>
	.table {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		font-size: 12.5px;
	}

	.head,
	.row {
		display: grid;
		/* waiting column must fit its own nowrap header ("Waiting ▼") — a
       too-narrow track lets the sticky header bleed into the DB column */
		grid-template-columns: 64px 130px 84px 110px 90px 90px 1fr;
		gap: 10px;
		align-items: center;
		padding: 0 10px;
	}

	.head {
		flex-shrink: 0;
		background: var(--bg-soft);
		position: sticky;
		top: 0;
		z-index: 1;
	}

	.sort {
		padding: 8px 0;
		text-align: left;
		font-weight: 600;
		color: var(--fg-strong);
		white-space: nowrap;
		border-radius: 0;
	}
	button.sort:hover {
		color: var(--accent);
	}
	.sort.col-num {
		text-align: right;
	}
	.arrow {
		font-size: 9px;
		color: var(--accent);
	}

	.rows {
		overflow: auto;
		flex: 1;
	}

	.row {
		width: 100%;
		text-align: left;
		padding-top: 4px;
		padding-bottom: 4px;
		border-top: 1px solid var(--hairline);
		border-radius: 0;
		color: var(--fg);
	}
	.row:hover {
		background: var(--bg-hover);
	}
	/* red is reserved for what matters: queries over the slow threshold */
	.row.slow {
		background: color-mix(in srgb, var(--red) 8%, transparent);
	}

	.col-num {
		text-align: right;
	}
	.col-db,
	.col-query {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}

	.badge {
		padding: 0 8px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 600;
		background: color-mix(in srgb, var(--green) 18%, transparent);
		color: var(--green);
		white-space: nowrap;
	}
	/* noteworthy, not alarming — red stays reserved for over-threshold rows */
	.badge.idle {
		background: color-mix(in srgb, var(--yellow) 18%, transparent);
		color: var(--yellow);
	}

	.expand {
		position: relative;
		padding: 8px 12px;
		border-top: 1px solid var(--hairline);
		background: var(--bg-hard);
	}
	.expand-actions {
		position: absolute;
		top: 6px;
		right: 8px;
	}
	.copy {
		display: grid;
		place-items: center;
		padding: 4px;
		color: var(--fg-muted);
	}
	.copy:hover {
		background: var(--bg-hover);
		color: var(--fg);
	}
	.expand dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 3px 12px;
		margin: 0 0 8px;
		font-size: 12px;
	}
	.expand dt {
		color: var(--fg-muted);
	}
	.expand dd {
		margin: 0;
	}
	.expand pre {
		margin: 0;
		padding: 8px;
		background: var(--bg);
		border-radius: var(--radius);
		font-size: 11.5px;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		max-height: 240px;
		overflow: auto;
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
