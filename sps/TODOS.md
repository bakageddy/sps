# TODOs

Concrete pending work. Backend items are Dinesh's; frontend items are Claude's.
Contract details live as doc comments in `src/lib/api/*.ts` — those files are
the spec; reconcile against them, not memory.

## Backend — blocking features

- [ ] **Event-driven `parse_logs`**: validate + return immediately; detached
      thread runs `thread::scope`, emits `ingest:started` / `ingest:file`
      (additive per-file deltas) / `ingest:error` / `ingest:finished`.
      Contract + Rust sketch: `src/lib/api/ingest-events.ts`.
      Kills: ParseReport::default() placeholder, mutex-held-during-parse,
      main-thread freeze.
- [ ] **Running-query commands** (11, requirements in
      `src/lib/api/runningquery.ts`):
      `runningquery_{mssql,pgsql,mssql_blocking,spwho2}_snapshots`,
      `runningquery_{mssql_queries,pgsql_queries,mssql_blocking,spwho2}(timestamp)`,
      `runningquery_{mssql,pgsql}_longrunning`, `runningquery_mssql_longtxns`
      — the stuckquery_* store bodies over the runningquery_* tables; sp_who2
      is the one new shape (`status::VARCHAR`, `lastbatch` ms epoch). Plus
      the hub resolver `connectiondump_runningqueries(timestamp, tolerance)`
      (see the incident entry). Frontend is live: /runningqueries and the
      incident page's Queries panel.
- [ ] **SQL console commands** (3, requirements in `src/lib/api/sql.ts`):
      `sql_query(sql, limit, offset)` (SELECT-only + single-statement gate
      ON THE PARSED STATEMENT, `LIMIT limit+1 OFFSET offset` probe →
      hasMore, NO DESCRIBE of the user statement, cells stringified in
      Rust from row values), `sql_schema()`
      (information_schema + COUNT(*)), `sql_export_csv(sql, path)` (`COPY
      (<sql>) TO path (FORMAT CSV, HEADER)`, returns rows written).
      Capability: `save()` is covered by the existing `dialog:default`.
      Frontend is live at /sql.
- [ ] **`healthmeter_info()`** (requirements in `src/lib/api/healthmeter.ts`):
      `SELECT val FROM main.healthmeter WHERE key = 'Server Time' LIMIT 1`,
      returned verbatim — `Result<Option<String>, String>`, None when the
      table's empty or has no such key. No zone/offset parsing on the Rust
      side; the table has no column identifying which capture a row
      belongs to, so which row comes back absent an ORDER BY is accepted as
      unspecified for now. Frontend is live: the Ingest page's Display
      timezone control (Bundle / Local / UTC / Custom + paste) runs the raw
      string through `extractZone()` itself, applied app-wide through
      `formatTimestamp`.
- [ ] **Notes commands** (3, requirements in `src/lib/api/notes.ts`):
      `notes_list()`, `notes_upsert(note)` (`INSERT OR REPLACE`, PK
      created_at), `notes_delete(created_at)`; table `main.notes(created_at
      UBIGINT PRIMARY KEY, text, updated_at, route)` in schema.sql — must
      SURVIVE re-ingest
      (never truncated with the log tables). Frontend is live: Ctrl+\
      editor (explicit Ctrl+S save) + sidebar section.
- [ ] **`save_database(path)`** (requirements in `src/lib/api/database.ts`):
      `ATTACH '<path>' AS target; COPY FROM DATABASE memory TO target;
      DETACH target;` on the live in-memory connection — copy only, no
      reopen/switch (the frontend calls `open_database(path)` after a
      successful save). Needs DuckDB ≥ 0.10 and an empty/new target path.
      Frontend is live: "Save as…" in the sidebar database control, shown
      only while the open database is in-memory.
- [ ] **cpumemstats**: schema DONE (platform-split: windows_cpu_stats /
      windows_memory_stats / linux_stats, totals denormalized per row).
      Remaining: column-map parser (header is ground truth; jagged Total rows
      differ per platform), commands per `src/lib/api/cpumemstats.ts`
      (decide the behavior for a Windows dump present in only one table).
- [ ] **`cpumem_path_series`**: per-executable aggregate series (per-dump
      SUM over processes sharing a path, or a name when path is NULL) —
      requirements in `src/lib/api/cpumemstats.ts`. Frontend: rollup rows
      are clickable and plot it.
