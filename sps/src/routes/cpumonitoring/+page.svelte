<script lang="ts">
	/**
	 * CPU monitoring analyzer — dump-centric drill-down:
	 *
	 *   dumps (left) → one dump's threads (middle) → clicked thread's
	 *   stack trace + its usage across ALL dumps (right)
	 *
	 * The page owns all server state and the selection path
	 * (dump timestamp → tid); components are presentational.
	 *
	 * Fetch philosophy: user-action fetches live in the event handlers that
	 * cause them; the $effects only react to state owned elsewhere (database
	 * opened, new data ingested from the topbar/Ingest page).
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { nearestByTimestamp } from "$lib/nearest";
	import {
		cpuDumps,
		cpuDumpThreads,
		cpuSeries,
		cpuStacktrace,
		type DumpSummary,
		type DumpThread,
	} from "$lib/api/cpumonitoring";
	import { cached } from "$lib/query-cache";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import DumpList from "$lib/components/DumpList.svelte";
	import ThreadTable from "$lib/components/ThreadTable.svelte";
	import LineChart, {
		type LineSeries,
	} from "$lib/components/LineChart.svelte";
	import StackTracePanel, {
		type TraceState,
	} from "$lib/components/StackTracePanel.svelte";

	// --- state: one straight line, mirroring the drill-down ---------------
	let errorMessage = $state<string | null>(null);

	let dumps = $state<DumpSummary[]>([]);
	let selectedDump = $state<number | null>(null);
	let threads = $state<DumpThread[]>([]);
	let selectedTid = $state<number | null>(null);
	let series = $state<LineSeries[]>([]);
	let chartMaximized = $state(false);
	let trace = $state<TraceState>({ status: "idle" });

	// cpumonitoring is logged as a singleton dump, but a dump's threads can
	// carry slightly different millisecond timestamps, so the backend's
	// GROUP BY timestamp fragments one logical dump into several adjacent
	// rows. Re-group here: consecutive dumps within 2s are the same dump.
	// (Ordering is ascending — cpu_dumps ORDER BY timestamp.)
	const GROUP_WINDOW_MS = 2000;

	interface DumpGroup extends DumpSummary {
		/** raw dump timestamps merged into this group, ascending */
		members: number[];
	}

	const groupedDumps = $derived.by<DumpGroup[]>(() => {
		const groups: DumpGroup[] = [];
		for (const d of dumps) {
			const last = groups.at(-1);
			const prev = last?.members.at(-1);
			if (
				last &&
				prev !== undefined &&
				d.timestamp - prev <= GROUP_WINDOW_MS
			) {
				// distinct threads fragmented across adjacent millis: sum the
				// counts and load (no double-count), keep the peak; the
				// earliest timestamp is the group's identity.
				last.members.push(d.timestamp);
				last.threads += d.threads;
				last.totalCpu += d.totalCpu;
				last.maxCpu = Math.max(last.maxCpu, d.maxCpu);
			} else {
				groups.push({
					timestamp: d.timestamp,
					threads: d.threads,
					totalCpu: d.totalCpu,
					maxCpu: d.maxCpu,
					members: [d.timestamp],
				});
			}
		}
		return groups;
	});

	function resetBelow(level: "dump" | "thread") {
		if (level === "dump") {
			selectedDump = null;
			threads = [];
		}
		selectedTid = null;
		series = [];
		trace = { status: "idle" };
	}

	async function refreshDumps() {
		dumps = await cached("cpu_dumps", cpuDumps);
	}

	// Database changed (topbar): drop the whole drill-down, load fresh.
	$effect(() => {
		if (db.state.status === "open") {
			resetBelow("dump");
			errorMessage = null;
			refreshDumps().catch((e) => (errorMessage = String(e)));
		} else {
			dumps = [];
			resetBelow("dump");
		}
	});

	// New data ingested: the dump list grows; the current selection stays
	// valid (dumps are immutable once parsed), so only refresh the list.
	$effect(() => {
		if (ingest.generation === 0) return;
		refreshDumps().catch((e) => (errorMessage = String(e)));
	});

	// Cross-analyzer link: another page navigated here with ?t=<ms>. The
	// effect waits until dumps are loaded, then selects the NEAREST dump
	// (the two analyzers' timestamps never match exactly). Consumed once —
	// the user's later clicks must not be overridden by the URL.
	let linkConsumed = $state(false);
	$effect(() => {
		if (linkConsumed) return;
		const raw = page.url.searchParams.get("t");
		if (raw === null) {
			linkConsumed = true;
			return;
		}
		const target = Number(raw);
		if (!Number.isFinite(target) || groupedDumps.length === 0) return; // wait for data
		const nearest = nearestByTimestamp(groupedDumps, target);
		if (nearest === null) return;
		linkConsumed = true;
		onselectdump(nearest.timestamp);
	});

	// --- drill-down actions ------------------------------------------------
	async function onselectdump(timestamp: number) {
		selectedDump = timestamp;
		resetBelow("thread");
		// The row is a merged group; fetch every fragment's threads and union
		// them back into the one dump the log actually captured.
		const group = groupedDumps.find((g) => g.timestamp === timestamp);
		const members = group?.members ?? [timestamp];
		try {
			const perFragment = await Promise.all(
				members.map((ts) =>
					cached(`cpu_dump_threads:${ts}`, () => cpuDumpThreads(ts)),
				),
			);
			// a tid lives in exactly one fragment; dedupe defensively and
			// restore the cpu-descending order the table expects.
			const byTid = new Map<number, DumpThread>();
			for (const list of perFragment)
				for (const t of list) if (!byTid.has(t.tid)) byTid.set(t.tid, t);
			threads = [...byTid.values()].sort((a, b) => b.cpu - a.cpu);
		} catch (e) {
			errorMessage = String(e);
		}
	}

	// Clicking a thread answers two questions at once: "what was it doing in
	// THIS dump?" (trace) and "was it always this hot?" (series, all dumps).
	async function onselectthread(tid: number) {
		if (selectedDump === null) return;
		selectedTid = tid;
		trace = { status: "loading", tid, timestamp: selectedDump };

		// allSettled, not all: the two queries are independent, so one failing
		// must not discard the other's result (all() rejects wholesale). The
		// trace is the primary answer; the chart is enrichment — its failure
		// degrades to an empty chart, logged instead of shown in the error bar
		// (cpu_series isn't implemented backend-side yet).
		const dump = selectedDump;
		const [traceResult, seriesResult] = await Promise.allSettled([
			cached(`cpu_stacktrace:${tid}:${dump}`, () =>
				cpuStacktrace(tid, dump),
			),
			cached(`cpu_series:${tid}`, () => cpuSeries(tid)),
		]);

		trace =
			traceResult.status === "fulfilled"
				? {
						status: "ready",
						tid,
						timestamp: selectedDump,
						frames: traceResult.value,
					}
				: { status: "error", message: String(traceResult.reason) };

		if (seriesResult.status === "fulfilled") {
			// Compose the chart's view type: API sends bare cpu points, we know
			// the identity and label.
			series = [
				{
					id: tid,
					label: `tid ${tid}`,
					points: seriesResult.value.map((p) => ({
						timestamp: p.timestamp,
						value: p.cpu,
					})),
				},
			];
		} else {
			series = [];
			console.warn("cpu_series failed:", seriesResult.reason);
		}
	}

	// Clicking a point on the chart = "show me the trace at THAT dump".
	async function onpointselect(tid: number, timestamp: number) {
		trace = { status: "loading", tid, timestamp };
		try {
			const frames = await cached(
				`cpu_stacktrace:${tid}:${timestamp}`,
				() => cpuStacktrace(tid, timestamp),
			);
			trace = { status: "ready", tid, timestamp, frames };
		} catch (e) {
			trace = { status: "error", message: String(e) };
		}
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

	<div class="toolbar">
		{#if selectedDump !== null}
			<button
				class="cross-link"
				onclick={() => goto(`/cpumemstats?t=${selectedDump}`)}
				title="Open CPU/Mem Statistics at the nearest dump"
				>processes at this time →</button
			>
		{:else}
			<span class="hint">select a dump to inspect its threads</span>
		{/if}
	</div>

	<div class="content">
		{#if chartMaximized}
			<LineChart
				{series}
				unit="%"
				emptyText="Click a thread to plot its CPU usage across dumps."
				{onpointselect}
				expanded
				ontoggleexpand={() => (chartMaximized = false)}
			/>
		{:else}
			<SplitPane direction="row" initial={0.2}>
				{#snippet a()}
					<DumpList
						dumps={groupedDumps}
						selected={selectedDump}
						onselect={onselectdump}
					/>
				{/snippet}
				{#snippet b()}
					<SplitPane direction="row" initial={0.45}>
						{#snippet a()}
							<ThreadTable
								{threads}
								selected={selectedTid}
								onselect={onselectthread}
							/>
						{/snippet}
						{#snippet b()}
							<SplitPane direction="column" initial={0.55}>
								{#snippet a()}
									<LineChart
										{series}
										unit="%"
										emptyText="Click a thread to plot its CPU usage across dumps."
										{onpointselect}
										ontoggleexpand={() =>
											(chartMaximized = true)}
									/>
								{/snippet}
								{#snippet b()}
									<StackTracePanel {trace} />
								{/snippet}
							</SplitPane>
						{/snippet}
					</SplitPane>
				{/snippet}
			</SplitPane>
		{/if}
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
		justify-content: flex-end;
		align-items: center;
		padding: 6px 12px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
		min-height: 37px;
	}

	.hint {
		font-size: 12px;
		color: var(--fg-muted);
	}

	.cross-link {
		padding: 4px 12px;
		font-size: 12px;
		background: none;
		border: none;
		border-radius: var(--radius);
		color: var(--fg-muted);
		cursor: pointer;
	}
	.cross-link:hover {
		background: var(--bg-hover);
		color: var(--fg);
	}

	.content {
		flex: 1;
		min-height: 0;
	}
</style>
