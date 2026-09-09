<script lang="ts">
  /**
   * Episode ↔ query correlation: pick a stuck episode, see the queries
   * the database was running at that moment. The stuck-query snapshot is
   * logged AT valve detection, so the warning timestamp and a snapshot
   * timestamp are near-identical; we join on nearest and SHOW the delta —
   * honesty matters, this is "what the DB was doing then", never "this
   * thread ran this query" (no tid↔session mapping exists in the logs).
   */
  import { stuckthreadListview, type StuckThread } from "$lib/api/stuckthread";
  import {
    stuckqueryMssqlSnapshots,
    stuckqueryPgsqlSnapshots,
    stuckqueryMssqlQueries,
    stuckqueryPgsqlQueries,
    stuckqueryMssqlBlocking,
    type MssqlQuery,
    type PgsqlQuery,
    type MssqlBlockingRow,
  } from "$lib/api/stuckquery";
  import { threadKey } from "$lib/stuckthread";
  import { nearestByTimestamp } from "$lib/nearest";
  import { formatDuration, formatTimestamp } from "$lib/format";
  import StuckTable from "$lib/components/StuckTable.svelte";
  import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
  import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
  import BlockingTree from "$lib/components/BlockingTree.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import { cached } from "$lib/query-cache";

  let errorMessage = $state<string | null>(null);
  let threads = $state<StuckThread[]>([]);
  /** merged snapshot moments, both flavors */
  let snapshots = $state<{ timestamp: number; kind: "mssql" | "pgsql" }[]>([]);
  let selected = $state<StuckThread | null>(null);

  /** the snapshot matched to the selected episode */
  let matched = $state<{ timestamp: number; kind: "mssql" | "pgsql" } | null>(null);
  let mssqlQueries = $state<MssqlQuery[]>([]);
  let pgsqlQueries = $state<PgsqlQuery[]>([]);
  let blocking = $state<MssqlBlockingRow[]>([]);

  const Mode = { Queries: "queries", Blocking: "blocking" } as const;
  type Mode = (typeof Mode)[keyof typeof Mode];
  let mode = $state<Mode>(Mode.Queries);

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });

  /** the episode moment snapshots were logged at: the warning's log time */
  const targetOf = (t: StuckThread) => t.begin ?? t.end ?? 0;

  const delta = $derived(
    selected === null || matched === null
      ? null
      : matched.timestamp - targetOf(selected),
  );

  async function refresh() {
    const [threadsResult, mssqlResult, pgsqlResult] = await Promise.allSettled([
      cached("stuckthread_listview", () => stuckthreadListview()),
      cached("stuckquery_mssql_snapshots", stuckqueryMssqlSnapshots),
      cached("stuckquery_pgsql_snapshots", stuckqueryPgsqlSnapshots),
    ]);
    if (threadsResult.status === "fulfilled") threads = threadsResult.value;
    else errorMessage = String(threadsResult.reason);

    const merged: { timestamp: number; kind: "mssql" | "pgsql" }[] = [];
    if (mssqlResult.status === "fulfilled")
      merged.push(...mssqlResult.value.map((s) => ({ timestamp: s.timestamp, kind: "mssql" as const })));
    else errorMessage = String(mssqlResult.reason);
    if (pgsqlResult.status === "fulfilled")
      merged.push(...pgsqlResult.value.map((s) => ({ timestamp: s.timestamp, kind: "pgsql" as const })));
    else errorMessage = String(pgsqlResult.reason);
    snapshots = merged;
  }

  async function onselect(thread: StuckThread) {
    selected = thread;
    mode = Mode.Queries;
    const snap = nearestByTimestamp(snapshots, targetOf(thread));
    matched = snap;
    if (snap === null) return;
    try {
      if (snap.kind === "mssql") {
        const [queries, chains] = await Promise.all([
          cached(`stuckquery_mssql_queries:${snap.timestamp}`, () =>
            stuckqueryMssqlQueries(snap.timestamp),
          ),
          cached(`stuckquery_mssql_blocking:${snap.timestamp}`, () =>
            stuckqueryMssqlBlocking(snap.timestamp),
          ),
        ]);
        if (matched?.timestamp !== snap.timestamp) return; // stale guard
        mssqlQueries = queries;
        blocking = chains;
      } else {
        const queries = await cached(`stuckquery_pgsql_queries:${snap.timestamp}`, () =>
          stuckqueryPgsqlQueries(snap.timestamp),
        );
        if (matched?.timestamp !== snap.timestamp) return;
        pgsqlQueries = queries;
      }
    } catch (e) {
      errorMessage = String(e);
    }
  }

  $effect(() => {
    if (db.state.status === "open") {
      errorMessage = null;
      selected = null;
      matched = null;
      refresh();
    } else {
      threads = [];
      snapshots = [];
      selected = null;
      matched = null;
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

  <SplitPane direction="row" initial={0.42}>
    {#snippet a()}
      <StuckTable
        {threads}
        selected={selected === null ? null : threadKey(selected)}
        {onselect}
        view={null}
      />
    {/snippet}
    {#snippet b()}
      <div class="content">
        {#if selected === null}
          <p class="empty">Select a stuck episode to see the queries running at that moment.</p>
        {:else if matched === null}
          <p class="empty">No stuck-query snapshots in this bundle.</p>
        {:else}
          <div class="toolbar">
            <span class="match mono">
              snapshot {formatTimestamp(timeFormat, matched.timestamp)}
              {#if delta !== null}
                <span class="delta" class:far={Math.abs(delta) > 5000}>
                  ({delta >= 0 ? "+" : "−"}{formatDuration(Math.abs(delta))}
                  {delta >= 0 ? "after" : "before"} the warning)
                </span>
              {/if}
            </span>
            {#if matched.kind === "mssql" && blocking.length > 0}
              <span class="chips" role="group" aria-label="View">
                <button
                  class:active={mode === Mode.Queries}
                  onclick={() => (mode = Mode.Queries)}
                >Queries</button>
                <button
                  class:active={mode === Mode.Blocking}
                  onclick={() => (mode = Mode.Blocking)}
                >Blocking</button>
              </span>
            {/if}
          </div>

          <div class="body">
            {#if matched.kind === "mssql"}
              {#if mode === Mode.Blocking}
                <BlockingTree rows={blocking} />
              {:else}
                <MssqlQueryTable queries={mssqlQueries} />
              {/if}
            {:else}
              <PgsqlQueryTable queries={pgsqlQueries} />
            {/if}
          </div>
        {/if}
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
    justify-content: space-between;
    gap: 12px;
    padding: 6px 10px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }
  .match {
    font-size: 12px;
  }
  .mono {
    font-family: var(--font-mono);
  }
  .delta {
    color: var(--fg-muted);
  }
  /* a match seconds away is a different moment — say so loudly */
  .delta.far {
    color: var(--red);
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
