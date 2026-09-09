<script lang="ts" module>
  /** View model one page builds from either flavor's snapshot rollup. */
  export interface SnapshotRow {
    timestamp: number;
    kind: "mssql" | "pgsql";
    /** e.g. "21 queries · 3 blocked" */
    detail: string;
    /** true = something is wrong in this snapshot (blocked/idle-in-txn) */
    alert: boolean;
  }
</script>

<script lang="ts">
  /**
   * Snapshot picker (cpumemstats dump-list pattern): one row per dump
   * moment, newest data comes pre-sorted from the backend.
   */
  import { formatTimestamp } from "$lib/format";

  interface Props {
    rows: SnapshotRow[];
    selected: number | null;
    onselect: (row: SnapshotRow) => void;
  }

  let { rows, selected, onselect }: Props = $props();

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });
</script>

<div class="list">
  {#each rows as row}
    <button
      class="row"
      class:selected={selected === row.timestamp}
      onclick={() => onselect(row)}
    >
      <span class="when mono">{formatTimestamp(timeFormat, row.timestamp)}</span>
      <span class="meta">
        <span class="kind">{row.kind}</span>
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
