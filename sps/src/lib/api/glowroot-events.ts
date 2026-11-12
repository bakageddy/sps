/**
 * Glowroot ingest events — emitted by the backend while a run is going.
 *
 * Unlike the `ingest:*` log events (additive deltas from many parser
 * threads), these carry CUMULATIVE totals: a run has exactly one writer
 * task, which is the only thing that emits them, so there is one source
 * and ordering is trivial. A late or duplicated event can never make the
 * numbers wrong, only momentarily stale.
 *
 * Every payload carries `runId`; the frontend ignores events for a run it
 * does not own. Coalesce `glowroot:progress` and `glowroot:listing` to at
 * most ~5 per second — they are serialized IPC, not a log.
 *
 * REQUIREMENTS: event names exact; per run the order is
 * started → listing* → listed → progress* → finished, with `problem` at any
 * point in between; `finished` fires exactly once, including for cancelled
 * and failed runs, and after the last progress event. Rust side:
 * `app.emit("glowroot:progress", payload)` (needs `tauri::Emitter`); payload
 * structs derive `Serialize + Clone` with `#[serde(rename_all = "camelCase")]`.
 */

import type { RunOutcome } from "./glowroot";

export const GlowrootEvent = {
	Started: "glowroot:started",
	Listing: "glowroot:listing",
	Listed: "glowroot:listed",
	Progress: "glowroot:progress",
	Problem: "glowroot:problem",
	Finished: "glowroot:finished",
} as const;

/** the run is registered (also in the history, outcome "running") */
export interface GlowrootStarted {
	runId: number;
}

/** Phase 1, discovering traces: points seen so far while sub-windows are
 *  still being listed. Only ever grows. */
export interface GlowrootListing {
	runId: number;
	found: number;
}

/** Phase 1 done, the plan is fixed: `total` traces will be fetched. */
export interface GlowrootListed {
	runId: number;
	/** after skipExisting and the maxTraces cap */
	total: number;
	/** traces dropped because they were already stored */
	skippedExisting: number;
	/** true when maxTraces cut the list (newest kept) */
	capped: boolean;
}

/** Phase 2, fetching: cumulative counters. fetched = stored + failed + in
 *  the writer's queue. */
export interface GlowrootProgress {
	runId: number;
	fetched: number;
	/** committed to the database */
	stored: number;
	failed: number;
	/** requests in flight right now (≤ workers) */
	inFlight: number;
}

/** A non-fatal problem. The first 200 per run are emitted one by one, the
 *  rest only show up in `failed`. */
export interface GlowrootProblem {
	runId: number;
	agentId: string;
	/** null for a listing-level problem (a truncated sub-window, say) */
	traceId: string | null;
	message: string;
}

export interface GlowrootFinished {
	runId: number;
	outcome: Exclude<RunOutcome, "running">;
	total: number;
	stored: number;
	failed: number;
	elapsedMs: number;
	/** why a run "failed" (session expired, collector unreachable, …);
	 *  null otherwise */
	message: string | null;
}
