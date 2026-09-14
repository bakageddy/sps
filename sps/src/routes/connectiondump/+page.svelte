<script lang="ts">
	/**
	 * Connection Dumps analyzer — stuckqueries anatomy: dump list on the
	 * left, content on the right with three modes:
	 *
	 *  - Pool: occupancy step chart with the capacity ceiling and the alarm
	 *    timeline (High CPU / High Memory / No ManagedConnections ticks,
	 *    suppressed dumps translucent). used == capacity IS exhaustion.
	 *  - Dump: the selected dump's holder traces (who has a connection and
	 *    the acquisition stack).
	 *  - Holders: hold episodes across dumps — same (thread id, startTime)
	 *    recurring with growing duration is the incident no single dump shows.
	 */
	import {
		connectiondumpSignals,
		connectiondumpPoolStats,
		connectiondumpSnapshots,
		connectiondumpTraces,
		connectiondumpHolders,
		type ConnDumpSignal,
		type ConnDumpPoolStats,
		type ConnDumpSnapshot,
		type ConnDumpTrace,
		type ConnDumpHolder,
	} from "$lib/api/connectiondump";
	import { causeColor } from "$lib/connectiondump";
	import SnapshotList, {
		snapshotKey,
		type SnapshotRow,
	} from "$lib/components/SnapshotList.svelte";
	import PoolChart from "$lib/components/PoolChart.svelte";
	import ConnDumpTraceTable from "$lib/components/ConnDumpTraceTable.svelte";
	import HoldersTable from "$lib/components/HoldersTable.svelte";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { cached } from "$lib/query-cache";
	import { persisted } from "$lib/persisted.svelte";
	import { formatDuration, formatTimestamp } from "$lib/format";

	let errorMessage = $state<string | null>(null);
	let signals = $state<ConnDumpSignal[]>([]);
	let poolStats = $state<ConnDumpPoolStats[]>([]);
	let snapshots = $state<ConnDumpSnapshot[]>([]);
	let holders = $state<ConnDumpHolder[]>([]);
	let selected = $state<SnapshotRow | null>(null);
	let traces = $state<ConnDumpTrace[]>([]);

	const Mode = { Pool: "pool", Dump: "dump", Holders: "holders" } as const;
	type Mode = (typeof Mode)[keyof typeof Mode];
	let mode = $state<Mode>(Mode.Pool);

	// slow-hold threshold (seconds, persisted) — rows above it flag red
	const slowThreshold = persisted("conndump-slow-s", 60);

	/** chart zoom window; null = full range */
	let view = $state<[number, number] | null>(null);

	const rows = $derived.by<SnapshotRow[]>(() =>
		snapshots.map((s) => ({
			timestamp: s.timestamp,
			kind: "dump" as const,
			detail:
				`${s.traceCount} in use · max ${formatDuration(s.maxDuration)}` +
				(s.total > 0 ? ` · pool ${s.used}/${s.total}` : ""),
			// exhaustion (or one hold past the threshold) is worth red
			alert:
				(s.total > 0 && s.used >= s.total) ||
				s.maxDuration > slowThreshold.value * 1000,
		})),
	);

	const domain = $derived.by<[number, number]>(() => {
		let lo = Infinity;
		let hi = -Infinity;
		for (const s of poolStats) {
			lo = Math.min(lo, s.timestamp);
			hi = Math.max(hi, s.timestamp);
		}
		for (const s of signals) {
			lo = Math.min(lo, s.timestamp);
			hi = Math.max(hi, s.timestamp);
		}
		if (lo === Infinity) return [0, 1];
		return hi > lo ? [lo, hi] : [lo, lo + 1];
	});

	const window_ = $derived(view ?? domain);

	// causes present in the data, for the legend
	const causes = $derived([...new Set(signals.map((s) => s.cause))]);

	// time axis under the chart (same policy as the concurrency page)
	const ticks = $derived.by(() => {
		const n = 8;
		const [lo, hi] = window_;
		return Array.from({ length: n + 1 }, (_, i) => ({
			pct: (i / n) * 100,
			t: lo + ((hi - lo) * i) / n,
		}));
	});
	const tickFormat = $derived.by(() => {
		const span = window_[1] - window_[0];
		if (span > 86_400_000)
			return new Intl.DateTimeFormat(undefined, {
				month: "short",
				day: "numeric",
				hour: "2-digit",
				minute: "2-digit",
				hourCycle: "h23",
			});
		if (span > 3_600_000)
			return new Intl.DateTimeFormat(undefined, {
				hour: "2-digit",
				minute: "2-digit",
				hourCycle: "h23",
			});
		return new Intl.DateTimeFormat(undefined, {
			hour: "2-digit",
			minute: "2-digit",
			second: "2-digit",
			hourCycle: "h23",
		});
	});

	async function refresh() {
		const [signalsResult, statsResult, snapshotsResult, holdersResult] =
			await Promise.allSettled([
				cached("connectiondump_signals", () => connectiondumpSignals()),
				cached("connectiondump_pool_stats", () =>
					connectiondumpPoolStats(),
				),
				cached("connectiondump_snapshots", connectiondumpSnapshots),
				cached("connectiondump_holders", () => connectiondumpHolders()),
			]);
		if (signalsResult.status === "fulfilled") signals = signalsResult.value;
		else errorMessage = String(signalsResult.reason);
		if (statsResult.status === "fulfilled") poolStats = statsResult.value;
		else errorMessage = String(statsResult.reason);
		if (snapshotsResult.status === "fulfilled")
			snapshots = snapshotsResult.value;
		else errorMessage = String(snapshotsResult.reason);
		if (holdersResult.status === "fulfilled") holders = holdersResult.value;
		else errorMessage = String(holdersResult.reason);

		if (selected === null && rows.length > 0) loadTraces(rows[0]);
	}

	async function loadTraces(row: SnapshotRow) {
		selected = row;
		const key = snapshotKey(row);
		try {
			const result = await cached(
				`connectiondump_traces:${row.timestamp}`,
				() => connectiondumpTraces(row.timestamp),
			);
			// stale guard: a slower response must not clobber a newer selection
			if (selected !== null && snapshotKey(selected) !== key) return;
			traces = result;
		} catch (e) {
			errorMessage = String(e);
		}
	}

	function onselect(row: SnapshotRow) {
		mode = Mode.Dump;
		loadTraces(row);
	}

	/** holder drill-down: open the dump it was last seen in */
	function onjump(holder: ConnDumpHolder) {
		const target = rows.find((r) => r.timestamp === holder.lastSeen);
		if (target === undefined) return;
		onselect(target);
	}

	$effect(() => {
		if (db.state.status === "open") {
			errorMessage = null;
			selected = null;
			view = null;
			refresh();
		} else {
			signals = [];
			poolStats = [];
			snapshots = [];
			holders = [];
			traces = [];
			selected = null;
		}
	});

	$effect(() => {
		if (ingest.generation === 0) return;
		selected = null;
		refresh();
	});
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
							class:active={mode === Mode.Pool}
							onclick={() => (mode = Mode.Pool)}>Pool</button
						>
						<button
							class:active={mode === Mode.Dump}
							onclick={() => (mode = Mode.Dump)}>Dump</button
						>
						<button
							class:active={mode === Mode.Holders}
							onclick={() => (mode = Mode.Holders)}>Holders</button
						>
					</span>

					<span class="right">
						{#if mode === Mode.Pool}
							<span class="legend">
								{#each causes as cause (cause)}
									<span class="key">
										<span
											class="swatch"
											style:background={causeColor(cause)}
										></span>{cause}
									</span>
								{/each}
							</span>
							<button
								class="reset"
								onclick={() => (view = null)}
								disabled={view === null}>reset</button
							>
						{:else}
							<label class="threshold">
								slow hold threshold
								<input
									type="number"
									min="1"
									max="86400"
									bind:value={slowThreshold.value}
								/>
								s
							</label>
						{/if}
					</span>
				</div>

				<div class="body">
					{#if mode === Mode.Pool}
						{#if poolStats.length === 0 && signals.length === 0}
							<p class="empty">
								No connection dumps — parse a bundle with
								performance logs from the Ingest page.
							</p>
						{:else}
							<div class="pool">
								<div class="chart">
									<PoolChart
										stats={poolStats}
										{signals}
										{domain}
										{view}
										onviewchange={(v) => (view = v)}
									/>
								</div>
								<div class="axis">
									{#each ticks as tick, i (i)}
										<span class="tick" style:left="{tick.pct}%"
											>{formatTimestamp(
												tickFormat,
												tick.t,
											)}</span
										>
									{/each}
								</div>
							</div>
						{/if}
					{:else if mode === Mode.Holders}
						<HoldersTable
							rows={holders}
							slowThresholdMs={slowThreshold.value * 1000}
							{onjump}
						/>
					{:else if selected === null}
						<p class="empty">Select a dump.</p>
					{:else}
						<ConnDumpTraceTable
							{traces}
							slowThresholdMs={slowThreshold.value * 1000}
						/>
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

	.legend {
		display: flex;
		gap: 10px;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.key {
		display: inline-flex;
		align-items: center;
		gap: 4px;
	}
	.swatch {
		width: 8px;
		height: 8px;
		border-radius: 2px;
	}

	.reset {
		padding: 2px 10px;
		font-size: 11px;
		font-weight: 600;
		border-radius: 999px;
		background: var(--accent);
		color: var(--bg-hard);
	}
	.reset:hover:not(:disabled) {
		opacity: 0.85;
	}
	.reset:disabled {
		background: var(--bg-hard);
		color: var(--fg-muted);
		opacity: 0.5;
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

	.body {
		flex: 1;
		min-height: 0;
	}

	.pool {
		display: flex;
		flex-direction: column;
		height: 100%;
		padding: 8px 12px 12px;
	}
	.chart {
		flex: 1;
		min-height: 0;
	}
	.axis {
		position: relative;
		height: 16px;
		flex-shrink: 0;
		/* the chart's y-label gutter is 30px (PoolChart .gutter) — ticks
		   must align with the plot, not the gutter */
		margin-left: 30px;
		margin-top: 4px;
	}
	.tick {
		position: absolute;
		transform: translateX(-50%);
		font-size: 10px;
		font-family: var(--font-mono);
		color: var(--fg-muted);
	}
	.tick:first-child {
		transform: none;
	}
	.tick:last-child {
		transform: translateX(-100%);
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
</style>
