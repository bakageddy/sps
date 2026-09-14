<script lang="ts">
	/**
	 * Pool occupancy band: step chart of used connections with the capacity
	 * (max_connections) as a dashed ceiling — used touching the ceiling IS
	 * exhaustion. Alarm signals render as ticks along the top edge, colored
	 * by cause (suppressed alarms translucent).
	 *
	 * Sweeps to zoom like every chart-shaped surface in the app;
	 * double-click resets. Same skeleton as StuckConcurrency on purpose.
	 */
	import type {
		ConnDumpPoolStats,
		ConnDumpSignal,
	} from "$lib/api/connectiondump";
	import { causeColor } from "$lib/connectiondump";

	interface Props {
		/** ascending timestamp */
		stats: ConnDumpPoolStats[];
		signals: ConnDumpSignal[];
		/** full data domain */
		domain: [number, number];
		view: [number, number] | null;
		onviewchange: (view: [number, number] | null) => void;
	}

	let { stats, signals, domain, view, onviewchange }: Props = $props();

	const window_ = $derived(view ?? domain);

	// Stats clipped to the window, carrying the last row into its start so
	// the step is continuous at the left edge.
	const visible = $derived.by(() => {
		const [lo, hi] = window_;
		const out: ConnDumpPoolStats[] = [];
		let carried: ConnDumpPoolStats | null = null;
		for (const s of stats) {
			if (s.timestamp <= lo) {
				carried = s;
				continue;
			}
			if (out.length === 0 && carried !== null)
				out.push({ ...carried, timestamp: lo });
			if (s.timestamp > hi) break;
			out.push(s);
		}
		if (out.length === 0 && carried !== null)
			out.push({ ...carried, timestamp: lo });
		return out;
	});

	const visibleSignals = $derived(
		signals.filter(
			(s) => s.timestamp >= window_[0] && s.timestamp <= window_[1],
		),
	);

	// y max = pool capacity (used can never exceed it); fall back to used
	// in case a stats line ever lost its max_connections.
	const yMax = $derived(
		Math.max(1, ...visible.map((s) => Math.max(s.total, s.used))),
	);

	// 0 at the floor, yMax at 88% height — headroom keeps the capacity line
	// clear of the signal ticks living along the top edge.
	const yPct = (v: number) => 100 - (v / yMax) * 88;

	const xPct = (t: number) => {
		const [lo, hi] = window_;
		return ((t - lo) / (hi - lo || 1)) * 100;
	};

	// integer y ticks at a nice step (1/2/5 × 10^k), ~6 lines
	const yTicks = $derived.by(() => {
		const raw = yMax / 6;
		const pow = 10 ** Math.floor(Math.log10(Math.max(1, raw)));
		const step =
			[1, 2, 5, 10].map((m) => m * pow).find((s) => s >= raw) ?? pow * 10;
		const ticks: number[] = [];
		for (let v = step; v <= yMax; v += step) ticks.push(v);
		return ticks;
	});

	// step-after paths in a 0..100 viewBox (preserveAspectRatio="none")
	function stepPath(value: (s: ConnDumpPoolStats) => number): string {
		if (visible.length === 0) return "";
		let d = `M ${xPct(visible[0].timestamp).toFixed(2)} ${yPct(value(visible[0])).toFixed(2)}`;
		for (let i = 1; i < visible.length; i++) {
			d += ` H ${xPct(visible[i].timestamp).toFixed(2)} V ${yPct(value(visible[i])).toFixed(2)}`;
		}
		return d + " H 100";
	}
	const usedPath = $derived(stepPath((s) => s.used));
	const usedArea = $derived(
		usedPath === "" ? "" : `${usedPath} V 100 H ${xPct(visible[0].timestamp).toFixed(2)} Z`,
	);
	const capacityPath = $derived(stepPath((s) => s.total));

	const signalTimeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	// --- sweep-zoom (same capture-free pattern as StuckConcurrency) -----------
	let band = $state<HTMLDivElement>();
	let dragFracStart = $state<number | null>(null);
	let dragFracCurrent = $state<number | null>(null);
	const DRAG_THRESHOLD = 6; // px

	const dragging = $derived.by(() => {
		if (dragFracStart === null || dragFracCurrent === null || !band)
			return false;
		return (
			Math.abs(dragFracCurrent - dragFracStart) * band.clientWidth >
			DRAG_THRESHOLD
		);
	});

	function frac(clientX: number): number {
		const rect = band!.getBoundingClientRect();
		return Math.min(1, Math.max(0, (clientX - rect.left) / rect.width));
	}

	function onpointerdown(event: PointerEvent) {
		if (event.button !== 0) return;
		dragFracStart = frac(event.clientX);
		dragFracCurrent = dragFracStart;
	}

	function onwindowmove(event: PointerEvent) {
		if (dragFracStart === null || event.buttons === 0) return;
		dragFracCurrent = frac(event.clientX);
	}

	function onwindowup() {
		if (dragFracStart !== null && dragFracCurrent !== null && dragging) {
			const [lo, hi] = window_;
			const span = hi - lo;
			const [a, b] = [
				Math.min(dragFracStart, dragFracCurrent),
				Math.max(dragFracStart, dragFracCurrent),
			];
			onviewchange([lo + a * span, lo + b * span]);
		}
		dragFracStart = null;
		dragFracCurrent = null;
	}
