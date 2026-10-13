<script lang="ts">
	/**
	 * Signals per day, GitHub-contributions style: weeks as columns, Sun→Sat
	 * as rows, each cell shaded by how many connection-dump signals fired
	 * that day RELATIVE to the bundle's busiest day (5 levels, quantized
	 * against the max — a quiet bundle still shows contrast).
	 *
	 * Counts real dumps; "Skipping to dump" lines are tallied separately
	 * for the tooltip since they say "still hot", not "new incident". Days
	 * are cut in the display timezone. Click a day → that day becomes the
	 * global time window and the incident page opens on it.
	 *
	 * The grid spans the FULL calendar year(s) the data touches — Jan 1 to
	 * Dec 31, every month present — not just the weeks with signals, exactly
	 * like GitHub's contribution graph. A four-day bundle still shows where
	 * in the year it sits; the grid scrolls sideways and opens on the data.
	 */
	import { goto } from "$app/navigation";
	import { connectiondumpSignals, type ConnDumpSignal } from "$lib/api/connectiondump";
	import { cached } from "$lib/query-cache";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { timeWindow } from "$lib/timewindow.svelte";
	import { dayStart, resolveZone, zonedDate } from "$lib/timezone.svelte";
	import { formatTimestamp } from "$lib/format";

	let signals = $state<ConnDumpSignal[]>([]);

	async function load() {
		try {
			signals = await cached("connectiondump_signals|true", () =>
				connectiondumpSignals(true),
			);
		} catch {
			signals = []; // the heatmap is decoration; the analyzers report errors
		}
	}
	$effect(() => {
		if (db.state.status === "open") {
			void db.epoch;
			load();
		} else signals = [];
	});
	$effect(() => {
		if (ingest.generation === 0) return;
		load();
	});

	const DAY = 86_400_000;

	interface Day {
		start: number;
		real: number;
		suppressed: number;
		byCause: Map<string, number>;
	}

	// bucket by display-zone day; resolveZone() is read inside so a
	// timezone change re-cuts the days
	const days = $derived.by(() => {
		void resolveZone();
		const map = new Map<number, Day>();
		for (const s of signals) {
			const k = dayStart(s.timestamp);
			let d = map.get(k);
			if (!d) {
				d = { start: k, real: 0, suppressed: 0, byCause: new Map() };
				map.set(k, d);
			}
			if (s.suppressed) d.suppressed += 1;
			else {
				d.real += 1;
				d.byCause.set(s.cause, (d.byCause.get(s.cause) ?? 0) + 1);
			}
		}
		return map;
	});

	/** zone-midnight of month/day (1-based) in `year`, robust to the zone
	 *  pushing noon-UTC across a date line */
	function zoneDay(year: number, month: number, day: number, zone: string | undefined): number {
		let ms = dayStart(Date.UTC(year, month - 1, day, 12));
		// noon UTC can land a calendar day early/late in zones past ±12 (Dec 31
		// in +14 is already next year's Jan 1) — compare the FULL date, year
		// included, so the correction goes the right way across a year wrap
		const [y, mo, d] = zonedDate(ms + 12 * 3_600_000, zone);
		const got = (y * 12 + mo) * 31 + d;
		const want = (year * 12 + month) * 31 + day;
		if (got > want) ms -= DAY;
		else if (got < want) ms += DAY;
		return ms;
	}

	// the grid: every week from the Sunday on/before Jan 1 of the first
	// data year to the Saturday on/after Dec 31 of the last — full years,
	// one column per week; `first`/`last` stay the DATA bounds for the header
	const grid = $derived.by(() => {
		if (days.size === 0) return null;
		const starts = [...days.keys()];
		const first = Math.min(...starts);
		const last = Math.max(...starts);
		const zone = resolveZone();
		const [firstYear] = zonedDate(first + 12 * 3_600_000, zone);
		const [lastYear] = zonedDate(last + 12 * 3_600_000, zone);
		const yearStart = zoneDay(firstYear, 1, 1, zone);
		const yearEnd = zoneDay(lastYear, 12, 31, zone);
		// weekday of a zone-midnight, asked at noon so DST shifts can't move it
		const wdFormat = new Intl.DateTimeFormat("en-US", { timeZone: zone, weekday: "short" });
		const NAMES = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
		const weekday = (ms: number) => NAMES.indexOf(wdFormat.format(ms + 12 * 3_600_000));
		const gridStart = yearStart - weekday(yearStart) * DAY;
		const gridEnd = yearEnd + (6 - weekday(yearEnd)) * DAY;
		const max = Math.max(1, ...[...days.values()].map((d) => d.real));
		// real -1 = padding outside the shown year(s) (blank, like GitHub);
		// 0 = a real day in the year with nothing on it
		type Cell = Day | { start: number; real: number };
		const weeks: { start: number; days: Cell[] }[] = [];
		let firstWeek = 0;
		for (let w = gridStart; w <= gridEnd; w += 7 * DAY) {
			const col: Cell[] = [];
			for (let i = 0; i < 7; i++) {
				const s = dayStart(w + i * DAY + 12 * 3_600_000); // re-anchor: DST can shift a day by an hour
				const d = days.get(s);
				col.push(d ?? { start: s, real: s < yearStart || s > yearEnd ? -1 : 0 });
			}
			if (w <= first && first < w + 7 * DAY) firstWeek = weeks.length;
			weeks.push({ start: w, days: col });
		}
		return { weeks, max, first, last, firstWeek };
	});

	/** 0 = none, 1..4 relative to the busiest day */
	const level = (real: number, max: number) =>
		real <= 0 ? 0 : Math.max(1, Math.ceil((real / max) * 4));

	const monthFormat = new Intl.DateTimeFormat(undefined, { month: "short" });
	const dayFormat = new Intl.DateTimeFormat(undefined, { dateStyle: "medium" });

	// month label above the first week that starts a new month
	const monthLabels = $derived.by(() => {
		if (grid === null) return [];
		const out: { index: number; label: string }[] = [];
		let prev = "";
		grid.weeks.forEach((w, i) => {
			const m = formatTimestamp(monthFormat, w.days[0].start + 12 * 3_600_000);
			if (m !== prev) {
				out.push({ index: i, label: m });
				prev = m;
			}
		});
		return out;
	});

	function tooltip(d: Day | { start: number; real: number }): string {
		const when = formatTimestamp(dayFormat, d.start + 12 * 3_600_000);
		if (!("byCause" in d)) return d.real < 0 ? when : `${when}: no signals`;
		const causes = [...d.byCause.entries()]
			.toSorted((a, b) => b[1] - a[1])
			.map(([c, n]) => `${n} ${c}`)
			.join(", ");
		return `${when}: ${d.real} signal${d.real === 1 ? "" : "s"} (${causes})${
			d.suppressed > 0 ? ` · ${d.suppressed.toLocaleString()} suppressed` : ""
		}`;
	}

	function open(d: { start: number; real: number }) {
		if (d.real < 0) return;
		timeWindow.value = [d.start, d.start + DAY - 1];
		goto("/connectiondump/incident");
	}

	const total = $derived([...days.values()].reduce((n, d) => n + d.real, 0));

	// --- horizontal scrolling -------------------------------------------
	// WebKitGTK's overlay scrollbar hides until hovered and only moves on
	// Shift+wheel, so a wide grid looked stuck. The wheel scrolls the grid
	// sideways directly, the bar is always visible, and ‹ › step a week.
	let scroller = $state<HTMLDivElement>();
	let canScroll = $state(false);
	function onwheel(e: WheelEvent) {
		const el = scroller;
		if (!el || el.scrollWidth <= el.clientWidth) return;
		if (Math.abs(e.deltaY) > Math.abs(e.deltaX)) {
			e.preventDefault();
			el.scrollLeft += e.deltaY;
		}
	}
	function step(weeks: number) {
		scroller?.scrollBy({ left: weeks * 15, behavior: "smooth" });
	}
	// open scrolled to the DATA, not the end of the year: the bundle's first
	// week lands roughly a third of the way in, so what follows it is visible
	const WEEK_PX = 15; // 12px cell + 3px gap, matches the CSS
	$effect(() => {
		const g = grid;
		const el = scroller;
		if (!el || g === null) return;
		requestAnimationFrame(() => {
			canScroll = el.scrollWidth > el.clientWidth;
			el.scrollLeft = Math.max(0, g.firstWeek * WEEK_PX - el.clientWidth / 3);
		});
	});
	const WEEKDAYS = ["", "Mon", "", "Wed", "", "Fri", ""];
