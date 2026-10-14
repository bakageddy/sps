<script lang="ts" module>
	/**
	 * Severity colouring used everywhere on the fragmentation page. The
	 * rebuild line is the user's `minRatio` setting (default 30 %, SQL
	 * Server's guidance); reorganize stays fixed at 5 % — nobody tunes it.
	 */
	export const REORGANIZE_AT = 5;

	export type Severity = "rebuild" | "reorganize" | "ok";

	export function severity(ratio: number, rebuildAt: number): Severity {
		if (ratio >= rebuildAt) return "rebuild";
		if (ratio >= REORGANIZE_AT) return "reorganize";
		return "ok";
	}

	export const severityColor: Record<Severity, string> = {
		rebuild: "var(--red)",
		reorganize: "var(--yellow)",
		ok: "var(--green)",
	};

	/** stable identity of an index across captures (schema+table+index) */
	export const indexKey = (r: { schema: string; table: string; index: string }) =>
		`${r.schema}.${r.table}.${r.index}`;
</script>

<script lang="ts">
	/**
	 * The indices of ONE capture: sortable, filterable, one row selectable.
	 * Clicking a row is "show me this index over time" — the page fetches
	 * its series for the chart.
	 *
	 * Rows crossing BOTH shared thresholds (fragmentation-settings) carry a
	 * flag; the small-index floor also hides rows under it, since their
	 * ratio is noise. Selection lives in the parent; sort/filter are local.
	 */
	import type { FragIndex } from "$lib/api/fragmentation";
	import { minPages, minRatio } from "$lib/fragmentation-settings.svelte";
	import { virtualWindow } from "$lib/virtual";
	import { stepIndex } from "$lib/listkeys";
	import { sortRows } from "$lib/sort";

	interface Props {
		rows: FragIndex[];
		/** indexKey() of the inspected index, or null */
		selected: string | null;
		onselect: (row: FragIndex) => void;
	}

	let { rows, selected, onselect }: Props = $props();

	type SortKey = "schema" | "table" | "index" | "ratio" | "pageCount";
	let sortKey = $state<SortKey>("ratio");
	let sortDescending = $state(true);
	let filter = $state("");
	/** show indices under the page floor too (off by default: noise) */
	let showSmall = $state(false);

	function sortBy(key: SortKey) {
		if (sortKey === key) {
			sortDescending = !sortDescending; // second click flips direction
		} else {
			sortKey = key;
			sortDescending = key === "ratio" || key === "pageCount";
		}
	}

	const floor = $derived(Number.isFinite(minPages.value) ? minPages.value : 0);
	const isFlagged = (r: FragIndex) => r.pageCount >= floor && r.ratio >= minRatio.value;

	const visible = $derived.by(() => {
		const needle = filter.trim().toLowerCase();
		const filtered = rows.filter(
			(r) =>
				(showSmall || r.pageCount >= floor) &&
				(!needle ||
					r.table.toLowerCase().includes(needle) ||
					r.index.toLowerCase().includes(needle) ||
					r.schema.toLowerCase().includes(needle)),
		);
		return sortRows(filtered, (r) => r[sortKey], sortDescending);
	});

	const hiddenSmall = $derived(
		showSmall ? 0 : rows.length - rows.filter((r) => r.pageCount >= floor).length,
	);

	const columns: { key: SortKey; label: string; num?: boolean }[] = [
		{ key: "schema", label: "Schema" },
		{ key: "table", label: "Table" },
		{ key: "index", label: "Index" },
		{ key: "ratio", label: "Fragmentation", num: true },
		{ key: "pageCount", label: "Pages", num: true },
	];

	// Virtualized: a capture can list every index of a large schema (tens of
	// thousands of rows). ROW/HEADER are the fixed pixel heights the CSS
	// enforces — the window math depends on them being exact.
	const ROW = 30;
	const HEADER = 34;
	let viewport = $state(0);
	let scrollTop = $state(0);
	const win = $derived(
		virtualWindow(scrollTop - HEADER, viewport, ROW, visible.length),
	);
	const slice = $derived(visible.slice(win.start, win.end));

	// j/k / arrows step the selection when the table is focused
	let scroller = $state<HTMLDivElement>();
	function onkeydown(e: KeyboardEvent) {
		const cur = visible.findIndex((r) => indexKey(r) === selected);
		const next = stepIndex(e, cur, visible.length);
		if (next === null) return;
		e.preventDefault();
		onselect(visible[next]);
		if (!scroller) return;
		const top = HEADER + next * ROW;
		if (top - HEADER < scroller.scrollTop) scroller.scrollTop = top - HEADER;
		else if (top + ROW > scroller.scrollTop + viewport)
			scroller.scrollTop = top + ROW - viewport;
	}

	const pages = new Intl.NumberFormat();
</script>

