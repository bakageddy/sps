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
	 * Running queries will join the hub once that parser exists.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		connectiondumpSignals,
		connectiondumpSnapshots,
		connectiondumpTraces,
		connectiondumpThreaddump,
		connectiondumpCpumonitoring,
		connectiondumpCpumemstats,
		INCIDENT_TOLERANCE_MS,
		type ConnDumpSignal,
	} from "$lib/api/connectiondump";
	import { threaddumpThreads, threaddumpTrace } from "$lib/api/threaddump";
	import { cpuDumpThreads } from "$lib/api/cpumonitoring";
	import { cpuMemCpuProcesses } from "$lib/api/cpumemstats";
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
	}
	let anchors = $state<Anchors | null>(null);
	let census = $state<IncidentThread[]>([]);
	let processes = $state<UsageRow[]>([]);

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

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	const rows = $derived<SnapshotRow[]>(
		signals.map((s) => ({
			timestamp: s.timestamp,
			kind: "signal" as const,
			detail: s.cause + (s.suppressed ? " · suppressed" : ""),
			// pool exhaustion is the incident worth red
			alert: s.cause === "No ManagedConnections",
		})),
	);

	async function refresh() {
		try {
			signals = await cached("connectiondump_signals", () =>
				connectiondumpSignals(),
			);
		} catch (e) {
			errorMessage = String(e);
		}
	}

	function resetIncident() {
		anchors = null;
		census = [];
		processes = [];
		selectedTid = null;
		trace = { status: "idle" };
	}

	function onselect(row: SnapshotRow) {
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
		const [tdR, cpuR, memR, snapsR] = await Promise.allSettled([
			cached(`connectiondump_threaddump:${ts}:${tol}`, () =>
				connectiondumpThreaddump(ts, tol),
			),
			cached(`connectiondump_cpumonitoring:${ts}:${tol}`, () =>
				connectiondumpCpumonitoring(ts, tol),
			),
			cached(`connectiondump_cpumemstats:${ts}:${tol}`, () =>
				connectiondumpCpumemstats(ts, tol),
			),
			cached("connectiondump_snapshots", connectiondumpSnapshots),
		]);
		if (stale()) return;
		const a: Anchors = {
			threaddump: settled(tdR, null),
			cpumonitoring: settled(cpuR, null),
			cpumemstats: settled(memR, null),
			traces: latestWithin(settled(snapsR, []), ts, tol)?.timestamp ?? null,
		};
		anchors = a;

		// wave 2: each anchor's rows through the existing commands
		const [threadsR, cpuThreadsR, tracesR, procR] = await Promise.allSettled([
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

	// ?t=<ms> from another analyzer: select the nearest signal, once.
	let linkConsumed = $state(false);
	$effect(() => {
		if (linkConsumed) return;
		const raw = page.url.searchParams.get("t");
		if (raw === null) {
			linkConsumed = true;
			return;
		}
		const target = Number(raw);
		if (!Number.isFinite(target) || signals.length === 0) return;
		const nearest = nearestByTimestamp(signals, target);
		if (nearest === null) return;
		linkConsumed = true;
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
			<SnapshotList {rows} selected={selectedKey} {onselect} />
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
								onclick={() => (mode = Mode.Queries)}>Queries</button
							>
						</span>
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
						{:else}
							<p class="empty">
								Running queries at the incident — arrives with the
								running-queries parser. (Stuck queries ride the valve
								trigger, not this one; see Stuck Threads → Queries.)
							</p>
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
