/**
 * Thread dumps: the jstack-style full census — EVERY thread's state, lock
 * relationship and stack at one moment. The other logs sample slices
 * (cpumonitoring keeps the hot threads, stuckthread the flagged ones,
 * connectiondump the connection holders); this one has all of them, which
 * is what makes it the spine of the incident census in api/connectiondump.
 *
 * Standalone analyzer at /threaddump (CPUMonitoring anatomy) with deadlock
 * detection derived FRONTEND-side from (tid, lockOwnerTid) — there is no
 * deadlock command on purpose.
 *
 * tid is the same Java thread id cpumonitoring / stuckthread / connectiondump
 * traces use, so it joins across logs by exact equality.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise),
 * camelCase JSON, u64 ms timestamps.
 */

import { invoke } from "@tauri-apps/api/core";

/** parser::threaddump::State::as_str() values, verbatim. */
export type ThreadState =
	| "NEW"
	| "RUNNABLE"
	| "TERMINATED"
	| "TIMED_WAITING"
	| "WAITING"
	| "BLOCKED";

// ---------------------------------------------------------------------------
// Dump rollups (one row per dump, for the dump list)
// ---------------------------------------------------------------------------

export interface ThreadDumpSummary {
	/** ms epoch of the dump; identifies the dump everywhere */
	timestamp: number;
	/** thread entries in this dump (all states) */
	threads: number;
	runnable: number;
	blocked: number;
	waiting: number;
	timedWaiting: number;
}

/**
 * ```rust
 * #[tauri::command]
 * fn threaddump_dumps(state: ...) -> Result<Vec<ThreadDumpSummary>, String>
 * ```
 * REQUIREMENTS: one row per distinct timestamp (GROUP BY; the per-state
 * counts are conditional COUNTs over the same group), ordered by timestamp
 * ascending; empty Vec when nothing parsed.
 */
export function threaddumpDumps(): Promise<ThreadDumpSummary[]> {
	return invoke("threaddump_dumps");
}

// ---------------------------------------------------------------------------
// Census: every thread of one dump
// ---------------------------------------------------------------------------

/**
 * One thread of one dump. `tid`, `name`, `state` mirror
 * parser::threaddump::Thread; the lock fields are the State payload
 * FLATTENED onto the row (struct variants + `#[serde(tag = "state")]` on
 * State + `#[serde(flatten)]` on Thread.state produces exactly this shape,
 * or a DTO does — the wire is the same either way).
 *
 * Nullability follows the variants:
 *   BLOCKED        → waitingOn, lock, lockOwnerTid, lockOwnerName all set
 *   WAITING        → waitingOn set; lock / owner present only when parked
 *                    on an owned lock (the parser's Options)
 *   TIMED_WAITING  → waitingOn when parked on an object, else null
 *   NEW/RUNNABLE/TERMINATED → all four null
 */
export interface ThreadDumpThread {
	tid: number;
	name: string;
	state: ThreadState;
	/** the Object the thread is waiting on / blocked on */
	waitingOn: string | null;
	/** the Lock (monitor) involved, when known */
	lock: string | null;
	/** Java tid of the lock owner — the edge of the wait-for graph */
	lockOwnerTid: number | null;
	lockOwnerName: string | null;
	/** Thread.trace.is_some() — false = dumped without a stack */
	hasTrace: boolean;
}

/**
 * REQUIREMENTS: rows WHERE timestamp = $1 exactly (the frontend only asks
 * for timestamps from threaddump_dumps); ordered with BLOCKED first, then
 * WAITING, TIMED_WAITING, RUNNABLE, then the rest — and by tid within a
 * state — so the interesting threads sort to the top by default.
 */
export function threaddumpThreads(
	timestamp: number,
): Promise<ThreadDumpThread[]> {
	return invoke("threaddump_threads", { timestamp });
}

// ---------------------------------------------------------------------------
// Trace: one thread's stack, WITH its lock lines interleaved
// ---------------------------------------------------------------------------

/**
 * Mirrors parser::threaddump::Element — a stack is a sequence of frames
 * with "- locked <obj>" / "- waiting to lock <obj>" lines BETWEEN them, and
 * their position carries the lock story, so the trace stays one ordered
 * sequence rather than two arrays. Discriminated on `kind`
 * (`#[serde(tag = "kind", rename_all = "lowercase")]` on Element).
 *
 * Frame(Cow, Cow) is (method, source) — same naming as cpumonitoring's
 * StackFrame so the two panels read alike.
 */
export type ThreadDumpElement =
	| { kind: "frame"; method: string; source: string }
	| { kind: "lock"; object: string };

/**
 * ```rust
 * #[tauri::command]
 * fn threaddump_trace(tid: u64, timestamp: u64, state: ...)
 *     -> Result<Option<Vec<ThreadDumpElement>>, String>
 * ```
 * REQUIREMENTS: elements ordered by idx ascending, frames and lock lines in
 * their ORIGINAL interleaving; null (None) = the thread was dumped without
 * a trace (mirrors Thread.trace: Option, same convention as cpu_stacktrace).
 * A returned array always has at least one element.
 */
export function threaddumpTrace(
	tid: number,
	timestamp: number,
): Promise<ThreadDumpElement[] | null> {
	return invoke("threaddump_trace", { tid, timestamp });
}
