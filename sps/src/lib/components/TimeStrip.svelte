<script lang="ts">
	/**
	 * Global time strip — every dump of every log kind as a dot on one
	 * shared axis, one lane per kind. Sweep to set the app-wide time
	 * window (lib/timewindow.svelte.ts); double-click clears it. Lane
	 * labels jump to that analyzer.
	 *
	 * Bounded DOM whatever the bundle size: one <path> per lane carries all
	 * its dots. Dots are drawn in PIXEL space (x from the measured width),
	 * not a stretched viewBox, so they stay round.
	 */
	import { goto } from "$app/navigation";
	import { connectiondumpSignals } from "$lib/api/connectiondump";
	import { threaddumpDumps } from "$lib/api/threaddump";
	import { cpuDumps } from "$lib/api/cpumonitoring";
	import { cpuMemDumps } from "$lib/api/cpumemstats";
	import { stuckthreadListview } from "$lib/api/stuckthread";
	import {
		stuckqueryMssqlSnapshots,
		stuckqueryPgsqlSnapshots,
	} from "$lib/api/stuckquery";
	import {
		runningqueryMssqlSnapshots,
		runningqueryPgsqlSnapshots,
	} from "$lib/api/runningquery";
	import { cached } from "$lib/query-cache";
	import { db } from "$lib/database.svelte";
	import { ingest } from "$lib/ingest.svelte";
	import { persisted } from "$lib/persisted.svelte";
	import { timeWindow, bundleDomain } from "$lib/timewindow.svelte";
	import { formatTimestamp } from "$lib/format";

	interface Lane {
		key: string;
		label: string;
		href: string;
		color: string;
		times: number[];
	}

	const LANE_H = 14;
	const PAD_TOP = 4;

	let lanes = $state<Lane[]>([]);
	const collapsed = persisted("time-strip-collapsed", true);

	/** unwrap an allSettled slot; failures leave an empty lane (strip is decoration) */
	const ok = <T,>(r: PromiseSettledResult<T[]>): T[] =>
		r.status === "fulfilled" ? r.value : [];

	async function load() {
		const [sig, td, cpu, mem, stuck, sqm, sqp, rqm, rqp] = await Promise.allSettled([
			cached("connectiondump_signals|false", () => connectiondumpSignals(false)),
			cached("threaddump_dumps", threaddumpDumps),
			cached("cpu_dumps", cpuDumps),
			cached("cpumem_dumps", cpuMemDumps),
			cached("stuckthread_listview", () => stuckthreadListview()),
			cached("stuckquery_mssql_snapshots", stuckqueryMssqlSnapshots),
			cached("stuckquery_pgsql_snapshots", stuckqueryPgsqlSnapshots),
			cached("runningquery_mssql_snapshots", runningqueryMssqlSnapshots),
			cached("runningquery_pgsql_snapshots", runningqueryPgsqlSnapshots),
		]);
		const ts = (rows: { timestamp: number }[]) => rows.map((r) => r.timestamp);
		lanes = [
			{ key: "signals", label: "signals", href: "/connectiondump/incident", color: "var(--chart-2)", times: ts(ok(sig)) },
			{ key: "threaddump", label: "thread dumps", href: "/threaddump", color: "var(--chart-4)", times: ts(ok(td)) },
			{ key: "cpu", label: "cpu", href: "/cpumonitoring", color: "var(--yellow)", times: ts(ok(cpu)) },
			{ key: "cpumem", label: "cpu/mem", href: "/cpumemstats", color: "var(--chart-6)", times: ts(ok(mem)) },
			{
				key: "stuck",
				label: "stuck threads",
				href: "/stuckthreads",
				color: "var(--alert)",
				times: ok(stuck).flatMap((s) => (s.begin === null ? [] : [s.begin])),
			},
			{ key: "stuckq", label: "stuck queries", href: "/stuckqueries", color: "var(--red)", times: [...ts(ok(sqm)), ...ts(ok(sqp))] },
			{ key: "rq", label: "running queries", href: "/runningqueries", color: "var(--green)", times: [...ts(ok(rqm)), ...ts(ok(rqp))] },
		].filter((l) => l.times.length > 0);
	}

	$effect(() => {
		if (db.state.status === "open") {
			void db.epoch;
			load();
		} else lanes = [];
	});
	$effect(() => {
		if (ingest.generation === 0) return;
		load();
	});

	const domain = $derived.by<[number, number] | null>(() => {
		let lo = Infinity;
		let hi = -Infinity;
		for (const l of lanes)
			for (const t of l.times) {
				if (t < lo) lo = t;
				if (t > hi) hi = t;
			}
		return lo === Infinity ? null : [lo, hi === lo ? lo + 1 : hi];
	});

	$effect(() => {
		bundleDomain.value = domain;
	});

	// a window from a previous bundle that no longer overlaps this one is
	// stale — drop it rather than show every list empty
	$effect(() => {
		const w = timeWindow.value;
		const d = domain;
		if (w === null || d === null) return;
		if (w[1] < d[0] || w[0] > d[1]) timeWindow.value = null;
	});

	let width = $state(0);
	const xOf = (t: number) => {
		const d = domain!;
		return ((t - d[0]) / (d[1] - d[0])) * width;
	};

	// one path per lane: "M x-r y a r r 0 1 0 2r 0 a r r 0 1 0 -2r 0" per dot
	const R = 2.5;
	const paths = $derived(
		domain === null || width === 0
			? []
			: lanes.map((l, i) => {
					const y = PAD_TOP + i * LANE_H + LANE_H / 2;
					let d = "";
					for (const t of l.times) {
						const x = xOf(t).toFixed(1);
						d += `M${(Number(x) - R).toFixed(1)} ${y}a${R} ${R} 0 1 0 ${2 * R} 0a${R} ${R} 0 1 0 ${-2 * R} 0`;
					}
					return { lane: l, d };
				}),
	);
	const height = $derived(PAD_TOP * 2 + lanes.length * LANE_H);

	// --- sweep -----------------------------------------------------------
	let band = $state<HTMLDivElement>();
	let dragStart = $state<number | null>(null);
	let dragNow = $state<number | null>(null);
	const dragging = $derived(
		dragStart !== null && dragNow !== null && Math.abs(dragNow - dragStart) > 4,
	);
	function px(clientX: number): number {
		const rect = band!.getBoundingClientRect();
		return Math.min(rect.width, Math.max(0, clientX - rect.left));
	}
	function onpointerdown(e: PointerEvent) {
		if (e.button !== 0 || domain === null) return;
		dragStart = px(e.clientX);
		dragNow = dragStart;
	}
	function onwindowmove(e: PointerEvent) {
		if (dragStart === null || e.buttons === 0) return;
		dragNow = px(e.clientX);
	}
	function onwindowup() {
		if (dragStart !== null && dragNow !== null && dragging && domain !== null) {
			const [a, b] = [Math.min(dragStart, dragNow), Math.max(dragStart, dragNow)];
			const span = domain[1] - domain[0];
			timeWindow.value = [
				Math.round(domain[0] + (a / width) * span),
				Math.round(domain[0] + (b / width) * span),
			];
		}
		dragStart = null;
		dragNow = null;
	}

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
	const windowPx = $derived.by(() => {
		const w = timeWindow.value;
		if (w === null || domain === null || width === 0) return null;
		return [xOf(w[0]), xOf(w[1])] as [number, number];
	});
