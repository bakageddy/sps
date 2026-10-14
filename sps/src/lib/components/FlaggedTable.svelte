<script lang="ts">
	/**
	 * Indices flagged in at least one capture, ranked. The one column that
	 * matters beyond the ratio is "flagged in N of M captures": every
	 * capture means chronic (rebuild it), only the newest means it just
	 * crossed the line, none of the recent ones means someone already
	 * rebuilt it. Clicking a row plots the index on the page's chart.
	 */
	import type { FragFlag } from "$lib/api/fragmentation";
	import { minRatio } from "$lib/fragmentation-settings.svelte";
	import { formatTimestamp } from "$lib/format";
	import { sortRows } from "$lib/sort";
	import SortButton from "$lib/components/SortButton.svelte";
	import { indexKey, severity, severityColor } from "$lib/components/FragmentationTable.svelte";

	interface Props {
		rows: FragFlag[];
		/** ms epoch of the newest capture in the store (null = none) */
		latestCapture: number | null;
		/** indexKey() of the inspected index, or null */
		selected: string | null;
		onselect: (row: FragFlag) => void;
	}

	let { rows, latestCapture, selected, onselect }: Props = $props();

	type Status = "chronic" | "current" | "cleared";
	function status(r: FragFlag): Status {
		if (r.flaggedCaptures === r.captures) return "chronic";
		if (latestCapture !== null && r.lastFlagged === latestCapture) return "current";
		return "cleared";
	}
	const statusColor: Record<Status, string> = {
		chronic: "var(--red)",
		current: "var(--yellow)",
		cleared: "var(--fg-muted)",
	};
	const statusHint: Record<Status, string> = {
		chronic: "flagged in every capture that contains it",
		current: "flagged in the newest capture, not in all",
		cleared: "was flagged earlier, not in the newest capture",
	};

	type SortKey = "table" | "index" | "flaggedCaptures" | "latestRatio" | "latestPages" | "lastFlagged" | "status";
	let sortKey = $state<SortKey>("latestRatio");
	let sortDescending = $state(true);
	let filter = $state("");

	function sortBy(key: SortKey) {
		if (sortKey === key) sortDescending = !sortDescending;
		else {
			sortKey = key;
			sortDescending = key !== "table" && key !== "index" && key !== "status";
		}
	}

	const statusRank: Record<Status, number> = { chronic: 0, current: 1, cleared: 2 };
	const visible = $derived.by(() => {
		const needle = filter.trim().toLowerCase();
		const filtered = needle
			? rows.filter((r) => indexKey(r).toLowerCase().includes(needle))
			: rows;
		return sortRows(
			filtered,
			(r) => (sortKey === "status" ? statusRank[status(r)] : r[sortKey]),
			sortDescending,
		);
	});

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "short",
		hourCycle: "h23",
	});
	const num = new Intl.NumberFormat();

	const head = (key: SortKey, label: string) => ({ key, label });
</script>

<div class="wrap">
	<div class="controls">
		<input class="filter" type="search" placeholder="Filter by schema, table or index…" bind:value={filter} />
		<span class="count mono">{visible.length}</span>
	</div>
	<div class="scroller">
		<table>
			<thead>
				<tr>
					<th><SortButton {...head("table", "Table")} active={sortKey === "table"} descending={sortDescending} onclick={() => sortBy("table")} /></th>
					<th><SortButton {...head("index", "Index")} active={sortKey === "index"} descending={sortDescending} onclick={() => sortBy("index")} /></th>
					<th><SortButton {...head("status", "Status")} active={sortKey === "status"} descending={sortDescending} onclick={() => sortBy("status")} /></th>
					<th class="num"><SortButton {...head("flaggedCaptures", "Flagged in")} active={sortKey === "flaggedCaptures"} descending={sortDescending} onclick={() => sortBy("flaggedCaptures")} /></th>
					<th class="num"><SortButton {...head("latestRatio", "Latest")} active={sortKey === "latestRatio"} descending={sortDescending} onclick={() => sortBy("latestRatio")} /></th>
					<th class="num"><SortButton {...head("latestPages", "Pages")} active={sortKey === "latestPages"} descending={sortDescending} onclick={() => sortBy("latestPages")} /></th>
					<th><SortButton {...head("lastFlagged", "Last flagged")} active={sortKey === "lastFlagged"} descending={sortDescending} onclick={() => sortBy("lastFlagged")} /></th>
				</tr>
			</thead>
			<tbody>
				{#each visible as r (indexKey(r))}
					{@const st = status(r)}
					{@const sev = severity(r.latestRatio, minRatio.value)}
					<tr class:selected={selected === indexKey(r)} onclick={() => onselect(r)} title={indexKey(r)}>
						<td class="name"><span class="muted">{r.schema}.</span>{r.table}</td>
						<td class="name">{r.index}</td>
						<td><span class="status" style:color={statusColor[st]} title={statusHint[st]}>{st}</span></td>
						<td class="num mono">{r.flaggedCaptures} / {r.captures}</td>
						<td class="num mono" style:color={severityColor[sev]}>{r.latestRatio.toFixed(1)}%</td>
						<td class="num mono">{num.format(r.latestPages)}</td>
						<td class="mono">{formatTimestamp(timeFormat, r.lastFlagged)}</td>
					</tr>
				{:else}
					<tr>
						<td colspan="7" class="empty">
							{rows.length === 0
								? "No index crosses both thresholds in any capture."
								: "No match."}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
</div>

<style>
	.wrap {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.controls {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 8px;
	}
	.filter {
		flex: 1;
		min-width: 0;
		padding: 6px 10px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
	}
	.count {
		font-size: 12px;
		color: var(--fg-muted);
	}
	.scroller {
		overflow: auto;
		overscroll-behavior: contain;
		flex: 1;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 13px;
	}
	thead {
		position: sticky;
		top: 0;
		z-index: 1;
		background: var(--bg-soft);
	}
	th {
		padding: 8px 10px;
		text-align: left;
		font-weight: 600;
		color: var(--fg-strong);
		white-space: nowrap;
	}
	th.num,
	td.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
	td {
		padding: 5px 10px;
		border-top: 1px solid var(--hairline);
		white-space: nowrap;
	}
	.name {
		max-width: 260px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.muted {
		color: var(--fg-muted);
	}
	.status {
		font-size: 12px;
		font-weight: 600;
	}
	tbody tr {
		cursor: pointer;
	}
	tbody tr:hover {
		background: var(--bg-hover);
	}
	tr.selected {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}
	.empty {
		text-align: center;
		color: var(--fg-muted);
		padding: 24px;
		cursor: default;
	}
</style>
