<script lang="ts" module>
  /**
   * Generic time-series line chart (D3-for-math, Svelte-for-DOM — see the
   * instance script). Generalized from the original CpuChart when a second
   * and third chart (process CPU / process memory) wanted the same body:
   * the domain-specific parts turned out to be only the series LABEL and
   * the value UNIT, so those became inputs and the component became
   * reusable. That's the usual path to a good generic component — extract
   * it from the second/third concrete need, don't design it up front.
   */
  export interface MetricPoint {
    /** ms epoch */
    timestamp: number;
    value: number;
  }

  export interface LineSeries {
    /** identity passed back through onpointselect (a tid, a pid, ...) */
    id: number;
    /** legend/tooltip label, e.g. "tid 5227" or "java.exe (484)" */
    label: string;
    points: MetricPoint[];
  }
</script>

<script lang="ts">
  import Icon from "$lib/components/Icon.svelte";
  import { scaleUtc, scaleLinear } from "d3-scale";
  import { line as d3line, area as d3area } from "d3-shape";
  import { bisector, extent, max, min } from "d3-array";

  interface Props {
    series: LineSeries[];
    /** value unit suffix on ticks and tooltip, e.g. "%" or " MB" */
    unit: string;
    /** shown when series is empty */
    emptyText?: string;
    /** Called when the user clicks a sample point. */
    onpointselect?: (id: number, timestamp: number) => void;
    /** true while this chart fills the whole analysis area */
    expanded?: boolean;
    /** presence renders the expand/restore button; the PAGE owns what
     * "expanded" means (it swaps its pane layout for this chart alone) */
    ontoggleexpand?: () => void;
  }

  let {
    series,
    unit,
    emptyText = "Select something to plot.",
    onpointselect,
    expanded = false,
    ontoggleexpand,
  }: Props = $props();

  let width = $state(0);
  let height = $state(0);

  const margin = { top: 12, right: 16, bottom: 28 };

  // --- Zoom (glowroot-style: drag a time range, double-click to reset) --
  let zoom = $state<[number, number] | null>(null);
  let dragStart = $state<number | null>(null);
  let dragCurrent = $state<number | null>(null);
  const DRAG_THRESHOLD = 8; // px — below this, a "drag" is just a click

  const dragging = $derived(
    dragStart !== null &&
      dragCurrent !== null &&
      Math.abs(dragCurrent - dragStart) > DRAG_THRESHOLD,
  );

  // Everything downstream (scales, bins, hover) sees only the zoomed
  // window, so the y-axis rescales to the visible data too.
  const visibleSeries = $derived.by(() => {
    if (zoom === null) return series;
    const [z0, z1] = zoom;
    return series.map((s) => ({
      ...s,
      points: s.points.filter((p) => p.timestamp >= z0 && p.timestamp <= z1),
    }));
  });

  const allPoints = $derived(visibleSeries.flatMap((s) => s.points));

  // --- y domain: zero-baseline only when the data actually lives near
  // zero. A series hovering at ~15,000 MB against a 0-baseline renders as
  // a flat line with 95% empty chart below it; tightening the domain
  // around the data (padded) makes its variation visible. CPU-style data
  // (min near 0) keeps the honest zero baseline.
  const yDomain = $derived.by<[number, number]>(() => {
    if (allPoints.length === 0) return [0, 1];
    const lo = min(allPoints, (p) => p.value) ?? 0;
    const hi = max(allPoints, (p) => p.value) ?? 1;
    if (hi === lo) {
      const pad = hi === 0 ? 1 : Math.abs(hi) * 0.05;
      return [Math.min(0, lo - pad), hi + pad];
    }
    if (lo > hi * 0.3) {
      const pad = (hi - lo) * 0.08;
      return [Math.max(0, lo - pad), hi + pad];
    }
    return [0, hi];
  });

  // Ticks come from a range-less scale so the left margin can be computed
  // FROM the labels (widest label decides) without a circular dependency.
  const yTicks = $derived(scaleLinear().domain(yDomain).nice().ticks(5));

  // Compact labels: "16K MB", not a clipped "15663 MB".
  const compactY = new Intl.NumberFormat(undefined, {
    notation: "compact",
    maximumFractionDigits: 1,
  });

  const marginLeft = $derived.by(() => {
    const widest = Math.max(
      0,
      ...yTicks.map((t) => (compactY.format(t) + unit).length),
    );
    return Math.min(96, Math.max(44, 14 + widest * 6.8));
  });

  const x = $derived(
    scaleUtc()
      .domain(
        zoom ??
          (allPoints.length > 0
            ? (extent(allPoints, (p) => p.timestamp) as [number, number])
            : [Date.now() - 1, Date.now()]),
      )
      .range([marginLeft, Math.max(marginLeft + 1, width - margin.right)]),
  );

  const y = $derived(
    scaleLinear()
      .domain(yDomain)
      .nice()
      .range([Math.max(margin.top + 1, height - margin.bottom), margin.top]),
  );

  const toPath = $derived(
    d3line<MetricPoint>()
      .x((p) => x(p.timestamp))
      .y((p) => y(p.value)),
  );

  // ------------------------------------------------------------------
  // Overplotting defense: with more points than pixels, a line zigzags
  // inside every pixel column and renders as a noise blob. Above the
  // density threshold we bin per pixel column and draw min→max as a band
  // (spikes survive exactly — max is kept, never averaged) plus a mean
  // line for the trend. Sparse series keep the plain line.
  // ------------------------------------------------------------------
  interface Bin {
    px: number;
    min: number;
    max: number;
    mean: number;
  }

  const plotWidth = $derived(Math.max(1, width - marginLeft - margin.right));

  function binPerPixel(points: MetricPoint[]): Bin[] {
    const buckets = new Map<number, { min: number; max: number; sum: number; n: number }>();
    for (const p of points) {
      const px = Math.round(x(p.timestamp));
      const b = buckets.get(px);
      if (b === undefined) {
        buckets.set(px, { min: p.value, max: p.value, sum: p.value, n: 1 });
      } else {
        if (p.value < b.min) b.min = p.value;
        if (p.value > b.max) b.max = p.value;
        b.sum += p.value;
        b.n += 1;
      }
    }
    return [...buckets.entries()]
      .map(([px, b]) => ({ px, min: b.min, max: b.max, mean: b.sum / b.n }))
      .sort((a, b) => a.px - b.px);
  }

  const toBand = $derived(
    d3area<Bin>()
      .x((b) => b.px)
      .y0((b) => y(b.min))
      .y1((b) => y(b.max)),
  );
  const toMeanLine = $derived(
    d3line<Bin>()
      .x((b) => b.px)
      .y((b) => y(b.mean)),
  );

  // A series is "dense" when it averages 2+ points per pixel column.
  const rendered = $derived(
    visibleSeries.map((s) => ({
      series: s,
      bins: s.points.length > plotWidth * 2 ? binPerPixel(s.points) : null,
    })),
  );

  const xTicks = $derived(x.ticks(Math.max(2, Math.floor(width / 120))));

  // Axis labels adapt to the visible span: beyond a day, time-only labels
  // repeat ambiguously ("17:30 … 17:30"), so the date joins in.
  const spansDays = $derived.by(() => {
    const [start, end] = x.domain();
    return end.getTime() - start.getTime() > 24 * 60 * 60 * 1000;
  });
  const timeFormat = new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23", // 24-hour clock
  });
  const dayTimeFormat = new Intl.DateTimeFormat(undefined, {
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
    hourCycle: "h23",
  });
  const tickFormat = $derived(spansDays ? dayTimeFormat : timeFormat);

  interface Hover {
    series: LineSeries;
    point: MetricPoint;
    colorIndex: number;
  }

  let hover = $state<Hover | null>(null);

  const bisect = bisector<MetricPoint, number>((p) => p.timestamp).center;

  function eventPx(event: PointerEvent): number {
    const svg = event.currentTarget as SVGSVGElement;
    return event.clientX - svg.getBoundingClientRect().left;
  }

  function findNearest(event: PointerEvent): Hover | null {
    const t = x.invert(eventPx(event)).getTime();

    let best: Hover | null = null;
    let bestDistance = Infinity;
    for (const [i, s] of visibleSeries.entries()) {
      if (s.points.length === 0) continue;
      const point = s.points[bisect(s.points, t)];
      const distance = Math.abs(point.timestamp - t);
      if (distance < bestDistance) {
        bestDistance = distance;
        best = { series: s, point, colorIndex: i };
      }
    }
    return best;
  }

  // Click and drag share the button, so both are resolved on pointerup:
  // moved past the threshold = zoom to the dragged range, else = the old
  // point-click. Pointer capture keeps fast drags from escaping the svg.
  function onpointerdown(event: PointerEvent) {
    (event.currentTarget as Element).setPointerCapture(event.pointerId);
    dragStart = eventPx(event);
    dragCurrent = dragStart;
  }

  function onpointermove(event: PointerEvent) {
    if (dragStart !== null && event.buttons > 0) {
      dragCurrent = eventPx(event);
      hover = dragging ? null : findNearest(event);
    } else {
      hover = findNearest(event);
    }
  }

  function onpointerup(event: PointerEvent) {
    if (dragStart !== null && dragCurrent !== null && dragging) {
      const [a, b] = [Math.min(dragStart, dragCurrent), Math.max(dragStart, dragCurrent)];
      zoom = [x.invert(a).getTime(), x.invert(b).getTime()];
      hover = null;
    } else {
      const target = findNearest(event);
      if (target) onpointselect?.(target.series.id, target.point.timestamp);
    }
    dragStart = null;
    dragCurrent = null;
  }

  function color(i: number): string {
    return `var(--chart-${(i % 7) + 1})`;
  }
