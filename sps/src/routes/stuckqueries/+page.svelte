<script lang="ts">
  /**
   * Stuck Queries analyzer — cpumemstats anatomy: snapshot list on the
   * left (one row per dump moment), content on the right. Modes:
   *  - Queries:      the flavor's running-queries table for the snapshot
   *  - Blocking:     MSSQL blocking-chain tree (only when the snapshot
   *                  logged one; PGSQL never has it)
   *  - Long-running: executions present across MULTIPLE snapshots
   *
   * MSSQL and PGSQL stay separate end to end (no union shapes) — the
   * snapshot row carries its flavor and everything downstream branches.
   */
  import {
    stuckqueryMssqlSnapshots,
    stuckqueryPgsqlSnapshots,
    stuckqueryMssqlQueries,
    stuckqueryPgsqlQueries,
    stuckqueryMssqlBlocking,
    stuckqueryMssqlLongrunning,
    stuckqueryPgsqlLongrunning,
    type MssqlSnapshot,
    type PgsqlSnapshot,
    type MssqlQuery,
    type PgsqlQuery,
    type MssqlBlockingRow,
  } from "$lib/api/stuckquery";
  import SnapshotList, { type SnapshotRow } from "$lib/components/SnapshotList.svelte";
  import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
  import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
  import BlockingTree from "$lib/components/BlockingTree.svelte";
  import LongRunnersTable, {
    type LongRunnerRow,
  } from "$lib/components/LongRunnersTable.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import { cached } from "$lib/query-cache";

  let errorMessage = $state<string | null>(null);
  let mssqlSnaps = $state<MssqlSnapshot[]>([]);
  let pgsqlSnaps = $state<PgsqlSnapshot[]>([]);
  let selected = $state<SnapshotRow | null>(null);

  // per-selection content (only the selected flavor's state is populated)
  let mssqlQueries = $state<MssqlQuery[]>([]);
  let pgsqlQueries = $state<PgsqlQuery[]>([]);
  let blocking = $state<MssqlBlockingRow[]>([]);
  let longRunners = $state<LongRunnerRow[]>([]);

  const Mode = { Queries: "queries", Blocking: "blocking", Long: "long" } as const;
  type Mode = (typeof Mode)[keyof typeof Mode];
  let mode = $state<Mode>(Mode.Queries);

  const rows = $derived.by<SnapshotRow[]>(() => {
    const out: SnapshotRow[] = [
      ...mssqlSnaps.map((s) => ({
        timestamp: s.timestamp,
        kind: "mssql" as const,
        detail:
          `${s.queries} queries` +
          (s.blocked > 0 ? ` · ${s.blocked} blocked` : "") +
          (s.blockingRows > 0 ? ` · chain` : ""),
        alert: s.blocked > 0 || s.blockingRows > 0,
      })),
      ...pgsqlSnaps.map((s) => ({
        timestamp: s.timestamp,
        kind: "pgsql" as const,
        detail:
          `${s.queries} queries` +
          (s.waiting > 0 ? ` · ${s.waiting} waiting` : "") +
          (s.idleInTxn > 0 ? ` · ${s.idleInTxn} idle in txn` : ""),
        alert: s.waiting > 0 || s.idleInTxn > 0,
      })),
    ];
    return out.toSorted((a, b) => a.timestamp - b.timestamp);
  });

  /** blocking data exists for the selected snapshot */
  const hasBlocking = $derived.by(() => {
    if (selected === null || selected.kind !== "mssql") return false;
    const snap = mssqlSnaps.find((s) => s.timestamp === selected!.timestamp);
    return snap !== undefined && snap.blockingRows > 0;
  });

  async function refresh() {
    const [mssqlResult, pgsqlResult] = await Promise.allSettled([
      cached("stuckquery_mssql_snapshots", stuckqueryMssqlSnapshots),
      cached("stuckquery_pgsql_snapshots", stuckqueryPgsqlSnapshots),
    ]);
    if (mssqlResult.status === "fulfilled") mssqlSnaps = mssqlResult.value;
    else errorMessage = String(mssqlResult.reason);
    if (pgsqlResult.status === "fulfilled") pgsqlSnaps = pgsqlResult.value;
    else errorMessage = String(pgsqlResult.reason);

    if (selected === null && rows.length > 0) onselect(rows[0]);
    loadLongRunners();
  }

  async function onselect(row: SnapshotRow) {
    selected = row;
    if (mode === Mode.Blocking) mode = Mode.Queries; // re-decide per snapshot
    try {
      if (row.kind === "mssql") {
        const [queries, chains] = await Promise.all([
          cached(`stuckquery_mssql_queries:${row.timestamp}`, () =>
            stuckqueryMssqlQueries(row.timestamp),
          ),
          cached(`stuckquery_mssql_blocking:${row.timestamp}`, () =>
            stuckqueryMssqlBlocking(row.timestamp),
          ),
        ]);
        // stale guard: a slower response must not clobber a newer selection
        if (selected?.timestamp !== row.timestamp) return;
        mssqlQueries = queries;
        blocking = chains;
      } else {
        const queries = await cached(`stuckquery_pgsql_queries:${row.timestamp}`, () =>
          stuckqueryPgsqlQueries(row.timestamp),
        );
        if (selected?.timestamp !== row.timestamp) return;
        pgsqlQueries = queries;
      }
    } catch (e) {
      errorMessage = String(e);
    }
  }

  async function loadLongRunners() {
    const [mssqlResult, pgsqlResult] = await Promise.allSettled([
      cached("stuckquery_mssql_longrunning", stuckqueryMssqlLongrunning),
      cached("stuckquery_pgsql_longrunning", stuckqueryPgsqlLongrunning),
    ]);
    const out: LongRunnerRow[] = [];
    if (mssqlResult.status === "fulfilled") {
      for (const r of mssqlResult.value) {
        out.push({
          key: `mssql:${r.sessionId}:${r.txnId}`,
          who: `session ${r.sessionId}`,
          query: r.statement,
          snapshots: r.snapshots,
          firstSeen: r.firstSeen,
          lastSeen: r.lastSeen,
          maxMs: r.maxElapsedMs,
          flagCount: r.blockedIn,
          flagLabel: "blocked",
        });
      }
    }
    if (pgsqlResult.status === "fulfilled") {
      for (const r of pgsqlResult.value) {
        out.push({
          key: `pgsql:${r.pid}:${r.firstSeen}`,
          who: `pid ${r.pid}`,
          query: r.query,
          snapshots: r.snapshots,
          firstSeen: r.firstSeen,
          lastSeen: r.lastSeen,
          maxMs: r.maxQueryTimeMs,
          flagCount: r.idleInTxnIn,
          flagLabel: "idle in txn",
        });
      }
    }
    // ranking: most snapshots first, then longest
    longRunners = out.toSorted(
      (a, b) => b.snapshots - a.snapshots || (b.maxMs ?? 0) - (a.maxMs ?? 0),
    );
  }

  $effect(() => {
    if (db.state.status === "open") {
      errorMessage = null;
      selected = null;
      refresh();
    } else {
      mssqlSnaps = [];
      pgsqlSnaps = [];
      selected = null;
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    selected = null;
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

  <SplitPane direction="row" initial={0.24}>
    {#snippet a()}
      <SnapshotList {rows} selected={selected?.timestamp ?? null} {onselect} />
    {/snippet}
    {#snippet b()}
      <div class="content">
        <div class="toolbar">
          <span class="chips" role="group" aria-label="View">
            <button
              class:active={mode === Mode.Queries}
              onclick={() => (mode = Mode.Queries)}
            >Queries</button>
            {#if hasBlocking}
              <button
                class:active={mode === Mode.Blocking}
                onclick={() => (mode = Mode.Blocking)}
              >Blocking</button>
            {/if}
            <button
              class:active={mode === Mode.Long}
              onclick={() => (mode = Mode.Long)}
            >Long-running</button>
          </span>
        </div>

        <div class="body">
          {#if mode === Mode.Long}
            <LongRunnersTable rows={longRunners} />
          {:else if selected === null}
            <p class="empty">Select a snapshot.</p>
          {:else if mode === Mode.Blocking}
            <BlockingTree rows={blocking} />
          {:else if selected.kind === "mssql"}
            <MssqlQueryTable queries={mssqlQueries} />
          {:else}
            <PgsqlQueryTable queries={pgsqlQueries} />
          {/if}
        </div>
      </div>
    {/snippet}
  </SplitPane>
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

  .content {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .toolbar {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
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

  .body {
    flex: 1;
    min-height: 0;
  }

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
    font-size: 12.5px;
  }
</style>
