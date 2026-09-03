# Ideas

Parked concepts and design directions — not scheduled work (that's TODOS.md).
Each entry: the idea, why it might matter, and the trigger that should revive it.

## Ingestion performance

- **Arrow appender (`append_record_batch`, `appender-arrow` feature).**
  Replace per-row `append_row` with columnar batches built straight from mmap
  slices: ~3.5M per-value FFI/alloc events collapse into ~85 per-batch bulk
  copies. Pipeline becomes: zero-copy parse, single bulk copy at the ownership
  boundary, O(batches) allocations.
  *Trigger: flamegraph shows appender-side per-value conversion/allocation
  (CString towers) dominating the ingest 0.7s.*

- **Staged pipeline (parse thread → channel → writer thread).**
  Overlaps parse (0.3s measured) with append: total ≈ max(stages) not sum.
  Only worth it if BOTH stages remain significant after the Arrow fix.
  *Trigger: bench shows two comparable stage costs.*

- **Performance floor ("splice" ideal).** Every input byte read once, every
  output byte written once, plus irreducible transform CPU. Current design is
  AT the floor structurally — remaining time is transform CPU or
  overhead-above-floor. Use flamegraph to separate; stop optimizing at the floor.

- **Schema-on-read as boundary move.** DuckDB can query files in place
  (read_parquet etc.) — ingestion becomes metadata-only. Doesn't fit our
  custom format + repeated queries, but remember the shape: "can ingestion
  not exist?" is a legitimate first question for ingest-speed problems.

- **Keyset pagination on the dump lists** (built 2026-08-30, then removed
  in favor of caching alone — "for now"). Full design existed and worked:
  optional PageRequest {sortBy, descending, limit, cursor:{key, timestamp}}
  on cpu_dumps/cpumem_dumps (null = everything); frontend had a
  createPagedList runes factory (epoch staleness guard, exhausted-on-short-
  page) + IntersectionObserver infinite scroll, server-side sort (header
  click resets — a keyset cursor is only valid within one ordering).
  Keyset chosen over offset because DuckDB zonemaps prune the cursor
  predicate while OFFSET pays a top-N heap of n+k. Recoverable from git
  history around this date.
  *Trigger: dump lists grow big enough that fetching/rendering them whole
  visibly lags (thousands of rows), or memory pressure from the cached
  full lists.*

## Schema / analysis

- **Parse-time `dump_id` column.** The parser sees dump block structure; a
  dump_id (or dump_ts) column turns fuzzy timestamp grouping into plain
  GROUP BY and gives per-dump queries an exact clusterable key. Fixes the
  slow-logger aggregation wrinkle correctly (vs gaps-and-islands SQL, which
  works but reconstructs what the parser already knew).
  *Trigger: dumps visibly split/merged wrong in the UI, or before the schema
  is otherwise frozen.*

- **Gaps-and-islands sessionization (SQL alternative to dump_id).**
  lag() to mark >2s gaps, running sum() as island id, GROUP BY island.
  Contract impact if adopted: DumpSummary carries start/end timestamps,
  cpu_dump_threads takes a range, DumpThread rows carry their exact timestamp.

- **Bulk-load-then-index.** Data is immutable after ingest → keep schema bare
  for the appender, CREATE INDEX (cpumonitoring(tid), stacktraces(tid)) as a
  post-ingest finalize step. Zonemaps already cover timestamp predicates
  (ingestion is timestamp-ordered). ART index only helps the scattered-tid
  lookups (cpu_series).
  *Trigger: EXPLAIN ANALYZE shows cpu_series scans actually hurting.*

- **Frontend series cache.** Series are immutable per (tid|pid,
  ingest.generation) — cacheable with generation as the invalidation token.
  *Trigger: a series click ever feels slow. (Unlikely; DuckDB is microseconds.)*

## UI / features

- **SQL query executor page** (SELECT-only, backend-enforced; preview capped
  with `truncated` flag; CSV export via DuckDB's own
  `COPY (<sql>) TO 'path' (FORMAT CSV, HEADER)` so no rows cross IPC —
  frontend supplies path from the save dialog). Sketched contract:
  run_query(sql) -> {columns, rows: (string|null)[][], truncated};
  export_query_csv(sql, path) -> rows written.
  Open: preview cap fixed vs user LIMIT; query history via persisted();
  schema sidebar.

- **Tabs** — frontend-only, editor-style (tab bar = $state list of
  {id, kind, params}); keep-mounted (hidden) strategy preserves per-tab state.
  Shape undecided: tabs of dumps/threads within an analyzer vs analyzers as tabs.

- **cpumemstats chart point-click** → select that dump in the list (parity
  with cpumonitoring's chart-click → trace-at-that-dump).

- **`user` column in cpumemstats process table** (Linux dumps carry it;
  ProcessRow already ships it — five-minute frontend add).

- **Ingest "indexing…" phase in IngestState** if bulk-load-then-index lands
  (parse_logs gets a finalize tail; surface it via a new event).

- **Stuckquery correlation** — stuckquery logs print the queries running in
  the DB whenever a stuckthread fires; correlate them with episodes. Panel
  in episode details: "queries in flight at this moment" (nearest dump(s) to
  the episode window). SCHEMA ADVICE captured 2026-09: per-query rows with a
  `dump_timestamp` column (u64 ms, join key); raw query text unmodified
  (future FTS); capture the executing thread/tid if the log has it — a tid
  match turns "around the same time" into "this thread ran this query".
  *Trigger: stuckquery parser lands.*

- **Stuckthread episode extras** (small, post-aggregator-fix): re-report
  count on episodes ("warned 4×") once re-reports merge instead of
  splitting; duration histogram for the window; episode → nearest
  cpumonitoring dump via the existing `?t=` link pattern.
  *Trigger: aggregator re-report fix lands / Dinesh asks.*
