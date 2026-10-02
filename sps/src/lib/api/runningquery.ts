/**
 * Running-query snapshots: the periodic "RunningQueries" dumps
 * (com.zoho.mickey.db.RunningQueries) — every running statement at a
 * moment, on a timer, NOT on a trigger. Stuck queries (api/stuckquery.ts)
 * are the same table shapes logged only during a stuck thread; this log is
 * the always-on version, which is why it CAN join the connection-dump
 * incident hub (connectiondump_runningqueries) while stuck queries can't.
 *
 * Row types are shared with stuck queries on purpose: the parser persists
 * the same parser::query::{PGSQLQuery, MSSQLQuery, BlockingQuery} rows, so
 * the same tables render them. One shape is new here: sp_who2.
 *
 * A snapshot = one dump moment = one distinct `timestamp`. Two logger
 * threads dump ~100 ms apart; those are TWO snapshots — exact timestamps,
 * no grouping (the list just shows both rows).
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * camelCase JSON, u64 ms timestamps/durations.
 */

import { invoke } from "@tauri-apps/api/core";
import type {
	MssqlSnapshot,
	PgsqlSnapshot,
	BlockingSnapshot,
	MssqlQuery,
	PgsqlQuery,
	MssqlBlockingRow,
	MssqlLongRunner,
	PgsqlLongRunner,
	MssqlLongTxn,
} from "./stuckquery";

export type {
	MssqlSnapshot,
	PgsqlSnapshot,
	BlockingSnapshot,
	MssqlQuery,
	PgsqlQuery,
	MssqlBlockingRow,
	MssqlLongRunner,
	PgsqlLongRunner,
	MssqlLongTxn,
};

// ---------------------------------------------------------------------------
// sp_who2 (MSSQL only) — the one shape stuck queries never logged
// ---------------------------------------------------------------------------

/** One dump moment that logged an sp_who2 table. */
export interface SpWho2Snapshot {
	/** ms epoch of the dump */
	timestamp: number;
	/** rows in runningquery_mssql_spwho2 at this timestamp */
	sessions: number;
	/** rows with status != 'sleeping' */
	active: number;
	/** rows with blocked_by IS NOT NULL AND blocked_by != 0 */
	blocked: number;
}

/**
 * One sp_who2 row. Field names mirror parser::query::SPWho2Query verbatim
 * (camelCased by serde) — the parser is the source of truth.
 *
 * SERIALIZATION REQUIREMENT: status must reach the wire as the exact enum
 * label — "sleeping" | "BACKGROUND" | "RUNNABLE" | "SUSPENDED" | "DORMANT"
 * (mixed case is how sp_who2 prints them; the badge styling lowercases).
 * Native ENUM column → cast `status::VARCHAR` in the SELECT.
 */
export interface SpWho2Row {
	spid: number;
	status: string;
	login: string;
	hostname: string | null;
	/** blocking spid; null/0 = not blocked */
	blockedBy: number | null;
	dbname: string;
	command: string;
	/** ms of CPU */
	cputime: number;
	/** IO operations */
	diskio: number;
	/** ms epoch of the last batch */
	lastbatch: number;
	programName: string | null;
	requestId: number;
}

// ---------------------------------------------------------------------------
// Snapshot rollups (one row per dump moment, for the snapshot list)
// ---------------------------------------------------------------------------

/**
 * ```rust
 * #[tauri::command]
 * async fn runningquery_mssql_snapshots(state: ...) -> Result<Vec<MSSQLSnapshot>, String>
 * #[tauri::command]
 * async fn runningquery_pgsql_snapshots(state: ...) -> Result<Vec<PGSQLSnapshot>, String>
 * #[tauri::command]
 * async fn runningquery_mssql_blocking_snapshots(state: ...) -> Result<Vec<BlockingSnapshot>, String>
 * #[tauri::command]
 * async fn runningquery_spwho2_snapshots(state: ...) -> Result<Vec<SpWho2Snapshot>, String>
 * ```
 * REQUIREMENTS: identical to the stuckquery_* snapshot commands, over the
 * runningquery_* tables — one row per distinct timestamp (GROUP BY),
 * ordered by timestamp ascending; empty Vec when the table has no rows
 * (a bundle normally contains only one flavor). sp_who2 counts per the
 * SpWho2Snapshot field docs.
 */
