<script lang="ts">
	/**
	 * Thread Dumps → Queries — database snapshots and thread dumps side by
	 * side, linked by time (the cpumemstats Linked Dumps pattern).
	 *
	 * Left: ONE row per running-query tick (all the tables logged at that
	 * tick — active statements, blocking chains, sp_who2 — fold into the
	 * row), over the thread dumps. "Include stuck queries" adds the
	 * stuck-query snapshots as candidates too: they are the same tables
	 * logged on the valve trigger instead of the timer, so around a thread
	 * dump taken during a stuck episode they are often the closer sample.
	 *
	 * Clicking a snapshot in EITHER list also selects (and loads) the
	 * closest dump in the other list — but only within the link threshold
	 * (seconds, user-tunable, persisted). Outside it the other side clears
	 * rather than pretending an association exists.
	 *
	 *   middle: linked thread dump's census + the clicked thread's stack
	 *   right:  the snapshot's statements (Active / Blocking) over sp_who2
	 *           — both at once, because the sleeping-but-blocking session
	 *           only exists in sp_who2 while its victim only shows as a
	 *           statement
	 *
	 * Reading it: a BLOCKED / long-waiting thread in the middle whose stack
	 * sits in JDBC, and on the right the statement that connection was
	 * running — that pairing is what neither analyzer shows alone.
	 */
	import {
		runningqueryMssqlSnapshots,
		runningqueryPgsqlSnapshots,
		runningqueryMssqlBlockingSnapshots,
		runningquerySpwho2Snapshots,
		runningqueryMssqlQueries,
		runningqueryPgsqlQueries,
		runningqueryMssqlBlocking,
		runningquerySpwho2,
		type MssqlQuery,
		type PgsqlQuery,
		type MssqlBlockingRow,
		type SpWho2Row,
	} from "$lib/api/runningquery";
	import {
		stuckqueryMssqlSnapshots,
		stuckqueryPgsqlSnapshots,
		stuckqueryMssqlBlockingSnapshots,
		stuckqueryMssqlQueries,
		stuckqueryPgsqlQueries,
		stuckqueryMssqlBlocking,
	} from "$lib/api/stuckquery";
	import {
		threaddumpDumps,
		threaddumpThreads,
		threaddumpTrace,
		type ThreadDumpSummary,
		type ThreadDumpThread,
	} from "$lib/api/threaddump";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { cached } from "$lib/query-cache";
	import { nearestByTimestamp } from "$lib/nearest";
	import { persisted } from "$lib/persisted.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import SnapshotList, {
		snapshotKey,
		type SnapshotRow,
	} from "$lib/components/SnapshotList.svelte";
	import ThreadDumpList from "$lib/components/ThreadDumpList.svelte";
	import ThreadDumpTable from "$lib/components/ThreadDumpTable.svelte";
	import ThreadDumpTrace, {
		type ThreadTraceState,
	} from "$lib/components/ThreadDumpTrace.svelte";
	import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
	import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
	import BlockingTree from "$lib/components/BlockingTree.svelte";
	import SpWho2Table from "$lib/components/SpWho2Table.svelte";

	/** where a database snapshot came from: the timer or the valve trigger */
	type Source = "running" | "stuck";

	// --- state -------------------------------------------------------------
	let errorMessage = $state<string | null>(null);

	let runningRows = $state<SnapshotRow[]>([]);
	let stuckRows = $state<SnapshotRow[]>([]);
	let dumps = $state<ThreadDumpSummary[]>([]);

	/** stuck-query snapshots as link candidates too (persisted preference) */
	const includeStuck = persisted("rq-linked-include-stuck", false);
	/** max distance (seconds) for a snapshot and a dump to count as "the same moment" */
	const threshold = persisted("rq-linked-threshold-s", 30);

	/** the selected database snapshot */
	let selectedSnap = $state<{ source: Source; timestamp: number } | null>(null);
	let selectedDump = $state<number | null>(null);

	let threads = $state<ThreadDumpThread[]>([]);
	let selectedTid = $state<number | null>(null);
	let trace = $state<ThreadTraceState>({ status: "idle" });

	// the snapshot's tables — all fetched together; a bundle logs one
	// flavor, so the other flavor's arrays simply come back empty
	let mssqlQueries = $state<MssqlQuery[]>([]);
	let pgsqlQueries = $state<PgsqlQuery[]>([]);
	let blocking = $state<MssqlBlockingRow[]>([]);
	let spwho2 = $state<SpWho2Row[]>([]);

	const StmtView = { Active: "active", Blocking: "blocking" } as const;
	type StmtView = (typeof StmtView)[keyof typeof StmtView];
	let stmtView = $state<StmtView>(StmtView.Active);

	const rows = $derived<SnapshotRow[]>(
		(includeStuck.value ? [...runningRows, ...stuckRows] : runningRows).toSorted(
			(a, b) => a.timestamp - b.timestamp,
		),
	);

	const selectedKey = $derived(
		selectedSnap === null
			? null
			: snapshotKey({
					timestamp: selectedSnap.timestamp,
					kind: selectedSnap.source,
					detail: "",
					alert: false,
				}),
	);

	/** seconds between the two linked dumps, when both are selected */
	const linkDelta = $derived(
		selectedSnap !== null && selectedDump !== null
			? (selectedDump - selectedSnap.timestamp) / 1000
			: null,
	);

	// --- folding the per-table snapshot rollups into one row per moment ----
	interface Fold {
		queries: number;
		blocked: number;
		waiting: number;
		idleInTxn: number;
		chains: number;
		sessions: number;
	}
	function fold(
		mssql: { timestamp: number; queries: number; blocked: number }[],
		pgsql: { timestamp: number; queries: number; waiting: number; idleInTxn: number }[],
		blockingSnaps: { timestamp: number; chains: number; sessions: number }[],
		spwho2Snaps: { timestamp: number; sessions: number; blocked: number }[],
		kind: Source,
	): SnapshotRow[] {
		const at = new Map<number, Fold>();
		const get = (ts: number) => {
			let f = at.get(ts);
			if (!f) {
				f = { queries: 0, blocked: 0, waiting: 0, idleInTxn: 0, chains: 0, sessions: 0 };
				at.set(ts, f);
			}
			return f;
		};
		for (const s of mssql) {
			const f = get(s.timestamp);
			f.queries += s.queries;
			f.blocked += s.blocked;
		}
		for (const s of pgsql) {
			const f = get(s.timestamp);
			f.queries += s.queries;
			f.waiting += s.waiting;
			f.idleInTxn += s.idleInTxn;
		}
		for (const s of blockingSnaps) get(s.timestamp).chains += s.chains;
		for (const s of spwho2Snaps) {
			const f = get(s.timestamp);
			f.sessions += s.sessions;
			f.blocked = Math.max(f.blocked, s.blocked);
		}
		return [...at.entries()].map(([timestamp, f]) => {
			const parts = [`${f.queries} queries`];
			if (f.blocked > 0) parts.push(`${f.blocked} blocked`);
			if (f.waiting > 0) parts.push(`${f.waiting} waiting`);
			if (f.idleInTxn > 0) parts.push(`${f.idleInTxn} idle in txn`);
			if (f.chains > 0) parts.push(`${f.chains} chain${f.chains === 1 ? "" : "s"}`);
			if (f.sessions > 0) parts.push(`${f.sessions} sessions`);
			return {
				timestamp,
				kind,
				detail: parts.join(" · "),
				alert: f.blocked > 0 || f.waiting > 0 || f.chains > 0,
			};
		});
	}

	// --- lifecycle -----------------------------------------------------------
	function resetSelection() {
		selectedSnap = null;
		selectedDump = null;
		threads = [];
		selectedTid = null;
		trace = { status: "idle" };
		mssqlQueries = [];
		pgsqlQueries = [];
		blocking = [];
		spwho2 = [];
		stmtView = StmtView.Active;
	}

	function settled<T>(r: PromiseSettledResult<T>, fallback: T): T {
		if (r.status === "fulfilled") return r.value;
		errorMessage = String(r.reason);
		return fallback;
	}

	async function refresh() {
		const [rm, rp, rb, rs, sm, sp, sb, dumpsR] = await Promise.allSettled([
			cached("runningquery_mssql_snapshots", runningqueryMssqlSnapshots),
			cached("runningquery_pgsql_snapshots", runningqueryPgsqlSnapshots),
			cached(
				"runningquery_mssql_blocking_snapshots",
				runningqueryMssqlBlockingSnapshots,
			),
			cached("runningquery_spwho2_snapshots", runningquerySpwho2Snapshots),
			cached("stuckquery_mssql_snapshots", stuckqueryMssqlSnapshots),
			cached("stuckquery_pgsql_snapshots", stuckqueryPgsqlSnapshots),
			cached(
				"stuckquery_mssql_blocking_snapshots",
				stuckqueryMssqlBlockingSnapshots,
			),
			cached("threaddump_dumps", threaddumpDumps),
		]);
		runningRows = fold(
			settled(rm, []),
			settled(rp, []),
			settled(rb, []),
			settled(rs, []),
			"running",
		);
		stuckRows = fold(settled(sm, []), settled(sp, []), settled(sb, []), [], "stuck");
		dumps = settled(dumpsR, []);
	}

	$effect(() => {
		if (db.state.status === "open") {
			resetSelection();
			errorMessage = null;
			refresh();
		} else {
			runningRows = [];
			stuckRows = [];
			dumps = [];
			resetSelection();
		}
	});

	$effect(() => {
		if (ingest.generation === 0) return;
		refresh();
	});

	// --- linking ---------------------------------------------------------
	/** nearest timestamp in `candidates`, but only within the threshold */
	function associate<T extends { timestamp: number }>(
		target: number,
		candidates: T[],
	): T | null {
		const nearest = nearestByTimestamp(candidates, target);
		if (nearest === null) return null;
		return Math.abs(nearest.timestamp - target) <= threshold.value * 1000
			? nearest
			: null;
	}

	function loadThreads(timestamp: number | null) {
		selectedTid = null;
		trace = { status: "idle" };
		if (timestamp === null) {
			threads = [];
			return;
		}
		cached(`threaddump_threads:${timestamp}`, () => threaddumpThreads(timestamp))
			.then((rows) => {
				if (selectedDump === timestamp) threads = rows;
			})
			.catch((e) => void (errorMessage = String(e)));
	}

	/** every table of the snapshot at once — the right pane shows them together */
	async function loadSnapshot(snap: { source: Source; timestamp: number } | null) {
		stmtView = StmtView.Active;
		if (snap === null) {
			mssqlQueries = [];
			pgsqlQueries = [];
			blocking = [];
			spwho2 = [];
			return;
		}
		const ts = snap.timestamp;
		const running = snap.source === "running";
		const [m, p, b, s] = await Promise.allSettled(
			running
				? [
						cached(`runningquery_mssql_queries:${ts}`, () =>
							runningqueryMssqlQueries(ts),
						),
						cached(`runningquery_pgsql_queries:${ts}`, () =>
							runningqueryPgsqlQueries(ts),
						),
						cached(`runningquery_mssql_blocking:${ts}`, () =>
							runningqueryMssqlBlocking(ts),
						),
						cached(`runningquery_spwho2:${ts}`, () => runningquerySpwho2(ts)),
					]
				: [
						cached(`stuckquery_mssql_queries:${ts}`, () =>
							stuckqueryMssqlQueries(ts),
						),
						cached(`stuckquery_pgsql_queries:${ts}`, () =>
							stuckqueryPgsqlQueries(ts),
						),
						cached(`stuckquery_mssql_blocking:${ts}`, () =>
							stuckqueryMssqlBlocking(ts),
						),
						// stuck queries never log sp_who2
						Promise.resolve([] as SpWho2Row[]),
					],
		);
		// stale guard: the user may have moved on before the rows landed
		if (
			selectedSnap === null ||
			selectedSnap.timestamp !== ts ||
			selectedSnap.source !== snap.source
		)
			return;
		mssqlQueries = settled(m, []) as MssqlQuery[];
		pgsqlQueries = settled(p, []) as PgsqlQuery[];
		blocking = settled(b, []) as MssqlBlockingRow[];
		spwho2 = settled(s, []) as SpWho2Row[];
	}

	// Symmetric: clicking either list selects there and links the other side.
	function onselectsnap(row: SnapshotRow) {
		selectedSnap = { source: row.kind as Source, timestamp: row.timestamp };
		loadSnapshot(selectedSnap);
		selectedDump = associate(row.timestamp, dumps)?.timestamp ?? null;
		loadThreads(selectedDump);
	}

	function onselectdump(timestamp: number) {
		selectedDump = timestamp;
		loadThreads(timestamp);
		const near = associate(timestamp, rows);
		selectedSnap =
			near === null ? null : { source: near.kind as Source, timestamp: near.timestamp };
		loadSnapshot(selectedSnap);
	}

	async function onselectthread(tid: number) {
		if (selectedDump === null) return;
		const dump = selectedDump;
		selectedTid = tid;
		trace = { status: "loading", tid, timestamp: dump };
		try {
			const elements = await cached(`threaddump_trace:${tid}:${dump}`, () =>
				threaddumpTrace(tid, dump),
			);
			if (selectedTid !== tid || selectedDump !== dump) return;
			trace = { status: "ready", tid, timestamp: dump, elements };
		} catch (e) {
			trace = { status: "error", message: String(e) };
		}
	}

	const isPgsql = $derived(pgsqlQueries.length > 0 && mssqlQueries.length === 0);
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

	<div class="toolbar">
		<label class="threshold">
			link threshold
			<input
				type="number"
				min="1"
				max="600"
				bind:value={threshold.value}
				title="Max seconds between a snapshot and a dump to link them (applies to the next click)"
			/>
			s
		</label>
		<label
			class="check"
			title="Also offer stuck-query snapshots (valve trigger) as link candidates"
		>
			<input type="checkbox" bind:checked={includeStuck.value} />
			include stuck queries
			<span class="mono muted">{stuckRows.length}</span>
		</label>
		{#if linkDelta !== null}
			<span class="delta mono">
				linked · dump is {linkDelta >= 0 ? "+" : ""}{linkDelta.toFixed(1)}s
				from the {selectedSnap?.source === "stuck" ? "stuck snapshot" : "tick"}
			</span>
		{:else if selectedSnap !== null || selectedDump !== null}
			<span class="delta muted"
				>nothing within {threshold.value}s on the other side</span
			>
		{/if}
	</div>

	<div class="content">
		<SplitPane direction="row" initial={0.28}>
			{#snippet a()}
				<SplitPane direction="column" initial={0.5}>
					{#snippet a()}
						<div class="pane-block">
							<h3>Database snapshots</h3>
							<SnapshotList {rows} selected={selectedKey} onselect={onselectsnap} />
						</div>
					{/snippet}
					{#snippet b()}
						<div class="pane-block">
							<h3>Thread dumps</h3>
							<ThreadDumpList
								{dumps}
								selected={selectedDump}
								onselect={onselectdump}
							/>
						</div>
					{/snippet}
				</SplitPane>
			{/snippet}
			{#snippet b()}
				<SplitPane direction="row" initial={0.5}>
					{#snippet a()}
						<div class="pane-block">
							<h3>Threads</h3>
							<SplitPane direction="column" initial={0.6}>
								{#snippet a()}
									<ThreadDumpTable
										{threads}
										selected={selectedTid}
										onselect={onselectthread}
									/>
								{/snippet}
								{#snippet b()}
									<ThreadDumpTrace {trace} />
								{/snippet}
							</SplitPane>
						</div>
					{/snippet}
					{#snippet b()}
						{#if selectedSnap === null}
							<div class="pane-block">
								<h3>Queries</h3>
								<p class="empty">
									{selectedDump === null
										? "Select a snapshot or a thread dump."
										: "No database snapshot within the link threshold."}
								</p>
							</div>
						{:else}
							<!-- statements over sp_who2 when the tick logged one; the
							     statements alone otherwise (pgsql, or a stuck snapshot) -->
							{#snippet statements()}
								<div class="pane-block">
									<h3>
										{selectedSnap?.source === "stuck" ? "Stuck queries" : "Running queries"}
										{#if blocking.length > 0}
											<span class="toggle" role="group" aria-label="Table">
												<button
													class:active={stmtView === StmtView.Active}
													onclick={() => (stmtView = StmtView.Active)}
													>Active</button
												>
												<button
													class:active={stmtView === StmtView.Blocking}
													onclick={() => (stmtView = StmtView.Blocking)}
													>Blocking <span class="n">{blocking.length}</span></button
												>
											</span>
										{/if}
									</h3>
									{#if stmtView === StmtView.Blocking}
										<BlockingTree rows={blocking} />
									{:else if isPgsql}
										<PgsqlQueryTable queries={pgsqlQueries} />
									{:else}
										<MssqlQueryTable queries={mssqlQueries} />
									{/if}
								</div>
							{/snippet}
							{#if spwho2.length > 0}
								<SplitPane direction="column" initial={0.55}>
									{#snippet a()}
										{@render statements()}
									{/snippet}
									{#snippet b()}
										<div class="pane-block">
											<h3>sp_who2 <span class="n mono">{spwho2.length} sessions</span></h3>
											<SpWho2Table rows={spwho2} />
										</div>
									{/snippet}
								</SplitPane>
							{:else}
								{@render statements()}
							{/if}
						{/if}
					{/snippet}
				</SplitPane>
			{/snippet}
		</SplitPane>
	</div>
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

	.toolbar {
		display: flex;
		align-items: center;
		gap: 16px;
		padding: 6px 12px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.threshold,
	.check {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.check input {
		margin: 0;
	}
	.threshold input {
		width: 64px;
		padding: 3px 6px;
		font-family: var(--font-mono);
		font-size: 12px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		text-align: right;
	}
	.delta {
		font-size: 12px;
		color: var(--green);
	}
	.delta.muted,
	.muted {
		color: var(--fg-muted);
	}
	.mono {
		font-family: var(--font-mono);
	}

	.content {
		flex: 1;
		min-height: 0;
	}

	.pane-block {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.pane-block h3 {
		display: flex;
		align-items: center;
		gap: 12px;
		margin: 0;
		padding: 6px 10px;
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--fg-muted);
		background: var(--bg-hard);
		flex-shrink: 0;
	}
	.pane-block h3 .n {
		font-weight: 400;
		text-transform: none;
		letter-spacing: 0;
	}
	/* the h3 is a flex column header; the table below must be able to shrink */
	.pane-block > :global(:not(h3)) {
		flex: 1;
		min-height: 0;
	}

	/* Segmented pill: an inset track with a filled active segment. */
	.toggle {
		display: flex;
		gap: 2px;
		margin-left: auto;
		padding: 2px;
		background: var(--bg-soft);
		border-radius: 999px;
		text-transform: none;
		letter-spacing: 0;
	}
	.toggle button {
		padding: 1px 10px;
		font-size: 11px;
		border-radius: 999px;
		color: var(--fg-muted);
	}
	.toggle button:hover {
		color: var(--fg);
	}
	.toggle button.active {
		background: var(--accent);
		color: var(--bg-hard);
		font-weight: 600;
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
</style>
