# TODO

## Features
- [x] MCP Integration Ongoing
- [ ] HTTP 1.1 API Exposure
- [ ] Full Text Search FTS5 using SQLITE
- [ ] cd0.txt parser
- [ ] Query Analysis (runningqueries, stuckqueries) Parsers
- [ ] Query Plan Analysis (pg_log) Parsers
- [ ] Memory Dump Overview with memoryanalyzer
- [ ] Integrate sea-query for queries.
- [ ] Review Schema and Add appropriate indices after usage of the application

## TOOLS
- [x] get_stuckthread_range
- [x] get_stuckthread_summary
- [x] get_trace_by_id
- [x] get_stuckthread_aggregate
- [x] get trace by id(s)
- [ ] [DEFERRED] stuckthread_summary_range
- [ ] [DEFERRED] compare windows
- [ ] build flamegraph/waterfall graph (does it even make sense for a flamegraph for the stuckthread data)

## MCP Tool Suggestions for Analysis

These tools are designed around the reasoning flow an LLM follows during a JVM thread-dump analysis session:
1. **Overview**: How bad is it? When? How many?
2. **Patterns**: What's the dominant blocking pattern?
3. **Drill-down**: What stacks are involved?
4. **Correlation**: Who's blocking whom? Deadlock? Hot lock? Slow DB?
5. **Timeline**: How did this evolve?

### 1. Stack-Trace–Centric Tools (highest value)

- [ ] **`get_top_stacks_by_frequency`** — Group threads (across all dumps) by stack hash, return top N most common stacks. Single most valuable analysis primitive: instantly reveals "200 threads all blocked in `JdbcConnection.getConnection`".
    - Params: `limit`, `state_filter` (e.g., only BLOCKED), `time_range`
    - Returns: `Vec<{ trace_id, count, sample_thread_ids, peek_frames }>`
- [ ] **`search_threads_by_frame`** — Find all threads/stuckthreads whose stacks contain a specific class or method (e.g., "find anything calling `OracleStatement.execute`"). grep-style fuzzy search.
    - Params: `pattern` (LIKE/regex), `match_type` (method/class/file), `time_range`
    - Returns: matching thread/trace summaries
- [ ] **`get_stack_diff`** — Given two trace IDs (or two snapshots of the same thread), show the diff of frames. Answers "did this thread make progress?"
- [ ] **`get_hot_frames`** — Across all threads in a time range, return top N most-frequent methods/classes anywhere in the stack. Poor-man's flame graph in tabular form.
    - Returns: `Vec<{ method, occurrence_count, distinct_thread_count, sample_trace_ids }>`

### 2. Lock Contention Tools (deadlock detection — huge value)

The schema already has `wait_object_id`, `lock_object_id`, `owner_id` on `thread`, but no tool exposes this. This is gold for diagnosis.

- [ ] **`get_lock_contention_graph`** — For a snapshot, return the wait-for graph: `{ thread_id, waiting_for_object, owned_by_thread_id }` tuples. LLM can then reason about cycles or hot locks.
- [ ] **`detect_deadlocks`** — Cycle-detection in SQL (recursive CTE) or in Rust. Return detected deadlock cycles with the threads and locks involved. Even without a formal deadlock, the longest wait chain is super useful.
- [ ] **`get_hot_locks`** — Return objects (locks) sorted by how many threads are waiting on or blocked by them. "500 threads waiting on `OracleConnectionPool$1@0xdeadbeef`" → instant diagnosis.
    - Returns: `Vec<{ object_id, class, identity, waiters_count, blocked_count, owner_thread_id, snapshot_count }>`
- [ ] **`get_lock_owner_chain`** — Given a thread, follow `owner_id` recursively to find the root of its blockage. Often the root owner is doing something different (DB call, native I/O) — that's the real bug.

### 3. Thread Dump Snapshot Tools (currently a big gap!)

No tools currently expose regular `threaddump` data — only stuckthread.

- [ ] **`list_threaddumps`** — Get available threaddumps (id, snapshot number, timestamp, thread count, blocked count, waiting count). Entry point: "what snapshots exist?"
- [ ] **`get_threaddump_summary`** — For a specific dump: state distribution (counts of RUNNABLE/BLOCKED/WAITING/etc.), unique stack count, deepest stack, longest thread name. Vital-signs view of one dump.
- [ ] **`get_threads_in_dump`** — Filter threads in one dump by state, name pattern, or presence of a frame.
- [ ] **`compare_dumps`** — Given two dump IDs, show:
    - Threads present in both (with state change)
    - New threads
    - Disappeared threads
    - Threads with same stack vs. progressing
    
    Classic technique for finding stuck threads: take two dumps 30s apart, see who hasn't moved.