</script>

{#if grid !== null}
	<section class="heat">
		<header>
			<span class="sum mono">
				{total.toLocaleString()} signals ·
				{formatTimestamp(dayFormat, grid.first + 12 * 3_600_000)} → {formatTimestamp(
					dayFormat,
					grid.last + 12 * 3_600_000,
				)}
			</span>
			{#if canScroll}
				<span class="nav">
					<button onclick={() => step(-4)} title="Earlier" aria-label="Scroll to earlier weeks">‹</button>
					<button onclick={() => step(4)} title="Later" aria-label="Scroll to later weeks">›</button>
				</span>
			{/if}
			<span class="legend">
				less
				{#each [0, 1, 2, 3, 4] as l (l)}
					<span class="cell l{l}"></span>
				{/each}
				more
			</span>
		</header>
		<div class="scroll" bind:this={scroller} {onwheel}>
			<div class="grid" style:--weeks={grid.weeks.length}>
				<div class="months">
					{#each monthLabels as m (m.index)}
						<span style:grid-column={m.index + 1}>{m.label}</span>
					{/each}
				</div>
				<div class="weekdays">
					{#each WEEKDAYS as w, i (i)}
						<span>{w}</span>
					{/each}
				</div>
				<div class="weeks">
					{#each grid.weeks as w (w.start)}
						<div class="week">
							{#each w.days as d (d.start)}
								<button
									class="cell l{level(d.real, grid.max)}"
									class:outside={d.real < 0}
									title={tooltip(d)}
									aria-label={tooltip(d)}
									disabled={d.real < 0}
									onclick={() => open(d)}
								></button>
							{/each}
						</div>
					{/each}
				</div>
			</div>
		</div>
		<p class="hint">Click a day to open it in the incident view (it becomes the time window).</p>
	</section>
{:else}
	<p class="empty">
		No connection-dump signals yet — parse a bundle with performance logs and
		the days light up here.
	</p>
{/if}

<style>
	.heat {
		padding: 12px 14px;
		background: var(--bg-soft);
		border-radius: var(--radius);
	}
	header {
		display: flex;
		align-items: baseline;
		gap: 12px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}
	.sum {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.legend {
		margin-left: auto;
		display: flex;
		align-items: center;
		gap: 3px;
		font-size: 10.5px;
		color: var(--fg-muted);
	}
	.legend .cell {
		display: inline-block;
	}
	.mono {
		font-family: var(--font-mono);
	}

	.scroll {
		overflow-x: auto;
		overflow-y: hidden;
		padding-bottom: 6px;
		/* always-visible thin bar: the grid can be many screens wide */
		scrollbar-width: thin;
		scrollbar-color: var(--border) transparent;
	}
	.scroll::-webkit-scrollbar {
		height: 6px;
	}
	.scroll::-webkit-scrollbar-thumb {
		background: var(--border);
		border-radius: 3px;
	}
	.scroll::-webkit-scrollbar-track {
		background: transparent;
	}
	.nav {
		display: flex;
		gap: 2px;
	}
	.nav button {
		padding: 0 8px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--fg-muted);
		font-size: 12px;
		line-height: 18px;
	}
	.nav button:hover {
		color: var(--fg);
		background: var(--bg-hover);
	}
	.grid {
		display: grid;
		grid-template-columns: 28px 1fr;
		grid-template-rows: 14px 1fr;
		gap: 4px;
		width: max-content;
	}
	.months {
		grid-column: 2;
		display: grid;
		grid-template-columns: repeat(var(--weeks), 12px);
		gap: 3px;
		font-size: 10px;
		color: var(--fg-muted);
	}
	.months span {
		white-space: nowrap;
	}
	.weekdays {
		grid-column: 1;
		grid-row: 2;
		display: grid;
		grid-template-rows: repeat(7, 12px);
		gap: 3px;
		font-size: 9.5px;
		line-height: 12px;
		color: var(--fg-muted);
		text-align: right;
		padding-right: 2px;
	}
	.weeks {
		grid-column: 2;
		grid-row: 2;
		display: flex;
		gap: 3px;
	}
	.week {
		display: grid;
		grid-template-rows: repeat(7, 12px);
		gap: 3px;
	}

	.cell {
		width: 12px;
		height: 12px;
		padding: 0;
		border-radius: 2px;
		border: none;
		background: var(--bg-hard);
		cursor: pointer;
	}
	.cell.l1 {
		background: color-mix(in srgb, var(--accent) 30%, var(--bg-hard));
	}
	.cell.l2 {
		background: color-mix(in srgb, var(--accent) 52%, var(--bg-hard));
	}
	.cell.l3 {
		background: color-mix(in srgb, var(--accent) 76%, var(--bg-hard));
	}
	.cell.l4 {
		background: var(--accent);
	}
	.cell:hover:not(:disabled) {
		outline: 1px solid var(--fg);
		outline-offset: 0;
	}
	.cell.outside {
		background: transparent;
		cursor: default;
	}

	.hint {
		margin: 8px 0 0;
		font-size: 10.5px;
		color: var(--fg-muted);
	}
	.empty {
		margin: 0;
		padding: 12px 14px;
		background: var(--bg-soft);
		border-radius: var(--radius);
		font-size: 12px;
		color: var(--fg-muted);
	}
</style>
