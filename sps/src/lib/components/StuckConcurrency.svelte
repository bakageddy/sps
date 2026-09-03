<script lang="ts">
  /**
   * Concurrency band: a step area chart of "episodes active at this
   * instant", sharing the page's zoom window with the strip above it.
   * A plateau reads as thread-pool exhaustion; isolated spikes read as
   * independent slow requests.
   *
   * Sweeps to zoom like every chart-shaped surface in the app (a surface
   * that looks sweepable must sweep); double-click resets.
   */
  import type { ConcurrencyPoint } from "$lib/stuckthread";

  interface Props {
    /** full-domain steps, ascending t (lib/stuckthread concurrencySteps) */
    points: ConcurrencyPoint[];
    /** full data domain */
    domain: [number, number];
    view: [number, number] | null;
    onviewchange: (view: [number, number] | null) => void;
  }

  let { points, domain, view, onviewchange }: Props = $props();

  const window_ = $derived(view ?? domain);

  // Steps clipped to the window: carry the running count into its start.
  const visible = $derived.by(() => {
    const [lo, hi] = window_;
    const out: ConcurrencyPoint[] = [];
    let carried = 0;
    for (const p of points) {
      if (p.t <= lo) {
        carried = p.count;
        continue;
      }
      if (out.length === 0) out.push({ t: lo, count: carried });
      if (p.t > hi) break;
      out.push(p);
    }
    if (out.length === 0) out.push({ t: lo, count: carried });
    return out;
  });

  const maxCount = $derived(Math.max(1, ...visible.map((p) => p.count)));

  // y scale shared by the paths and the axis: 0 at the floor, maxCount at
  // 90% of the height (10% headroom so the peak never kisses the top edge)
  const yPct = (count: number) => 100 - (count / maxCount) * 90;

  // integer y ticks at a nice step (1/2/5 × 10^k), targeting ~8 lines so
  // the scale stays readable between gridlines (maxCount 100 → step 20,
  // not a single 50/100 pair)
  const yTicks = $derived.by(() => {
    const raw = maxCount / 8;
    const pow = 10 ** Math.floor(Math.log10(Math.max(1, raw)));
    const step = [1, 2, 5, 10].map((m) => m * pow).find((s) => s >= raw) ?? pow * 10;
    const ticks: number[] = [];
    for (let c = step; c <= maxCount; c += step) ticks.push(c);
    return ticks;
  });

  // step-after path in a 0..100 viewBox (preserveAspectRatio="none"
  // stretches it to the band, so only ratios matter here)
  const linePath = $derived.by(() => {
    const [lo, hi] = window_;
    const span = hi - lo || 1;
    const x = (t: number) => ((t - lo) / span) * 100;
    let d = `M 0 ${yPct(visible[0].count).toFixed(2)}`;
    for (let i = 1; i < visible.length; i++) {
      d += ` H ${x(visible[i].t).toFixed(2)} V ${yPct(visible[i].count).toFixed(2)}`;
    }
    return d + " H 100";
  });
  const areaPath = $derived(`${linePath} V 100 H 0 Z`);

  // --- sweep-zoom (same capture-free pattern as the strip) -------------------
  let band = $state<HTMLDivElement>();
  let dragFracStart = $state<number | null>(null);
  let dragFracCurrent = $state<number | null>(null);
  const DRAG_THRESHOLD = 6; // px

  const dragging = $derived.by(() => {
    if (dragFracStart === null || dragFracCurrent === null || !band) return false;
    return Math.abs(dragFracCurrent - dragFracStart) * band.clientWidth > DRAG_THRESHOLD;
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
  <!-- y labels live in a gutter LEFT of the plot, on the same yPct scale -->
  <div class="gutter" aria-hidden="true">
    {#each yTicks as count (count)}
      <span class="ylabel" style:top="{yPct(count)}%">{count}</span>
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
      <path class="area" d={areaPath} />
      <path class="line" d={linePath} />
    </svg>

    {#each yTicks as count (count)}
      <span class="ygrid" style:top="{yPct(count)}%"></span>
    {/each}

    <span class="label">concurrent</span>
    <span class="max">max {maxCount}</span>

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
    /* keep the stroke 1.5 SCREEN px despite the stretched viewBox */
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
    right: 6px; /* right-aligned against the plot edge */
    transform: translateY(-50%); /* centered on its gridline */
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-muted);
  }

  .label,
  .max {
    position: absolute;
    top: 3px;
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--fg-muted);
    pointer-events: none;
  }
  .label {
    left: 8px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .max {
    right: 8px;
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
