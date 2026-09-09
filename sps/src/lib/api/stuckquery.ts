/**
 * Stuck-query snapshots: the "Running queries information during stuck
 * thread" tables the app logs at every valve detection.
 *
 * NO union shapes on purpose (unlike cpumemstats): MSSQL and PGSQL rows
 * share almost nothing, so each flavor gets its own commands and types.
 * A snapshot = one dump moment = one distinct `timestamp`. PGSQL has no
 * blocking table — the blocking command exists for MSSQL only.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * camelCase JSON, u64 ms timestamps/durations.
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Snapshot rollups (one row per dump moment, for the snapshot list)
// ---------------------------------------------------------------------------

export interface MssqlSnapshot {
  /** ms epoch of the dump */
  timestamp: number;
  /** rows in stuckquery_mssql at this timestamp */
  queries: number;
  /** rows with blocked_by != 0 */
  blocked: number;
}

/** One dump moment that logged a "Currently Blocking Query Details" table. */
export interface BlockingSnapshot {
  /** ms epoch of the dump */
  timestamp: number;
  /** distinct head blockers at this timestamp */
  chains: number;
  /** total blocked sessions (rows) at this timestamp */
  sessions: number;
}

export interface PgsqlSnapshot {
  /** ms epoch of the dump */
  timestamp: number;
  /** rows in stuckquery_pgsql at this timestamp */
  queries: number;
  /** rows with waiting = true */
  waiting: number;
  /** rows with state = 'idle in transaction' */
  idleInTxn: number;
}

/**
 * ```rust
 * #[tauri::command]
 * fn stuckquery_mssql_snapshots(state: ...) -> Result<Vec<MssqlSnapshot>, String>
 * #[tauri::command]
 * fn stuckquery_pgsql_snapshots(state: ...) -> Result<Vec<PgsqlSnapshot>, String>
 * ```
 * REQUIREMENTS: one row per distinct timestamp (GROUP BY), ordered by
 * timestamp ascending; empty Vec when the table has no rows (a bundle
 * normally contains only one flavor).
 */
export function stuckqueryMssqlSnapshots(): Promise<MssqlSnapshot[]> {
  return invoke("stuckquery_mssql_snapshots");
}
export function stuckqueryPgsqlSnapshots(): Promise<PgsqlSnapshot[]> {
  return invoke("stuckquery_pgsql_snapshots");
}

/**
 * Blocking snapshots are a SEPARATE command (and separate list rows in the
 * UI) — not a count folded into the mssql snapshot rollup.
 *
 * ```rust
 * #[tauri::command]
 * fn stuckquery_mssql_blocking_snapshots(state: ...) -> Result<Vec<BlockingSnapshot>, String>
 * ```
 * REQUIREMENTS: one row per distinct timestamp in stuckquery_mssql_blocking
 * (GROUP BY: chains = COUNT(DISTINCT head_blocker), sessions = COUNT(*)),
 * ordered by timestamp ascending; empty Vec when nothing ever blocked.
 */
export function stuckqueryMssqlBlockingSnapshots(): Promise<BlockingSnapshot[]> {
  return invoke("stuckquery_mssql_blocking_snapshots");
}

// ---------------------------------------------------------------------------
// Per-snapshot query rows
// ---------------------------------------------------------------------------

/**
 * One row of "Currently Running Queries" (MSSQL). Field names mirror
 * parser::stuckquery::RunningQuery verbatim (camelCased by serde) — the
 * parser is the source of truth, this file follows it. All durations ms.
 *
 * SERIALIZATION REQUIREMENTS on the Rust side:
 *  - status must reach the wire lowercase ("runnable"…, rename_all) —
 *    the badge styling keys off the exact strings;
 *  - waitType must serialize as a PLAIN STRING (WaitType::as_str), not
 *    serde's default externally-tagged enum — Unknown(String) would
 *    otherwise arrive as {"Unknown": "..."}.
 */
export interface MssqlQuery {
  sessionId: number;
  /** runnable | running | rollback | sleeping | background | suspended */
  status: string;
  txnId: number;
  /** blocking session id; 0 = not blocked */
  blockedBy: number;
  waitType: string | null;
  waitResource: string | null;
  waitTimeMs: number;
  cpuTimeMs: number;
  logicalReads: number;
  reads: number;
  writes: number;
  elapsed: number;
  /** the statement excerpt from the table */
  statement: string;
  /** full batch text when logged */
  commandText: string;
  command: string;
  login: string;
  host: string;
  db: string;
  program: string;
  hostProcess: number;
  lastRequestEnd: number;
  loginTime: number;
  openTxn: number;
}

