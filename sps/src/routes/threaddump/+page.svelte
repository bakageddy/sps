<script lang="ts">
	/**
	 * Thread Dumps analyzer — CPUMonitoring anatomy:
	 *
	 *   dumps (left) → one dump's full thread census (middle) → the clicked
	 *   thread's stack with its lock lines in place (right)
	 *
	 * A "Locks" mode swaps the middle+right for the lock analysis panel
	 * (deadlock cycles + hot lock owners), derived frontend-side from the
	 * census rows — selecting a thread there drops you back into the census
	 * on that thread.
	 *
	 * Pure threaddump on purpose: cross-log decoration (cpu, connection
	 * holds) lives in the connectiondump incident view, not here.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { nearestByTimestamp } from "$lib/nearest";
	import {
		threaddumpDumps,
		threaddumpThreads,
		threaddumpTrace,
		threaddumpThreadSeries,
		type ThreadDumpSummary,
		type ThreadDumpThread,
		type ThreadDumpPoint,
	} from "$lib/api/threaddump";
	import { cached } from "$lib/query-cache";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import ThreadDumpList from "$lib/components/ThreadDumpList.svelte";
	import ThreadDumpTable from "$lib/components/ThreadDumpTable.svelte";
	import ThreadDumpTrace, {
		type ThreadTraceState,
	} from "$lib/components/ThreadDumpTrace.svelte";
	import DeadlockPanel from "$lib/components/DeadlockPanel.svelte";
	import ThreadTimeline from "$lib/components/ThreadTimeline.svelte";

	let errorMessage = $state<string | null>(null);
	let dumps = $state<ThreadDumpSummary[]>([]);
	let selectedDump = $state<number | null>(null);
	let threads = $state<ThreadDumpThread[]>([]);
	let selectedTid = $state<number | null>(null);
	let trace = $state<ThreadTraceState>({ status: "idle" });
	/** the selected thread across ALL dumps (cpumonitoring's series role) */
	let series = $state<ThreadDumpPoint[]>([]);
	/** which dump's stack the trace panel shows — a timeline click moves it
	 *  without changing the selected dump, exactly like a chart point click */
	let traceAt = $state<number | null>(null);

	const Mode = { Threads: "threads", Locks: "locks" } as const;
	type Mode = (typeof Mode)[keyof typeof Mode];
	let mode = $state<Mode>(Mode.Threads);

	function resetBelow(level: "dump" | "thread") {
		if (level === "dump") {
			selectedDump = null;
			threads = [];
		}
		selectedTid = null;
		trace = { status: "idle" };
		series = [];
		traceAt = null;
	}

	async function refreshDumps() {
		dumps = await cached("threaddump_dumps", threaddumpDumps);
	}

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

	$effect(() => {
		if (ingest.generation === 0) return;
		refreshDumps().catch((e) => (errorMessage = String(e)));
	});

	// Cross-analyzer link (?t=<ms>): select the nearest dump once data is in.
	let linkConsumed = $state(false);
	$effect(() => {
		if (linkConsumed) return;
		const raw = page.url.searchParams.get("t");
		if (raw === null) {
			linkConsumed = true;
			return;
		}
		const target = Number(raw);
		if (!Number.isFinite(target) || dumps.length === 0) return;
		const nearest = nearestByTimestamp(dumps, target);
		if (nearest === null) return;
		linkConsumed = true;
		onselectdump(nearest.timestamp);
	});

	async function onselectdump(timestamp: number) {
		selectedDump = timestamp;
		resetBelow("thread");
		try {
			const result = await cached(`threaddump_threads:${timestamp}`, () =>
				threaddumpThreads(timestamp),
			);
			if (selectedDump !== timestamp) return; // stale guard
			threads = result;
		} catch (e) {
			errorMessage = String(e);
		}
	}

	/** load one (tid, dump) stack into the trace panel, stale-guarded on tid */
	async function loadTrace(tid: number, dump: number) {
		traceAt = dump;
		trace = { status: "loading", tid, timestamp: dump };
		try {
			const elements = await cached(`threaddump_trace:${tid}:${dump}`, () =>
				threaddumpTrace(tid, dump),
			);
			if (selectedTid !== tid || traceAt !== dump) return;
			trace = { status: "ready", tid, timestamp: dump, elements };
		} catch (e) {
			trace = { status: "error", message: String(e) };
		}
	}

	// Clicking a thread answers two questions at once (cpumonitoring's
	// pattern): "what was it doing in THIS dump?" (trace) and "what did it
	// do across ALL dumps?" (series → timeline). allSettled: the series is
	// enrichment; its failure degrades to an empty strip, logged not shown.
	async function onselectthread(tid: number) {
		if (selectedDump === null) return;
		mode = Mode.Threads; // a Locks-panel click lands here
		selectedTid = tid;
		series = [];
		const [, seriesResult] = await Promise.allSettled([
			loadTrace(tid, selectedDump),
			cached(`threaddump_thread_series:${tid}`, () =>
				threaddumpThreadSeries(tid),
			),
		]);
		if (selectedTid !== tid) return;
		if (seriesResult.status === "fulfilled") series = seriesResult.value;
		else console.warn("threaddump_thread_series failed:", seriesResult.reason);
	}

	/** timeline click = "show me this thread's stack at THAT dump" */
	function onselectpoint(timestamp: number) {
		if (selectedTid === null) return;
		loadTrace(selectedTid, timestamp);
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
		<span class="chips" role="group" aria-label="View">
			<button
				class:active={mode === Mode.Threads}
				onclick={() => (mode = Mode.Threads)}>Threads</button
			>
			<button
				class:active={mode === Mode.Locks}
				onclick={() => (mode = Mode.Locks)}
				disabled={selectedDump === null}>Locks</button
			>
		</span>
		<span class="right">
			{#if selectedDump !== null}
				<button
					class="cross-link"
					onclick={() => goto(`/cpumonitoring?t=${selectedDump}`)}
					title="Open CPU Monitoring at the nearest dump"
					>cpu at this time →</button
				>
				<button
					class="cross-link"
					onclick={() => goto(`/connectiondump/incident?t=${selectedDump}`)}
					title="Open the incident view around this moment"
					>incident →</button
				>
			{:else}
				<span class="hint">select a dump to inspect its threads</span>
			{/if}
		</span>
	</div>

	<div class="content">
		<SplitPane direction="row" initial={0.22}>
			{#snippet a()}
				<ThreadDumpList
					{dumps}
					selected={selectedDump}
					onselect={onselectdump}
				/>
			{/snippet}
			{#snippet b()}
				{#if mode === Mode.Locks}
					<DeadlockPanel {threads} onselect={onselectthread} />
				{:else}
					<SplitPane direction="row" initial={0.5}>
						{#snippet a()}
							<ThreadDumpTable
								{threads}
								selected={selectedTid}
								onselect={onselectthread}
							/>
						{/snippet}
						{#snippet b()}
							<SplitPane direction="column" initial={0.3}>
								{#snippet a()}
									<ThreadTimeline
										{dumps}
										points={series}
										selected={traceAt}
										onselect={onselectpoint}
									/>
								{/snippet}
								{#snippet b()}
									<ThreadDumpTrace {trace} />
								{/snippet}
							</SplitPane>
						{/snippet}
					</SplitPane>
				{/if}
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
		justify-content: space-between;
		gap: 12px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
		min-height: 37px;
	}
	.right {
		display: flex;
		align-items: center;
		gap: 8px;
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
	.chips button:hover:not(:disabled) {
		color: var(--fg);
	}
	.chips button.active {
		background: var(--accent);
		color: var(--bg-hard);
		font-weight: 600;
	}
	.chips button:disabled {
		opacity: 0.5;
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
