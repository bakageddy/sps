<script lang="ts" module>
  /**
   * View model one page builds from the snapshot rollups. The same
   * timestamp can appear TWICE — once as a query snapshot and once as a
   * "blocking" row (blocking tables are fetched and displayed separately)
   * — so identity is kind + timestamp, never timestamp alone.
   */
  export interface SnapshotRow {
    timestamp: number;
    kind: "mssql" | "pgsql" | "blocking";
    /** e.g. "21 queries · 3 blocked" */
    detail: string;
    /** true = something is wrong in this snapshot (blocked/idle-in-txn) */
    alert: boolean;
  }

  export const snapshotKey = (row: SnapshotRow) => `${row.kind}:${row.timestamp}`;
</script>

<script lang="ts">
  /**
   * Snapshot picker (cpumemstats dump-list pattern): one row per dump
   * moment, newest data comes pre-sorted from the backend.
   */
  import { formatTimestamp } from "$lib/format";

  interface Props {
    rows: SnapshotRow[];
    /** snapshotKey() of the selected row, or null */
    selected: string | null;
    onselect: (row: SnapshotRow) => void;
  }

  let { rows, selected, onselect }: Props = $props();

  // The ONLY sort key is the timestamp — snapshots are moments in time,
  // nothing else about them orders meaningfully. The header toggles it.
  let descending = $state(false);
  const sorted = $derived(
    descending ? rows.toSorted((a, b) => b.timestamp - a.timestamp) : rows,
  );

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });
</script>

<div class="list">
  <div class="header">
    <button class="sort" onclick={() => (descending = !descending)}>
      Timestamp <span class="arrow">{descending ? "▼" : "▲"}</span>
    </button>
    <span class="count mono">{rows.length}</span>
  </div>
  {#each sorted as row}
    <button
      class="row"
      class:selected={selected === snapshotKey(row)}
      onclick={() => onselect(row)}
    >
      <span class="when mono">{formatTimestamp(timeFormat, row.timestamp)}</span>
      <span class="meta">
        <span class="kind" class:blocking={row.kind === "blocking"}>{row.kind}</span>
        <span class="detail" class:alert={row.alert}>{row.detail}</span>
      </span>
    </button>
  {:else}
    <p class="empty">No stuck-query snapshots.</p>
  {/each}
</div>

<style>
  .list {
    overflow: auto;
    height: 100%;
  }

  .header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    background: var(--bg-soft);
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--fg-muted);
    position: sticky;
    top: 0;
    z-index: 1;
  }
  .count {
    font-size: 11px;
  }
  .sort {
    padding: 0;
    font: inherit;
    color: inherit;
    text-transform: inherit;
    letter-spacing: inherit;
    border-radius: 0;
  }
  .sort:hover {
    color: var(--accent);
  }
  .arrow {
    font-size: 9px;
    color: var(--accent);
  }

  .row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 6px 10px;
    text-align: left;
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

  .when {
    font-size: 12px;
  }
  .mono {
    font-family: var(--font-mono);
  }

  .meta {
    display: flex;
    gap: 8px;
    align-items: baseline;
  }
  .kind {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--fg-muted);
    padding: 0 6px;
    border-radius: 999px;
    background: var(--bg-hard);
  }
  .kind.blocking {
    background: color-mix(in srgb, var(--red) 18%, transparent);
    color: var(--red);
  }
  .detail {
    font-size: 11.5px;
    color: var(--fg-muted);
  }
  .detail.alert {
    color: var(--red);
  }

  .empty {
    padding: 24px 12px;
    text-align: center;
    color: var(--fg-muted);
    font-size: 12.5px;
  }
</style>