- [x] **Stuck-thread commands** — DONE as `stuckthread_listview(from, to)`
      (aggregation in Rust: `get_stuckthread_aggregates`) +
      `stuckthread_trace(tid, begin)`. `src/lib/api/stuckthread.ts` mirrors
      the implemented shapes; frontend derives geometry only
      (`src/lib/stuckthread.ts`). Pages: /stuckthreads (strip + episodes /
      paths rollup + details/copy) and /stuckthreads/concurrency.
- [ ] **Stuck-query commands** (nine, requirements in
      `src/lib/api/stuckquery.ts`; pages live at /stuckqueries and
      /stuckthreads/queries; parser DONE): per flavor, NO unions —
      `stuckquery_{mssql,pgsql}_snapshots` (GROUP BY timestamp rollups),
      `stuckquery_mssql_blocking_snapshots` (SEPARATE command + separate
      list rows in the UI: chains + sessions per timestamp),
      `stuckquery_{mssql,pgsql}_queries(timestamp)`,
      `stuckquery_mssql_blocking(timestamp)` (pgsql has no blocking table),
      `stuckquery_{mssql,pgsql}_longrunning` (executions present in >1
      snapshot; identity mssql = session_id+txn_id, pgsql = pid+query),
      `stuckquery_mssql_longtxns` (GROUP BY session_id+txn_id, >1 snapshot,
      queries = DISTINCT statements in first-seen order).
- [ ] **Connection-dump commands** (five, requirements in
      `src/lib/api/connectiondump.ts`; page live at /connectiondump;
      parser DONE: Signal / PoolStats / Trace entries in
      `parser/connectiondump.rs`): `connectiondump_signals(from?,
      to?)` (alarm timeline; parser still needs a `suppressed` flag to split
      real dumps from "Skipping to dump" lines; Cause serialized via
      as_str, not enum-tagged),
      `connectiondump_pool_stats(from?, to?)` (used/free/total series),
      `connectiondump_snapshots()` + `connectiondump_traces(timestamp)`
      (browser pair, stuckqueries pattern), `connectiondump_holders(from?,
      to?)` (episodes grouped by **(thread id, startTime)** — id alone is a
      reused Java thread id, NOT a connection id; same id + same startTime
      across dumps = one continuing hold. Persist `appFrame` = first
      non-plumbing stack frame at store time; dedup full stacks by hash).
