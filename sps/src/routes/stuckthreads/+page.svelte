<script lang="ts">
  /**
   * Stuck Threads analyzer — DevTools network-tab shape: filter toolbar,
   * zoomable waterfall strip (all time visualization lives there), episode
   * table below it, details side panel on row click (request, thread,
   * timings, stack trace).
   *
   * Episodes come aggregated from Rust (stuckthread_listview, mirrored in
   * src/lib/api/stuckthread.ts); the frontend derives geometry only
   * (lib/stuckthread.ts). One cached full-range fetch; windowing is
   * client-side.
   */
  import { MediaQuery } from "svelte/reactivity";
  import {
    stuckthreadListview,
    stuckthreadTrace,
    type StuckThread,
  } from "$lib/api/stuckthread";
  import { bounds, threadBar, threadKey, pathRollup, type StuckBar } from "$lib/stuckthread";
  import StuckOverview from "$lib/components/StuckOverview.svelte";
  import StuckPathTable from "$lib/components/StuckPathTable.svelte";
  import TimeRangePicker from "$lib/components/TimeRangePicker.svelte";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import { cached } from "$lib/query-cache";
  import { formatDuration, formatTimestamp } from "$lib/format";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import StuckTable from "$lib/components/StuckTable.svelte";
  import StackTracePanel, {
    type TraceState,
  } from "$lib/components/StackTracePanel.svelte";
  import Icon from "$lib/components/Icon.svelte";

  let errorMessage = $state<string | null>(null);
  let threads = $state<StuckThread[]>([]);
  let selected = $state<StuckThread | null>(null);
  let trace = $state<TraceState>({ status: "idle" });
  /** overview brush window; null = full range */
  let view = $state<[number, number] | null>(null);

  const bars = $derived(threads.map(threadBar));

  // Table mode: per-episode list or per-path rollup.
  const ViewMode = { Episodes: "episodes", Paths: "paths" } as const;
  type ViewMode = (typeof ViewMode)[keyof typeof ViewMode];
  let viewMode = $state<ViewMode>(ViewMode.Episodes);

  // --- time window controls -------------------------------------------------
  // Stuck episodes live on the seconds scale; the full log domain is hours.
  // So the table NEVER defaults to the full domain — it opens on the last
  // 5 minutes, and the toolbar cluster (presets / magnifiers / slider)
  // drives the same `view` state the drag-zoom writes.
  const MINUTE = 60_000;
  const PRESETS = [
    { label: "5m", ms: 5 * MINUTE },
    { label: "10m", ms: 10 * MINUTE },
    { label: "30m", ms: 30 * MINUTE },
  ];
  /** slider fully zoomed in = a 10 s window */
  const MIN_SPAN = 10_000;

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
  const fullSpan = $derived(domain[1] - domain[0]);
  const windowSpan = $derived(view === null ? fullSpan : view[1] - view[0]);

  /** the last `ms` of the log — presets anchor at the data's end */
  function lastWindow(ms: number): [number, number] | null {
    if (ms >= fullSpan) return null;
    return [domain[1] - ms, domain[1]];
  }

  /** resize the window around its current center (magnifiers + slider) */
  function applySpan(ms: number) {
    const span = Math.max(MIN_SPAN, ms);
    if (span >= fullSpan) {
      view = null;
      return;
    }
    const center = view === null ? domain[1] - span / 2 : (view[0] + view[1]) / 2;
    let lo = center - span / 2;
    let hi = center + span / 2;
    if (lo < domain[0]) [lo, hi] = [domain[0], domain[0] + span];
    if (hi > domain[1]) [lo, hi] = [domain[1] - span, domain[1]];
    view = [lo, hi];
  }

  const presetActive = (ms: number) =>
    view !== null && Math.abs(windowSpan - ms) < 500 && Math.abs(view[1] - domain[1]) < 500;

  // --- DevTools-style filter toolbar ---------------------------------------
  const Status = { All: "all", Done: "done", Stuck: "stuck" } as const;
  type Status = (typeof Status)[keyof typeof Status];
  let filterText = $state("");
  let statusFilter = $state<Status>(Status.All);

  const filteredThreads = $derived.by(() => {
    const needle = filterText.trim().toLowerCase();
    return threads.filter((t) => {
      if (statusFilter === Status.Done && t.end === null) return false;
      if (statusFilter === Status.Stuck && t.end !== null) return false;
      if (needle === "") return true;
      return (
        (t.request ?? "").toLowerCase().includes(needle) ||
        t.name.toLowerCase().includes(needle) ||
        String(t.tid).includes(needle)
      );
    });
  });

  // Rollup respects both the toolbar filters and the zoom window.
  const rollup = $derived.by(() => {
    // capture: TS drops the null-narrowing of `view` inside the closure
    const window_ = view;
    const windowed =
      window_ === null
        ? filteredThreads
        : filteredThreads.filter((t) => {
            const [start, end] = bounds(t);
            return start <= window_[1] && end >= window_[0];
          });
    return pathRollup(windowed);
  });

  // Bars join to rows on (tid, start) — threadBar builds them that way.
  const overviewSelected = $derived(
    selected === null ? null : { tid: selected.tid, timestamp: bounds(selected)[0] },
  );

  function onselectbar(bar: StuckBar) {
    const thread = threads.find(
      (t) => t.tid === bar.tid && bounds(t)[0] === bar.timestamp,
    );
    if (thread) onselect(thread);
  }

  const portrait = new MediaQuery("(orientation: portrait)");

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });

  async function refresh() {
    try {
      // one cached full-range fetch; zooming windows client-side. The
      // handler's (from, to) frame stays available for huge logs later.
      threads = await cached("stuckthread_listview", () => stuckthreadListview());
    } catch (e) {
      errorMessage = String(e);
    }
    view = lastWindow(PRESETS[0].ms); // default: last 5 minutes
  }

  $effect(() => {
    if (db.state.status === "open") {
      selected = null;
      trace = { status: "idle" };
      errorMessage = null;
      refresh();
    } else {
      threads = [];
      selected = null;
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refresh();
  });

  async function onselect(thread: StuckThread) {
    selected = thread;
    if (thread.begin === null) {
      // completion-only episode: no warning event, so no trace exists
      trace = { status: "ready", tid: thread.tid, timestamp: bounds(thread)[0], frames: null };
      return;
    }
    const tid = thread.tid;
    const timestamp = thread.begin;
    trace = { status: "loading", tid, timestamp };
    try {
      const frames = await cached(`stuckthread_trace:${tid}:${timestamp}`, () =>
        stuckthreadTrace(tid, timestamp),
      );
      // the handler returns an empty Vec for "no trace captured"
      trace = { status: "ready", tid, timestamp, frames: frames.length === 0 ? null : frames };
    } catch (e) {
      trace = { status: "error", message: String(e) };
    }
  }

  // --- copy episode as text (for tickets/chat) ------------------------------
  let copied = $state(false);

  async function copyEpisode() {
    if (selected === null) return;
    const active = [selected.activeStart, selected.activeEnd].filter((n) => n !== null);
    const lines = [
      `Request: ${selected.request ?? "unknown"}`,
      `Thread: ${selected.name || "(empty)"}`,
      `TID: ${selected.tid}`,
      ...(active.length > 0 ? [`Active threads: ${active.join(" -> ")}`] : []),
      `Started: ${formatTimestamp(timeFormat, bounds(selected)[0])}`,
      selected.end !== null
        ? `Completed: ${formatTimestamp(timeFormat, selected.end)}`
        : `Never completed in log`,
      `Stuck for: ${formatDuration(selected.duration)}`,
    ];
    if (trace.status === "ready" && trace.frames !== null) {
      lines.push("", "Stack trace:");
      for (const frame of trace.frames) lines.push(`  at ${frame.method} (${frame.source})`);
    }
    try {
      await navigator.clipboard.writeText(lines.join("\n"));
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch (e) {
      errorMessage = `Clipboard write failed: ${e}`;
    }
  }
</script>

<div class="page">
  {#if errorMessage}
    <div class="error-bar" role="alert">
      {errorMessage}
      <button onclick={() => (errorMessage = null)} aria-label="Dismiss">✕</button>
    </div>
  {/if}

  <!-- DevTools anatomy: filter toolbar, overview strip, table, side panel. -->
  <div class="toolbar">
    <input
      class="filter"
      type="search"
      placeholder="Filter by request, thread or tid…"
      bind:value={filterText}
    />
    <span class="chips" role="group" aria-label="Status filter">
      <button
        class:active={statusFilter === Status.All}
        onclick={() => (statusFilter = Status.All)}
      >All</button>
      <button
        class:active={statusFilter === Status.Done}
        onclick={() => (statusFilter = Status.Done)}
      >Completed</button>
      <button
        class:active={statusFilter === Status.Stuck}
        onclick={() => (statusFilter = Status.Stuck)}
      >Stuck</button>
    </span>

    <span class="chips" role="group" aria-label="Table mode">
      <button
        class:active={viewMode === ViewMode.Episodes}
        onclick={() => (viewMode = ViewMode.Episodes)}
      >Episodes</button>
      <button
        class:active={viewMode === ViewMode.Paths}
        onclick={() => (viewMode = ViewMode.Paths)}
      >Paths</button>
    </span>

    <span class="zoom" role="group" aria-label="Time window">
      <TimeRangePicker {domain} {view} onviewchange={(v) => (view = v)} />
      {#each PRESETS as preset (preset.ms)}
        <button
          class="preset"
          class:active={presetActive(preset.ms)}
          onclick={() => (view = lastWindow(preset.ms))}
        >{preset.label}</button>
      {/each}
      <button
        class="zbtn"
        onclick={() => applySpan(windowSpan * 2)}
        disabled={view === null}
        title="Zoom out"
        aria-label="Zoom out"
      ><Icon name="zoomOut" size={13} /></button>
      <button
        class="zbtn"
        onclick={() => applySpan(windowSpan / 2)}
        disabled={windowSpan <= MIN_SPAN}
        title="Zoom in"
        aria-label="Zoom in"
      ><Icon name="zoomIn" size={13} /></button>
      <button
        class="reset"
        onclick={() => (view = null)}
        disabled={view === null}
        title="Reset zoom to full range"
      >reset</button>
    </span>

    <span class="count">{filteredThreads.length} / {threads.length}</span>
  </div>

  <!-- Strip over table as a vertical split — the strip height is draggable. -->
  <div class="body">
    <!-- A render error below (e.g. from corrupt rows) lands here as a
         visible message instead of silently killing the page. -->
    <svelte:boundary>
      {#snippet failed(error, reset)}
        <div class="crash" role="alert">
          <p>Stuck Threads crashed while rendering:</p>
          <pre>{error instanceof Error ? (error.stack ?? error.message) : String(error)}</pre>
          <button onclick={reset}>try again</button>
        </div>
      {/snippet}
    <SplitPane direction="column" initial={0.18}>
      {#snippet a()}
        <div class="strip">
          <StuckOverview
            {bars}
            selected={overviewSelected}
            onselect={onselectbar}
            {view}
            onviewchange={(v) => (view = v)}
          />
        </div>
      {/snippet}
      {#snippet b()}
  <div class="content">
    {#if viewMode === ViewMode.Paths}
      <StuckPathTable
        rollups={rollup}
        onpick={(path) => {
          filterText = path;
          viewMode = ViewMode.Episodes;
        }}
      />
    {:else if selected === null}
      <StuckTable threads={filteredThreads} selected={null} {onselect} {view} />
    {:else}
      <SplitPane direction={portrait.current ? "column" : "row"} initial={0.55}>
        {#snippet a()}
          <StuckTable
            threads={filteredThreads}
            selected={selected === null ? null : threadKey(selected)}
            {onselect}
            {view}
          />
        {/snippet}
        {#snippet b()}
          <!-- Snippets are functions, so TS drops the outer null-narrowing
               (same rule as any closure); re-establish it locally. -->
          {#if selected !== null}
          <div class="details">
            <header>
              <h3>Episode</h3>
              <span class="header-actions">
                <button
                  class="hbtn"
                  onclick={copyEpisode}
                  title="Copy episode as text"
                  aria-label="Copy episode as text"
                ><Icon name={copied ? "check" : "copy"} size={12} /></button>
                <button
                  class="hbtn close"
                  onclick={() => (selected = null)}
                  aria-label="Close details"
                ><Icon name="chevronLeft" size={12} /></button>
              </span>
            </header>

            <dl>
              <dt>Request</dt>
              <dd class="mono wrap">{selected.request ?? "unknown (warning lost)"}</dd>
              <dt>Thread</dt>
              <dd class="mono wrap">{selected.name || "(empty in log)"}</dd>
              <dt>TID</dt>
              <dd class="mono">{selected.tid}</dd>
              <dt>Started</dt>
              <dd class="mono">{formatTimestamp(timeFormat, bounds(selected)[0])}</dd>
              <dt>Status</dt>
              <dd>
                {#if selected.end !== null}
                  <span class="badge done">completed {formatTimestamp(timeFormat, selected.end)}</span>
                {:else}
                  <span class="badge open">never completed in log</span>
                {/if}
              </dd>
              <dt>Stuck for</dt>
              <dd class="mono">{formatDuration(selected.duration)}</dd>
              {#if selected.activeStart !== null || selected.activeEnd !== null}
                <!-- a COUNT of threads active alongside this one, not a
                     duration: "12 → 8" = at warning → at completion -->
                <dt>Active threads</dt>
                <dd class="mono">
                  {[selected.activeStart, selected.activeEnd]
                    .filter((n) => n !== null)
                    .join(" → ")}
                </dd>
              {/if}
            </dl>

            <div class="trace">
              <StackTracePanel {trace} />
            </div>
          </div>
          {/if}
        {/snippet}
      </SplitPane>
    {/if}
  </div>
      {/snippet}
    </SplitPane>
    </svelte:boundary>
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
    flex-wrap: wrap; /* range picker + chips can exceed narrow windows */
    gap: 8px 12px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }
  .filter {
    flex: 1;
    max-width: 360px;
    padding: 5px 10px;
    background: var(--bg-hard);
    border: none;
    border-radius: var(--radius);
    font-size: 12.5px;
  }
  .chips {
    display: flex;
    gap: 2px;
    padding: 2px;
    background: var(--bg-hard);
    border-radius: 999px;
  }
  .chips button {
    padding: 2px 12px;
    font-size: 11.5px;
    border-radius: 999px;
    color: var(--fg-muted);
  }
  .chips button:hover {
    color: var(--fg);
  }
  .chips button.active {
    background: var(--accent);
    color: var(--bg-hard);
    font-weight: 600;
  }
  .zoom {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
  }
  .preset {
    padding: 2px 9px;
    font-size: 11px;
    font-family: var(--font-mono);
    border-radius: 999px;
    color: var(--fg-muted);
    background: var(--bg-hard);
  }
  .preset:hover {
    color: var(--fg);
  }
  .preset.active {
    background: var(--accent);
    color: var(--bg-hard);
    font-weight: 600;
  }
  .zbtn {
    display: grid;
    place-items: center;
    padding: 4px;
    color: var(--fg-muted);
    border-radius: var(--radius);
  }
  .zbtn:hover:not(:disabled) {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .zbtn:disabled {
    opacity: 0.35;
  }

  .count {
    font-size: 11.5px;
    font-family: var(--font-mono);
    color: var(--fg-muted);
  }

  .body {
    flex: 1;
    min-height: 0; /* lets the SplitPane shrink instead of overflowing */
  }

  .crash {
    padding: 16px;
    color: var(--red);
    font-size: 12.5px;
  }
  .crash pre {
    padding: 10px;
    background: var(--bg-hard);
    border-radius: var(--radius);
    overflow: auto;
    font-size: 11.5px;
    white-space: pre-wrap;
  }
  .crash button {
    padding: 3px 12px;
    border-radius: 999px;
    background: var(--accent);
    color: var(--bg-hard);
    font-weight: 600;
  }

  .strip {
    height: 100%; /* fills its pane; the split divider owns the sizing */
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

  .content {
    height: 100%;
    min-height: 0;
  }

  .details {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .details header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: var(--bg-hard);
    flex-shrink: 0;
  }
  .details h3 {
    margin: 0;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--fg-muted);
  }
  .header-actions {
    display: flex;
    gap: 2px;
  }
  .hbtn {
    display: grid;
    place-items: center;
    padding: 4px;
    color: var(--fg-muted);
  }
  .hbtn:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }
  .close {
    transform: rotate(180deg); /* chevron points toward the panel edge */
  }

  dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 14px;
    margin: 0;
    padding: 10px 12px;
    font-size: 12.5px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }
  dt {
    color: var(--fg-muted);
  }
  dd {
    margin: 0;
    min-width: 0;
  }
  .wrap {
    overflow-wrap: anywhere;
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .badge {
    padding: 1px 8px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 600;
  }
  .badge.done {
    background: color-mix(in srgb, var(--green) 18%, transparent);
    color: var(--green);
  }
  .badge.open {
    background: color-mix(in srgb, var(--red) 18%, transparent);
    color: var(--red);
  }

  .trace {
    flex: 1;
    min-height: 0;
  }
</style>
