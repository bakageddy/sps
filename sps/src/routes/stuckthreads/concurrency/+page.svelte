<script lang="ts">
  /**
   * Concurrency page: full-height step chart of "episodes stuck at this
   * instant" over the whole log. The shape is the diagnosis — a plateau
   * near the server's maxThreads is thread-pool exhaustion, isolated
   * spikes are independent slow requests.
   *
   * Shares the cached stuckthread_listview fetch with the main analyzer,
   * so flipping between the pages costs nothing.
   */
  import { stuckthreadListview, type StuckThread } from "$lib/api/stuckthread";
  import { bounds, concurrencySteps } from "$lib/stuckthread";
  import StuckConcurrency from "$lib/components/StuckConcurrency.svelte";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import { cached } from "$lib/query-cache";
  import { formatTimestamp } from "$lib/format";

  let errorMessage = $state<string | null>(null);
  let threads = $state<StuckThread[]>([]);
  /** zoom window; null = full range */
  let view = $state<[number, number] | null>(null);

  const steps = $derived(concurrencySteps(threads));

  const domain = $derived.by<[number, number]>(() => {
    let start = Infinity;
    let end = -Infinity;
    for (const t of threads) {
      const [s, e] = bounds(t);
      start = Math.min(start, s);
      end = Math.max(end, e);
    }
    if (start === Infinity) return [0, 1];
    return end > start ? [start, end] : [start, start + 1];
  });

  const window_ = $derived(view ?? domain);

  /**
   * Window stats in one pass: the peak (and when it was reached) plus the
   * TIME-WEIGHTED average — each count weighted by how long it held, so a
   * one-second spike doesn't count like a ten-minute plateau.
   */
  const stats = $derived.by(() => {
    const [lo, hi] = window_;

    // value carried into the window from earlier steps
    let current = 0;
    for (const p of steps) {
      if (p.t <= lo) current = p.count;
      else break;
    }

    let peak = { count: current, t: lo };
    let area = 0;
    let prevT = lo;
    for (const p of steps) {
      if (p.t <= lo) continue;
      if (p.t >= hi) break;
      area += current * (p.t - prevT);
      prevT = p.t;
      current = p.count;
      if (current > peak.count) peak = { count: current, t: p.t };
    }
    area += current * (hi - prevT);

    const span = hi - lo;
    return { peak, average: span > 0 ? area / span : 0 };
  });

  const ticks = $derived.by(() => {
    const n = 8;
    const [lo, hi] = window_;
    return Array.from({ length: n + 1 }, (_, i) => ({
      pct: (i / n) * 100,
      t: lo + ((hi - lo) * i) / n,
    }));
  });

  // Tick precision follows the window (same policy as the overview strip).
  const timeFormat = $derived.by(() => {
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
    try {
      threads = await cached("stuckthread_listview", () => stuckthreadListview());
    } catch (e) {
      errorMessage = String(e);
    }
  }

  $effect(() => {
    if (db.state.status === "open") {
      errorMessage = null;
      refresh();
    } else {
      threads = [];
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refresh();
  });
</script>

<div class="page">
  {#if errorMessage}
    <div class="error-bar" role="alert">
      {errorMessage}
      <button onclick={() => (errorMessage = null)} aria-label="Dismiss">✕</button>
    </div>
  {/if}

  <div class="toolbar">
    <span class="stat">
      peak <strong>{stats.peak.count}</strong> concurrent
      {#if stats.peak.count > 0}
        at <span class="mono">{formatTimestamp(timeFormat, stats.peak.t)}</span>
      {/if}
      · avg <strong>{stats.average.toFixed(1)}</strong>
    </span>
    <button class="reset" onclick={() => (view = null)} disabled={view === null}>
      reset
    </button>
  </div>

  {#if threads.length === 0}
    <p class="empty">No stuck-thread events — parse a serverout log from the Ingest page.</p>
  {:else}
    <div class="axis">
      {#each ticks as tick, i (i)}
        <span class="tick" style:left="{tick.pct}%">{formatTimestamp(timeFormat, tick.t)}</span>
      {/each}
    </div>
    <div class="chart">
      <StuckConcurrency points={steps} {domain} {view} onviewchange={(v) => (view = v)} />
    </div>
  {/if}
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 8px 12px 12px;
  }

  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 12px;
    margin-bottom: 8px;
    font-size: 12px;
    color: var(--red);
    background: color-mix(in srgb, var(--red) 12%, transparent);
    border-radius: var(--radius);
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
    justify-content: space-between;
    gap: 12px;
    padding: 2px 0 8px;
    flex-shrink: 0;
  }
  .stat {
    font-size: 12.5px;
    color: var(--fg-muted);
  }
  .stat strong {
    color: var(--fg-strong);
    font-family: var(--font-mono);
  }
  .mono {
    font-family: var(--font-mono);
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

  .axis {
    position: relative;
    height: 16px;
    flex-shrink: 0;
    /* the chart's y-label gutter is 30px (StuckConcurrency .gutter) —
       the time ticks must align with the plot, not the gutter */
    margin-left: 30px;
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

  .chart {
    flex: 1;
    min-height: 0;
  }

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
  }
</style>