/** One row of "Currently Running Queries" (PGSQL). */
export interface PgsqlQuery {
  pid: number;
  queryTime: number | null;
  txnTime: number | null;
  dbName: string;
  /** active | idle in transaction */
  state: string;
  waiting: boolean;
  query: string;
  /** ms epoch of the last state change */
  stateChange: number;
  applicationName: string | null;
  clientHost: string | null;
  clientPort: number | null;
}

/**
 * ```rust
 * #[tauri::command]
 * fn stuckquery_mssql_queries(timestamp: u64, state: ...) -> Result<Vec<MssqlQuery>, String>
 * #[tauri::command]
 * fn stuckquery_pgsql_queries(timestamp: u64, state: ...) -> Result<Vec<PgsqlQuery>, String>
 * ```
 * REQUIREMENTS: rows WHERE timestamp = $1 exactly (the frontend only asks
 * for timestamps it got from the snapshots command); MSSQL ordered by
 * elapsed descending, PGSQL by query_time descending (nulls last).
 */
export function stuckqueryMssqlQueries(timestamp: number): Promise<MssqlQuery[]> {
  return invoke("stuckquery_mssql_queries", { timestamp });
}
export function stuckqueryPgsqlQueries(timestamp: number): Promise<PgsqlQuery[]> {
  return invoke("stuckquery_pgsql_queries", { timestamp });
}

// ---------------------------------------------------------------------------
// Blocking chain (MSSQL only — PGSQL logs no blocking table)
// ---------------------------------------------------------------------------

/**
 * One row of "Currently Blocking Query Details". Field names mirror
 * parser::stuckquery::BlockingQuery verbatim (camelCased by serde).
 * waitType: same plain-string serialization requirement as MssqlQuery.
 */
export interface MssqlBlockingRow {
  /** session at the root of this chain */
  headBlocker: number;
  sessionId: number;
  txnId: number;
  /** direct blocker of this session */
  blockingSessionId: number;
  waitType: string | null;
  /** ms */
  waitDuration: number;
  waitResource: string | null;
  statementStartOffset: number;
  statementEndOffset: number;
  planHandle: string;
  sqlHandle: string;
  mostRecentSqlHandle: string;
  /** depth in the chain (1 = directly under the head blocker) */
  level: number;
  blockerQueryOrMostRecentQuery: string;
}

/**
 * ```rust
 * #[tauri::command]
 * fn stuckquery_mssql_blocking(timestamp: u64, state: ...) -> Result<Vec<MssqlBlockingRow>, String>
 * ```
 * REQUIREMENTS: rows WHERE timestamp = $1; ordered by head_blocker, then
 * level ascending (the frontend indents by level within each head-blocker
 * group); empty Vec when the snapshot logged no blocking table.
 */
export function stuckqueryMssqlBlocking(timestamp: number): Promise<MssqlBlockingRow[]> {
  return invoke("stuckquery_mssql_blocking", { timestamp });
}

// ---------------------------------------------------------------------------
// Long runners: queries observed across MULTIPLE snapshots
// ---------------------------------------------------------------------------

/**
 * One execution seen in several snapshots (MSSQL identity: the same
 * session_id + txn_id still running = the same execution).
 */
export interface MssqlLongRunner {
  sessionId: number;
  txnId: number;
  statement: string;
  login: string;
  /** distinct snapshots this execution appears in */
  snapshots: number;
  /** ms epoch of the first/last snapshot containing it */
  firstSeen: number;
  lastSeen: number;
  maxElapsedMs: number;
  maxCpuTimeMs: number;
  /** snapshots in which it was blocked (blocked_by != 0) */
  blockedIn: number;
}

/** One execution seen in several snapshots (PGSQL identity: pid + query). */
export interface PgsqlLongRunner {
  pid: number;
  query: string;
  dbName: string;
  snapshots: number;
  firstSeen: number;
  lastSeen: number;
  maxQueryTimeMs: number | null;
  /** snapshots in which state = 'idle in transaction' */
  idleInTxnIn: number;
}

/**
 * ```rust
 * #[tauri::command]
 * fn stuckquery_mssql_longrunning(state: ...) -> Result<Vec<MssqlLongRunner>, String>
 * #[tauri::command]
 * fn stuckquery_pgsql_longrunning(state: ...) -> Result<Vec<PgsqlLongRunner>, String>
 * ```
 * REQUIREMENTS: GROUP BY the identity above; only groups present in MORE
 * THAN ONE distinct timestamp; ordered by snapshots desc, then
 * maxElapsedMs / maxQueryTimeMs desc.
 */
export function stuckqueryMssqlLongrunning(): Promise<MssqlLongRunner[]> {
  return invoke("stuckquery_mssql_longrunning");
}
export function stuckqueryPgsqlLongrunning(): Promise<PgsqlLongRunner[]> {
  return invoke("stuckquery_pgsql_longrunning");
}
