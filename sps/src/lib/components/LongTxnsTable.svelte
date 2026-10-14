<script lang="ts">
	/**
	 * Long-running transactions (MSSQL): one row per session+txn observed
	 * across multiple snapshots; expanding shows every distinct statement
	 * logged under it, in first-seen order — the transaction's story.
	 * Fixed ranking (most snapshots, then longest observed span) on purpose.
	 */
	import type { MssqlLongTxn } from "$lib/api/stuckquery";
	import { formatDuration, formatTimestamp } from "$lib/format";
	import { sortRows } from "$lib/sort";
	import SortButton from "$lib/components/SortButton.svelte";
	import { copyText } from "$lib/clipboard";
	import Icon from "$lib/components/Icon.svelte";

	interface Props {
		rows: MssqlLongTxn[];
		/** jump to the snapshot where this transaction was last seen */
		onjump?: (row: MssqlLongTxn) => void;
	}

	let { rows, onjump }: Props = $props();

	const txnKey = (r: MssqlLongTxn) => `${r.sessionId}:${r.txnId}`;

	// default = the incoming ranking; any column re-sorts (stable on ties)
	type SortKey = "sessionId" | "txnId" | "snapshots" | "span" | "queries" | "login";
	let sortKey = $state<SortKey | null>(null);
	let sortDescending = $state(true);
	function sortBy(key: SortKey) {
		if (sortKey === key) sortDescending = !sortDescending;
		else {
			sortKey = key;
			sortDescending = key !== "login";
		}
	}
	const sorted = $derived(
		sortRows(
			rows,
			sortKey === null
				? null
				: (r) =>
						sortKey === "span"
							? r.lastSeen - r.firstSeen
							: sortKey === "queries"
								? r.queries.length
								: r[sortKey!],
			sortDescending,
		),
	);

	let expanded = $state<string | null>(null);
	let copied = $state<string | null>(null);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	async function copyTxn(r: MssqlLongTxn) {
		const lines = [
			`Transaction ${r.txnId} — session ${r.sessionId} (${r.login || "unknown login"})`,
			`Seen in ${r.snapshots} snapshots, ${formatTimestamp(timeFormat, r.firstSeen)} → ${formatTimestamp(timeFormat, r.lastSeen)}`,
			"",
			...r.queries.flatMap((q, i) => [`-- statement ${i + 1}`, q, ""]),
		];
		if (await copyText(lines.join("\n"))) {
			copied = txnKey(r);
			setTimeout(() => (copied = null), 1500);
		}
	}
</script>

<div class="table">
	<div class="head">
		<span class="col-num"><SortButton label="Session" active={sortKey === "sessionId"} descending={sortDescending} onclick={() => sortBy("sessionId")} /></span>
		<span class="col-num"><SortButton label="Txn" active={sortKey === "txnId"} descending={sortDescending} onclick={() => sortBy("txnId")} /></span>
		<span class="col-num"><SortButton label="Snapshots" active={sortKey === "snapshots"} descending={sortDescending} onclick={() => sortBy("snapshots")} /></span>
		<span class="col-num"><SortButton label="Observed span" active={sortKey === "span"} descending={sortDescending} onclick={() => sortBy("span")} /></span>
		<span class="col-num"><SortButton label="Queries" active={sortKey === "queries"} descending={sortDescending} onclick={() => sortBy("queries")} /></span>
		<span><SortButton label="Login" active={sortKey === "login"} descending={sortDescending} onclick={() => sortBy("login")} /></span>
	</div>

	<div class="rows">
		{#each sorted as r (txnKey(r))}
			<button
				class="row"
				onclick={() => (expanded = expanded === txnKey(r) ? null : txnKey(r))}
			>
				<span class="col-num mono">{r.sessionId}</span>
				<span class="col-num mono">{r.txnId}</span>
				<span class="col-num mono">{r.snapshots}</span>
				<span class="col-num mono">{formatDuration(r.lastSeen - r.firstSeen)}</span>
				<span class="col-num mono">{r.queries.length}</span>
				<span class="login mono">{r.login || "—"}</span>
			</button>
			{#if expanded === txnKey(r)}
				<div class="expand">
					<div class="expand-actions">
						<button
							class="copy"
							onclick={() => copyTxn(r)}
							title="Copy transaction as text"
							aria-label="Copy transaction as text"
						><Icon name={copied === txnKey(r) ? "check" : "copy"} size={12} /></button>
					</div>
					<dl>
						<dt>First seen</dt>
						<dd class="mono">{formatTimestamp(timeFormat, r.firstSeen)}</dd>
						<dt>Last seen</dt>
						<dd class="mono">{formatTimestamp(timeFormat, r.lastSeen)}</dd>
					</dl>
					<ol>
						{#each r.queries as q}
							<li><pre>{q}</pre></li>
						{/each}
					</ol>
					{#if onjump}
						<button class="jump" onclick={() => onjump(r)}>
							view last snapshot →
						</button>
					{/if}
				</div>
			{/if}
		{:else}
			<p class="empty">
				No transaction appears in more than one snapshot — nothing was
				open across dumps.
			</p>
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
		grid-template-columns: 70px 110px 80px 110px 64px 1fr;
		gap: 10px;
		align-items: center;
		padding: 0 10px;
	}

	.head {
		flex-shrink: 0;
		padding-top: 8px;
		padding-bottom: 8px;
		background: var(--bg-soft);
		font-weight: 600;
		color: var(--fg-strong);
		position: sticky;
		top: 0;
		z-index: 1;
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

	.col-num {
		text-align: right;
	}
	.login {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		color: var(--fg-muted);
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
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
	.expand ol {
		margin: 0;
		padding-left: 20px;
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.expand pre {
		/* explicit: <pre> otherwise falls back to the UA monospace */
		font-family: var(--font-mono);
		margin: 0;
		padding: 8px;
		background: var(--bg);
		border-radius: var(--radius);
		font-size: 11.5px;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		max-height: 160px;
		overflow: auto;
	}
	.jump {
		margin-top: 8px;
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.jump:hover {
		background: var(--bg-hover);
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
