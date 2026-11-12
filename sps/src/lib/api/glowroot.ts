/**
 * Glowroot trace ingestion — pull Web/Schedule traces from a Glowroot
 * collector (an embedded agent or Glowroot Central, 0.14.0+) into the open
 * database, over plain HTTP/1.1 JSON. Endpoint and field reference:
 * docs/glowroot-http-api.md.
 *
 * Shape of the feature: the commands below are short request/response calls
 * (connect, log in, pick an agent, count). The long one, `glowroot_ingest`,
 * only STARTS a run and resolves at once; everything after that arrives as
 * `glowroot:*` events (./glowroot-events.ts) from a single writer task.
 *
 * Conventions as everywhere: camelCase JSON, `Err` serialized to a readable
 * String (a rejected promise), times are ms epoch UTC (Glowroot already
 * reports UTC millis, no zone handling).
 *
 * The collector session (cookie jar, base URL, cached layout) lives in the
 * backend. At most one session exists at a time: `glowroot_connect` replaces
 * it. The password is passed once to `glowroot_login` and never stored by
 * either side.
 */

import { invoke } from "@tauri-apps/api/core";

/** What `/backend/layout` told us about the collector. */
export interface Collector {
	/** normalised base URL: scheme + host + port + context path, no trailing slash */
	url: string;
	/** layout.glowrootVersion, e.g. "0.14.8, built 2025-…" */
	version: string;
	/** true = Glowroot Central (agents are addressed by id), false = embedded */
	central: boolean;
	loginEnabled: boolean;
	loggedIn: boolean;
	/** layout.redirectToLogin: anonymous has no permissions, so nothing works
	 *  until `glowroot_login` succeeds */
	needsLogin: boolean;
	/** the collector's own default zone, informational only */
	defaultTimeZoneId: string;
}

/** One selectable agent or rollup (Central only). */
export interface Agent {
	/** agent id, or rollup id (ends with "::") */
	id: string;
	display: string;
	/** true for a rollup: listing spans it, each trace keeps its own agent id */
	rollup: boolean;
	/** a top-level rollup that `glowroot_agents(topLevelId)` can expand */
	hasChildren: boolean;
}

/** How a text filter matches. Always sent over IPC as one of these STRINGS,
 *  never a number or index; they are also the wire values of Glowroot's
 *  `*-comparator` query parameters, so the backend passes them through.
 *  Rust: an enum with `#[serde(rename_all = "kebab-case")]` (variants Begins,
 *  Equals, Ends, Contains, NotContains) round-trips them as is. */
export const COMPARATORS = ["begins", "equals", "ends", "contains", "not-contains"] as const;
export type Comparator = (typeof COMPARATORS)[number];

/** One text filter: the text and how to compare it. The UI never sends an
 *  empty `text`; an unused filter is null instead. */
export interface TextFilter {
	text: string;
	comparator: Comparator;
}

export interface AgentLayout {
	/** the types the agent has (stored ∪ live ∪ configured default), in the
	 *  collector's order — e.g. "Background", "Schedule", "Web". Never
	 *  hard-coded by the UI. */
	transactionTypes: string[];
	/** trace attribute names per transaction type, from the layout's
	 *  traceAttributeNames; feeds the attribute filter's suggestions. A type
	 *  without attributes may be missing. */
	traceAttributeNames: Record<string, string[]>;
	/** what the logged-in user may read on this agent rollup; the UI disables
	 *  the matching toggles and warns when `traces` is false */
	permissions: {
		traces: boolean;
		errorTraces: boolean;
		trace: boolean;
		queries: boolean;
		threadProfile: boolean;
	};
}

export interface IngestOptions {
	/** "" on embedded; an agent id or rollup id on Central */
	agentRollupId: string;
	/** non-empty subset of the agent's transactionTypes */
	transactionTypes: string[];
	/** exact transaction-name filter, null = all names */
	transactionName: string | null;
	/** the filters below are ANDed with each other and with the duration
	 *  band; null = not used */
	headline: TextFilter | null;
	user: TextFilter | null;
	/** applies to the error family only (Glowroot ignores it on slow
	 *  listings in practice, unverified): the slow family is listed without
	 *  it */
	errorMessage: TextFilter | null;
	/** a trace attribute (name from AgentLayout.traceAttributeNames or typed)
	 *  with a text filter on its value; both parts are always present */
	attribute: { name: string; value: TextFilter } | null;
	/** which point families to list; at least one is true */
	slow: boolean;
	error: boolean;
	/** inclusive bounds, ms epoch; from < to; `to` is a concrete instant,
	 *  never 0 ("now" is resolved by the UI so a run is reproducible) */
	from: number;
	to: number;
	/** duration band in ms; high = null means unbounded */
	durationMillisLow: number;
	durationMillisHigh: number | null;
	/** cap on traces fetched; the NEWEST by capture time win when it bites */
	maxTraces: number;
	/** concurrent requests = HTTP/1.1 connections, 1..=32 */
	workers: number;
	/** which parts to fetch besides the header, which is always stored */
	entries: boolean;
	queries: boolean;
	profiles: boolean;
	/** drop traces already stored for that agent before the cap is applied */
	skipExisting: boolean;
}

export interface TraceCount {
	transactionType: string;
	slow: number;
	error: number;
}

export type RunOutcome = "running" | "completed" | "cancelled" | "failed";

