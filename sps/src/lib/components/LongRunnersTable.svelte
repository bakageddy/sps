<script lang="ts" module>
  /**
   * Flavor-neutral view model: the page maps MssqlLongRunner /
   * PgsqlLongRunner into this (DTO vs view-model split — the wire shapes
   * differ, the ranking table doesn't need to care).
   */
  export interface LongRunnerRow {
    /** unique per row (e.g. "mssql:53:35776964631" / "pgsql:40952:…") */
    key: string;
    /** "session 53" / "pid 40952" */
    who: string;
    query: string;
    /** distinct snapshots this execution appears in */
    snapshots: number;
    firstSeen: number;
    lastSeen: number;
    /** max observed elapsed/query time, ms (null when unlogged) */
    maxMs: number | null;
    /** count of snapshots with the flavor's red flag, with its label */
    flagCount: number;
    flagLabel: string;
  }
</script>

<script lang="ts">
  /**
   * Executions observed across MULTIPLE snapshots — a query that shows up
   * in every dump with growing elapsed time IS the incident. Fixed sort
   * (most snapshots, then longest) on purpose: this is a ranking.
   */
  import { formatDuration, formatTimestamp } from "$lib/format";

  interface Props {
    rows: LongRunnerRow[];
    /** jump to the snapshot where this execution was last seen */
    onjump?: (row: LongRunnerRow) => void;
  }

  let { rows, onjump }: Props = $props();

  let expanded = $state<string | null>(null);

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });
</script>

<div class="table">
  <div class="head">
    <span>Who</span>
    <span class="col-num">Snapshots</span>
    <span class="col-num">Longest</span>
    <span class="col-num">Flagged</span>
    <span>Query</span>
  </div>

  <div class="rows">
    {#each rows as row (row.key)}
      <button
        class="row"
        onclick={() => (expanded = expanded === row.key ? null : row.key)}
      >
        <span class="mono">{row.who}</span>
        <span class="col-num mono">{row.snapshots}</span>
        <span class="col-num mono">{row.maxMs === null ? "—" : formatDuration(row.maxMs)}</span>
        <span class="col-num mono" class:bad={row.flagCount > 0}>
          {row.flagCount > 0 ? `${row.flagCount} ${row.flagLabel}` : "—"}
        </span>
        <span class="query mono">{row.query}</span>
      </button>
      {#if expanded === row.key}
        <div class="expand">
          <dl>
            <dt>First seen</dt><dd class="mono">{formatTimestamp(timeFormat, row.firstSeen)}</dd>
            <dt>Last seen</dt><dd class="mono">{formatTimestamp(timeFormat, row.lastSeen)}</dd>
          </dl>
          <pre>{row.query}</pre>
          {#if onjump}
            <button class="jump" onclick={() => onjump(row)}>
              view last snapshot →
            </button>
          {/if}
        </div>
      {/if}
    {:else}
      <p class="empty">
        No execution appears in more than one snapshot — nothing was stuck
        across dumps.
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
    grid-template-columns: 110px 80px 90px 110px 1fr;
    gap: 10px;
    align-items: center;
    padding: 0 10px;
  }

  .head {
    flex-shrink: 0;
    padding-top: 8px;
    padding-bottom: 8px;
    background: var(--bg-soft);
    font-weight: 600;
    color: var(--fg-strong);
    position: sticky;
    top: 0;
    z-index: 1;
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

  .col-num {
    text-align: right;
  }
  .query {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--fg-muted);
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .bad {
    color: var(--red);
    font-weight: 600;
  }

  .expand {
    padding: 8px 12px;
    border-top: 1px solid var(--hairline);
    background: var(--bg-hard);
  }
  .expand dl {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 3px 12px;
    margin: 0 0 8px;
    font-size: 12px;
  }
  .expand dt {
    color: var(--fg-muted);
  }
  .expand dd {
    margin: 0;
  }
  .expand pre {
    margin: 0;
    padding: 8px;
    background: var(--bg);
    border-radius: var(--radius);
    font-size: 11.5px;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
    max-height: 240px;
    overflow: auto;
  }

  .jump {
    margin-top: 8px;
    padding: 2px 12px;
    border-radius: 999px;
    background: var(--bg);
    color: var(--accent);
    font-size: 11.5px;
    font-weight: 600;
  }
  .jump:hover {
    background: var(--bg-hover);
  }

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
  }
</style>
