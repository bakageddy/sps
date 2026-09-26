/**
 * Connection-dump presentation helpers (pure functions, no state).
 */

/**
 * Stack-frame prefixes that are connection/persistence plumbing — the fixed
 * sandwich every trace shares. The first frame NOT matching any of these is
 * the "owner" frame: the application code that actually wanted the
 * connection. Sole home of this heuristic — the backend ships raw stacks
 * (ConnDumpHolder.stackTrace) and never derives owner frames.
 */
const PLUMBING = [
	"java.",
	"jdk.",
	"com.zoho.cp.",
	"com.zoho.mickey.",
	"com.adventnet.ds.",
	"com.adventnet.db.",
	"com.adventnet.persistence",
	"com.adventnet.mfw.",
	"com.adventnet.authorization.",
	"com.manageengine.mdh.QueryInterceptor",
	// thin DataAccess wrappers — naming them would tell you nothing
	"com.adventnet.servicedesk.utils.DataAccessUtil",
	"com.manageengine.sdpod.v3api.utils.DataAccessUtil",
	"com.adventnet.servicedesk.utils.ResourcesUtil",
	"com.adventnet.servicedesk.utils.DBUtilities",
	"com.manageengine.servicedesk.utils.SDQueryExecutor",
	"com.manageengine.servicedesk.v3api.utils.DBUtilitiesOP",
];

/** First non-plumbing frame of a stack, or null when it's plumbing all the
 *  way down. */
export function appFrame(stack: string[]): string | null {
	for (const frame of stack) {
		if (!PLUMBING.some((p) => frame.startsWith(p))) return frame;
	}
	return null;
}

/**
 * HTTP worker thread names carry the request: "<url>-<epoch>_###_<thread>".
 * Returns the url part for those, the name unchanged otherwise.
 */
export function threadLabel(threadName: string): string {
	const i = threadName.indexOf("_###_");
	if (i === -1) return threadName;
	const head = threadName.slice(0, i);
	// strip the trailing "-<epoch millis>" stamp if present
	return head.replace(/-\d{10,}$/, "");
}

/**
 * Alarm tick color per cause (CSS custom property reference). This is a
 * CATEGORICAL legend, so the hues must stay distinct from each other —
 * NMC takes the palette's warm orange slot (chart-2) rather than the
 * semantic --alert, which is yellow and would collide with High CPU.
 */
export function causeColor(cause: string): string {
	switch (cause) {
		case "No ManagedConnections":
			return "var(--chart-2)";
		case "High CPU":
			return "var(--yellow)";
		case "High Memory Consumption":
			return "var(--chart-4)";
		default: // URL invocation and anything new
			return "var(--chart-6)";
	}
}

// ---------------------------------------------------------------------------
// Incident census (frontend join)
// ---------------------------------------------------------------------------

import type { ThreadDumpThread } from "$lib/api/threaddump";
import type { DumpThread } from "$lib/api/cpumonitoring";
import type { ConnDumpTrace } from "$lib/api/connectiondump";

/**
 * One row of the incident census: the threaddump thread (the spine — the
 * only log that has EVERY thread) decorated with what the other logs knew
 * about it at the incident. null = that log has no row for this tid at
 * its anchor: cpumonitoring only logs hot threads, connectiondump only
 * connection holders — null is normal, not missing data.
 */
export interface IncidentThread extends ThreadDumpThread {
	/** cpumonitoring cpu % at the incident's cpumonitoring anchor */
	cpu: number | null;
	/** ms this thread had held a pooled connection at the trace anchor */
	heldFor: number | null;
}

/**
 * Join by EXACT tid — the same Java thread id across all three logs. The
 * backend resolves WHICH dump of each log is "this incident"
 * (connectiondump_<subsystem> resolvers); this does the row-level join
 * those anchors make possible. Order is the threaddump's (BLOCKED first).
 */
export function buildCensus(
	threads: ThreadDumpThread[],
	cpu: DumpThread[],
	traces: ConnDumpTrace[],
): IncidentThread[] {
	const cpuByTid = new Map(cpu.map((t) => [t.tid, t.cpu]));
	// a thread holds at most one connection at a time; keep the longest
	// defensively should a dump ever list a tid twice
	const heldByTid = new Map<number, number>();
	for (const tr of traces) {
		const prev = heldByTid.get(tr.id);
		if (prev === undefined || tr.duration > prev)
			heldByTid.set(tr.id, tr.duration);
	}
	return threads.map((t) => ({
		...t,
		cpu: cpuByTid.get(t.tid) ?? null,
		heldFor: heldByTid.get(t.tid) ?? null,
	}));
}

/**
 * connectiondump's OWN anchor needs no backend command: the LATEST trace
 * dump within the window, from the snapshots already loaded — the same
 * latest-in-window rule the backend resolvers apply to the other logs, so
 * all anchors are the final sample of the burst (see the contract).
 */
export function latestWithin<T extends { timestamp: number }>(
	items: T[],
	target: number,
	toleranceMs: number,
): T | null {
	let best: T | null = null;
	for (const it of items) {
		if (Math.abs(it.timestamp - target) > toleranceMs) continue;
		if (best === null || it.timestamp > best.timestamp) best = it;
	}
	return best;
}
