/** Nearest row by timestamp — cross-analyzer links can never join on
 * equality: the log kinds sample on independent clocks. */
export function nearestByTimestamp<T extends { timestamp: number }>(
  rows: T[],
  target: number,
): T | null {
  if (rows.length === 0) return null;
  return rows.reduce((best, row) =>
    Math.abs(row.timestamp - target) < Math.abs(best.timestamp - target) ? row : best,
  );
}