</script>

<svelte:window onpointermove={onwindowmove} onpointerup={onwindowup} />

{#if lanes.length > 0}
	<div class="strip" class:collapsed={collapsed.value}>
		<div class="bar">
			<button
				class="fold"
				onclick={() => (collapsed.value = !collapsed.value)}
				title={collapsed.value ? "Show the time strip" : "Hide the time strip"}
				aria-expanded={!collapsed.value}>{collapsed.value ? "▸" : "▾"}</button
			>
			<span class="title">time window</span>
			{#if timeWindow.value !== null}
				<span class="range mono">
					{formatTimestamp(timeFormat, timeWindow.value[0])} → {formatTimestamp(
						timeFormat,
						timeWindow.value[1],
					)}
				</span>
				<button class="clear" onclick={() => (timeWindow.value = null)}
					>clear</button
				>
			{:else if domain !== null}
				<span class="range muted mono">
					full bundle · {formatTimestamp(timeFormat, domain[0])} → {formatTimestamp(
						timeFormat,
						domain[1],
					)}
				</span>
				<span class="hint muted">sweep to narrow every analyzer</span>
			{/if}
		</div>

		{#if !collapsed.value}
			<div class="lanes">
				<div class="labels" style:padding-top="{PAD_TOP}px">
					{#each lanes as l (l.key)}
						<button
							class="label"
							style:height="{LANE_H}px"
							style:color={l.color}
							onclick={() => goto(l.href)}
							title="{l.times.length} · open {l.label}">{l.label}</button
						>
					{/each}
				</div>
				<!-- svelte-ignore a11y_no_static_element_interactions -->
				<div
					class="band"
					bind:this={band}
					bind:clientWidth={width}
					style:height="{height}px"
					{onpointerdown}
					ondblclick={() => (timeWindow.value = null)}
				>
					<svg {width} {height} aria-hidden="true">
						{#if windowPx}
							<rect
								class="window"
								x={windowPx[0]}
								y="0"
								width={Math.max(1, windowPx[1] - windowPx[0])}
								{height}
							/>
						{/if}
						{#each paths as p (p.lane.key)}
							<path d={p.d} fill={p.lane.color} />
						{/each}
					</svg>
					{#if dragging && dragStart !== null && dragNow !== null}
						<span
							class="brush"
							style:left="{Math.min(dragStart, dragNow)}px"
							style:width="{Math.abs(dragNow - dragStart)}px"
						></span>
					{/if}
				</div>
			</div>
		{/if}
	</div>
{/if}

<style>
	.strip {
		flex-shrink: 0;
		border-bottom: 1px solid var(--hairline);
		background: var(--bg-soft);
	}
	.bar {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 3px 10px;
		font-size: 11px;
	}
	.fold {
		padding: 0 4px;
		color: var(--fg-muted);
		font-size: 11px;
	}
	.title {
		text-transform: uppercase;
		letter-spacing: 0.06em;
		font-weight: 600;
		color: var(--fg-muted);
	}
	.range {
		font-size: 11px;
		color: var(--accent);
	}
	.muted {
		color: var(--fg-muted);
	}
	.hint {
		font-size: 10.5px;
		margin-left: auto;
	}
	.mono {
		font-family: var(--font-mono);
	}
	.clear {
		padding: 0 10px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font-size: 10.5px;
		font-weight: 600;
	}
	.clear:hover {
		background: var(--bg-hover);
	}

	.lanes {
		display: flex;
		padding: 0 10px 4px;
	}
	.labels {
		display: flex;
		flex-direction: column;
		width: 110px;
		flex-shrink: 0;
	}
	.label {
		padding: 0 6px 0 0;
		text-align: right;
		font-family: var(--font-mono);
		font-size: 10px;
		line-height: 1;
		border-radius: 0;
		opacity: 0.85;
	}
	.label:hover {
		opacity: 1;
		text-decoration: underline;
	}
	.band {
		position: relative;
		flex: 1;
		min-width: 0;
		background: var(--bg-hard);
		border-radius: var(--radius);
		cursor: crosshair;
		touch-action: none;
		overflow: hidden;
	}
	svg {
		display: block;
	}
	.window {
		fill: color-mix(in srgb, var(--accent) 18%, transparent);
		stroke: var(--accent);
		stroke-width: 1;
	}
	.brush {
		position: absolute;
		top: 0;
		bottom: 0;
		background: color-mix(in srgb, var(--accent) 18%, transparent);
		border-left: 1px solid var(--accent);
		border-right: 1px solid var(--accent);
		pointer-events: none;
	}
</style>
