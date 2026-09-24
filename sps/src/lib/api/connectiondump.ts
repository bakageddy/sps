/**
 * Connection dumps: the ConnectionDump performance log — threshold alarms
 * (High CPU / High Memory / No ManagedConnections / URL invocation), pool
 * occupancy stats, and per-dump "In use TraceInfo" holder traces.
 *
 * The interesting analysis is HOLDS, not snapshots: the trace `id` is the
 * JAVA THREAD ID of the holder (threads are pooled and reused — the same id
 * acquires connections hundreds of times), so a hold's identity is
 * (id, startTime): the same thread id with the same acquisition timestamp
 * across dumps is one continuing hold, and its growing duration is the
 * incident.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * camelCase JSON, u64 ms timestamps/durations.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Signals: the alarm timeline
// ---------------------------------------------------------------------------

/**
 * One threshold alarm — both real dumps ("Going to dump...") and coalesced
 * ones ("Skipping to dump..."). Field names mirror
 * parser::connectiondump::ConnectionDumpEntry::Signal (camelCased).
 *
 * cause is the connectiondump_cause ENUM label, read straight from the
 * column (see src-tauri/schema.sql) — exactly one of:
 *   "High CPU" | "No ManagedConnections" | "High Memory Consumption" |
 *   "URL invocation"
 * These strings ARE the wire contract; causeColor() in
 * src/lib/connectiondump.ts must match them verbatim.
 *
 * suppressed = true is the "Skipping to dump" preamble (threshold exceeded
 * but no body captured); false is a real dump.
 */
export interface ConnDumpSignal {
	/** ms epoch of the log line */
	timestamp: number;
	/** logging thread id (demux key parser-side; opaque to the UI) */
	tid: number;
	cause: string;
	/** true = "Skipping to dump" (alarm without a captured body) */
	suppressed: boolean;
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_signals(from: Option<u64>, to: Option<u64>, state: ...)
 *     -> Result<Vec<ConnDumpSignal>, String>
 * ```
 * REQUIREMENTS: rows with from <= timestamp <= to (each bound applied only
 * when Some); ordered by timestamp ascending; empty Vec when nothing parsed.
 */
export function connectiondumpSignals(
	from?: number,
	to?: number,
): Promise<ConnDumpSignal[]> {
	return invoke("connectiondump_signals", { from, to });
}

// ---------------------------------------------------------------------------
// Pool stats: occupancy series
// ---------------------------------------------------------------------------

/**
 * One "Connection Pool Stats" line. Field names mirror
 * parser::connectiondump::ConnectionDumpEntry::ConnectionPoolStats
 * (camelCased; `total` = max_connections).
 */
export interface ConnDumpPoolStats {
	/** ms epoch of the log line */
	timestamp: number;
	tid: number;
	used: number;
	free: number;
	total: number;
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_pool_stats(from: Option<u64>, to: Option<u64>, state: ...)
 *     -> Result<Vec<ConnDumpPoolStats>, String>
 * ```
 * REQUIREMENTS: same range semantics as connectiondump_signals; ordered by
 * timestamp ascending (the chart step-plots in order and bisects).
 */
export function connectiondumpPoolStats(
	from?: number,
	to?: number,
): Promise<ConnDumpPoolStats[]> {
	return invoke("connectiondump_pool_stats", { from, to });
}

// ---------------------------------------------------------------------------
// Snapshot rollups + per-snapshot traces (browser pair)
// ---------------------------------------------------------------------------

/** One trace dump moment, rolled up for the snapshot list. */
export interface ConnDumpSnapshot {
	/** ms epoch of the "In use TraceInfo" line */
	timestamp: number;
	/** traces in this dump */
	traceCount: number;
	/** longest held-for among them, ms */
	maxDuration: number;
	/**
	 * pool occupancy at this dump — from the ConnectionPoolStats entry of
	 * the SAME dump (same tid, the adjacent line); 0/0 when it went missing.
	 */
	used: number;
	total: number;
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_snapshots(state: ...) -> Result<Vec<ConnDumpSnapshot>, String>
 * ```
 * REQUIREMENTS: one row per distinct trace-dump timestamp (GROUP BY),
 * ordered by timestamp ascending; empty Vec when no dumps parsed.
 */
export function connectiondumpSnapshots(): Promise<ConnDumpSnapshot[]> {
	return invoke("connectiondump_snapshots");
}

/**
 * One "In use TraceInfo" entry. Field names mirror
 * parser::connectiondump::Trace verbatim (rename_all = camelCase already on
 * the struct — it is wire-ready as-is). `id` is the holder's JAVA THREAD ID,
 * not a connection id.
 */
export interface ConnDumpTrace {
	/** held-for at dump time, ms */
	duration: number;
	/** ms epoch of the connection acquisition */
	startTime: number;
	stackTrace: string[];
	id: number;
	invokedBy: string | null;
	threadName: string;
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_traces(timestamp: u64, state: ...)
 *     -> Result<Vec<ConnDumpTrace>, String>
 * ```
 * REQUIREMENTS: traces WHERE dump timestamp = $1 exactly (the frontend only
 * asks for timestamps from connectiondump_snapshots); ordered by duration
 * descending; stack frames in original top-to-bottom order.
 */
export function connectiondumpTraces(
	timestamp: number,
): Promise<ConnDumpTrace[]> {
	return invoke("connectiondump_traces", { timestamp });
}

// ---------------------------------------------------------------------------
// Holders: continuing holds observed across dumps
// ---------------------------------------------------------------------------

/**
 * One hold episode: traces grouped by (id, startTime) — same thread id AND
 * same acquisition timestamp = the same hold continuing across dumps. A
 * recycled thread id can never collide (it cannot reproduce the same
 * acquisition millisecond).
 *
 * The "owner" frame is derived from `stackTrace` via appFrame() in
 * src/lib/connectiondump.ts — the plumbing heuristic lives ONLY there.
 */
export interface ConnDumpHolder {
	/** holder's Java thread id */
	id: number;
	/** ms epoch of the acquisition (episode identity with id) */
	startTime: number;
	/** longest observed held-for, ms (MAX(duration) across appearances) */
	duration: number;
	/** distinct dumps this hold appears in; backend guarantees > 1 */
	dumpCount: number;
	/** stack frames of the hold, original top-to-bottom (idx) order */
	stackTrace: string[];
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_holders(from: Option<u64>, to: Option<u64>, state: ...)
 *     -> Result<Vec<ConnDumpHolder>, String>
 * ```
 * REQUIREMENTS: GROUP BY (id, start_time) over traces whose DUMP timestamp
 * falls in range (same range semantics as signals); ONLY episodes observed
 * in more than one dump (dumpCount > 1 guaranteed — single-dump holds are
 * excluded backend-side). `stackTrace` = frames of any ONE appearance in
 * idx order (a hold's stack is identical across its appearances — never the
 * concatenation of all of them). No ordering requirement — the frontend
 * sorts by duration.
 */
export function connectiondumpHolders(
	from?: number,
	to?: number,
): Promise<ConnDumpHolder[]> {
	return invoke("connectiondump_holders", { from, to });
}
