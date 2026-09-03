/**
 * View-model helpers over the Rust-aggregated episodes (StuckThread).
 * Aggregation itself lives in the backend; this module only derives
 * display geometry and identity.
 */
import type { StuckThread } from "$lib/api/stuckthread";

/** Minimal geometry the overview strip draws. */
export interface StuckBar {
  tid: number;
  /** = episode start */
  timestamp: number;
  durationMs: number;
}

/**
 * Episode geometry [start, end], ms epoch, best effort per row shape:
 *  - paired:           [begin, end]
 *  - never completed:  [begin, begin + duration]
 *  - completion only:  [end - duration, end]
 */
export function bounds(t: StuckThread): [number, number] {
  const start = t.begin ?? (t.end !== null ? t.end - t.duration : 0);
  const end = t.end ?? (t.begin ?? 0) + t.duration;
  return end > start ? [start, end] : [start, start + 1];
}

/** Stable row identity: tid + the first event timestamp we know of. */
export const threadKey = (t: StuckThread) => `${t.tid}:${t.begin ?? t.end}`;

export function threadBar(t: StuckThread): StuckBar {
  const [start, end] = bounds(t);
  return { tid: t.tid, timestamp: start, durationMs: end - start };
}

// ---------------------------------------------------------------------------
// Concurrency: how many episodes are active at each instant
// ---------------------------------------------------------------------------

/** One step of the concurrency curve: `count` holds from `t` onward. */
export interface ConcurrencyPoint {
  t: number;
  count: number;
}

/**
 * Sweep-line over episode bounds: +1 at start, -1 at end, sorted by time
 * (ends before starts at equal timestamps, so a back-to-back handoff
 * doesn't spike). A plateau near the server's maxThreads is the signature
 * of thread-pool exhaustion.
 */
export function concurrencySteps(threads: StuckThread[]): ConcurrencyPoint[] {
  const deltas: [number, number][] = [];
  for (const t of threads) {
    const [start, end] = bounds(t);
    deltas.push([start, 1], [end, -1]);
  }
  deltas.sort((a, b) => a[0] - b[0] || a[1] - b[1]);

  const points: ConcurrencyPoint[] = [];
  let count = 0;
  for (const [t, delta] of deltas) {
    count += delta;
    const last = points[points.length - 1];
    if (last !== undefined && last.t === t) last.count = count;
    else points.push({ t, count });
  }
  return points;
}

// ---------------------------------------------------------------------------
// Rollup by request path
// ---------------------------------------------------------------------------

export interface PathRollup {
  path: string;
  episodes: number;
  /** episodes never seen completing */
  open: number;
  maxMs: number;
  totalMs: number;
}

/** request path without host; completion-only rows group under one bucket */
export function requestPath(t: StuckThread): string {
  if (t.request === null) return "(unknown request)";
  try {
    return new URL(t.request).pathname;
  } catch {
    return t.request;
  }
}

/** Group episodes per path, worst offenders (total stuck time) first. */
export function pathRollup(threads: StuckThread[]): PathRollup[] {
  const buckets = new Map<string, PathRollup>();
  for (const t of threads) {
    const path = requestPath(t);
    let bucket = buckets.get(path);
    if (bucket === undefined) {
      bucket = { path, episodes: 0, open: 0, maxMs: 0, totalMs: 0 };
      buckets.set(path, bucket);
    }
    bucket.episodes += 1;
    if (t.end === null) bucket.open += 1;
    bucket.maxMs = Math.max(bucket.maxMs, t.duration);
    bucket.totalMs += t.duration;
  }
  return [...buckets.values()].toSorted((a, b) => b.totalMs - a.totalMs);
}
