<script lang="ts">
  /**
   * Episode list over the Rust-aggregated rows: sortable Name / Status /
   * TID / Started / Time columns. Pure data table — time visualization
   * lives in the overview strip; the table FOLLOWS the active window by
   * hiding episodes that don't intersect it.
   */
  import type { StuckThread } from "$lib/api/stuckthread";
  import { bounds, threadKey } from "$lib/stuckthread";
  import { formatDuration, formatTimestamp } from "$lib/format";

  interface Props {
    threads: StuckThread[];
    /** threadKey() of the selected row, or null */
    selected: string | null;
    onselect: (thread: StuckThread) => void;
    /** active time window; rows outside it are hidden. null = all */
    view?: [number, number] | null;
  }

  let { threads, selected, onselect, view = null }: Props = $props();

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

  function sortValue(t: StuckThread): string | number {
    switch (sortKey) {
      case "name":
        return label(t);
      case "status":
        return t.end === null ? 0 : 1;
      case "tid":
        return t.tid;
      case "start":
        return bounds(t)[0];
      case "duration":
        return t.duration;
    }
  }

  const visible = $derived.by(() => {
    const windowed =
      view === null
        ? threads
        : threads.filter((t) => {
            const [start, end] = bounds(t);
            return start <= view[1] && end >= view[0];
          });
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
    { key: "duration", label: "Time", class: "col-duration" },
  ];

  /** request path without host — the interesting part at column width */
  function label(t: StuckThread): string {
    if (t.request === null) return t.name || "(warning lost — unknown request)";
    try {
      return new URL(t.request).pathname;
    } catch {
      return t.request;
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
    <!-- unkeyed on purpose: rows are stateless, and duplicate identities in
         corrupt data (same tid+begin twice) must not crash the each -->
    {#each visible as thread}
      <button
        class="row"
        class:selected={selected === threadKey(thread)}
        onclick={() => onselect(thread)}
        title={thread.request ?? thread.name}
      >
        <span class="col-name">{label(thread)}</span>
        <span class="col-status">
          <span class="badge" class:done={thread.end !== null} class:open={thread.end === null}>
            {thread.end !== null ? "done" : "stuck"}
          </span>
        </span>
        <span class="col-tid mono">{thread.tid}</span>
        <span class="col-start mono">{formatTimestamp(timeFormat, bounds(thread)[0])}</span>
        <span class="col-duration mono">{formatDuration(thread.duration)}</span>
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
    grid-template-columns: minmax(150px, 1fr) 80px 64px 90px 100px;
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
  .sort.col-duration {
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
  .col-duration {
    text-align: right;
  }
  .col-duration {
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
