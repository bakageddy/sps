<script lang="ts">
	/**
	 * Index fragmentation analyzer — capture-centric drill-down:
	 *
	 *   captures (left) → one capture's indices OR the flagged list across
	 *   captures (middle, toggle) → clicked index's ratio across ALL
	 *   captures + the capture's severity summary (right)
	 *
	 * The question the page answers: "which indices need a rebuild, and is
	 * it getting worse between captures?" The two thresholds that define
	 * "flagged" live in fragmentation-settings and are edited in the
	 * toolbar; the table markers, the summary and the flagged query all
	 * read the same two numbers.
	 *
	 * Fetch philosophy (same as cpumonitoring): user-action fetches live in
	 * the handlers; $effects only react to state owned elsewhere.
	 */
	import { page } from "$app/state";
	import { goto } from "$app/navigation";
	import { pushSelection } from "$lib/navhistory.svelte";
	import { nearestByTimestamp } from "$lib/nearest";
	import {
		fragmentationSnapshots,
		fragmentationIndices,
		fragmentationIndexSeries,
		fragmentationFlagged,
		type FragSnapshot,
		type FragIndex,
		type FragFlag,
	} from "$lib/api/fragmentation";
	import { minPages, minRatio } from "$lib/fragmentation-settings.svelte";
	import { cached } from "$lib/query-cache";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { formatTimestamp } from "$lib/format";
	import SplitPane from "$lib/components/SplitPane.svelte";
	import SnapshotList, {
		snapshotKey,
		type SnapshotRow,
	} from "$lib/components/SnapshotList.svelte";
	import FragmentationTable, {
		REORGANIZE_AT,
		indexKey,
		severity,
		severityColor,
	} from "$lib/components/FragmentationTable.svelte";
	import FlaggedTable from "$lib/components/FlaggedTable.svelte";
	import LineChart, {
		type LineSeries,
	} from "$lib/components/LineChart.svelte";

	// --- state: one straight line, mirroring the drill-down ---------------
	let errorMessage = $state<string | null>(null);

	let snapshots = $state<FragSnapshot[]>([]);
	let selectedSnapshot = $state<number | null>(null);
	let indices = $state<FragIndex[]>([]);
	/** the inspected index — identity survives capture changes */
	let selectedIndex = $state<{ schema: string; table: string; index: string } | null>(null);
	let series = $state<LineSeries[]>([]);
	let chartMaximized = $state(false);

	/** middle pane: this capture's rows, or the flagged list across captures */
	let view = $state<"capture" | "flagged">("capture");
	let flagged = $state<FragFlag[]>([]);
	let flaggedLoading = $state(false);

	const latestCapture = $derived(snapshots.at(-1)?.timestamp ?? null);

	const rows = $derived<SnapshotRow[]>(
		snapshots.map((s) => ({
			timestamp: s.timestamp,
			kind: "frag",
			detail: `${s.indices} indices · max ${s.maxRatio.toFixed(1)}%`,
			alert: s.maxRatio >= minRatio.value,
		})),
	);

	// severity census of the selected capture over indices above the floor
	const summary = $derived.by(() => {
		let flaggedCount = 0;
		let reorganize = 0;
		let ok = 0;
		let pages = 0;
		for (const r of indices) {
			pages += r.pageCount;
			if (r.pageCount < minPages.value) continue;
			const s = severity(r.ratio, minRatio.value);
			if (s === "rebuild") flaggedCount++;
			else if (s === "reorganize") reorganize++;
			else ok++;
		}
		return { flagged: flaggedCount, reorganize, ok, pages, total: indices.length };
	});

	function resetBelow(level: "snapshot" | "index") {
		if (level === "snapshot") {
			selectedSnapshot = null;
			indices = [];
		}
		selectedIndex = null;
		series = [];
	}

	async function refreshSnapshots() {
		snapshots = await cached("fragmentation_snapshots", fragmentationSnapshots);
	}

	// the flagged list depends on the thresholds: re-query when they change
	// (persisted settings, so this also runs once on mount) and when data
	// changes. Only while that view is showing — no point otherwise.
	async function refreshFlagged() {
		const p = minPages.value;
		const r = minRatio.value;
		flaggedLoading = true;
		try {
			flagged = await cached(`fragmentation_flagged:${p}:${r}`, () =>
				fragmentationFlagged(p, r),
			);
		} catch (e) {
			errorMessage = String(e);
		} finally {
			flaggedLoading = false;
		}
	}
	$effect(() => {
		if (view !== "flagged" || db.state.status !== "open") return;
		void minPages.value;
		void minRatio.value;
		void db.epoch;
		void ingest.generation;
		refreshFlagged();
	});

	// Database changed (topbar): drop the whole drill-down, load fresh.
	$effect(() => {
		if (db.state.status === "open") {
			resetBelow("snapshot");
			errorMessage = null;
			refreshSnapshots().catch((e) => (errorMessage = String(e)));
		} else {
			snapshots = [];
			flagged = [];
			resetBelow("snapshot");
		}
	});

	// New data ingested: more captures; the selection stays valid.
	$effect(() => {
		if (ingest.generation === 0) return;
		refreshSnapshots().catch((e) => (errorMessage = String(e)));
	});

	// Cross-analyzer link / palette "jump to time": ?t=<ms> selects the
	// nearest capture once data is here; consumed once per value.
	let consumedLink = $state<string | null>(null);
	$effect(() => {
		const raw = page.url.searchParams.get("t");
		if (raw === null || raw === consumedLink) return;
		const target = Number(raw);
		if (!Number.isFinite(target) || snapshots.length === 0) return;
		const nearest = nearestByTimestamp(snapshots, target);
		if (nearest === null) return;
		consumedLink = raw;
		onselectsnapshot(nearest.timestamp);
	});

	// --- drill-down actions ------------------------------------------------
	async function onselectsnapshot(timestamp: number) {
		consumedLink = String(timestamp);
		pushSelection(timestamp);
		selectedSnapshot = timestamp;
		// keep the inspected index: "same index, other capture" is the
		// comparison this page exists for
		try {
			indices = await cached(`fragmentation_indices:${timestamp}`, () =>
				fragmentationIndices(timestamp),
			);
		} catch (e) {
			errorMessage = String(e);
		}
	}

	async function onselectindex(row: { schema: string; table: string; index: string }) {
		selectedIndex = { schema: row.schema, table: row.table, index: row.index };
		series = [];
		try {
			const points = await cached(
				`fragmentation_index_series:${indexKey(row)}`,
				() => fragmentationIndexSeries(row.schema, row.table, row.index),
			);
			series = [
				{
					id: 0,
					label: `${row.table}.${row.index}`,
					points: points.map((p) => ({
						timestamp: p.timestamp,
						value: p.ratio,
					})),
				},
			];
		} catch (e) {
			series = [];
			console.warn("fragmentation_index_series failed:", e);
		}
	}

	// a point on the trend = "show me that capture" (index stays selected)
	function onpointselect(_id: number, timestamp: number) {
		onselectsnapshot(timestamp);
	}

	// threshold inputs: ignore empty/NaN while typing, keep the last good value
	function setPages(e: Event) {
		const v = Number((e.currentTarget as HTMLInputElement).value);
		if (Number.isFinite(v) && v >= 0) minPages.value = Math.round(v);
	}
	function setRatio(e: Event) {
		const v = Number((e.currentTarget as HTMLInputElement).value);
		if (Number.isFinite(v) && v >= 0 && v <= 100) minRatio.value = v;
	}

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
	const num = new Intl.NumberFormat();