/** One row of run history, persisted in the database. */
export interface IngestRun {
	id: number;
	collectorUrl: string;
	agentRollupId: string;
	/** ms epoch */
	startedAt: number;
	/** null while running */
	finishedAt: number | null;
	outcome: RunOutcome;
	total: number;
	stored: number;
	failed: number;
	options: IngestOptions;
}

/**
 * Read the collector's layout and open a fresh session (new cookie jar),
 * replacing any existing one.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_connect(url: String, state: ...) -> Result<Collector, String>
 * ```
 * REQUIREMENTS: accept a bare "host:port" (assume http) and strip a trailing
 * slash; a context path ("/glowroot") is kept. Err text says what failed:
 * unreachable, TLS, not JSON ("not a Glowroot collector"), non-2xx status.
 * Short connect timeout (10 s).
 */
export function glowrootConnect(url: string): Promise<Collector> {
	return invoke("glowroot_connect", { url });
}

/**
 * Log in on the current session and return the re-read layout.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_login(username: String, password: String, state: ...)
 *     -> Result<Collector, String>
 * ```
 * REQUIREMENTS: Glowroot answers a failed login with HTTP 200 and an
 * incorrectLogin flag, not a 401 — turn that into
 * Err("incorrect username or password"). The session cookie is whatever
 * Set-Cookie sent (default name GLOWROOT_SESSION_ID); the cookie jar keeps
 * it. Err if there is no session (connect first).
 */
export function glowrootLogin(username: string, password: string): Promise<Collector> {
	return invoke("glowroot_login", { username, password });
}

/**
 * Drop the session. Best-effort sign-out; never fails because the collector
 * is gone.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_disconnect(state: ...) -> Result<(), String>
 * ```
 */
export function glowrootDisconnect(): Promise<void> {
	return invoke("glowroot_disconnect");
}

/**
 * List agents that reported in `[from, to]` (Central only).
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_agents(top_level_id: Option<String>, from: u64, to: u64,
 *     state: ...) -> Result<Vec<Agent>, String>
 * ```
 * REQUIREMENTS: `topLevelId` null = the top-level agents/rollups, an id =
 * that rollup's children. Err "not a Central collector" on embedded. Order
 * by display name, case-insensitive.
 */
export function glowrootAgents(
	topLevelId: string | null,
	from: number,
	to: number,
): Promise<Agent[]> {
	return invoke("glowroot_agents", { topLevelId, from, to });
}

/**
 * Transaction types and permissions for one agent rollup.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_agent_layout(agent_rollup_id: String, state: ...)
 *     -> Result<AgentLayout, String>
 * ```
 * REQUIREMENTS: embedded reads it from the layout already cached by connect
 * (agentRollupId is ""); Central asks the collector for that rollup's
 * layout.
 */
export function glowrootAgentLayout(agentRollupId: string): Promise<AgentLayout> {
	return invoke("glowroot_agent_layout", { agentRollupId });
}

/**
 * How many traces the given options would list, per type, before anything
 * heavy starts. Ignores workers/parts/maxTraces/skipExisting.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_count(options: IngestOptions, state: ...)
 *     -> Result<Vec<TraceCount>, String>
 * ```
 * REQUIREMENTS: one count request per selected type and family, in the
 * order of `options.transactionTypes`; a family the options turned off
 * reports 0. Uses the same filters as listing (duration band, name, headline, user,
 * error message, attribute, window)
 * so the number matches what the run will find.
 */
export function glowrootCount(options: IngestOptions): Promise<TraceCount[]> {
	return invoke("glowroot_count", { options });
}

/**
 * Start a run and return its id. Resolves as soon as the run is registered
 * and `glowroot:started` has been emitted.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_ingest(options: IngestOptions, app: tauri::AppHandle,
 *     state: ...) -> Result<u64, String>
 * ```
 * REQUIREMENTS: validate first (Err before anything is registered): a
 * session exists, from < to, workers in 1..=32, at least one type and one
 * family, a database is open. Only ONE run at a time: Err "a run is already
 * in progress" otherwise. Must not block the command for the duration of
 * the run. Everything after is `glowroot:*` events; `glowroot:finished`
 * fires exactly once, whatever the outcome. The run row is inserted into
 * the history at start (outcome "running") and finalised at the end.
 */
export function glowrootIngest(options: IngestOptions): Promise<number> {
	return invoke("glowroot_ingest", { options });
}

/**
 * Ask a running run to stop.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_cancel(run_id: u64, state: ...) -> Result<(), String>
 * ```
 * REQUIREMENTS: cooperative and idempotent. Resolves immediately; the run
 * still emits `glowroot:finished` with outcome "cancelled" after it has
 * flushed what was already fetched. Unknown or finished id = Ok(()).
 */
export function glowrootCancel(runId: number): Promise<void> {
	return invoke("glowroot_cancel", { runId });
}

/**
 * Past runs, newest first.
 *
 * ```rust
 * #[tauri::command]
 * async fn glowroot_runs(state: ...) -> Result<Vec<IngestRun>, String>
 * ```
 * REQUIREMENTS: from the database, so history survives restarts together
 * with the traces. A run left "running" by a crash is reported as "failed".
 */
export function glowrootRuns(): Promise<IngestRun[]> {
	return invoke("glowroot_runs");
}