</script>

<div class="chart" bind:clientWidth={width} bind:clientHeight={height}>
  {#if series.length === 0}
    <p class="empty">{emptyText}</p>
  {:else if width > 0 && height > 0}
    <!-- Charts are inherently pointer-driven; the data itself stays reachable
         through the (keyboard-accessible) tables, so suppressing the
         pointer-only lint on the svg is a deliberate trade-off. -->
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions, a11y_no_noninteractive_element_interactions -->
    <svg
      {width}
      {height}
      role="img"
      aria-label="Usage over time"
      {onpointerdown}
      {onpointermove}
      {onpointerup}
      onpointerleave={() => (hover = null)}
      ondblclick={() => (zoom = null)}
    >
      {#each yTicks as tick (tick)}
        <line
          class="grid"
          x1={marginLeft}
          x2={width - margin.right}
          y1={y(tick)}
          y2={y(tick)}
        />
        <text class="tick" x={marginLeft - 8} y={y(tick)} dy="0.32em" text-anchor="end">
          {compactY.format(tick)}{unit}
        </text>
      {/each}

      {#each xTicks as tick (tick.getTime())}
        <text
          class="tick"
          x={x(tick)}
          y={height - margin.bottom + 16}
          text-anchor="middle"
        >
          {tickFormat.format(tick)}
        </text>
      {/each}

      {#each rendered as r, i (r.series.id)}
        {#if r.bins}
          <!-- dense: min–max band + mean line -->
          <path class="band" d={toBand(r.bins)} style:fill={color(i)} />
          <path class="series mean" d={toMeanLine(r.bins)} style:stroke={color(i)} />
        {:else}
          <path class="series" d={toPath(r.series.points)} style:stroke={color(i)} />
        {/if}
      {/each}

      {#if dragging && dragStart !== null && dragCurrent !== null}
        <rect
          class="zoom-selection"
          x={Math.min(dragStart, dragCurrent)}
          y={margin.top}
          width={Math.abs(dragCurrent - dragStart)}
          height={Math.max(0, height - margin.top - margin.bottom)}
        />
      {/if}

      {#if hover}
        <line
          class="crosshair"
          x1={x(hover.point.timestamp)}
          x2={x(hover.point.timestamp)}
          y1={margin.top}
          y2={height - margin.bottom}
        />
        <circle
          cx={x(hover.point.timestamp)}
          cy={y(hover.point.value)}
          r="4"
          style:fill={color(hover.colorIndex)}
        />
      {/if}
    </svg>

    {#if hover}
      <!-- HTML tooltip on purpose: text layout is painful in SVG. -->
      <div
        class="tooltip"
        style:left="{Math.min(x(hover.point.timestamp) + 12, width - 180)}px"
        style:top="{y(hover.point.value) - 8}px"
      >
        <strong>{hover.series.label}</strong>
        <span>
          {hover.point.value.toFixed(2)}{unit} at {tickFormat.format(hover.point.timestamp)}
        </span>
      </div>
    {/if}

    {#if zoom}
      <button
        class="zoom-reset"
        style:left="{marginLeft + 8}px"
        onclick={() => (zoom = null)}
        title="Reset zoom (or double-click the chart)"
      >reset</button>
    {/if}

    <div class="corner">
      <div class="legend">
        {#each series as s, i (s.id)}
          <span class="legend-item">
            <span class="swatch" style:background={color(i)}></span>
            {s.label}
          </span>
        {/each}
      </div>
      {#if ontoggleexpand}
        <button
          class="expand"
          onclick={ontoggleexpand}
          title={expanded ? "Restore layout" : "Expand chart"}
        >
          <Icon name={expanded ? "compress" : "expand"} size={12} />
        </button>
      {/if}
    </div>
  {/if}
</div>

<style>
  .chart {
    position: relative;
    width: 100%;
    height: 100%;
    min-height: 120px;
  }

  svg {
    display: block;
    cursor: crosshair; /* hints that the plot is drag-selectable */
    touch-action: none;
  }

  .zoom-selection {
    fill: var(--accent);
    opacity: 0.15;
    stroke: var(--accent);
    stroke-opacity: 0.6;
  }

  .zoom-reset {
    position: absolute;
    top: 6px; /* left set inline, just right of the y-axis labels */
    padding: 3px 12px;
    font-size: 11px;
    font-weight: 600;
    background: var(--accent);
    border: none;
    border-radius: var(--radius);
    color: var(--bg-hard);
    cursor: pointer;
  }
  .zoom-reset:hover {
    opacity: 0.85;
  }

  .grid {
    stroke: var(--border);
    stroke-dasharray: 2 4;
  }

  .tick {
    fill: var(--fg-muted);
    font-size: 11px;
    font-family: var(--font-mono);
  }

  .series {
    fill: none;
    stroke-width: 1.5;
  }
  .series.mean {
    stroke-width: 1;
  }
  .band {
    opacity: 0.35;
    stroke: none;
  }

  .crosshair {
    stroke: var(--fg-muted);
    stroke-dasharray: 3 3;
  }

  .tooltip {
    position: absolute;
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 10px;
    background: var(--bg-hard);
    border: none;
    box-shadow: var(--shadow); /* elevation, not outline */
    border-radius: var(--radius);
    font-size: 12px;
    pointer-events: none;
    white-space: nowrap;
  }

  .corner {
    position: absolute;
    top: 4px;
    right: 12px;
    display: flex;
    align-items: flex-start;
    gap: 10px;
    max-width: 70%;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: 4px 12px;
    font-size: 12px;
    color: var(--fg-muted);
    pointer-events: none;
  }

  .expand {
    display: grid;
    place-items: center;
    padding: 4px;
    background: var(--bg-soft);
    border: none;
    border-radius: var(--radius);
    color: var(--fg-muted);
    cursor: pointer;
  }
  .expand:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .legend-item {
    display: inline-flex;
    align-items: center;
    gap: 5px;
  }
  .swatch {
    width: 10px;
    height: 3px;
    border-radius: 2px;
  }

  .empty {
    display: grid;
    place-items: center;
    height: 100%;
    margin: 0;
    color: var(--fg-muted);
    font-size: 13px;
    text-align: center;
  }
</style>