</script>

<div class="page">
	{#if errorMessage}
		<div class="error-bar" role="alert">
			{errorMessage}
			<button onclick={() => (errorMessage = null)} aria-label="Dismiss">✕</button>
		</div>
	{/if}

	<div class="toolbar">
		<div class="views" role="tablist">
			<button
				role="tab"
				class:active={view === "capture"}
				aria-selected={view === "capture"}
				onclick={() => (view = "capture")}>This capture</button
			>
			<button
				role="tab"
				class:active={view === "flagged"}
				aria-selected={view === "flagged"}
				onclick={() => (view = "flagged")}
				>Flagged across captures{#if flaggedLoading}…{:else if view === "flagged"} ({flagged.length}){/if}</button
			>
		</div>

		<div class="thresholds" title="An index is flagged when it is at least this fragmented AND at least this big. Both are yours to tune; the markers, the summary and the flagged list follow.">
			<span class="legend">
				<span style:color={severityColor.rebuild}>⚑</span> flag at ≥
			</span>
			<input type="number" min="0" max="100" step="1" value={minRatio.value} onchange={setRatio} />%
			<span class="legend">on ≥</span>
			<input type="number" min="0" step="100" value={minPages.value} onchange={setPages} />
			<span class="legend">pages</span>
			<span class="legend dim">· <span style:color={severityColor.reorganize}>■</span> ≥{REORGANIZE_AT}% reorganize</span>
		</div>

		{#if selectedSnapshot !== null}
			<button
				class="cross-link"
				onclick={() => goto(`/runningqueries?t=${selectedSnapshot}`)}
				title="Open Running Queries at the nearest tick"
				>queries at this time →</button
			>
		{/if}
	</div>

	<div class="content">
		{#if chartMaximized}
			<LineChart
				{series}
				unit="%"
				emptyText="Click an index to plot its fragmentation across captures."
				{onpointselect}
				expanded
				ontoggleexpand={() => (chartMaximized = false)}
			/>
		{:else}
			<SplitPane direction="row" initial={0.22}>
				{#snippet a()}
					<SnapshotList
						{rows}
						selected={selectedSnapshot === null
							? null
							: snapshotKey({ kind: "frag", timestamp: selectedSnapshot, detail: "", alert: false })}
						onselect={(row) => onselectsnapshot(row.timestamp)}
					/>
				{/snippet}
				{#snippet b()}
					<SplitPane direction="row" initial={0.62}>
						{#snippet a()}
							{#if view === "capture"}
								<FragmentationTable
									rows={indices}
									selected={selectedIndex === null ? null : indexKey(selectedIndex)}
									onselect={onselectindex}
								/>
							{:else}
								<FlaggedTable
									rows={flagged}
									{latestCapture}
									selected={selectedIndex === null ? null : indexKey(selectedIndex)}
									onselect={onselectindex}
								/>
							{/if}
						{/snippet}
						{#snippet b()}
							<div class="side">
								{#if selectedSnapshot !== null}
									<section class="summary">
										<h3 class="mono">{formatTimestamp(timeFormat, selectedSnapshot)}</h3>
										<dl>
											<dt style:color={severityColor.rebuild}>flagged</dt>
											<dd class="mono">{summary.flagged}</dd>
											<dt style:color={severityColor.reorganize}>reorganize</dt>
											<dd class="mono">{summary.reorganize}</dd>
											<dt style:color={severityColor.ok}>healthy</dt>
											<dd class="mono">{summary.ok}</dd>
											<dt>indices</dt>
											<dd class="mono">{num.format(summary.total)}</dd>
											<dt>pages</dt>
											<dd class="mono">{num.format(summary.pages)}</dd>
										</dl>
										<p class="hint">counts over indices with ≥ {num.format(minPages.value)} pages</p>
									</section>
								{/if}
								<div class="chart">
									<LineChart
										{series}
										unit="%"
										emptyText="Click an index to plot its fragmentation across captures."
										{onpointselect}
										ontoggleexpand={() => (chartMaximized = true)}
									/>
								</div>
							</div>
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
		align-items: center;
		gap: 16px;
		padding: 6px 12px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
		min-height: 37px;
		font-size: 12px;
	}

	.views {
		display: flex;
		gap: 2px;
		padding: 2px;
		background: var(--bg-hard);
		border-radius: var(--radius);
	}
	.views button {
		padding: 3px 10px;
		font-size: 12px;
		background: none;
		border: none;
		border-radius: var(--radius);
		color: var(--fg-muted);
		cursor: pointer;
		white-space: nowrap;
	}
	.views button.active {
		background: var(--bg-soft);
		color: var(--fg-strong);
	}

	.thresholds {
		display: flex;
		align-items: center;
		gap: 5px;
		white-space: nowrap;
	}
	.thresholds input {
		width: 64px;
		padding: 3px 6px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		font-family: var(--font-mono);
		font-size: 12px;
		text-align: right;
	}
	.legend {
		color: var(--fg-muted);
	}
	.dim {
		opacity: 0.8;
		margin-left: 6px;
	}

	.cross-link {
		margin-left: auto;
		padding: 4px 12px;
		font-size: 12px;
		background: none;
		border: none;
		border-radius: var(--radius);
		color: var(--fg-muted);
		cursor: pointer;
		white-space: nowrap;
	}
	.cross-link:hover {
		background: var(--bg-hover);
		color: var(--fg);
	}

	.content {
		flex: 1;
		min-height: 0;
	}

	.side {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.summary {
		padding: 10px 12px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.summary h3 {
		margin: 0 0 8px;
		font-size: 12px;
		font-weight: 600;
		color: var(--fg-strong);
	}
	dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 2px 12px;
		margin: 0;
		font-size: 12px;
	}
	dt {
		color: var(--fg-muted);
	}
	dd {
		margin: 0;
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	.hint {
		margin: 8px 0 0;
		font-size: 11px;
		color: var(--fg-muted);
	}
	.chart {
		flex: 1;
		min-height: 0;
	}
</style>
