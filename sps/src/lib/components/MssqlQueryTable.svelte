<script lang="ts">
  /**
   * "Currently Running Queries" (MSSQL flavor) for one snapshot.
   * Sortable; clicking a row expands it in place to show the full
   * statement text and the fields that don't earn a column.
   */
  import type { MssqlQuery } from "$lib/api/stuckquery";
  import { formatDuration } from "$lib/format";

  interface Props {
    queries: MssqlQuery[];
  }

  let { queries }: Props = $props();

  type SortKey = "session" | "status" | "blockedBy" | "wait" | "cpu" | "elapsed";
  let sortKey = $state<SortKey>("elapsed");
  let sortDescending = $state(true);
  /** sessionId of the expanded row, or null */
  let expanded = $state<number | null>(null);

  function sortBy(key: SortKey) {
    if (sortKey === key) sortDescending = !sortDescending;
    else {
      sortKey = key;
      sortDescending = key !== "session" && key !== "status";
    }
  }

  function sortValue(q: MssqlQuery): string | number {
    switch (sortKey) {
      case "session":
        return q.sessionId;
      case "status":
        return q.status;
      case "blockedBy":
        return q.blockedBy;
      case "wait":
        return q.waitType ?? "";
      case "cpu":
        return q.cpuTimeMs;
      case "elapsed":
        return q.elapsedMs;
    }
  }

  const sorted = $derived(
    queries.toSorted((a, b) => {
      const va = sortValue(a);
      const vb = sortValue(b);
      const order = va < vb ? -1 : va > vb ? 1 : 0;
      return sortDescending ? -order : order;
    }),
  );

  const columns: { key: SortKey; label: string; class: string }[] = [
    { key: "session", label: "Session", class: "col-num" },
    { key: "status", label: "Status", class: "col-status" },
    { key: "blockedBy", label: "Blocked by", class: "col-num" },
    { key: "wait", label: "Wait type", class: "col-wait" },
    { key: "cpu", label: "CPU", class: "col-num" },
    { key: "elapsed", label: "Elapsed", class: "col-num" },
  ];
</script>

<div class="table">
  <div class="head">
    {#each columns as col (col.key)}
      <button class="sort {col.class}" onclick={() => sortBy(col.key)}>
        {col.label}
        {#if sortKey === col.key}<span class="arrow">{sortDescending ? "▼" : "▲"}</span>{/if}
      </button>
    {/each}
    <span class="sort col-stmt">Statement</span>
  </div>

  <div class="rows">
    {#each sorted as q}
      <button
        class="row"
        class:blocked={q.blockedBy !== 0}
        onclick={() => (expanded = expanded === q.sessionId ? null : q.sessionId)}
      >
        <span class="col-num mono">{q.sessionId}</span>
        <span class="col-status"><span class="badge {q.status}">{q.status}</span></span>
        <span class="col-num mono">{q.blockedBy === 0 ? "—" : q.blockedBy}</span>
        <span class="col-wait mono" title={q.waitResource ?? undefined}>{q.waitType ?? "—"}</span>
        <span class="col-num mono">{formatDuration(q.cpuTimeMs)}</span>
        <span class="col-num mono">{formatDuration(q.elapsedMs)}</span>
        <span class="col-stmt mono">{q.statement}</span>
      </button>
      {#if expanded === q.sessionId}
        <div class="expand">
          <dl>
            <dt>Txn</dt><dd class="mono">{q.txnId}</dd>
            <dt>Login</dt><dd class="mono">{q.login || "—"}</dd>
            <dt>Command</dt><dd class="mono">{q.command || "—"}</dd>
            <dt>Wait</dt>
            <dd class="mono">{q.waitType ?? "—"} {q.waitResource ?? ""} ({formatDuration(q.waitTimeMs)})</dd>
            <dt>Reads</dt>
            <dd class="mono">{q.logicalReads} logical · {q.reads} physical · {q.writes} writes</dd>
          </dl>
          <pre>{q.commandText || q.statement}</pre>
        </div>
      {/if}
    {:else}
      <p class="empty">No queries in this snapshot.</p>
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
    grid-template-columns: 64px 88px 80px 130px 80px 80px 1fr;
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
  button.sort:hover {
    color: var(--accent);
  }
  .sort.col-num {
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
  .row.blocked {
    background: color-mix(in srgb, var(--red) 8%, transparent);
  }

  .col-num {
    text-align: right;
  }
  .col-wait,
  .col-stmt {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
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
    background: var(--bg-hard);
    color: var(--fg-muted);
  }
  .badge.suspended {
    background: color-mix(in srgb, var(--yellow) 18%, transparent);
    color: var(--yellow);
  }
  .badge.runnable,
  .badge.running {
    background: color-mix(in srgb, var(--green) 18%, transparent);
    color: var(--green);
  }
  .badge.rollback {
    background: color-mix(in srgb, var(--red) 18%, transparent);
    color: var(--red);
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

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
  }
</style>
