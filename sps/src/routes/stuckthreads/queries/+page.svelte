<script lang="ts">
  /**
   * Episode ↔ query matcher, linked-dumps anatomy: the left column stacks
   * the stuck-episode list OVER the query-snapshot list (both fully
   * visible); the right side shows the linked snapshot's queries or
   * blocking chains.
   *
   * Linking is SYMMETRIC and threshold-gated (seconds, persisted), exactly
   * like cpumemstats' Linked Dumps: clicking an episode highlights the
   * nearest snapshot within the threshold, clicking a snapshot highlights
   * the nearest episode. Outside the threshold the other side clears —
   * honesty over a far match. And it stays "what the DB was doing at that
   * instant", never "this thread ran this query" (no tid↔session mapping
   * exists in the logs).
   */
  import {
    stuckthreadListview,
    stuckthreadTrace,
    type StuckThread,
  } from "$lib/api/stuckthread";
  import StackTracePanel, {
    type TraceState,
  } from "$lib/components/StackTracePanel.svelte";
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
  import StuckTable from "$lib/components/StuckTable.svelte";
  import SnapshotList, {
    snapshotKey,
    type SnapshotRow,
  } from "$lib/components/SnapshotList.svelte";
  import MssqlQueryTable from "$lib/components/MssqlQueryTable.svelte";
  import PgsqlQueryTable from "$lib/components/PgsqlQueryTable.svelte";
  import BlockingTree from "$lib/components/BlockingTree.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import { cached } from "$lib/query-cache";
  import { persisted } from "$lib/persisted.svelte";
  import { page } from "$app/state";

  let errorMessage = $state<string | null>(null);
  let threads = $state<StuckThread[]>([]);
  let snapshots = $state<SnapshotRow[]>([]);

  let selectedThread = $state<StuckThread | null>(null);
  let selectedSnap = $state<SnapshotRow | null>(null);

  /** max distance (seconds) for an episode and a snapshot to be linked */
  const threshold = persisted("stuckquery-threshold-s", 30);

  let mssqlQueries = $state<MssqlQuery[]>([]);
  let pgsqlQueries = $state<PgsqlQuery[]>([]);
  let blocking = $state<MssqlBlockingRow[]>([]);
  let trace = $state<TraceState>({ status: "idle" });

  const Mode = { Queries: "queries", Blocking: "blocking" } as const;
  type Mode = (typeof Mode)[keyof typeof Mode];
  let mode = $state<Mode>(Mode.Queries);

  /** the episode moment snapshots were logged at: the warning's log time */
  const targetOf = (t: StuckThread) => t.begin ?? t.end ?? 0;

  /** seconds between the linked pair, when both sides are selected */
  const linkDelta = $derived(
    selectedThread !== null && selectedSnap !== null
      ? (selectedSnap.timestamp - targetOf(selectedThread)) / 1000
      : null,
  );

  /**
   * Episodes are fetched ONLY for the snapshot time range (padded by the
   * max threshold setting) — an episode with no snapshot anywhere near it
   * can never link, so fetching and rendering thousands of them here was
   * pure cost. The main /stuckthreads page remains the full list.
   */
  const RANGE_PAD_MS = 600_000; // = the threshold input's max (600 s)

  async function refresh() {
    // snapshots first: they define which episodes are worth fetching
    const [mssqlResult, pgsqlResult] = await Promise.allSettled([
      cached("stuckquery_mssql_snapshots", stuckqueryMssqlSnapshots),
      cached("stuckquery_pgsql_snapshots", stuckqueryPgsqlSnapshots),
    ]);

    const merged: SnapshotRow[] = [];
    if (mssqlResult.status === "fulfilled")
      merged.push(
        ...mssqlResult.value.map((s) => ({
          timestamp: s.timestamp,
          kind: "mssql" as const,
          detail:
            `${s.queries} queries` + (s.blocked > 0 ? ` · ${s.blocked} blocked` : ""),
          alert: false,
        })),
      );
    else errorMessage = String(mssqlResult.reason);
    if (pgsqlResult.status === "fulfilled")
      merged.push(
        ...pgsqlResult.value.map((s) => ({
          timestamp: s.timestamp,
          kind: "pgsql" as const,
          detail:
            `${s.queries} queries` +
            (s.waiting > 0 ? ` · ${s.waiting} waiting` : "") +
            (s.idleInTxn > 0 ? ` · ${s.idleInTxn} idle in txn` : ""),
          alert: false,
        })),
      );
    else errorMessage = String(pgsqlResult.reason);
    snapshots = merged.toSorted((a, b) => a.timestamp - b.timestamp);

    if (snapshots.length === 0) {
      threads = []; // nothing to link against — don't fetch episodes at all
      return;
    }
    const from = Math.max(0, snapshots[0].timestamp - RANGE_PAD_MS);
    const to = snapshots[snapshots.length - 1].timestamp + RANGE_PAD_MS;
    try {
      threads = await cached(`stuckthread_listview:${from}:${to}`, () =>
        stuckthreadListview(from, to),
      );
    } catch (e) {
      errorMessage = String(e);
    }
  }

  function resetSelection() {
    selectedThread = null;
    selectedSnap = null;
    mssqlQueries = [];
    pgsqlQueries = [];
    blocking = [];
    trace = { status: "idle" };
  }

  /** set the episode side and load its captured stack trace */
  async function selectThread(thread: StuckThread | null) {
    selectedThread = thread;
    if (thread === null) {
      trace = { status: "idle" };
      return;
    }
    if (thread.begin === null) {
      // completion-only episode: no warning event, so no trace exists
      trace = { status: "ready", tid: thread.tid, timestamp: targetOf(thread), frames: null };
      return;
    }
    const tid = thread.tid;
    const timestamp = thread.begin;
    trace = { status: "loading", tid, timestamp };
    try {
      const frames = await cached(`stuckthread_trace:${tid}:${timestamp}`, () =>
        stuckthreadTrace(tid, timestamp),
      );
      if (selectedThread === null || threadKey(selectedThread) !== threadKey(thread)) return;
      // the handler returns an empty Vec for "no trace captured"
      trace = { status: "ready", tid, timestamp, frames: frames.length === 0 ? null : frames };
    } catch (e) {
      trace = { status: "error", message: String(e) };
    }
  }

  $effect(() => {
    if (db.state.status === "open") {
      errorMessage = null;
      resetSelection();
      refresh();
    } else {
      threads = [];
      snapshots = [];
      resetSelection();
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refresh();
  });

  // Cross-page link: /stuckqueries navigated here with ?t=<ms>. Waits for
  // snapshots, selects the nearest one (should be exact — the sender picked
  // it from the same data), consumed ONCE so later clicks aren't overridden.
  let linkConsumed = $state(false);
  $effect(() => {
    if (linkConsumed) return;
    const raw = page.url.searchParams.get("t");
    if (raw === null) {
      linkConsumed = true;
      return;
    }
    const target = Number(raw);
    if (!Number.isFinite(target) || snapshots.length === 0) return; // wait for data
    const nearest = nearestByTimestamp(snapshots, target);
    if (nearest === null) return;
    linkConsumed = true;
    onselectsnapshot(nearest);
  });

  // --- symmetric linking (linked-dumps pattern) ------------------------------
  function loadSnapshot(snap: SnapshotRow | null) {
    mode = Mode.Queries;
    if (snap === null) return;
    const key = snapshotKey(snap);
    const guard = () => selectedSnap !== null && snapshotKey(selectedSnap) === key;
    if (snap.kind === "mssql") {
      Promise.all([
        cached(`stuckquery_mssql_queries:${snap.timestamp}`, () =>
          stuckqueryMssqlQueries(snap.timestamp),
        ),
        cached(`stuckquery_mssql_blocking:${snap.timestamp}`, () =>
          stuckqueryMssqlBlocking(snap.timestamp),
        ),
      ])
        .then(([queries, chains]) => {
          if (!guard()) return; // stale: selection moved on
          mssqlQueries = queries;
          blocking = chains;
        })
        .catch((e) => void (errorMessage = String(e)));
    } else {
      cached(`stuckquery_pgsql_queries:${snap.timestamp}`, () =>
        stuckqueryPgsqlQueries(snap.timestamp),
      )
        .then((queries) => {
          if (!guard()) return;
          pgsqlQueries = queries;
        })
        .catch((e) => void (errorMessage = String(e)));
    }
  }

  function onselectepisode(thread: StuckThread) {
    void selectThread(thread);
    const nearest = nearestByTimestamp(snapshots, targetOf(thread));
    selectedSnap =
      nearest !== null &&
      Math.abs(nearest.timestamp - targetOf(thread)) <= threshold.value * 1000
        ? nearest
        : null;
    loadSnapshot(selectedSnap);
  }

  function onselectsnapshot(snap: SnapshotRow) {
    selectedSnap = snap;
    loadSnapshot(snap);
    const candidates = threads.map((t) => ({ timestamp: targetOf(t), t }));
    const nearest = nearestByTimestamp(candidates, snap.timestamp);
    void selectThread(
      nearest !== null &&
        Math.abs(nearest.timestamp - snap.timestamp) <= threshold.value * 1000
        ? nearest.t
        : null,
    );
  }
</script>

<div class="page">
  {#if errorMessage}
    <div class="error-bar" role="alert">
      {errorMessage}
      <button onclick={() => (errorMessage = null)} aria-label="Dismiss">✕</button>
    </div>
  {/if}

  <div class="toolbar">
    <label class="threshold">
      link threshold
      <input
        type="number"
        min="1"
        max="600"
        bind:value={threshold.value}
        title="Max seconds between a warning and a snapshot to link them (applies to the next click)"
      />
      s
    </label>
    {#if linkDelta !== null}
      <span class="delta mono">
        linked · Δ {linkDelta >= 0 ? "+" : ""}{linkDelta.toFixed(1)}s
      </span>
    {:else if selectedThread !== null || selectedSnap !== null}
      <span class="delta muted">
        nothing within {threshold.value}s on the other side
      </span>
    {/if}
  </div>

  <div class="content">
    <SplitPane direction="row" initial={0.32}>
      {#snippet a()}
        <SplitPane direction="column" initial={0.5}>
          {#snippet a()}
            <div class="pane-block">
              <h3>Stuck episodes</h3>
              <StuckTable
                {threads}
                selected={selectedThread === null ? null : threadKey(selectedThread)}
                onselect={onselectepisode}
                view={null}
              />
            </div>
          {/snippet}
          {#snippet b()}
            <div class="pane-block">
              <h3>Query snapshots</h3>
              <SnapshotList
                rows={snapshots}
                selected={selectedSnap === null ? null : snapshotKey(selectedSnap)}
                onselect={onselectsnapshot}
              />
            </div>
          {/snippet}
        </SplitPane>
      {/snippet}
      {#snippet b()}
        <SplitPane direction="column" initial={0.62}>
          {#snippet a()}
            <div class="detail">
              {#if selectedSnap === null}
                <p class="empty">
                  {snapshots.length === 0
                    ? "No stuck-query snapshots in this bundle."
                    : "Select an episode or a snapshot to see the queries running at that moment."}
                </p>
              {:else}
                {#if selectedSnap.kind === "mssql" && blocking.length > 0}
                  <div class="detail-toolbar">
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
                  </div>
                {/if}
                <div class="detail-body">
                  {#if selectedSnap.kind === "mssql"}
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
          {#snippet b()}
            <div class="pane-block">
              <h3>Stuck thread stack trace</h3>
              <div class="trace-host">
                <StackTracePanel {trace} />
              </div>
            </div>
          {/snippet}
        </SplitPane>
      {/snippet}
    </SplitPane>
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
    gap: 16px;
    padding: 4px 10px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }
  .threshold {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--fg-muted);
  }
  .threshold input {
    width: 56px;
    padding: 2px 6px;
    background: var(--bg-hard);
    border: none;
    border-radius: var(--radius);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: 11.5px;
    text-align: right;
  }
  .delta {
    font-size: 11.5px;
  }
  .delta.muted {
    color: var(--fg-muted);
  }
  .mono {
    font-family: var(--font-mono);
  }

  .content {
    flex: 1;
    min-height: 0;
  }

  .pane-block {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .pane-block h3 {
    margin: 0;
    padding: 6px 10px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--fg-muted);
    background: var(--bg-hard);
    flex-shrink: 0;
  }
  /* The lists own the rest of their pane. height:auto matters: the list
     components set height:100%, which here would resolve against the WHOLE
     pane-block (h3 included) — overflowing the pane by one header and
     clipping the scrollbar's bottom. Flex sizing must win instead. */
  .pane-block > :global(*:last-child) {
    flex: 1;
    min-height: 0;
    height: auto;
  }

  .detail {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .detail-toolbar {
    display: flex;
    align-items: center;
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
  .detail-body {
    flex: 1;
    min-height: 0;
  }
  .trace-host {
    flex: 1;
    min-height: 0;
    overflow: auto;
  }

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
    font-size: 12.5px;
  }
</style>
