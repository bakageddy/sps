/**
 * Format a ms-epoch timestamp, surviving corrupt values. Intl throws a
 * RangeError past ±8.64e15 (the JS Date range) — and a u64 subtraction
 * wrapping in a release-build parser produces exactly such values. One
 * bad row must label itself, not crash the page.
 */
export function formatTimestamp(fmt: Intl.DateTimeFormat, ms: number): string {
  return Number.isFinite(ms) && Math.abs(ms) <= 8.64e15 ? fmt.format(ms) : "corrupt ts";
}

/** Human-scale duration: 843 ms → "843 ms", 10222 → "10.2 s", 154000 → "2m 34s". */
export function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms} ms`;
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)} s`;
  const m = Math.floor(ms / 60_000);
  return `${m}m ${Math.round((ms % 60_000) / 1000)}s`;
}