<div class="wrap">
	<div class="controls">
		<input
			class="filter"
			type="search"
			placeholder="Filter by table, index or schema…"
			bind:value={filter}
		/>
		{#if hiddenSmall > 0 || showSmall}
			<label class="small" title="Indices under the page floor ({pages.format(floor)} pages) — fragmentation on tiny indices is noise">
				<input type="checkbox" bind:checked={showSmall} />
				show small
			</label>
		{/if}
		<span class="count mono">
			{visible.length}{#if hiddenSmall > 0}<span class="hidden"> · {hiddenSmall} small hidden</span>{/if}
		</span>
	</div>

	<div
		class="scroller"
		role="listbox"
		aria-label="Indices"
		tabindex="0"
		bind:this={scroller}
		bind:clientHeight={viewport}
		{onkeydown}
		onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
	>
		<div class="header">
			{#each columns as col (col.key)}
				<button class="sort" class:num={col.num} onclick={() => sortBy(col.key)}>
					{col.label}
					{#if sortKey === col.key}
						<span class="arrow">{sortDescending ? "▼" : "▲"}</span>
					{/if}
				</button>
			{/each}
		</div>
		<div class="spacer" style:height="{win.padTop}px"></div>
		{#each slice as r (indexKey(r))}
			{@const sev = severity(r.ratio, minRatio.value)}
			<button
				class="row"
				class:selected={selected === indexKey(r)}
				class:flagged={isFlagged(r)}
				onclick={() => onselect(r)}
				title={indexKey(r)}
			>
				<span class="cell muted">{r.schema}</span>
				<span class="cell">{r.table}</span>
				<span class="cell">{r.index}</span>
				<span class="cell ratio">
					<span class="bar" style:width="{Math.min(100, r.ratio)}%" style:background={severityColor[sev]}></span>
					{#if isFlagged(r)}<span class="flag" title="Flagged: ≥{minRatio.value}% on ≥{pages.format(floor)} pages">⚑</span>{/if}
					<span class="pct mono" style:color={severityColor[sev]}>{r.ratio.toFixed(1)}%</span>
				</span>
				<span class="cell num mono">{pages.format(r.pageCount)}</span>
			</button>
		{:else}
			<p class="empty">
				{rows.length === 0 ? "Select a capture on the left." : "No match."}
			</p>
		{/each}
		<div class="spacer" style:height="{win.padBottom}px"></div>
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
	.small {
		display: flex;
		align-items: center;
		gap: 4px;
		font-size: 12px;
		color: var(--fg-muted);
		white-space: nowrap;
		cursor: pointer;
	}
	.count {
		font-size: 12px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.hidden {
		opacity: 0.7;
	}

	.scroller {
		overflow: auto;
		overflow-anchor: none;
		overscroll-behavior: contain;
		flex: 1;
		outline: none;
	}
	.scroller:focus-visible {
		box-shadow: inset 0 0 0 1px var(--accent);
	}

	/* one grid template shared by header and rows, so columns line up
	   without a <table> (virtualised rows can't be <tr>s under a sticky
	   <thead> without fighting the browser) */
	.header,
	.row {
		display: grid;
		grid-template-columns: minmax(70px, 0.6fr) minmax(140px, 1.4fr) minmax(160px, 1.6fr) 160px 90px;
		align-items: center;
	}

	.header {
		position: sticky;
		top: 0;
		z-index: 1;
		height: 34px; /* = HEADER in the script */
		box-sizing: border-box;
		background: var(--bg-soft);
		font-size: 13px;
	}
	.sort {
		height: 100%;
		padding: 0 10px;
		text-align: left;
		background: none;
		border: none;
		cursor: pointer;
		font-weight: 600;
		color: var(--fg-strong);
		white-space: nowrap;
	}
	.sort.num {
		text-align: right;
	}
	.sort:hover {
		background: var(--bg-hover);
	}
	.arrow {
		font-size: 9px;
		color: var(--accent);
	}

	.row {
		height: 30px; /* = ROW in the script */
		box-sizing: border-box;
		width: 100%;
		padding: 0;
		background: none;
		border: none;
		border-top: 1px solid var(--hairline);
		color: var(--fg);
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}
	.row:hover {
		background: var(--bg-hover);
	}
	.row.selected {
		background: color-mix(in srgb, var(--accent) 12%, transparent);
	}
	.row.flagged {
		box-shadow: inset 3px 0 0 var(--red);
	}

	.cell {
		padding: 0 10px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.muted {
		color: var(--fg-muted);
	}
	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}

	/* the ratio cell is a bar with the number over it: the colour says
	   rebuild/reorganize/ok at a glance, the number says how much */
	.ratio {
		position: relative;
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 6px;
		height: 100%;
	}
	.bar {
		position: absolute;
		left: 10px;
		right: 10px;
		top: 7px;
		bottom: 7px;
		max-width: calc(100% - 20px);
		opacity: 0.22;
		border-radius: 2px;
	}
	.flag {
		position: relative;
		color: var(--red);
		font-size: 12px;
	}
	.pct {
		position: relative;
		font-size: 12px;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}

	.empty {
		text-align: center;
		color: var(--fg-muted);
		padding: 24px;
		margin: 0;
	}
</style>
