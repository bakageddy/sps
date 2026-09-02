<script lang="ts">
  /**
   * DevTools-style episode list: sortable Name / Status / TID / Started /
   * Time columns. Pure data table by design — every time-position visual
   * (bars, axis, sweep-zoom) lives in the overview strip; the table only
   * FOLLOWS the active window by hiding rows outside it.
   */
  import type { StuckSpan } from "$lib/api/stuckthread";
  import { formatDuration } from "$lib/format";

  interface Props {
    spans: StuckSpan[];
    /** key of the selected span, or null */
    selected: string | null;
    onselect: (span: StuckSpan) => void;
    /** active time window; rows outside it are hidden. null = all */
    view?: [number, number] | null;
  }

  let { spans, selected, onselect, view = null }: Props = $props();

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
  });

  // --- sorting + windowing ---------------------------------------------------
  type SortKey = "name" | "status" | "tid" | "start" | "duration";
  let sortKey = $state<SortKey>("start");
  let sortDescending = $state(false);

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortDescending = !sortDescending;
    } else {
      sortKey = key;
      sortDescending = key === "duration"; // biggest offenders first
    }
  }

  function sortValue(s: StuckSpan): string | number {
    switch (sortKey) {
      case "name":
        return shortRequest(s);
      case "status":
        return s.end === null ? 0 : 1;
      case "tid":
        return s.tid;
      case "start":
        return s.start;
      case "duration":
        return s.durationMs;
    }
  }

  const visible = $derived.by(() => {
    const windowed =
      view === null
        ? spans
        : spans.filter(
            (s) => s.start <= view[1] && (s.end ?? s.start + s.durationMs) >= view[0],
          );
    return windowed.toSorted((a, b) => {
      const va = sortValue(a);
      const vb = sortValue(b);
      const order = va < vb ? -1 : va > vb ? 1 : 0;
      return sortDescending ? -order : order;
    });
  });

  const columns: { key: SortKey; label: string; class: string }[] = [
    { key: "name", label: "Name", class: "col-name" },
    { key: "status", label: "Status", class: "col-status" },
    { key: "tid", label: "TID", class: "col-tid" },
    { key: "start", label: "Started", class: "col-start" },
    { key: "duration", label: "Time", class: "col-time" },
  ];

  /** request path without host — the interesting part at column width */
  function shortRequest(span: StuckSpan): string {
    if (span.request === null) return "(begin lost — unknown request)";
    try {
      return new URL(span.request).pathname;
    } catch {
      return span.request;
    }
  }
</script>

<div class="table">
  <div class="head">
    {#each columns as col (col.key)}
      <button class="sort {col.class}" onclick={() => sortBy(col.key)}>
        {col.label}
        {#if sortKey === col.key}
          <span class="arrow">{sortDescending ? "▼" : "▲"}</span>
        {/if}
      </button>
    {/each}
  </div>

  <div class="rows">
    {#each visible as span (span.key)}
      <button
        class="row"
        class:selected={selected === span.key}
        onclick={() => onselect(span)}
        title={span.request ?? span.key}
      >
        <span class="col-name">{shortRequest(span)}</span>
        <span class="col-status">
          <span class="badge" class:done={span.end !== null} class:open={span.end === null}>
            {span.end !== null ? "done" : "stuck"}
          </span>
        </span>
        <span class="col-tid mono">{span.tid}</span>
        <span class="col-start mono">{timeFormat.format(span.start)}</span>
        <span class="col-time mono">{formatDuration(span.durationMs)}</span>
      </button>
    {:else}
      <p class="empty">
        {view === null
          ? "No stuck threads — parse a serverout log from the Ingest page."
          : "No episodes in the selected window — zoom out or reset."}
      </p>
    {/each}
  </div>
</div>

<style>
  .table {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: 12.5px;
  }

  .head,
  .row {
    display: grid;
    grid-template-columns: minmax(150px, 1fr) 80px 64px 90px 90px;
    gap: 10px;
    align-items: center;
    padding: 0 10px;
  }

  .head {
    flex-shrink: 0;
    background: var(--bg-soft);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .sort {
    padding: 8px 0;
    text-align: left;
    font-weight: 600;
    color: var(--fg-strong);
    white-space: nowrap;
    border-radius: 0;
  }
  .sort:hover {
    color: var(--accent);
  }
  /* numeric columns are right-aligned; their headers must line up */
  .sort.col-tid,
  .sort.col-start,
  .sort.col-time {
    text-align: right;
  }
  .arrow {
    font-size: 9px;
    color: var(--accent);
  }

  .rows {
    overflow: auto;
    flex: 1;
  }

  .row {
    width: 100%;
    text-align: left;
    padding-top: 4px;
    padding-bottom: 4px;
    border-top: 1px solid var(--hairline);
    border-radius: 0;
    color: var(--fg);
  }
  .row:hover {
    background: var(--bg-hover);
  }
  .row.selected {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .col-name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .col-tid,
  .col-start,
  .col-time {
    text-align: right;
  }
  .col-time {
    color: var(--yellow);
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 12px;
  }

  .badge {
    padding: 0 8px;
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

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
  }
</style>
