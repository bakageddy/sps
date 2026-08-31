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
- [ ] **cpumemstats**: schema DONE (platform-split: windows_cpu_stats /
      windows_memory_stats / linux_stats, totals denormalized per row).
      Remaining: column-map parser (header is ground truth; jagged Total rows
      differ per platform), commands per `src/lib/api/cpumemstats.ts`
      (decide the behavior for a Windows dump present in only one table).
- [ ] **`cpumem_path_series`**: per-executable aggregate series (per-dump
      SUM over processes sharing a path, or a name when path is NULL) —
      requirements in `src/lib/api/cpumemstats.ts`. Frontend: rollup rows
      are clickable and plot it.
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

## Frontend (Claude)

- [ ] Nothing blocking. On request: `user` column in cpumemstats table,
      chart point-click → dump selection, ingest "indexing…" phase
      (see IDEAS.md).
