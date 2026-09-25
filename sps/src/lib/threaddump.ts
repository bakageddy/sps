/**
 * Thread-dump presentation + analysis helpers (pure functions, no state).
 */

import type { ThreadDumpThread, ThreadState } from "$lib/api/threaddump";

/** Badge color per state — BLOCKED is the alarm, WAITING the caution. */
export function stateColor(state: ThreadState): string {
	switch (state) {
		case "BLOCKED":
			return "var(--red)";
		case "WAITING":
			return "var(--yellow)";
		case "TIMED_WAITING":
			return "var(--chart-4)";
		case "RUNNABLE":
			return "var(--green)";
		default: // NEW, TERMINATED
			return "var(--fg-muted)";
	}
}

/**
 * Deadlock detection over one dump's census.
 *
 * The wait-for graph has ONE outgoing edge per thread (a thread waits on at
 * most one lock owner), so it is a functional graph and every cycle is
 * found by walking each thread's owner chain once: if the walk re-enters a
 * node already on the CURRENT path, the path from that node onward is a
 * cycle. Nodes are marked done globally so each cycle is reported once and
 * chains merely leading INTO a cycle (victims of the deadlock, not members)
 * are not reported as cycles. O(n).
 *
 * Returns each cycle as its member threads in wait order — t[i] waits on a
 * lock owned by t[i+1], and the last waits on the first.
 */
export function findDeadlocks(
	threads: ThreadDumpThread[],
): ThreadDumpThread[][] {
	const byTid = new Map(threads.map((t) => [t.tid, t]));
	const done = new Set<number>();
	const cycles: ThreadDumpThread[][] = [];

	for (const start of threads) {
		if (done.has(start.tid) || start.lockOwnerTid === null) continue;

		const path: ThreadDumpThread[] = [];
		const onPath = new Map<number, number>(); // tid -> index in path
		let cur: ThreadDumpThread | undefined = start;

		while (cur && !done.has(cur.tid)) {
			const seenAt = onPath.get(cur.tid);
			if (seenAt !== undefined) {
				cycles.push(path.slice(seenAt));
				break;
			}
			onPath.set(cur.tid, path.length);
			path.push(cur);
			cur =
				cur.lockOwnerTid === null ? undefined : byTid.get(cur.lockOwnerTid);
		}
		for (const t of path) done.add(t.tid);
	}
	return cycles;
}

/**
 * Threads blocked (directly or transitively) by a given tid — the "how many
 * are stuck behind this one" number for a hot lock owner.
 */
export function blockedBehind(
	threads: ThreadDumpThread[],
	ownerTid: number,
): number {
	const waiters = new Map<number, number[]>();
	for (const t of threads) {
		if (t.lockOwnerTid === null) continue;
		const list = waiters.get(t.lockOwnerTid);
		if (list) list.push(t.tid);
		else waiters.set(t.lockOwnerTid, [t.tid]);
	}
	const seen = new Set<number>();
	const stack = [ownerTid];
	while (stack.length > 0) {
		for (const w of waiters.get(stack.pop()!) ?? []) {
			if (!seen.has(w)) {
				seen.add(w);
				stack.push(w);
			}
		}
	}
	return seen.size;
}
