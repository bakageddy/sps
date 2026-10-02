<script lang="ts">
	/**
	 * Running Queries analyzer — the periodic "RunningQueries" dumps.
	 * Same anatomy as Stuck Queries (snapshot list left, content right)
	 * and the same row shapes, plus sp_who2 — this log is the always-on
	 * version of that one: a timer, not a valve trigger, so a snapshot
	 * exists near every connection-dump signal (the incident page joins
	 * it; "open incident →" jumps the other way).
	 *
	 * MSSQL and PGSQL stay separate end to end (no union shapes) — the
	 * snapshot row carries its flavor and everything downstream branches.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { pushSelection } from "$lib/navhistory.svelte";
	import {
		runningqueryMssqlSnapshots,
		runningqueryPgsqlSnapshots,
		runningqueryMssqlBlockingSnapshots,
		runningquerySpwho2Snapshots,
		runningqueryMssqlQueries,
		runningqueryPgsqlQueries,
		runningqueryMssqlBlocking,
		runningquerySpwho2,
		runningqueryMssqlLongrunning,
		runningqueryPgsqlLongrunning,
		runningqueryMssqlLongtxns,
		type MssqlLongTxn,
		type MssqlSnapshot,
		type PgsqlSnapshot,
		type BlockingSnapshot,
		type SpWho2Snapshot,
		type MssqlQuery,
		type PgsqlQuery,
		type MssqlBlockingRow,
		type SpWho2Row,
	} from "$lib/api/runningquery";
	import SnapshotList, {
		snapshotKey,
		type SnapshotRow,
	} from "$lib/components/SnapshotList.svelte";
	import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
	import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
	import BlockingTree from "$lib/components/BlockingTree.svelte";
	import SpWho2Table from "$lib/components/SpWho2Table.svelte";
	import LongRunnersTable, {
		type LongRunnerRow,
	} from "$lib/components/LongRunnersTable.svelte";
	import LongTxnsTable from "$lib/components/LongTxnsTable.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { cached } from "$lib/query-cache";
	import { nearestByTimestamp } from "$lib/nearest";
	import { slowThreshold } from "$lib/stuckquery-settings.svelte";

	let errorMessage = $state<string | null>(null);
	let mssqlSnaps = $state<MssqlSnapshot[]>([]);
	let pgsqlSnaps = $state<PgsqlSnapshot[]>([]);
	let blockingSnaps = $state<BlockingSnapshot[]>([]);
	let spwho2Snaps = $state<SpWho2Snapshot[]>([]);
	let selected = $state<SnapshotRow | null>(null);

	// per-selection content (only the selected row's kind is populated)
	let mssqlQueries = $state<MssqlQuery[]>([]);
	let pgsqlQueries = $state<PgsqlQuery[]>([]);
	let blocking = $state<MssqlBlockingRow[]>([]);
	let spwho2 = $state<SpWho2Row[]>([]);
	let longRunners = $state<LongRunnerRow[]>([]);
	let longTxns = $state<MssqlLongTxn[]>([]);

	const Mode = { Snapshot: "snapshot", Long: "long", Txns: "txns" } as const;
	type Mode = (typeof Mode)[keyof typeof Mode];
	let mode = $state<Mode>(Mode.Snapshot);

	// Unlike stuck-query snapshots (logged only while something is stuck,
	// so "blocked" is the norm), these are periodic samples: a tick with
	// anything blocked or waiting IS the notable one and earns the alert.
	const rows = $derived.by<SnapshotRow[]>(() => {
		const out: SnapshotRow[] = [
			...mssqlSnaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "mssql" as const,
				detail:
					`${s.queries} queries` +
					(s.blocked > 0 ? ` · ${s.blocked} blocked` : ""),
				alert: s.blocked > 0,
			})),
			...pgsqlSnaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "pgsql" as const,
				detail:
					`${s.queries} queries` +
					(s.waiting > 0 ? ` · ${s.waiting} waiting` : "") +
					(s.idleInTxn > 0 ? ` · ${s.idleInTxn} idle in txn` : ""),
				alert: s.waiting > 0,
			})),
			...blockingSnaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "blocking" as const,
				detail: `${s.chains} chain${s.chains === 1 ? "" : "s"} · ${s.sessions} sessions`,
				alert: true,
			})),
			...spwho2Snaps.map((s) => ({
				timestamp: s.timestamp,
				kind: "spwho2" as const,
				detail:
					`${s.sessions} sessions · ${s.active} active` +
					(s.blocked > 0 ? ` · ${s.blocked} blocked` : ""),
				alert: s.blocked > 0,
			})),
		];
		return out.toSorted((a, b) => a.timestamp - b.timestamp);
	});

	async function refresh() {
		const [mssqlR, pgsqlR, blockingR, spwho2R] = await Promise.allSettled([
			cached("runningquery_mssql_snapshots", runningqueryMssqlSnapshots),
			cached("runningquery_pgsql_snapshots", runningqueryPgsqlSnapshots),
			cached(
				"runningquery_mssql_blocking_snapshots",
				runningqueryMssqlBlockingSnapshots,
			),
			cached("runningquery_spwho2_snapshots", runningquerySpwho2Snapshots),
		]);
		mssqlSnaps = settled(mssqlR, []);
		pgsqlSnaps = settled(pgsqlR, []);
		blockingSnaps = settled(blockingR, []);
		spwho2Snaps = settled(spwho2R, []);

		if (selected === null && rows.length > 0) onselect(rows[0]);
		loadLongRunners();
	}

	/** unwrap an allSettled slot; a rejection surfaces once and yields the fallback */
	function settled<T>(r: PromiseSettledResult<T>, fallback: T): T {
		if (r.status === "fulfilled") return r.value;
		errorMessage = String(r.reason);
		return fallback;
	}

	async function onselect(row: SnapshotRow) {
		// the selection is a history entry: back/forward walk the incidents.
		// Mark it consumed first so the ?t= handler below doesn't echo the
		// selection back when the URL changes.
		consumedLink = String(row.timestamp);
		pushSelection(row.timestamp);
		selected = row;
		const key = snapshotKey(row);
		// stale guard: a slower response must not clobber a newer selection
		const stale = () => selected !== null && snapshotKey(selected) !== key;
		try {
			if (row.kind === "mssql") {
				const queries = await cached(
					`runningquery_mssql_queries:${row.timestamp}`,
					() => runningqueryMssqlQueries(row.timestamp),
				);
				if (stale()) return;
				mssqlQueries = queries;
			} else if (row.kind === "pgsql") {
				const queries = await cached(
					`runningquery_pgsql_queries:${row.timestamp}`,
					() => runningqueryPgsqlQueries(row.timestamp),
				);
				if (stale()) return;
				pgsqlQueries = queries;
			} else if (row.kind === "blocking") {
				const chains = await cached(
					`runningquery_mssql_blocking:${row.timestamp}`,
					() => runningqueryMssqlBlocking(row.timestamp),
				);
				if (stale()) return;
				blocking = chains;
			} else {
				const sessions = await cached(
					`runningquery_spwho2:${row.timestamp}`,
					() => runningquerySpwho2(row.timestamp),
				);
				if (stale()) return;
				spwho2 = sessions;
			}
		} catch (e) {
			errorMessage = String(e);
		}
	}

	async function loadLongRunners() {
		const [mssqlR, pgsqlR, txnsR] = await Promise.allSettled([
			cached("runningquery_mssql_longrunning", runningqueryMssqlLongrunning),
			cached("runningquery_pgsql_longrunning", runningqueryPgsqlLongrunning),
			cached("runningquery_mssql_longtxns", runningqueryMssqlLongtxns),
		]);
		longTxns = settled(txnsR, []);
		const out: LongRunnerRow[] = [];
		for (const r of settled(mssqlR, [])) {
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
		for (const r of settled(pgsqlR, [])) {
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
			blockingSnaps = [];
			spwho2Snaps = [];
			selected = null;
		}
	});

	$effect(() => {
		if (ingest.generation === 0) return;
		selected = null;
		refresh();
	});

	// ?t=<ms> from the incident page: select the nearest snapshot, once.
	// re-entrant: a NEW ?t= (another analyzer's link, the palette's "jump
	// to time") re-selects; the same value is consumed once
	let consumedLink = $state<string | null>(null);
	$effect(() => {
		const raw = page.url.searchParams.get("t");
		if (raw === null || raw === consumedLink) return;
		const target = Number(raw);
		if (!Number.isFinite(target) || rows.length === 0) return;
		const nearest = nearestByTimestamp(rows, target);
		if (nearest === null) return;
		consumedLink = raw;
		mode = Mode.Snapshot;
		onselect(nearest);
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
						{#if mode === Mode.Snapshot && selected !== null}
							<button
								class="link"
								onclick={() =>
									goto(
										`/connectiondump/incident?t=${selected!.timestamp}`,
									)}
								title="Open the connection-dump incident nearest this snapshot"
								>open incident →</button
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
						<p class="empty">
							{rows.length === 0
								? "No running-query dumps — parse a bundle with RunningQueries logs from the Ingest page."
								: "Select a snapshot."}
						</p>
					{:else if selected.kind === "blocking"}
						<BlockingTree rows={blocking} />
					{:else if selected.kind === "spwho2"}
						<SpWho2Table rows={spwho2} />
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
	.link {
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.link:hover {
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