### 4. Timeline / Temporal Tools

- [ ] **`get_stuckthread_timeline`** — Histogram of stuckthread starts bucketed by time (e.g., per-minute counts). Lets the LLM say "the problem started at 14:23".
    - Params: `bucket_ms`, `time_range`
    - Returns: `Vec<{ bucket_start_unix_ms, count, distinct_request_count, avg_duration }>`
- [ ] **`get_thread_history`** — Given a thread_id, return all its appearances across dumps in time order, with state and trace_id at each point. Trace one thread's lifecycle.
- [ ] **`get_stuck_periods`** — Find time windows where stuckthread count exceeds a threshold. "Bad windows" of the application.

### 5. Request / Endpoint Correlation

- [ ] **`get_slow_requests`** — Group by `request` with avg/max/p95 durations and counts. (`get_stuckthread_aggregate` does this; reframing as "slow requests" gives clearer semantics.)
- [ ] **`get_requests_with_common_stack`** — Find requests whose stuckthreads share a common stack prefix. "All `/api/orders` calls block on the same code path."

### 6. Schema / Meta Tools (don't skip these)

- [ ] **`get_database_overview`** — Counts of: threaddumps, stuckthreads, unique stacktraces, unique objects, time range covered, distinct thread names, distinct requests. Recommended as the **first** tool the LLM calls.
- [ ] **`get_schema`** — Return the schema (or a curated semantic version). Even a hardcoded prose description as a tool helps the LLM compose follow-up queries.
- [ ] **`run_sql_query`** (sandboxed) — Read-only SQL execution tool with a safety check (reject `INSERT`/`UPDATE`/`DELETE`/`DROP`/`ATTACH`/`PRAGMA write`). Power-user escape hatch when fixed tools don't fit. Alternative: constrained version `run_aggregate_query { table, group_by[], filter[], measure[] }`.

### Quality-of-life improvements to existing tools

- [ ] `get_stuckthreads_between_range` returns full `Trace` for every thread — enormous and exceeds context. Add `peek_only: bool` (first 10 frames) and `limit`. Default to peek.
- [ ] `Trace::get_by_id` has an N+1 problem with `Object::get_by_id`. JOIN instead.
- [ ] **🐛 BUG**: `get_longest_stuck_thread` uses `ORDER BY active_duration_ms LIMIT 1` — that's ascending, returns the *shortest*. Should be `DESC`.
- [ ] Return trace digests (short fingerprint hash, e.g., `crc32` of top 5 frames) in summary tools. Lets the LLM group "these 30 stuckthreads share digest `0xabcd1234`" without dumping full traces.
- [ ] Add cursor/pagination to range queries. Default `limit: 50` with `offset` saves context.

### Tool-design principles to apply across all tools

1. **Return IDs, not full payloads, in list/aggregate tools.** Let the LLM ask for details only for IDs it cares about. (`get_trace_by_ids` is great for this — keep the pattern.)
2. **Include UTC strings alongside `unix_ms`.** Already done in `StuckThreadSummary`; do it everywhere.
3. **Add "Use this when..." examples in tool descriptions.** Models pick the right tool faster.
4. **Make filters consistent.** Every list tool should accept the same `start/end/limit` shape.
5. **Return a `peek` field** (first 3–5 frames) in any "list of threads" tool so the LLM has signal without fetching full traces.

### Priority order

| Priority | Tool | Why |
|----|----|----|
| 🥇 P0 | `get_top_stacks_by_frequency` | Single most useful query for any thread-dump analysis |
| 🥇 P0 | `get_database_overview` | LLM entry point — tells it what to ask next |
| 🥇 P0 | `list_threaddumps` + `get_threaddump_summary` + `get_threads_in_dump` | Threaddumps are unreachable currently |
| 🥈 P1 | `get_hot_locks` + `detect_deadlocks` | Killer feature for JVM diagnosis |
| 🥈 P1 | `compare_dumps` | Classic "is it stuck?" technique |
| 🥈 P1 | `search_threads_by_frame` | LLMs love substring search |
| 🥉 P2 | `get_stuckthread_timeline` | "When did it start?" |
| 🥉 P2 | `get_thread_history` | Drill-down |
| 🥉 P2 | `run_sql_query` (sandboxed) | Power-user escape hatch |

