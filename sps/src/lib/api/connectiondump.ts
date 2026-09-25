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

// ---------------------------------------------------------------------------
// Incidents: connectiondump as the correlation hub
// ---------------------------------------------------------------------------
//
// Every performance dump fires on a trigger (High CPU / NMC / High Memory /
// manual URL invocation) and connectiondump RECORDS that trigger as a
// signal — so the signals timeline is the incident index, and every other
// log's dump taken within the tolerance of a signal belongs to that
// incident. Stuck threads are deliberately NOT part of this hub (they keep
// their own matcher at /stuckthreads/queries).
//
// Correlation is done backend-side (Dinesh): the frontend hands over a
// signal timestamp and gets back already-matched data. Two commands:
//  - connectiondump_incident: resolves the anchor — which dump of each
//    subsystem is "this incident" — so the frontend can reuse the existing
//    per-timestamp commands (cpumem_cpu_processes, stuckquery_*_queries…)
//    for the panels that need no tid join, and deep-link into those pages.
//  - connectiondump_incident_threads: the one panel that DOES need a tid
//    join — the threaddump census decorated with cpumonitoring / hold data.

/** ms: dumps of one trigger land within this of the signal (measured 2–4s). */
export const INCIDENT_TOLERANCE_MS = 4000;

/** The nearest dump of each subsystem within INCIDENT_TOLERANCE_MS of the
 *  signal; null = that log has no dump in the window (or isn't in the bundle). */
export interface IncidentAnchors {
	threaddump: number | null;
	cpumonitoring: number | null;
	cpumemstats: number | null;
	stuckqueryMssql: number | null;
	stuckqueryPgsql: number | null;
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_incident(timestamp: u64, state: ...)
 *     -> Result<IncidentAnchors, String>
 * ```
 * REQUIREMENTS: $1 is a signal timestamp from connectiondump_signals. For
 * each subsystem, the dump timestamp minimizing |dump_ts - $1| subject to
 * |dump_ts - $1| <= INCIDENT_TOLERANCE_MS, else null. cpumonitoring dumps =
 * distinct cpumonitoring.timestamp; cpumemstats = distinct dump timestamps
 * across its platform tables; stuckquery = distinct snapshot timestamps per
 * flavor. Never errors on a missing log — that's a null, not an Err.
 */
export function connectiondumpIncident(
	timestamp: number,
): Promise<IncidentAnchors> {
	return invoke("connectiondump_incident", { timestamp });
}

import type { ThreadDumpThread } from "./threaddump";

/**
 * One row of the incident census: the threaddump thread (all of
 * ThreadDumpThread — the spine) decorated by tid with what the other logs
 * knew about it at this incident. null = that log has no row for this tid
 * at its matched dump (cpumonitoring only logs hot threads; connectiondump
 * only connection holders) — null is normal, not an error.
 */
export interface IncidentThread extends ThreadDumpThread {
	/** cpumonitoring cpu % at the incident's cpumonitoring dump */
	cpu: number | null;
	/** ms the thread had held a pooled connection at the incident's trace dump */
	heldFor: number | null;
}

/**
 * ```rust
 * #[tauri::command]
 * fn connectiondump_incident_threads(timestamp: u64, state: ...)
 *     -> Result<Vec<IncidentThread>, String>
 * ```
 * REQUIREMENTS: $1 is a signal timestamp. Resolve the threaddump anchor as
 * in connectiondump_incident; empty Vec if there is none. Rows = every
 * thread of that threaddump (LEFT side), LEFT JOINed by EXACT tid with:
 *   - cpumonitoring rows at the incident's cpumonitoring anchor → cpu;
 *   - connectiondump_stacktraces rows (id = tid) at the nearest trace dump
 *     within tolerance → heldFor = that row's duration.
 * A thread absent from a side gets null there. Same ordering as
 * threaddump_threads (BLOCKED first…), with cpu descending as the tiebreak
 * within a state so the hot ones lead.
 */
export function connectiondumpIncidentThreads(
	timestamp: number,
): Promise<IncidentThread[]> {
	return invoke("connectiondump_incident_threads", { timestamp });
}