- [ ] **NMC correlation — threaddump link (future)**: when the pool is
      exhausted, a thread dump in the window shows the WAITER side directly
      (threads parked in `ConnectionPool.getConnection` /
      `getConnDetailFromPool`); cd0 traces are the HOLDER side. Join by
      thread name/id (`http-nio-8080-exec-11` style names appear in both) —
      needs the threaddump parser first. Same window pattern also joins
      stuckthreads (victims) and stuckqueries (why holders won't release).
- [ ] **Thread-dump commands** (three, requirements in
      `src/lib/api/threaddump.ts`; page live at /threaddump; parser DONE):
      `threaddump_dumps()` (GROUP BY timestamp + per-state conditional
      counts), `threaddump_threads(timestamp)` (census; State payload
      FLATTENED onto the row — struct variants + `#[serde(tag="state")]` +
      `#[serde(flatten)]` on Thread.state, or a DTO; wire names
      waitingOn/lock/lockOwnerTid/lockOwnerName, BLOCKED-first ordering),
      `threaddump_trace(tid, timestamp)` (Option<Vec<Element>>, frames and
      lock lines in ORIGINAL interleaving, `#[serde(tag="kind")]`, Frame
      tuple → {method, source}). Deadlock detection is FRONTEND
      (`src/lib/threaddump.ts`) — no command.
      PLUS `threaddump_thread_series(tid)` → the thread's row per dump it
      appears in (same flattened State payload; plain SELECT WHERE tid =
      $1, no join to traces). Powers the per-thread timeline on /threaddump
      (cpumonitoring model: strip over trace; consecutive rows with the
      same state/waitingOn/lockOwnerTid = unchanged = stuck). NO STACK
      FINGERPRINTING anywhere — Dinesh's call; stacks are never hashed.
- [ ] **Connection-dump incident resolvers** (three now, one later;
      requirements in `src/lib/api/connectiondump.ts`; page live at
      /connectiondump/incident): each `connectiondump_<subsystem>(signalTs,
      tolerance) -> Option<u64>` = the LATEST dump of that log within
      `tolerance` ms of signalTs (MAX over BETWEEN — not nearest: each
      trigger dumps cpumonitoring ×2 / cpumemstats ×3 / threaddump ×3, and
      the last sample is the settled one). `Result<Option<u64>, String>`:
      no dump in window → Ok(None) (empty table included); DB/query
      failure → Err like every other command.
      `tolerance` is a PARAMETER (live knob); frontend default is still
      4000 = first-dump alignment only — RAISE to the burst span once the
      repeat interval is known (needed anyway to confirm the cpumonitoring
      2s fragment-merge can't swallow a repeat). Subsystems:
      `threaddump`, `cpumonitoring`, `cpumemstats`, `runningqueries`
      (UNION of the four runningquery_* tables' distinct timestamps; the
      log is periodic, so LATEST-in-window = the tick just after the
      signal). NO joined census command — the frontend joins
      threaddump ⋈ cpu ⋈ holds by exact tid (`lib/connectiondump.ts
      buildCensus`) from the existing per-timestamp commands.
      EXCLUDED from the hub on purpose: stuck threads AND stuck queries —
      both ride the valve trigger, never a performance-dump signal.
- [ ] **Overview commands**: `cpumem_total_cpu` / `cpumem_total_memory` —
      requirements in `src/lib/api/cpumemstats.ts` (frontend page is live at
      /cpumemstats/overview).
- [ ] **`database_info` shape**: return `Option<DatabaseInfo { path }>`
      (always Some — store always exists; path None = in-memory). Currently
      `Option<String>`, which breaks the sidebar chip.

## Backend — future features

- [ ] **Full-text search over stacktraces** (cpumonitoring_stacktraces first).
      Later expands to search across threaddumps, stuckthreads, and cpu
      threads once their parsers exist (most of those log kinds are still
      TODO parser-side). Frontend will need a search page/contract when the
      backend shape settles.

## Backend — correctness

### Stuckthread (from review 2026-09-01/02; Dinesh fixing)

- [ ] Parser: header path violates the PROGRESS INVARIANT — a line not
      starting with `[` is never consumed, so next() returns the same
      Err(HeaderNotFound) forever (infinite loop). Real serverout files are
      mixed-content, so this fires on normal input. (The `while ... break`
      is an `if` in costume — original skip-until-header intent lost.)
- [ ] Parser: add a mixed-content fixture (valve entries with foreign log
      lines between them + one corrupt entry) asserting termination, one
      Err per bad segment, and correct event count. Current fixtures are
      pure valve lines and cannot catch the above.
- [ ] Parser: `ParseTID` error message is a copy-paste of HeaderNotFound's
      ("Cannot find header...") — lies at the boundary. Also
      `timestamp - duration` is a naked u64 subtraction (garbled duration
      → debug panic).
- [ ] Parser: comment the atomically-written-record invariant where
      `has_stacktrace` classifies Begin/End (it's load-bearing).
- [x] ~~Old aggregator (`get_stuckthreads_aggregate_minimal`) bugs
      (double-emit, dropped orphaned ends, HashMap flush order)~~ —
      OBSOLETE: replaced by `get_stuckthread_aggregates`.
- [ ] NEW aggregator (`get_stuckthread_aggregates`): the "found a pending
      begin" branch fires for ANY row with that tid — a re-reported WARNING
      closes the pending episode as if it completed (one real episode →
      several short "completed" ones) and then opens a fresh one. Check
      `request.is_none()` before pairing; merge re-reports instead.
- [ ] NEW aggregator: `begin` is the raw warning log-line timestamp — the
      valve fires AFTER the threshold, so episodes render ~threshold late.
      Decide: keep as-is (frontend displays what it gets) or derive
      begin = ts - duration. Write the decision down either way.
- [ ] Pairing safety net (end names are EMPTY so name-join is impossible):
      verify pairs by deriving start from both sides (begin.ts - begin.dur
      ≈ end.ts - end.dur within epsilon); mispairs from missing ends fail
      this check by construction.
- [ ] CONFIRMED in the field: corrupt timestamps (~1.8e19 ms, i.e. wrapped
      u64 subtraction in a release build) reached the frontend and crashed
      the /stuckthreads render until it grew guards (`formatTimestamp`,
      unkeyed eaches, `<svelte:boundary>`). The guards make it survivable,
      not correct — fix the subtraction at the source (see the ParseTID /
      naked-u64 parser item above).

- [ ] `get_stackframes`: add `ORDER BY idx` (row order is not guaranteed
      without it; preserve_insertion_order is likely, not promised).
- [ ] `get_cpu_series`: add `ORDER BY timestamp` (chart line + bisect assume
      sorted points).
- [ ] Error message policy sweep: pick embed-source OR chain-rendering
      (recommended: chain — bare per-layer Display + `report()` helper at the
      handler boundary walking `source()`), apply to ALL variants.
      Currently mixed; `#[error("Parsing Error")]` drops its source's story.
      Also: "quering" → "querying".
- [ ] Decide file-level atomicity for ingest: partial-file-on-corruption is
      the current (silent) choice; transaction-per-file gives all-or-nothing
      while keeping streaming. Write the decision as a comment on
      `append_cpumonitoring`.
- [ ] Ban panics in command-reachable code: `state.lock().unwrap()`,
      `store.get().unwrap()` in the parse thread. A panic in a Tauri command
      on Linux aborts the process (nounwind FFI boundary). Consider
      `#![warn(clippy::unwrap_used)]`.

## Backend — performance (measure first)

- [ ] Flamegraph the ingest 0.7s: `cargo flamegraph --release -- parse ...`.
      Wrapper per-value allocation (CString towers under append_row) →
      Arrow appender (IDEAS.md). Time inside libduckdb → it's the DB working;
      check preserve_insertion_order=false and re-bench against in-memory DB
      to isolate the file-backed WAL/checkpoint tax.
- [ ] Pin the pool-size invariant in a comment:
      `pool = 12 log kinds; no query headroom needed because ingest ≤ 1s` —
      and note dev (debug) builds violate the ≤1s assumption daily.

## Hygiene

- [ ] `util.rs`: remove the `let something = Vec::new()` lifetime probe
      (doesn't compile as-is); future probes → `#[cfg(test)]` or
      `_named // TODO(dinesh)` so WIP is legible.
- [ ] Split `util.rs` into `ingest` (discovery + orchestration + pipeline)
      and an `fs`/mmap helper; make `Store.0` private behind methods
      (`init`, `path`, `pool_clone`/`with_conn`).
- [ ] Move shared row types out of `handlers::types` if store keeps importing
      them (storage layer depending on wire layer is inverted).
- [ ] Zoom permission: if Ctrl+± rejects, add
      `"core:webview:allow-set-webview-zoom"` to `capabilities/default.json`.

## Backend — index fragmentation page (contract: `src/lib/api/fragmentation.ts`)

- [ ] `fragmentation_snapshots() -> Vec<FragSnapshot>` — one row per
      capture (index count, max ratio, total pages), timestamp ascending.
- [ ] `fragmentation_indices(timestamp: u64) -> Vec<FragIndex>` — one
      capture's rows, ratio descending.
- [ ] `fragmentation_index_series(schema, table, index) -> Vec<FragPoint>`
      — one index across captures, timestamp ascending.
      Frontend treats `ratio` as 0–100 (avg_fragmentation_in_percent); if
      the log prints 0–1, scale in the parser, not here.
- [ ] `fragmentation_flagged(min_pages: u64, min_ratio: f64) -> Vec<FragFlag>`
      — one row per index flagged (ratio ≥ min_ratio AND pages ≥ min_pages)
      in at least one capture, with capture counts, first/last flagged
      time and the newest reading; latest ratio descending. Thresholds
      come from the UI (defaults 200 pages / 30 %).

## Backend — Glowroot ingest (contracts: `src/lib/api/glowroot.ts`, `glowroot-events.ts`; wire reference: `docs/glowroot-http-api.md`)

The frontend page (`/glowroot`, last in the sidebar beside the SQL Console) is done and waits on these. Nothing under
`src-tauri/` was touched. The `ingest` module stub and the commented-out
`pub mod ingest;` in `lib.rs` are yours.

**Commands** (exact names and argument shapes are in the contract file):
- [ ] `glowroot_connect(url)`, `glowroot_login(username, password)`,
      `glowroot_disconnect()` — one session at a time in `AppState`: a
      `reqwest` client with a cookie jar, HTTP/1.1 only, idle pool of 32 per
      host, 10 s connect timeout, 60 s request timeout, plus the base URL and
      the cached layout. Failed login is HTTP 200 with an incorrect-login
      flag, not a 401.
- [ ] `glowroot_agents(top_level_id, from, to)` — Central only; top-level
      rollups, or one rollup's children. Sorted by display name.
- [ ] `glowroot_agent_layout(agent_rollup_id)` — transaction types and the
      permission flags. Embedded reads the layout cached at connect.
- [ ] `glowroot_count(options)` — one trace-count request per selected type
      and family; same filters as listing (duration band, name, headline,
      user, error message, attribute, window); families switched off report 0.
- [ ] `glowroot_ingest(options) -> run id` — validate, register the run,
      emit `glowroot:started`, spawn the work, resolve. One run at a time.
- [ ] `glowroot_cancel(run_id)` — cooperative and idempotent.
- [ ] `glowroot_runs()` — history from the database, newest first; a run
      still marked running after a restart is reported as failed.

**Events** (cumulative totals from the single writer, at most about five per
second; every payload carries the run id): `glowroot:started`, `listing`,
`listed`, `progress`, `problem`, `finished`. Order per run: started, listing
repeatedly, listed, progress repeatedly, finished, with problems in
between. `finished` fires exactly once for every outcome, after the last
progress event. First 200 problems individually, the rest only counted.

**Filters.** `headline`, `user`, `errorMessage` and `attribute` in the options
are optional text filters (text plus a comparator: begins, equals, ends,
contains, not-contains; strings on the wire, so a serde enum with
`rename_all = "kebab-case"`), ANDed with each other and with the duration band.
They map to the headline, user, error-message and attribute parameters and
their comparator parameters in the points and count requests (see section 3
of `docs/glowroot-http-api.md`). Send a filter only when it is set. The
error-message filter goes to the error family only; the slow family is
listed without it. The attribute filter always has a name and a value. Not
verified against the source: whether Glowroot accepts an attribute name with
an empty value, and whether it ignores the error-message filter on slow
listings. The UI never sends either case.

**Listing.** Per selected type and family, ask for points with a limit of
5000. When the limit is exceeded, split the window in half and recurse; stop
splitting under one second and report the truncation as a problem. Ignore
partial points. De-duplicate on agent id plus trace id across families and
sub-windows, sort by capture time newest first, drop traces already stored
when asked, then cut to the cap.

**Fetching.** A semaphore sized to `workers`; per trace: header first, then
entries, queries and both profiles only when the option is on and the
header's existence flag says yes. Forward the check-live-traces flag when the
point carried it. Retry three times with backoff on timeouts and 5xx. A 404
or expired answer for a part is not a failure: store the header with the
flag. Resolve truncated query texts once per hash (shared map). Check the
cancellation token before every request.

**Writing.** One writer thread owns the appenders and is the only emitter.
Fetch tasks hand finished traces over a channel. Flush about every 500
traces. On cancel, drain what is already queued, flush, finalise the run as
cancelled. Re-ingesting a stored trace replaces its rows. A 401/403 mid-run
(expired session) ends the run as failed with that message.

**Tables** (add to `schema.sql` with `IF NOT EXISTS` and name them in
`store/tables.rs`, or the CI schema gate fails). Times are UTC ms, durations
are nanoseconds unless the name says otherwise:
- [ ] `glowroot_runs` — id, collector URL, agent rollup id, options as JSON,
      started and finished ms, outcome, total, stored, failed.
- [ ] `glowroot_traces` — agent id, trace id, run id, collector URL, agent
      display, transaction type and name, headline, user, start and capture
      ms, duration, async and partial flags, error message and exception
      class, main and aux thread stats (cpu, blocked, waited, allocated),
      entry, query and profile-sample counts with their limit-exceeded flags,
      the three existence flags, location stack as a JSON list.
- [ ] `glowroot_trace_attributes` — agent id, trace id, name, value (one row
      per value).
- [ ] `glowroot_trace_timers` — agent id, trace id, index, parent index,
      depth, kind (main, aux, async), name, total, count, extended.
- [ ] `glowroot_trace_entries` — agent id, trace id, index, parent index,
      depth, start offset, duration, message, query text hash, prefix,
      suffix, error message, location stack, detail as JSON.
- [ ] `glowroot_trace_queries` — agent id, trace id, query type, query text
      hash, total duration, execution count, total rows (nullable).
- [ ] `glowroot_query_texts` — hash (unique), full text, truncated flag.
      Texts the server sent without a hash get one computed locally.
- [ ] `glowroot_trace_profiles` — agent id, trace id, kind (main, aux),
      unfiltered sample count, tree as JSON (flattening waits for the trace
      browser).

**Dependencies to add:** `reqwest` (json, cookies, rustls-tls), `tokio-util`
(cancellation token). Tauri's async runtime already provides tokio.

## Frontend (Claude)

- [ ] Trace browser for the ingested Glowroot traces (list, filter, detail
      with timer and entry trees, per-trace queries) once the backend lands.
- [ ] Nothing else blocking. On request: `user` column in cpumemstats table,
      chart point-click → dump selection, ingest "indexing…" phase
      (see IDEAS.md).
