/**
 * CPUMemStatistics queries, matched to the platform-split storage:
 *   windows_cpu_stats(timestamp, total, path, cpu, pid, name)
 *   windows_memory_stats(timestamp, total, path, mem, pid, name)
 *   linux_stats(timestamp, total_cpu, total_mem, user, name, pid, cpu, mem, path)
 *
 * The frontend is platform-agnostic: every command returns merged rows
 * covering both platforms; the frontend never knows which table a row
 * came from.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * `#[serde(rename_all = "camelCase")]`, u64 ms timestamps.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirror these as Rust structs)
// ---------------------------------------------------------------------------

/** One row per dump — feeds the dump list. */
export interface CpuMemDumpSummary {
  /** ms epoch of the dump block */
  timestamp: number;
  totalCpu: number;
  /** MB */
  totalMemory: number;
}

/** One process in one metric's table. `value` is CPU % or MB depending on
 * which list it came from. */
export interface ProcessUsage {
  pid: number;
  name: string;
  /** Linux rows only; null for Windows rows */
  user: string | null;
  value: number;
  /** null = the log had no path for this process (Option<Cow> backend-side) */
  path: string | null;
}

/** One point of a per-process series; `value` unit follows the series. */
export interface MetricPoint {
  /** ms epoch */
  timestamp: number;
  value: number;
}

/** A pid's history across all dumps, one array per metric. */
export interface ProcessSeries {
  cpu: MetricPoint[];
  memory: MetricPoint[];
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/**
 * Every CPUMemStatistics dump, ordered by timestamp ascending.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_dumps(
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<CpuMemDumpSummary>, String>
 * ```
 * REQUIREMENTS: one row per dump across BOTH platforms; ordered by
 * timestamp ascending. Decide (and pin) the behavior for a Windows dump
 * present in only one of its two tables.
 */
export function cpuMemDumps(): Promise<CpuMemDumpSummary[]> {
  return invoke("cpumem_dumps");
}

/**
 * One dump's CPU list, ordered by value descending. The frontend fetches
 * both metrics concurrently on dump click, so the CPU|Memory toggle stays
 * a local flip.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_cpu_processes(
 *     timestamp: u64,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<ProcessUsage>, String>
 * ```
 * REQUIREMENTS: rows from BOTH platforms' cpu data; ordered by value
 * descending; `user` null for Windows rows.
 */
export function cpuMemCpuProcesses(timestamp: number): Promise<ProcessUsage[]> {
  return invoke("cpumem_cpu_processes", { timestamp });
}

/**
 * One dump's Memory list (MB), ordered by value descending.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_memory_processes(
 *     timestamp: u64,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<ProcessUsage>, String>
 * ```
 * REQUIREMENTS: same as the cpu list, over the memory data.
 */
export function cpuMemMemoryProcesses(timestamp: number): Promise<ProcessUsage[]> {
  return invoke("cpumem_mem_processes", { timestamp });
}

/**
 * Overall total CPU across every dump — feeds the overview page.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_total_cpu(
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<MetricPoint>, String>
 * ```
 * REQUIREMENTS: one point per dump, spanning both platforms; ordered by
 * timestamp ascending (charts assume sorted).
 */
export function cpuMemTotalCpu(): Promise<MetricPoint[]> {
  return invoke("cpumem_total_cpu");
}

/**
 * Overall total memory (MB) across every dump — feeds the overview page.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_total_memory(
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<MetricPoint>, String>
 * ```
 * REQUIREMENTS: same as cpumem_total_cpu, over the memory totals.
 */
export function cpuMemTotalMemory(): Promise<MetricPoint[]> {
  return invoke("cpumem_total_memory");
}

/**
 * One executable's aggregate history across all dumps — the per-dump SUM
 * of every process sharing the path (all postgres.exe workers as one
 * line). Feeds the chart when a per-path rollup row is clicked.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_path_series(
 *     path: Option<String>,
 *     name: Option<String>,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<ProcessSeries, String>
 * ```
 * REQUIREMENTS: exactly one of path/name is Some — match rows by path when
 * path is given; by name AND path IS NULL when name is given (mirrors how
 * the frontend rollup groups). One point per dump per metric (sum), each
 * array ordered by timestamp ascending; spans both platforms.
 */
export function cpuMemPathSeries(
  path: string | null,
  name: string | null,
): Promise<ProcessSeries> {
  return invoke("cpumem_path_series", { path, name });
}

/**
 * One pid's CPU and Memory history across all dumps — feeds both charts on
 * a single click. Points ordered by timestamp ascending (REQUIRED: the
 * chart's line + hover bisect assume sorted); either array may be empty.
 *
 * ```rust
 * #[tauri::command]
 * async fn cpumem_series(
 *     pid: u64,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<ProcessSeries, String>
 * ```
 * REQUIREMENT: each array ordered by timestamp ascending (charts assume
 * sorted); spans both platforms' tables.
 */
export function cpuMemSeries(pid: number): Promise<ProcessSeries> {
  return invoke("cpumem_series", { pid });
}
