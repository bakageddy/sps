<script lang="ts">
	/**
	 * Incident view — connectiondump as the correlation hub.
	 *
	 * Left: the signals timeline (every trigger — cause + time). Selecting
	 * one resolves the incident in two waves:
	 *   1. the backend answers, per subsystem, WHICH dump was taken within
	 *      the tolerance of the signal (connectiondump_<subsystem>, each an
	 *      Option<u64> anchor); connectiondump's own trace anchor comes
	 *      from the snapshots already loaded;
	 *   2. the existing per-timestamp commands fetch each anchor's rows,
	 *      and lib/connectiondump.buildCensus joins threaddump ⋈ cpu ⋈
	 *      holds by exact tid.
	 *
	 * Stuck threads and stuck queries are deliberately absent: both ride
	 * the VALVE trigger, not the performance-dump trigger, so they never
	 * co-occur with a signal (/stuckthreads/queries is their matcher).
	 * Running queries DO join: they are periodic, so there is always a
	 * tick near a signal — the Queries panel shows that tick's tables.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { pushSelection } from "$lib/navhistory.svelte";
	import {
		connectiondumpSignals,
		connectiondumpSnapshots,
		connectiondumpTraces,
		connectiondumpThreaddump,
		connectiondumpCpumonitoring,
		connectiondumpCpumemstats,
		connectiondumpRunningqueries,
		INCIDENT_TOLERANCE_MS,
		type ConnDumpSignal,
	} from "$lib/api/connectiondump";
	import { threaddumpThreads, threaddumpTrace } from "$lib/api/threaddump";
	import { cpuDumpThreads } from "$lib/api/cpumonitoring";
	import { cpuMemCpuProcesses } from "$lib/api/cpumemstats";
	import {
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
		buildCensus,
		causeColor,
		latestWithin,
		type IncidentThread,
	} from "$lib/connectiondump";
	import { nearestByTimestamp } from "$lib/nearest";
	import { formatTimestamp } from "$lib/format";
	import { cached } from "$lib/query-cache";
	import { persisted } from "$lib/persisted.svelte";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import Icon from "$lib/components/Icon.svelte";
	import SnapshotList, {
		snapshotKey,
		type SnapshotRow,
	} from "$lib/components/SnapshotList.svelte";
	import IncidentThreadTable from "$lib/components/IncidentThreadTable.svelte";
	import ThreadDumpTrace, {
		type ThreadTraceState,
	} from "$lib/components/ThreadDumpTrace.svelte";
	import ProcessTable, {
		type UsageRow,
	} from "$lib/components/ProcessTable.svelte";
	import { buildIncidentReport, reportStackThreads } from "$lib/incident-report";
	import { cpuStacktrace } from "$lib/api/cpumonitoring";
	import { copyText } from "$lib/clipboard";
	import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
	import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
	import BlockingTree from "$lib/components/BlockingTree.svelte";
	import SpWho2Table from "$lib/components/SpWho2Table.svelte";

	let errorMessage = $state<string | null>(null);
	let signals = $state<ConnDumpSignal[]>([]);
	let selected = $state<ConnDumpSignal | null>(null);

	/** which dump of each subsystem is "this incident"; null = none in window */
	interface Anchors {
		threaddump: number | null;
		cpumonitoring: number | null;
		cpumemstats: number | null;
		/** connectiondump's own trace dump (holders) */
		traces: number | null;
		/** running-query tick (periodic log, so normally always present) */
		runningqueries: number | null;
	}
	let anchors = $state<Anchors | null>(null);
	let census = $state<IncidentThread[]>([]);
	let processes = $state<UsageRow[]>([]);

	// the running-query tick's tables; a bundle logs ONE flavor, so either
	// the pgsql list or the three mssql ones are populated
	let rqMssql = $state<MssqlQuery[]>([]);
	let rqPgsql = $state<PgsqlQuery[]>([]);
	let rqBlocking = $state<MssqlBlockingRow[]>([]);
	let rqSpwho2 = $state<SpWho2Row[]>([]);
	const QueryView = {
		Active: "active",
		Blocking: "blocking",
		Sessions: "sessions",
	} as const;
	type QueryView = (typeof QueryView)[keyof typeof QueryView];
	let queryView = $state<QueryView>(QueryView.Active);

	let selectedTid = $state<number | null>(null);
	let trace = $state<ThreadTraceState>({ status: "idle" });

	const Mode = {
		Threads: "threads",
		Processes: "processes",
		Queries: "queries",
	} as const;
	type Mode = (typeof Mode)[keyof typeof Mode];
	let mode = $state<Mode>(Mode.Threads);

	// The incident window, in SECONDS for the input; sent to the backend in
	// ms on every resolve. Persisted: it's a property of the bundle's clock
	// drift, and you tune it once per investigation.
	const tolerance = persisted(
		"incident-tolerance-s",
		INCIDENT_TOLERANCE_MS / 1000,
	);
	const toleranceMs = $derived(Math.round(tolerance.value * 1000));

	// "Skipping to dump" signals hidden by default — they're the bulk of the
	// timeline (tens of thousands) and each is "the last incident is still
	// going", not a new one. Same persisted key as the connectiondump page.
	const showSuppressed = persisted("conndump-show-suppressed", false);
	const visibleSignals = $derived(
		signals.filter((s) => showSuppressed.value || !s.suppressed),
	);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	const rows = $derived<SnapshotRow[]>(
		visibleSignals.map((s) => ({
			timestamp: s.timestamp,
			kind: "signal" as const,
			detail: s.cause + (s.suppressed ? " · suppressed" : ""),
			// pool exhaustion is the incident worth red
			alert: s.cause === "No ManagedConnections",
		})),
	);

	async function refresh() {
		// the include flag is part of the cache key, so flipping the toggle
		// is a distinct (and cached) fetch
		const include = showSuppressed.value;
		try {
			signals = await cached(`connectiondump_signals|${include}`, () =>
				connectiondumpSignals(include),
			);
		} catch (e) {
			errorMessage = String(e);
		}
	}

	function resetIncident() {
		anchors = null;
		census = [];
		processes = [];
		rqMssql = [];
		rqPgsql = [];
		rqBlocking = [];
		rqSpwho2 = [];
		selectedTid = null;
		trace = { status: "idle" };
	}

	function onselect(row: SnapshotRow) {
		// the selection is a history entry: back/forward walk the incidents.
		// Mark it consumed first so the ?t= handler below doesn't echo the
		// selection back when the URL changes.
		consumedLink = String(row.timestamp);
		pushSelection(row.timestamp);
		const sig = signals.find((s) => s.timestamp === row.timestamp);
		if (sig) selected = sig;
	}

	// Resolution is DERIVED from (selected signal, tolerance): changing
	// either re-resolves, so widening the window re-anchors the incident
	// live. The effect only reads those two; everything it writes is
	// downstream state, so it can't loop.
	$effect(() => {
		const sig = selected;
		const tol = toleranceMs;
		if (sig === null) return;
		resolve(sig.timestamp, tol);
	});

	/** unwrap an allSettled slot; a rejection surfaces once and yields the fallback */
	function settled<T>(r: PromiseSettledResult<T>, fallback: T): T {
		if (r.status === "fulfilled") return r.value;
		errorMessage = String(r.reason);
		return fallback;
	}

	async function resolve(ts: number, tol: number) {
		resetIncident();
		// stale = the user moved on (other signal OR other window) while we
		// were in flight; either invalidates this resolution
		const stale = () =>
			selected === null || selected.timestamp !== ts || toleranceMs !== tol;

		// wave 1: anchors
		const [tdR, cpuR, memR, rqR, snapsR] = await Promise.allSettled([
			cached(`connectiondump_threaddump:${ts}:${tol}`, () =>
				connectiondumpThreaddump(ts, tol),
			),
			cached(`connectiondump_cpumonitoring:${ts}:${tol}`, () =>
				connectiondumpCpumonitoring(ts, tol),
			),
			cached(`connectiondump_cpumemstats:${ts}:${tol}`, () =>
				connectiondumpCpumemstats(ts, tol),
			),
			cached(`connectiondump_runningqueries:${ts}:${tol}`, () =>
				connectiondumpRunningqueries(ts, tol),
			),
			cached("connectiondump_snapshots", connectiondumpSnapshots),
		]);
		if (stale()) return;
		const a: Anchors = {
			threaddump: settled(tdR, null),
			cpumonitoring: settled(cpuR, null),
			cpumemstats: settled(memR, null),
			runningqueries: settled(rqR, null),
			traces: latestWithin(settled(snapsR, []), ts, tol)?.timestamp ?? null,
		};
		anchors = a;

		// wave 2: each anchor's rows through the existing commands. The
		// running-query tick fetches all four flavors — three of them come
		// back empty for a bundle's other database, which is cheap.
		const rq = a.runningqueries;
		const [
			threadsR,
			cpuThreadsR,
			tracesR,
			procR,
			rqMssqlR,
			rqPgsqlR,
			rqBlockingR,
			rqSpwho2R,
		] = await Promise.allSettled([
			a.threaddump === null
				? Promise.resolve([])
				: cached(`threaddump_threads:${a.threaddump}`, () =>
						threaddumpThreads(a.threaddump!),
					),
			a.cpumonitoring === null
				? Promise.resolve([])
				: cached(`cpu_dump_threads:${a.cpumonitoring}`, () =>
						cpuDumpThreads(a.cpumonitoring!),
					),
			a.traces === null
				? Promise.resolve([])
				: cached(`connectiondump_traces:${a.traces}`, () =>
						connectiondumpTraces(a.traces!),
					),
			a.cpumemstats === null
				? Promise.resolve([])
				: cached(`cpumem_cpu_processes:${a.cpumemstats}`, () =>
						cpuMemCpuProcesses(a.cpumemstats!),
					),
			rq === null
				? Promise.resolve([])
				: cached(`runningquery_mssql_queries:${rq}`, () =>
						runningqueryMssqlQueries(rq),
					),
			rq === null
				? Promise.resolve([])
				: cached(`runningquery_pgsql_queries:${rq}`, () =>
						runningqueryPgsqlQueries(rq),
					),
			rq === null
				? Promise.resolve([])
				: cached(`runningquery_mssql_blocking:${rq}`, () =>
						runningqueryMssqlBlocking(rq),
					),
			rq === null
				? Promise.resolve([])
				: cached(`runningquery_spwho2:${rq}`, () => runningquerySpwho2(rq)),
		]);
		if (stale()) return;
		census = buildCensus(
			settled(threadsR, []),
			settled(cpuThreadsR, []),
			settled(tracesR, []),
		);
		// ProcessUsage -> the table's view row (drop `user`)
		processes = settled(procR, []).map((p) => ({
			pid: p.pid,
			name: p.name,
			value: p.value,
			path: p.path,
		}));
		rqMssql = settled(rqMssqlR, []);
		rqPgsql = settled(rqPgsqlR, []);
		rqBlocking = settled(rqBlockingR, []);
		rqSpwho2 = settled(rqSpwho2R, []);
	}

	async function onselectthread(tid: number) {
		if (anchors === null || anchors.threaddump === null) return;
		selectedTid = tid;
		const dump = anchors.threaddump;
		trace = { status: "loading", tid, timestamp: dump };
		try {
			const elements = await cached(`threaddump_trace:${tid}:${dump}`, () =>
				threaddumpTrace(tid, dump),
			);
			if (selectedTid !== tid) return;
			trace = { status: "ready", tid, timestamp: dump, elements };
		} catch (e) {
			trace = { status: "error", message: String(e) };
		}
	}

	// the resolved incident as Markdown, for the ticket
	let copiedReport = $state(false);
	let reportBusy = $state(false);

	/** stack lines for one thread: the thread dump's trace (frames and lock
	 *  lines in order), else cpumonitoring's captured stack, else nothing */
	async function stackLines(tid: number, a: Anchors): Promise<string[]> {
		if (a.threaddump !== null) {
			try {
				const els = await cached(`threaddump_trace:${tid}:${a.threaddump}`, () =>
					threaddumpTrace(tid, a.threaddump!),
				);
				if (els && els.length > 0)
					return els.map((e) =>
						e.kind === "frame" ? `at ${e.method}(${e.source})` : `- ${e.object}`,
					);
			} catch {
				/* fall through to cpumonitoring */
			}
		}
		if (a.cpumonitoring !== null) {
			try {
				const frames = await cached(`cpu_stacktrace:${tid}:${a.cpumonitoring}`, () =>
					cpuStacktrace(tid, a.cpumonitoring!),
				);
				if (frames && frames.length > 0)
					return frames.map((f) => `at ${f.method}(${f.source})`);
			} catch {
				/* no stack for this thread */
			}
		}
		return [];
	}

	async function copyReport() {
		if (selected === null || anchors === null || reportBusy) return;
		reportBusy = true;
		try {
			// stacks for the threads the report prints — fetched on demand,
			// in parallel; the census alone never loads them
			const a = anchors;
			const wanted = reportStackThreads(census);
			const fetched = await Promise.all(
				wanted.map(async (t) => [t.tid, await stackLines(t.tid, a)] as const),
			);
			const stacks = new Map(fetched.filter(([, lines]) => lines.length > 0));
			const text = buildIncidentReport({
				signal: selected,
				toleranceMs,
				anchors: a,
				census,
				processes,
				mssql: rqMssql,
				pgsql: rqPgsql,
				blocking: rqBlocking,
				spwho2: rqSpwho2,
				stacks,
			});
			if (await copyText(text)) {
				copiedReport = true;
				setTimeout(() => (copiedReport = false), 1500);
			} else errorMessage = "Clipboard write failed.";
		} finally {
			reportBusy = false;
		}
	}

	$effect(() => {
		if (db.state.status === "open") {
			errorMessage = null;
			selected = null;
			resetIncident();
			refresh();
		} else {
			signals = [];
			selected = null;
			resetIncident();
		}
	});

	$effect(() => {
		if (ingest.generation === 0) return;
		refresh();
	});

	// toggle flipped: re-pull the signals with the new filter
	$effect(() => {
		void showSuppressed.value;
		if (db.state.status !== "open") return;
		refresh();
	});

	// ?t=<ms> from another analyzer: select the nearest signal, once.
	// re-entrant: a NEW ?t= (another analyzer's link, the palette's "jump
	// to time") re-selects; the same value is consumed once
	let consumedLink = $state<string | null>(null);
	$effect(() => {
		const raw = page.url.searchParams.get("t");
		if (raw === null || raw === consumedLink) return;
		const target = Number(raw);
		if (!Number.isFinite(target) || signals.length === 0) return;
		const nearest = nearestByTimestamp(visibleSignals, target);
		if (nearest === null) return;
		consumedLink = raw;
		selected = nearest;
	});

	const selectedKey = $derived(
		selected === null
			? null
			: snapshotKey({
					timestamp: selected.timestamp,
					kind: "signal",
					detail: "",
					alert: false,
				}),
	);
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
			<div class="listcol">
				<label class="toggle" title="Also list the 'Skipping to dump' alarms">
					<input type="checkbox" bind:checked={showSuppressed.value} />
					show suppressed
					<span class="mono muted">{visibleSignals.length}</span>
				</label>
				<div class="listbody">
					<SnapshotList {rows} selected={selectedKey} {onselect} />
				</div>
			</div>
		{/snippet}
		{#snippet b()}
			<div class="content">
				{#if selected === null}
					<p class="empty">Select a signal to resolve its incident.</p>
				{:else}
					<div class="header">
						<span
							class="cause"
							style:background={causeColor(selected.cause)}
							>{selected.cause}</span
						>
						<span class="mono when"
							>{formatTimestamp(timeFormat, selected.timestamp)}</span
						>
						{#if selected.suppressed}
							<span class="muted">dump suppressed</span>
						{/if}
						<span class="anchors">
							{#if anchors !== null}
								{@const links = [
									{ label: "threaddump", ts: anchors.threaddump, href: "/threaddump" },
									{ label: "cpu", ts: anchors.cpumonitoring, href: "/cpumonitoring" },
									{ label: "processes", ts: anchors.cpumemstats, href: "/cpumemstats" },
									{ label: "queries", ts: anchors.runningqueries, href: "/runningqueries" },
									{ label: "holders", ts: anchors.traces, href: null },
								]}
								{#each links as l (l.label)}
									{#if l.ts === null}
										<span class="anchor miss">{l.label} —</span>
									{:else if l.href === null}
										<span
											class="anchor hit"
											title="Trace dump at {formatTimestamp(timeFormat, l.ts)}"
											>{l.label} ✓</span
										>
									{:else}
										<button
											class="anchor hit link"
											onclick={() => goto(`${l.href}?t=${l.ts}`)}
											title="Open {l.label} at {formatTimestamp(timeFormat, l.ts)}"
											>{l.label} ✓ →</button
										>
									{/if}
								{/each}
							{:else}
								<span class="muted">resolving…</span>
							{/if}
						</span>
						<button
							class="report"
							onclick={copyReport}
							disabled={anchors === null || reportBusy}
							title="Copy this incident as Markdown (anchors, blocked/hot threads with stacks, holders, processes, queries)"
							><Icon name={copiedReport ? "check" : "copy"} size={12} />
							{reportBusy ? "report…" : "report"}</button
						>
					</div>

					<div class="toolbar">
						<span class="chips" role="group" aria-label="Panel">
							<button
								class:active={mode === Mode.Threads}
								onclick={() => (mode = Mode.Threads)}
								>Threads <span class="n mono">{census.length}</span></button
							>
							<button
								class:active={mode === Mode.Processes}
								onclick={() => (mode = Mode.Processes)}
								>Processes <span class="n mono">{processes.length}</span
								></button
							>
							<button
								class:active={mode === Mode.Queries}
								onclick={() => (mode = Mode.Queries)}
								>Queries <span class="n mono"
									>{rqMssql.length + rqPgsql.length}</span
								></button
							>
						</span>
						{#if mode === Mode.Queries && rqMssql.length + rqBlocking.length + rqSpwho2.length > 0}
							<!-- mssql logs three tables per tick; pgsql just one -->
							<span class="chips sub" role="group" aria-label="Table">
								<button
									class:active={queryView === QueryView.Active}
									onclick={() => (queryView = QueryView.Active)}
									>Active <span class="n mono">{rqMssql.length}</span></button
								>
								<button
									class:active={queryView === QueryView.Blocking}
									onclick={() => (queryView = QueryView.Blocking)}
									>Blocking <span class="n mono">{rqBlocking.length}</span
									></button
								>
								<button
									class:active={queryView === QueryView.Sessions}
									onclick={() => (queryView = QueryView.Sessions)}
									>sp_who2 <span class="n mono">{rqSpwho2.length}</span
									></button
								>
							</span>
						{/if}
						<label
							class="tolerance"
							title="How far from the signal a subsystem's dump may be to count as this incident"
						>
							window ±
							<input
								type="number"
								min="0.5"
								max="120"
								step="0.5"
								bind:value={tolerance.value}
							/>
							s
						</label>
					</div>

					<div class="body">
						{#if mode === Mode.Threads}
							<SplitPane direction="row" initial={0.55}>
								{#snippet a()}
									<IncidentThreadTable
										threads={census}
										selected={selectedTid}
										onselect={onselectthread}
									/>
								{/snippet}
								{#snippet b()}
									<ThreadDumpTrace {trace} />
								{/snippet}
							</SplitPane>
						{:else if mode === Mode.Processes}
							{#if anchors?.cpumemstats == null}
								<p class="empty">
									No CPU/Mem statistics dump within the incident window.
								</p>
							{:else}
								<ProcessTable
									{processes}
									valueLabel="CPU %"
									selected={null}
									onselect={() =>
										goto(`/cpumemstats?t=${anchors!.cpumemstats}`)}
								/>
							{/if}
						{:else if anchors?.runningqueries == null}
							<p class="empty">
								No running-query tick within the incident window. (Stuck
								queries ride the valve trigger, not this one; see Stuck
								Threads → Queries.)
							</p>
						{:else if rqPgsql.length > 0}
							<PgsqlQueryTable queries={rqPgsql} />
						{:else if queryView === QueryView.Blocking}
							<BlockingTree rows={rqBlocking} />
						{:else if queryView === QueryView.Sessions}
							<SpWho2Table rows={rqSpwho2} />
						{:else}
							<MssqlQueryTable queries={rqMssql} />
						{/if}
					</div>
				{/if}
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
	.listcol {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.listbody {
		flex: 1;
		min-height: 0;
	}
	.toggle {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		font-size: 11.5px;
		color: var(--fg-muted);
		flex-shrink: 0;
	}
	.toggle input {
		margin: 0;
	}
	.toggle .mono {
		margin-left: auto;
		font-size: 11px;
	}
	.header {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 10px;
		padding: 8px 12px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.cause {
		padding: 1px 10px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--bg-hard);
	}
	.when {
		font-size: 12.5px;
		color: var(--fg-strong);
	}
	.mono {
		font-family: var(--font-mono);
	}
	.muted {
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.anchors {
		display: flex;
		gap: 6px;
		margin-left: auto;
	}
	.anchor {
		padding: 1px 10px;
		border-radius: 999px;
		font-size: 11px;
		font-family: var(--font-mono);
	}
	.anchor.hit {
		background: var(--bg-hard);
		color: var(--accent);
	}
	.anchor.link:hover {
		background: var(--bg-hover);
	}
	.anchor.miss {
		color: var(--fg-muted);
		opacity: 0.6;
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
	.tolerance {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.tolerance input {
		width: 56px;
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
	/* second chip group hugs the first; the tolerance knob stays right */
	.chips.sub {
		margin-right: auto;
	}
	.report {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		margin-left: auto;
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
		flex-shrink: 0;
	}
	.report:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	.report:disabled {
		color: var(--fg-muted);
		opacity: 0.6;
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
	.n {
		font-size: 10.5px;
		opacity: 0.8;
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
