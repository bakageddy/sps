/**
 * CPU-monitoring queries — the contract between this frontend and the Rust
 * side. Each function wraps one Tauri command; the comment above it is the
 * signature to implement in src-tauri. Conventions (same as before):
 *
 *   - Err type serialized to String (`.map_err(|e| e.to_string())`) — it
 *     arrives as a rejected promise, never in the return type.
 *   - Response structs derive Serialize with `#[serde(rename_all = "camelCase")]`.
 *   - Timestamps are u64 ms since epoch.
 *   - Store lives behind AppDb (see api/database.ts); commands error when
 *     no database is open.
 *
 * The navigation model is DUMP-CENTRIC: a cpumonitoring log is a sequence
 * of dumps (one timestamped block of thread entries). The page lists dumps,
 * drills into one dump's threads, then into one thread's stack trace; the
 * time-series chart is the secondary "was this thread always hot?" view.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirror these as Rust structs)
// ---------------------------------------------------------------------------

/** One row per dump — feeds the dump list. */
export interface DumpSummary {
  /** ms epoch of the dump's header line; identifies the dump everywhere */
  timestamp: number;
  /** number of thread entries in this dump */
  threads: number;
  maxCpu: number;
  /** sum of all thread usage in this dump — overall load at that moment */
  totalCpu: number;
}

/** One thread entry within a single dump. Deliberately has no trace info:
 * frames load lazily via cpu_stacktrace when a thread is clicked, keeping
 * this query join-free. */
export interface DumpThread {
  tid: number;
  name: string | null;
  /** RUNNABLE, WAITING, ... */
  state: string;
  cpu: number;
}

export interface CpuPoint {
  /** ms epoch */
  timestamp: number;
  cpu: number;
}

/** One stack frame captured alongside a sample. Arrives in trace order
 * (top frame first) — which is only true if the query orders by idx. */
export interface StackFrame {
  method: string;
  source: string;
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/**
 * Every dump in the store, ordered by timestamp ascending.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpu_dumps(
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<DumpSummary>, String>
 * ```
 */
export function cpuDumps(): Promise<DumpSummary[]> {
  return invoke("cpu_dumps");
}

/**
 * All thread entries of ONE dump, ordered by cpu descending.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpu_dump_threads(
 *     timestamp: u64,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<DumpThread>, String>
 * ```
 */
export function cpuDumpThreads(timestamp: number): Promise<DumpThread[]> {
  return invoke("cpu_dump_threads", { timestamp });
}

/**
 * ONE thread's usage across all dumps — feeds the chart when a thread is
 * clicked. Single tid on purpose (the UI only ever plots the clicked
 * thread), and the response is just the points — no name (the client has
 * it from the clicked row) and no tid echo (it's the argument).
 *
 * ```rust
 * #[tauri::command]
 * async fn cpu_series(
 *     tid: u64,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<CpuPoint>, String>
 * ```
 * REQUIREMENT: points ordered by timestamp ascending (chart assumes sorted).
 */
export function cpuSeries(tid: number): Promise<CpuPoint[]> {
  return invoke("cpu_series", { tid });
}

/**
 * Stack trace captured for one (tid, timestamp) sample, ordered by idx.
 * null (None) = the sample was recorded WITHOUT a trace — mirroring the
 * parser's `trace: Option<CPUTrace>` instead of flattening None into an
 * empty vec. A returned array always has at least one frame.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpu_stacktrace(
 *     tid: u64,
 *     timestamp: u64,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Option<Vec<StackFrame>>, String>
 * ```
 * REQUIREMENT: frames ordered by idx ascending.
 */
export function cpuStacktrace(
  tid: number,
  timestamp: number,
): Promise<StackFrame[] | null> {
  return invoke("cpu_stacktrace", { tid, timestamp });
}
