/**
 * Stuck-thread queries (Tomcat StuckThreadDetectionValve warnings).
 *
 * Mirrors the implemented handlers (src-tauri/src/handlers/stuckthread.rs):
 * `stuckthread_listview` returns episodes aggregated IN RUST (warning and
 * completion events paired by tid) within an optional time frame;
 * `stuckthread_trace` returns the frames captured with a warning event.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * camelCase JSON, u64 ms timestamps.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirror of handlers::types::AggregatedStuckthread)
// ---------------------------------------------------------------------------

/** One aggregated stuck episode. */
export interface StuckThread {
  tid: number;
  /** warning log-line ts; null = only the completion notice survived */
  begin: number | null;
  /** completion ts (>= begin + duration when paired); null = never completed */
  end: number | null;
  /** thread name as logged (empty when only a completion notice exists) */
  name: string;
  /** reported stuck duration, ms (from the closing event when paired) */
  duration: number;
  /** request URL from the warning; null when the warning was lost */
  request: string | null;
  /** valve's "active for" ms at the warning, when logged */
  activeStart: number | null;
  /** valve's "active for" ms at completion, when logged */
  activeEnd: number | null;
}

/** One stack frame of a warning's captured trace. */
export interface StuckFrame {
  method: string;
  source: string;
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/**
 * Aggregated episodes with timestamps inside [from, to] (ms epoch,
 * inclusive); omitted bounds mean the full log. Ordered by event time.
 *
 * ```rust
 * #[tauri::command]
 * fn stuckthread_listview(
 *     from: Option<u64>,
 *     to: Option<u64>,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<AggregatedStuckthread>, String>
 * ```
 */
export function stuckthreadListview(
  from?: number,
  to?: number,
): Promise<StuckThread[]> {
  return invoke("stuckthread_listview", { from: from ?? null, to: to ?? null });
}

/**
 * Stack trace captured with a warning event, in original frame order.
 * (tid, timestamp) = (thread.tid, thread.begin). An empty array means the
 * warning carried no trace — completion-only episodes never have one.
 *
 * ```rust
 * #[tauri::command]
 * fn stuckthread_trace(
 *     tid: u64,
 *     timestamp: u64,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<Frame>, String>
 * ```
 */
export function stuckthreadTrace(
  tid: number,
  timestamp: number,
): Promise<StuckFrame[]> {
  return invoke("stuckthread_trace", { tid, timestamp });
}