</script>

<svelte:window onpointermove={onwindowmove} onpointerup={onwindowup} />

<div class="wrap">
	<div class="gutter" aria-hidden="true">
		{#each yTicks as v (v)}
			<span class="ylabel" style:top="{yPct(v)}%">{v}</span>
		{/each}
	</div>

	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="band"
		bind:this={band}
		{onpointerdown}
		ondblclick={() => onviewchange(null)}
	>
		<svg viewBox="0 0 100 100" preserveAspectRatio="none" aria-hidden="true">
			<path class="area" d={usedArea} />
			<path class="capacity" d={capacityPath} />
			<path class="line" d={usedPath} />
		</svg>

		{#each yTicks as v (v)}
			<span class="ygrid" style:top="{yPct(v)}%"></span>
		{/each}

		<!-- alarm ticks along the top edge -->
		{#each visibleSignals as s, i (i)}
			<span
				class="signal"
				class:suppressed={s.suppressed}
				style:left="{xPct(s.timestamp)}%"
				style:background={causeColor(s.cause)}
				title="{s.cause}{s.suppressed ? ' (dump suppressed)' : ''} — {signalTimeFormat.format(s.timestamp)}"
			></span>
		{/each}

		<span class="label">connections in use</span>

		{#if dragging && dragFracStart !== null && dragFracCurrent !== null}
			<span
				class="brush"
				style:left="{Math.min(dragFracStart, dragFracCurrent) * 100}%"
				style:width="{Math.abs(dragFracCurrent - dragFracStart) * 100}%"
			></span>
		{/if}
	</div>
</div>

<style>
	.wrap {
		display: flex;
		height: 100%;
	}

	.gutter {
		position: relative;
		width: 30px;
		flex-shrink: 0;
	}

	.band {
		position: relative;
		flex: 1;
		min-width: 0;
		height: 100%;
		background: var(--bg-hard);
		border-radius: var(--radius);
		overflow: hidden;
		cursor: crosshair;
		touch-action: none;
	}

	svg {
		position: absolute;
		inset: 0;
		width: 100%;
		height: 100%;
	}

	.area {
		fill: color-mix(in srgb, var(--accent) 18%, transparent);
		stroke: none;
	}
	.line {
		fill: none;
		stroke: var(--accent);
		stroke-width: 1.5;
		vector-effect: non-scaling-stroke;
	}
	.capacity {
		fill: none;
		stroke: var(--red);
		stroke-width: 1;
		stroke-dasharray: 4 3;
		vector-effect: non-scaling-stroke;
	}

	.ygrid {
		position: absolute;
		left: 0;
		right: 0;
		height: 1px;
		background: var(--hairline);
		pointer-events: none;
	}
	.ylabel {
		position: absolute;
		right: 6px;
		transform: translateY(-50%);
		font-family: var(--font-mono);
		font-size: 10px;
		color: var(--fg-muted);
	}

	.signal {
		position: absolute;
		top: 0;
		width: 2px;
		height: 10px;
		transform: translateX(-50%);
	}
	.signal.suppressed {
		opacity: 0.4;
	}

	.label {
		position: absolute;
		top: 3px;
		left: 8px;
		font-family: var(--font-mono);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--fg-muted);
		pointer-events: none;
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
