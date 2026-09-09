<script lang="ts">
  /**
   * MSSQL blocking chains for one snapshot: one group per head blocker,
   * victims indented by their `level` in the chain. Nine out of ten
   * lock incidents are one head blocker — this view names it.
   *
   * PGSQL snapshots never render this (the app logs no blocking table
   * for Postgres).
   */
  import type { MssqlBlockingRow } from "$lib/api/stuckquery";
  import { formatDuration } from "$lib/format";

  interface Props {
    rows: MssqlBlockingRow[];
  }

  let { rows }: Props = $props();

  // rows arrive ordered by (head_blocker, level); group per head blocker
  const chains = $derived.by(() => {
    const groups = new Map<number, MssqlBlockingRow[]>();
    for (const row of rows) {
      const group = groups.get(row.headBlocker);
      if (group) group.push(row);
      else groups.set(row.headBlocker, [row]);
    }
    // worst chain (most victims) first
    return [...groups.entries()].toSorted((a, b) => b[1].length - a[1].length);
  });

  /** `${headBlocker}:${sessionId}` of the expanded row, or null */
  let expanded = $state<string | null>(null);
  const rowKey = (r: MssqlBlockingRow) => `${r.headBlocker}:${r.sessionId}`;
</script>

<div class="tree">
  {#each chains as [head, victims]}
    <div class="chain">
      <div class="head-row">
        <span class="crown">head blocker</span>
        <span class="mono session">session {head}</span>
        <span class="count">blocking {victims.length} session{victims.length === 1 ? "" : "s"}</span>
      </div>
      {#each victims as v}
        <button
          class="victim"
          style:padding-left="{12 + v.level * 22}px"
          onclick={() => (expanded = expanded === rowKey(v) ? null : rowKey(v))}
        >
          <span class="elbow">└─</span>
          <span class="mono session">{v.sessionId}</span>
          <span class="mono wait" title={v.waitResource ?? undefined}>{v.waitType ?? "—"}</span>
          <span class="mono dur">{formatDuration(v.waitDurationMs)}</span>
          <span class="mono query">{v.query}</span>
        </button>
        {#if expanded === rowKey(v)}
          <div class="expand" style:margin-left="{12 + v.level * 22}px">
            <dl>
              <dt>Blocked by</dt><dd class="mono">session {v.blockingSessionId}</dd>
              <dt>Txn</dt><dd class="mono">{v.txnId}</dd>
              <dt>Wait resource</dt><dd class="mono">{v.waitResource ?? "—"}</dd>
              <dt>Offsets</dt>
              <dd class="mono">{v.statementStartOffset} → {v.statementEndOffset}</dd>
            </dl>
            <pre>{v.query}</pre>
          </div>
        {/if}
      {/each}
    </div>
  {:else}
    <p class="empty">No blocking chains in this snapshot.</p>
  {/each}
</div>

<style>
  .tree {
    overflow: auto;
    height: 100%;
    font-size: 12.5px;
  }

  .chain {
    border-bottom: 1px solid var(--hairline);
    padding-bottom: 4px;
  }

  .head-row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 12px 4px;
  }
  .crown {
    padding: 0 8px;
    border-radius: 999px;
    font-size: 10.5px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    background: color-mix(in srgb, var(--red) 20%, transparent);
    color: var(--red);
  }
  .session {
    font-weight: 600;
  }
  .count {
    font-size: 11.5px;
    color: var(--fg-muted);
  }

  .victim {
    display: grid;
    grid-template-columns: auto 60px 140px 80px 1fr;
    gap: 10px;
    align-items: center;
    width: 100%;
    padding-top: 3px;
    padding-bottom: 3px;
    padding-right: 10px;
    text-align: left;
    border-radius: 0;
    color: var(--fg);
  }
  .victim:hover {
    background: var(--bg-hover);
  }

  .elbow {
    color: var(--fg-muted);
    font-family: var(--font-mono);
  }
  .wait {
    color: var(--yellow);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .dur {
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

  .expand {
    padding: 8px 12px;
    background: var(--bg-hard);
    border-radius: var(--radius);
    margin: 4px 12px 6px;
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