export function runningqueryMssqlSnapshots(): Promise<MssqlSnapshot[]> {
	return invoke("runningquery_mssql_snapshots");
}
export function runningqueryPgsqlSnapshots(): Promise<PgsqlSnapshot[]> {
	return invoke("runningquery_pgsql_snapshots");
}
export function runningqueryMssqlBlockingSnapshots(): Promise<
	BlockingSnapshot[]
> {
	return invoke("runningquery_mssql_blocking_snapshots");
}
export function runningquerySpwho2Snapshots(): Promise<SpWho2Snapshot[]> {
	return invoke("runningquery_spwho2_snapshots");
}

// ---------------------------------------------------------------------------
// Per-snapshot rows
// ---------------------------------------------------------------------------

/**
 * ```rust
 * #[tauri::command]
 * async fn runningquery_mssql_queries(timestamp: u64, state: ...) -> Result<Vec<MSSQLQuery<'static>>, String>
 * #[tauri::command]
 * async fn runningquery_pgsql_queries(timestamp: u64, state: ...) -> Result<Vec<PGSQLQuery<'static>>, String>
 * #[tauri::command]
 * async fn runningquery_mssql_blocking(timestamp: u64, state: ...) -> Result<Vec<BlockingQuery<'static>>, String>
 * #[tauri::command]
 * async fn runningquery_spwho2(timestamp: u64, state: ...) -> Result<Vec<SPWho2Query<'static>>, String>
 * ```
 * REQUIREMENTS: rows WHERE timestamp = $1 exactly (the frontend only asks
 * for timestamps it got from a snapshots command or the incident
 * resolver); MSSQL ordered by elapsed desc, PGSQL by query_time desc
 * (nulls last), blocking by head_blocker then level asc, sp_who2 by
 * cputime desc. Same serialization requirements as the stuckquery
 * equivalents (status lowercase, waitType plain string).
 */
export function runningqueryMssqlQueries(
	timestamp: number,
): Promise<MssqlQuery[]> {
	return invoke("runningquery_mssql_queries", { timestamp });
}
export function runningqueryPgsqlQueries(
	timestamp: number,
): Promise<PgsqlQuery[]> {
	return invoke("runningquery_pgsql_queries", { timestamp });
}
export function runningqueryMssqlBlocking(
	timestamp: number,
): Promise<MssqlBlockingRow[]> {
	return invoke("runningquery_mssql_blocking", { timestamp });
}
export function runningquerySpwho2(timestamp: number): Promise<SpWho2Row[]> {
	return invoke("runningquery_spwho2", { timestamp });
}

// ---------------------------------------------------------------------------
// Long runners / long transactions — across MULTIPLE snapshots
// ---------------------------------------------------------------------------

/**
 * ```rust
 * #[tauri::command]
 * async fn runningquery_mssql_longrunning(state: ...) -> Result<Vec<MSSQLLongRunningQuery>, String>
 * #[tauri::command]
 * async fn runningquery_pgsql_longrunning(state: ...) -> Result<Vec<PGSQLLongRunningQuery>, String>
 * #[tauri::command]
 * async fn runningquery_mssql_longtxns(state: ...) -> Result<Vec<MSSQLLongRunningTxn>, String>
 * ```
 * REQUIREMENTS: identical to stuckquery_{mssql,pgsql}_longrunning and
 * stuckquery_mssql_longtxns, over the runningquery_* tables. Because these
 * dumps are periodic, "present in > 1 distinct timestamp" here means
 * "still running at the next tick" — the ranking (snapshots desc, then
 * max elapsed / span desc) is the point of this analyzer.
 */
export function runningqueryMssqlLongrunning(): Promise<MssqlLongRunner[]> {
	return invoke("runningquery_mssql_longrunning");
}
export function runningqueryPgsqlLongrunning(): Promise<PgsqlLongRunner[]> {
	return invoke("runningquery_pgsql_longrunning");
}
export function runningqueryMssqlLongtxns(): Promise<MssqlLongTxn[]> {
	return invoke("runningquery_mssql_longtxns");
}
