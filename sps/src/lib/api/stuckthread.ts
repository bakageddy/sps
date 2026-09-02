/**
 * Stuck-thread queries (Tomcat StuckThreadDetectionValve warnings).
 *
 * Two granularities by design (DevTools anatomy): a MINIMAL feed for the
 * overview waterfall strip, and FULL span rows for the table + details.
 * Pairing begin/end events into episodes happens in Rust for both.
 * Traces load lazily per span.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * camelCase JSON, u64 ms timestamps.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirror these as Rust structs)
// ---------------------------------------------------------------------------

/** Minimal episode geometry for the overview strip. */
export interface StuckBar {
  tid: number;
  /** ms epoch the thread became active (= span start) */
  timestamp: number;
  /** best-known active duration, ms */
  durationMs: number;
}

/** One full episode row for the table + details panel. */
export interface StuckSpan {
  /** the thread name — unique episode key (embeds the request id) */
  key: string;
  tid: number;
  /** request URL; null when only the end event survived */
  request: string | null;
  /** ms epoch the thread actually became active */
  start: number;
  /** ms epoch of the completion notice; null = never seen completing */
  end: number | null;
  /** best-known active duration, ms */
  durationMs: number;
  /** true when a begin event (and therefore possibly a trace) exists */
  hasBegin: boolean;
  /** timestamp of the latest begin event — trace fetch key with tid */
  beginTimestamp: number | null;
}

/** One stack frame of a span's captured trace. */
export interface StuckFrame {
  method: string;
  source: string;
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/**
 * Minimal bars for the overview strip.
 *
 * ```rust
 * #[tauri::command]
 * async fn stuckthread_bars(
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<StuckBar>, String>
 * ```
 * REQUIREMENTS: one bar per episode; `timestamp` MUST equal the
 * corresponding span's `start` (same derivation: event ts - durationMs) —
 * the frontend joins bar clicks to table rows on (tid, timestamp);
 * ordered by timestamp ascending.
 */
export function stuckthreadBars(): Promise<StuckBar[]> {
  return invoke("stuckthread_bars");
}

/**
 * Full episode rows for the table and details panel.
 *
 * ```rust
 * #[tauri::command]
 * async fn stuckthread_spans(
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Vec<StuckSpan>, String>
 * ```
 * REQUIREMENTS (pairing semantics the UI assumes):
 *  - pair by thread NAME, not tid (tids get reused, names embed request id);
 *  - start = event timestamp - durationMs (the warning fires AFTER the
 *    threshold; the log-line time is late by construction);
 *  - begin without end  → end: null (renders as unresolved/red);
 *  - end without begin  → reconstruct start the same way; request null,
 *    hasBegin false;
 *  - repeated begins for one name → ONE span: earliest start, max
 *    durationMs, beginTimestamp of the LATEST begin (trace key);
 *  - ordered by start ascending.
 */
export function stuckthreadSpans(): Promise<StuckSpan[]> {
  return invoke("stuckthread_spans");
}

/**
 * Stack trace captured with a span's begin event, in original frame order.
 * null = the begin event carried no trace.
 *
 * ```rust
 * #[tauri::command]
 * async fn stuckthread_trace(
 *     tid: u64,
 *     timestamp: u64,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<Option<Vec<StuckFrame>>, String>
 * ```
 * REQUIREMENTS: (tid, timestamp) = (span.tid, span.beginTimestamp); a
 * returned array has at least one frame.
 */
export function stuckthreadTrace(
  tid: number,
  timestamp: number,
): Promise<StuckFrame[] | null> {
  return invoke("stuckthread_trace", { tid, timestamp });
}
