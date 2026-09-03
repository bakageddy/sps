<script lang="ts">
  /**
   * Overview strip = the zoom controller for the whole page (DevTools
   * pattern: the mini timeline brushes the detail view).
   *
   *  - drag horizontally  → select a time window (the table follows)
   *  - click a bar        → select that episode (drag vs click resolved
   *                         by a pixel threshold on pointerup, same trick
   *                         as LineChart)
   *  - wheel              → pan the window when zoomed
   *  - double-click       → reset to full range
   *
   * The strip renders the ACTIVE WINDOW (glowroot-style: the chart itself
   * rescales). An earlier fixed-map-plus-lens design kept reading as
   * "zoom is broken" — with a seconds-wide window on an hours-wide log the
   * lens moved sub-pixel, so zoom clicks changed nothing visible.
   */
  import type { StuckBar } from "$lib/stuckthread";
  import { formatTimestamp } from "$lib/format";

  interface Props {
    bars: StuckBar[];
    selected: { tid: number; timestamp: number } | null;
    onselect: (bar: StuckBar) => void;
    /** active zoom window, null = full range */
    view: [number, number] | null;
    onviewchange: (view: [number, number] | null) => void;
  }

  let { bars, selected, onselect, view, onviewchange }: Props = $props();

  /** the whole log — pan clamping + "zoomed all the way out" detection */
  const full = $derived.by<[number, number]>(() => {
    if (bars.length === 0) return [0, 1];
    const start = Math.min(...bars.map((b) => b.timestamp));
    const end = Math.max(...bars.map((b) => b.timestamp + b.durationMs));
    return end > start ? [start, end] : [start, start + 1];
  });

  /** what the strip actually draws */
  const domain = $derived<[number, number]>(view ?? full);

  const pct = (t: number) => ((t - domain[0]) / (domain[1] - domain[0])) * 100;
  const timeAt = (frac: number) => domain[0] + frac * (domain[1] - domain[0]);

  // Greedy lane packing over ALL bars, start-sorted — deliberately NOT
  // window-filtered: lane assignment must be stable across pan/zoom. A
  // per-window packing reshuffled lanes (and the lane count, i.e. every
  // line's y) whenever a bar entered or left the window, which read as
  // the chart reordering itself mid-pan.
  const lanes = $derived.by(() => {
    const laneEnds: number[] = [];
    const placed = bars
      .toSorted((a, b) => a.timestamp - b.timestamp)
      .map((bar) => {
        const end = bar.timestamp + bar.durationMs;
        let lane = laneEnds.findIndex((e) => e <= bar.timestamp);
        if (lane === -1) {
          lane = laneEnds.length;
          laneEnds.push(end);
        } else {
          laneEnds[lane] = end;
        }
        return { bar, lane };
      });
    return { placed, count: Math.max(1, laneEnds.length) };
  });

  /** only bars intersecting the window reach the DOM (lanes stay global) */
  const visibleBars = $derived(
    lanes.placed.filter(
      ({ bar }) => bar.timestamp <= domain[1] && bar.timestamp + bar.durationMs >= domain[0],
    ),
  );

  // --- fixed lane pitch + overflow ------------------------------------------
  // Lines never scale or overlap: each lane owns LANE_PITCH px, the strip
  // shows as many lanes as fit, and a counter says how many episodes are
  // clipped below. Dragging the split divider taller reveals them.
  const LANE_PITCH = 12; // 8px line + 4px breathing room
  const LINE_HEIGHT = 8;
  let stripHeight = $state(0);
  const laneCapacity = $derived(Math.max(1, Math.floor(stripHeight / LANE_PITCH)));
  const rendered = $derived(visibleBars.filter(({ lane }) => lane < laneCapacity));
  const hiddenCount = $derived(visibleBars.length - rendered.length);

  const ticks = $derived.by(() => {
    const n = 8;
    return Array.from({ length: n + 1 }, (_, i) => ({
      pct: (i / n) * 100,
      t: timeAt(i / n),
    }));
  });

  // Tick precision follows the window: a multi-day span needs the day or
  // wrapped HH:MM ticks are ambiguous; a minutes span needs seconds.
  const timeFormat = $derived.by(() => {
    const span = domain[1] - domain[0];
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

  const isSelected = (bar: StuckBar) =>
    selected !== null && selected.tid === bar.tid && selected.timestamp === bar.timestamp;

  // Entry lines are colored per-thread from the theme palette. Red is
  // deliberately absent — it means "stuck/unresolved" elsewhere on this
  // page and must not be handed out at random.
  const PALETTE = ["--blue", "--green", "--yellow", "--purple", "--aqua", "--accent"] as const;
  const barColor = (tid: number) => PALETTE[tid % PALETTE.length];

  // --- drag-to-zoom (capture-free: window listeners run the drag so the
  // bar buttons keep their clicks; a captured pointer would steal them) ---
  let strip = $state<HTMLDivElement>();
  let dragFracStart = $state<number | null>(null);
  let dragFracCurrent = $state<number | null>(null);
  let justDragged = false;
  const DRAG_THRESHOLD = 6; // px

  const dragging = $derived.by(() => {
    if (dragFracStart === null || dragFracCurrent === null || !strip) return false;
    return Math.abs(dragFracCurrent - dragFracStart) * strip.clientWidth > DRAG_THRESHOLD;
  });

  function frac(clientX: number): number {
    const rect = strip!.getBoundingClientRect();
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
      const [a, b] = [
        Math.min(dragFracStart, dragFracCurrent),
        Math.max(dragFracStart, dragFracCurrent),
      ];
      onviewchange([timeAt(a), timeAt(b)]);
      justDragged = true;
    }
    dragFracStart = null;
    dragFracCurrent = null;
  }

  function onbarclick(bar: StuckBar) {
    if (justDragged) {
      justDragged = false;
      return;
    }
    onselect(bar);
  }

  // Wheel pans the window along the full log.
  function onwheel(event: WheelEvent) {
    if (view === null) return;
    event.preventDefault();
    const span = view[1] - view[0];
    const fullSpan = full[1] - full[0];
    const step = (event.deltaY + event.deltaX) * (span / 500);
    let lo = view[0] + step;
    let hi = view[1] + step;
    if (lo < full[0]) [lo, hi] = [full[0], full[0] + span];
    if (hi > full[1]) [lo, hi] = [full[1] - span, full[1]];
    onviewchange(span >= fullSpan ? null : [lo, hi]);
  }
</script>

<svelte:window onpointermove={onwindowmove} onpointerup={onwindowup} />

<div class="overview">
  <div class="axis">
    {#each ticks as tick, i (i)}
      <span class="tick" style:left="{tick.pct}%">{formatTimestamp(timeFormat, tick.t)}</span>
    {/each}
  </div>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="strip"
    bind:this={strip}
    bind:clientHeight={stripHeight}
    {onpointerdown}
    {onwheel}
    ondblclick={() => onviewchange(null)}
  >
    {#each ticks as tick, i (i)}
      <span class="gridline" style:left="{tick.pct}%"></span>
    {/each}

    <!-- unkeyed: duplicate identities in corrupt data must not crash the each -->
    {#each rendered as { bar, lane }}
      <!-- a fixed-size line per episode, one LANE_PITCH row per lane -->
      <button
        class="bar"
        class:selected={isSelected(bar)}
        style:--bar-color="var({barColor(bar.tid)})"
        style:left="{pct(bar.timestamp)}%"
        style:width="{Math.max(0.3, pct(bar.timestamp + bar.durationMs) - pct(bar.timestamp))}%"
        style:top="{lane * LANE_PITCH + (LANE_PITCH - LINE_HEIGHT) / 2}px"
        onclick={() => onbarclick(bar)}
        aria-label="Stuck episode, tid {bar.tid}"
      ></button>
    {/each}

    {#if hiddenCount > 0}
      <!-- clipped lanes exist: fade the cutoff + count what's below -->
      <span class="fade"></span>
      <span class="overflow">▾ {hiddenCount} more</span>
    {/if}

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
  .overview {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    padding: 4px 10px 8px;
  }

  .axis {
    position: relative;
    height: 16px;
    flex-shrink: 0;
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

  .strip {
    position: relative;
    flex: 1;
    min-height: 24px;
    background: var(--bg-hard);
    border-radius: var(--radius);
    overflow: hidden;
    cursor: crosshair;
    touch-action: none;
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

  .gridline {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--hairline);
  }

  .fade {
    position: absolute;
    left: 0;
    right: 0;
    bottom: 0;
    height: 14px;
    background: linear-gradient(transparent, var(--bg-hard));
    pointer-events: none;
  }

  .overflow {
    position: absolute;
    right: 8px;
    bottom: 4px;
    padding: 1px 8px;
    border-radius: 999px;
    background: var(--bg-soft);
    color: var(--fg-muted);
    font-family: var(--font-mono);
    font-size: 10.5px;
    pointer-events: none;
  }

  .bar {
    position: absolute;
    height: 8px; /* a fixed line, never scaled — overflow is clipped + counted */
    min-width: 3px;
    border-radius: 2px;
    background: color-mix(in srgb, var(--bar-color) 35%, transparent);
    border: 1.5px solid var(--bar-color);
    padding: 0;
  }
  .bar:hover {
    background: var(--bar-color);
  }
  .bar.selected {
    background: var(--bar-color);
    outline: 2px solid var(--fg);
    outline-offset: 1px;
  }

</style>
