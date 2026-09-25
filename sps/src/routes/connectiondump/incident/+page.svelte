<script lang="ts">
	/**
	 * Incident view — connectiondump as the correlation hub.
	 *
	 * Left: the signals timeline (every trigger — cause + time). Selecting
	 * one asks the backend to resolve the incident: which dump of each
	 * subsystem was taken within the tolerance of that signal
	 * (connectiondump_incident), and the threaddump census decorated by tid
	 * with cpumonitoring / connection-hold data (connectiondump_incident_
	 * threads). The process and query panels reuse the existing
	 * per-timestamp commands against the resolved anchors — no new shapes.
	 *
	 * Stuck threads are deliberately absent (own matcher at
	 * /stuckthreads/queries).
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import {
		connectiondumpSignals,
		connectiondumpIncident,
		connectiondumpIncidentThreads,
		type ConnDumpSignal,
		type IncidentAnchors,
		type IncidentThread,
	} from "$lib/api/connectiondump";
	import { threaddumpTrace } from "$lib/api/threaddump";
	import { cpuMemCpuProcesses } from "$lib/api/cpumemstats";
	import {
		stuckqueryMssqlQueries,
		stuckqueryPgsqlQueries,
		stuckqueryMssqlBlocking,
		type MssqlQuery,
		type PgsqlQuery,
		type MssqlBlockingRow,
	} from "$lib/api/stuckquery";
	import { causeColor } from "$lib/connectiondump";
	import { nearestByTimestamp } from "$lib/nearest";
	import { formatTimestamp } from "$lib/format";
	import { cached } from "$lib/query-cache";
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
	import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
	import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
	import BlockingTree from "$lib/components/BlockingTree.svelte";

	let errorMessage = $state<string | null>(null);
	let signals = $state<ConnDumpSignal[]>([]);
	let selected = $state<ConnDumpSignal | null>(null);

	// resolved incident
	let anchors = $state<IncidentAnchors | null>(null);
	let census = $state<IncidentThread[]>([]);
	let processes = $state<UsageRow[]>([]);
	let mssqlQueries = $state<MssqlQuery[]>([]);
	let pgsqlQueries = $state<PgsqlQuery[]>([]);
	let blocking = $state<MssqlBlockingRow[]>([]);

	let selectedTid = $state<number | null>(null);
	let trace = $state<ThreadTraceState>({ status: "idle" });

	const Mode = {
		Threads: "threads",
		Processes: "processes",
		Queries: "queries",
	} as const;
	type Mode = (typeof Mode)[keyof typeof Mode];
	let mode = $state<Mode>(Mode.Threads);

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
		mssqlQueries = [];
		pgsqlQueries = [];
		blocking = [];
		selectedTid = null;
		trace = { status: "idle" };
	}

	async function onselect(row: SnapshotRow) {
		const sig = signals.find((s) => s.timestamp === row.timestamp);
		if (!sig) return;
		selected = sig;
		resetIncident();
		const ts = sig.timestamp;
		const stale = () => selected === null || selected.timestamp !== ts;

		const [anchorsResult, censusResult] = await Promise.allSettled([
			cached(`connectiondump_incident:${ts}`, () =>
				connectiondumpIncident(ts),
			),
			cached(`connectiondump_incident_threads:${ts}`, () =>
				connectiondumpIncidentThreads(ts),
			),
		]);
		if (stale()) return;
		if (censusResult.status === "fulfilled") census = censusResult.value;
		else errorMessage = String(censusResult.reason);
		if (anchorsResult.status !== "fulfilled") {
			errorMessage = String(anchorsResult.reason);
			return;
		}
		anchors = anchorsResult.value;

		// second wave: the panels that ride on existing per-timestamp commands
		const a = anchors;
		const [procResult, mssqlResult, blockingResult, pgsqlResult] =
			await Promise.allSettled([
				a.cpumemstats === null
					? Promise.resolve([])
					: cached(`cpumem_cpu_processes:${a.cpumemstats}`, () =>
							cpuMemCpuProcesses(a.cpumemstats!),
						),
				a.stuckqueryMssql === null
					? Promise.resolve([])
					: cached(`stuckquery_mssql_queries:${a.stuckqueryMssql}`, () =>
							stuckqueryMssqlQueries(a.stuckqueryMssql!),
						),
				a.stuckqueryMssql === null
					? Promise.resolve([])
					: cached(`stuckquery_mssql_blocking:${a.stuckqueryMssql}`, () =>
							stuckqueryMssqlBlocking(a.stuckqueryMssql!),
						),
				a.stuckqueryPgsql === null
					? Promise.resolve([])
					: cached(`stuckquery_pgsql_queries:${a.stuckqueryPgsql}`, () =>
							stuckqueryPgsqlQueries(a.stuckqueryPgsql!),
						),
			]);
		if (stale()) return;
		if (procResult.status === "fulfilled")
			// ProcessUsage -> the table's view row (drop `user`)
			processes = procResult.value.map((p) => ({
				pid: p.pid,
				name: p.name,
				value: p.value,
				path: p.path,
			}));
		if (mssqlResult.status === "fulfilled") mssqlQueries = mssqlResult.value;
		if (blockingResult.status === "fulfilled") blocking = blockingResult.value;
		if (pgsqlResult.status === "fulfilled") pgsqlQueries = pgsqlResult.value;
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
		onselect({ timestamp: nearest.timestamp, kind: "signal", detail: "", alert: false });
	});

	const hasQueries = $derived(
		mssqlQueries.length > 0 || pgsqlQueries.length > 0 || blocking.length > 0,
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
			<SnapshotList
				{rows}
				selected={selected === null
					? null
					: snapshotKey({
							timestamp: selected.timestamp,
							kind: "signal",
							detail: "",
							alert: false,
						})}
				{onselect}
			/>
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
									{ label: "mssql", ts: anchors.stuckqueryMssql, href: "/stuckqueries" },
									{ label: "pgsql", ts: anchors.stuckqueryPgsql, href: "/stuckqueries" },
								]}
								{#each links as l (l.label)}
									{#if l.ts !== null}
										<button
											class="anchor hit"
											onclick={() => goto(`${l.href}?t=${l.ts}`)}
											title="Open {l.label} at {formatTimestamp(timeFormat, l.ts)}"
											>{l.label} ✓</button
										>
									{:else}
										<span class="anchor miss">{l.label} —</span>
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
								onclick={() => (mode = Mode.Queries)}
								>Queries <span class="n mono"
									>{mssqlQueries.length + pgsqlQueries.length}</span
								></button
							>
						</span>
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
								<p class="empty">No CPU/Mem statistics dump within the incident window.</p>
							{:else}
								<ProcessTable
									{processes}
									valueLabel="CPU %"
									selected={null}
									onselect={() =>
										goto(`/cpumemstats?t=${anchors!.cpumemstats}`)}
								/>
							{/if}
						{:else if !hasQueries}
							<p class="empty">No stuck-query snapshot within the incident window.</p>
						{:else}
							<div class="queries">
								{#if blocking.length > 0}
									<BlockingTree rows={blocking} />
								{/if}
								{#if mssqlQueries.length > 0}
									<MssqlQueryTable queries={mssqlQueries} />
								{/if}
								{#if pgsqlQueries.length > 0}
									<PgsqlQueryTable queries={pgsqlQueries} />
								{/if}
							</div>
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
	.anchor.hit:hover {
		background: var(--bg-hover);
	}
	.anchor.miss {
		color: var(--fg-muted);
		opacity: 0.6;
	}
	.toolbar {
		display: flex;
		align-items: center;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
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
	.queries {
		height: 100%;
		overflow: auto;
	}
	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
</style>
