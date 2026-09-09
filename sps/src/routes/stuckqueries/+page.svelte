<script lang="ts">
	/**
	 * Stuck Queries analyzer — cpumemstats anatomy: snapshot list on the
	 * left, content on the right. Query snapshots and BLOCKING snapshots are
	 * separate commands and separate list rows (same timestamp can appear as
	 * both) — selecting a blocking row renders the chain tree directly.
	 * A "Long-running" mode shows executions present across MULTIPLE
	 * snapshots.
	 *
	 * MSSQL and PGSQL stay separate end to end (no union shapes) — the
	 * snapshot row carries its flavor and everything downstream branches.
	 */
	import {
		stuckqueryMssqlSnapshots,
		stuckqueryPgsqlSnapshots,
		stuckqueryMssqlBlockingSnapshots,
		stuckqueryMssqlQueries,
		stuckqueryPgsqlQueries,
		stuckqueryMssqlBlocking,
		stuckqueryMssqlLongrunning,
		stuckqueryPgsqlLongrunning,
		stuckqueryMssqlLongtxns,
		type MssqlLongTxn,
		type MssqlSnapshot,
		type PgsqlSnapshot,
		type BlockingSnapshot,
		type MssqlQuery,
		type PgsqlQuery,
		type MssqlBlockingRow,
	} from "$lib/api/stuckquery";
	import SnapshotList, {
		snapshotKey,
		type SnapshotRow,
	} from "$lib/components/SnapshotList.svelte";
	import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
	import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
	import BlockingTree from "$lib/components/BlockingTree.svelte";
	import LongRunnersTable, {
		type LongRunnerRow,
	} from "$lib/components/LongRunnersTable.svelte";
	import LongTxnsTable from "$lib/components/LongTxnsTable.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { cached } from "$lib/query-cache";
	import { slowThreshold } from "$lib/stuckquery-settings.svelte";
	import { goto } from "$app/navigation";

	let errorMessage = $state<string | null>(null);
	let mssqlSnaps = $state<MssqlSnapshot[]>([]);
	let pgsqlSnaps = $state<PgsqlSnapshot[]>([]);
	let blockingSnaps = $state<BlockingSnapshot[]>([]);
	let selected = $state<SnapshotRow | null>(null);

	// per-selection content (only the selected row's kind is populated)
	let mssqlQueries = $state<MssqlQuery[]>([]);
	let pgsqlQueries = $state<PgsqlQuery[]>([]);
	let blocking = $state<MssqlBlockingRow[]>([]);
	let longRunners = $state<LongRunnerRow[]>([]);
	let longTxns = $state<MssqlLongTxn[]>([]);

	const Mode = { Snapshot: "snapshot", Long: "long", Txns: "txns" } as const;
	type Mode = (typeof Mode)[keyof typeof Mode];
	let mode = $state<Mode>(Mode.Snapshot);

	const rows = $derived.by<SnapshotRow[]>(() => {
		const out: SnapshotRow[] = [
			// alert stays false for query snapshots: nearly every snapshot has
			// SOMETHING waiting (they're logged during stuck threads by
			// definition) — red everywhere is red nowhere. Blocking rows are the
			// genuinely important ones and keep the alert.
			...mssqlSnaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "mssql" as const,
				detail:
					`${s.queries} queries` +
					(s.blocked > 0 ? ` · ${s.blocked} blocked` : ""),
				alert: false,
			})),
			...pgsqlSnaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "pgsql" as const,
				detail:
					`${s.queries} queries` +
					(s.waiting > 0 ? ` · ${s.waiting} waiting` : "") +
					(s.idleInTxn > 0 ? ` · ${s.idleInTxn} idle in txn` : ""),
				alert: false,
			})),
			...blockingSnaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "blocking" as const,
				detail: `${s.chains} chain${s.chains === 1 ? "" : "s"} · ${s.sessions} sessions`,
				alert: true,
			})),
		];
		return out.toSorted((a, b) => a.timestamp - b.timestamp);
	});

	async function refresh() {
		const [mssqlResult, pgsqlResult, blockingResult] =
			await Promise.allSettled([
				cached("stuckquery_mssql_snapshots", stuckqueryMssqlSnapshots),
				cached("stuckquery_pgsql_snapshots", stuckqueryPgsqlSnapshots),
				cached(
					"stuckquery_mssql_blocking_snapshots",
					stuckqueryMssqlBlockingSnapshots,
				),
			]);
		if (mssqlResult.status === "fulfilled") mssqlSnaps = mssqlResult.value;
		else errorMessage = String(mssqlResult.reason);
		if (pgsqlResult.status === "fulfilled") pgsqlSnaps = pgsqlResult.value;
		else errorMessage = String(pgsqlResult.reason);
		if (blockingResult.status === "fulfilled")
			blockingSnaps = blockingResult.value;
		else errorMessage = String(blockingResult.reason);

		if (selected === null && rows.length > 0) onselect(rows[0]);
		loadLongRunners();
	}

	async function onselect(row: SnapshotRow) {
		selected = row;
		const key = snapshotKey(row);
		try {
			if (row.kind === "mssql") {
				const queries = await cached(
					`stuckquery_mssql_queries:${row.timestamp}`,
					() => stuckqueryMssqlQueries(row.timestamp),
				);
				// stale guard: a slower response must not clobber a newer selection
				if (selected !== null && snapshotKey(selected) !== key) return;
				mssqlQueries = queries;
			} else if (row.kind === "pgsql") {
				const queries = await cached(
					`stuckquery_pgsql_queries:${row.timestamp}`,
					() => stuckqueryPgsqlQueries(row.timestamp),
				);
				if (selected !== null && snapshotKey(selected) !== key) return;
				pgsqlQueries = queries;
			} else {
				const chains = await cached(
					`stuckquery_mssql_blocking:${row.timestamp}`,
					() => stuckqueryMssqlBlocking(row.timestamp),
				);
				if (selected !== null && snapshotKey(selected) !== key) return;
				blocking = chains;
			}
		} catch (e) {
			errorMessage = String(e);
		}
	}

	async function loadLongRunners() {
		const [mssqlResult, pgsqlResult, txnsResult] = await Promise.allSettled([
			cached("stuckquery_mssql_longrunning", stuckqueryMssqlLongrunning),
			cached("stuckquery_pgsql_longrunning", stuckqueryPgsqlLongrunning),
			cached("stuckquery_mssql_longtxns", stuckqueryMssqlLongtxns),
		]);
		if (txnsResult.status === "fulfilled") longTxns = txnsResult.value;
		const out: LongRunnerRow[] = [];
		if (mssqlResult.status === "fulfilled") {
			for (const r of mssqlResult.value) {
				out.push({
					key: `mssql:${r.sessionId}:${r.txnId}`,
					who: `session ${r.sessionId}`,
					query: r.statement,
					snapshots: r.snapshots,
					firstSeen: r.firstSeen,
					lastSeen: r.lastSeen,
					maxMs: r.maxElapsedMs,
					flagCount: r.blockedIn,
					flagLabel: "blocked",
				});
			}
		}
		if (pgsqlResult.status === "fulfilled") {
			for (const r of pgsqlResult.value) {
				out.push({
					key: `pgsql:${r.pid}:${r.firstSeen}`,
					who: `pid ${r.pid}`,
					query: r.query,
					snapshots: r.snapshots,
					firstSeen: r.firstSeen,
					lastSeen: r.lastSeen,
					maxMs: r.maxQueryTimeMs,
					flagCount: r.idleInTxnIn,
					flagLabel: "idle in txn",
				});
			}
		}
		// ranking: most snapshots first, then longest
		longRunners = out.toSorted(
			(a, b) =>
				b.snapshots - a.snapshots || (b.maxMs ?? 0) - (a.maxMs ?? 0),
		);
	}

	$effect(() => {
		if (db.state.status === "open") {
			errorMessage = null;
			selected = null;
			refresh();
		} else {
			mssqlSnaps = [];
			pgsqlSnaps = [];
			selected = null;
		}
	});

	$effect(() => {
		if (ingest.generation === 0) return;
		selected = null;
		refresh();
	});

	/** long-runner drill-down: open the snapshot it was last seen in */
	function onjumptosnapshot(runner: LongRunnerRow) {
		const kind = runner.key.startsWith("mssql") ? "mssql" : "pgsql";
		const target = rows.find(
			(r) => r.kind === kind && r.timestamp === runner.lastSeen,
		);
		if (target === undefined) return;
		mode = Mode.Snapshot;
		onselect(target);
	}

	/** long-transaction drill-down: same jump, mssql by definition */
	function onjumptxn(txn: MssqlLongTxn) {
		const target = rows.find(
			(r) => r.kind === "mssql" && r.timestamp === txn.lastSeen,
		);
		if (target === undefined) return;
		mode = Mode.Snapshot;
		onselect(target);
	}
</script>

<div class="page">
	{#if errorMessage}
		<div class="error-bar" role="alert">
			{errorMessage}
			<button onclick={() => (errorMessage = null)} aria-label="Dismiss"
				>✕</button
			>
		</div>
	{/if}

	<SplitPane direction="row" initial={0.24}>
		{#snippet a()}
			<SnapshotList
				{rows}
				selected={selected === null ? null : snapshotKey(selected)}
				{onselect}
			/>
		{/snippet}
		{#snippet b()}
			<div class="content">
				<div class="toolbar">
					<span class="chips" role="group" aria-label="View">
						<button
							class:active={mode === Mode.Snapshot}
							onclick={() => (mode = Mode.Snapshot)}
							>Snapshot</button
						>
						<button
							class:active={mode === Mode.Long}
							onclick={() => (mode = Mode.Long)}
							>Long-running</button
						>
						<button
							class:active={mode === Mode.Txns}
							onclick={() => (mode = Mode.Txns)}
							>Transactions</button
						>
					</span>

					<span class="right">
						{#if mode === Mode.Snapshot && selected !== null && selected.kind !== "blocking"}
							<button
								class="matcher-link"
								onclick={() =>
									goto(
										`/stuckthreads/queries?t=${selected!.timestamp}`,
									)}
								title="Open this snapshot in the episode matcher"
								>open in matcher →</button
							>
						{/if}
						<label class="threshold">
							slow query threshold
							<input
								type="number"
								min="1"
								max="86400"
								bind:value={slowThreshold.value}
							/>
							s
						</label>
					</span>
				</div>

				<div class="body">
					{#if mode === Mode.Long}
						<LongRunnersTable
							rows={longRunners}
							onjump={onjumptosnapshot}
						/>
					{:else if mode === Mode.Txns}
						<LongTxnsTable rows={longTxns} onjump={onjumptxn} />
					{:else if selected === null}
						<p class="empty">Select a snapshot.</p>
					{:else if selected.kind === "blocking"}
						<BlockingTree rows={blocking} />
					{:else if selected.kind === "mssql"}
						<MssqlQueryTable queries={mssqlQueries} />
					{:else}
						<PgsqlQueryTable queries={pgsqlQueries} />
					{/if}
				</div>
			</div>
		{/snippet}
	</SplitPane>
</div>

<style>
	.page {
		display: flex;
		flex-direction: column;
		height: 100%;
	}

	.error-bar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 6px 12px;
		font-size: 12px;
		color: var(--red);
		background: color-mix(in srgb, var(--red) 12%, transparent);
		border-bottom: 1px solid var(--hairline);
	}
	.error-bar button {
		background: none;
		border: none;
		cursor: pointer;
		color: inherit;
	}

	.content {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}

	.toolbar {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.right {
		display: flex;
		align-items: center;
		gap: 14px;
	}
	.matcher-link {
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.matcher-link:hover {
		background: var(--bg-hover);
	}
	.threshold {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.threshold input {
		width: 64px;
		padding: 2px 6px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		color: var(--fg);
		font-family: var(--font-mono);
		font-size: 11.5px;
		text-align: right;
	}
	.chips {
		display: flex;
		gap: 2px;
		padding: 2px;
		background: var(--bg-hard);
		border-radius: 999px;
	}
	.chips button {
		padding: 2px 12px;
		font-size: 11.5px;
		border-radius: 999px;
		color: var(--fg-muted);
	}
	.chips button:hover {
		color: var(--fg);
	}
	.chips button.active {
		background: var(--accent);
		color: var(--bg-hard);
		font-weight: 600;
	}

	.body {
		flex: 1;
		min-height: 0;
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
</style>
