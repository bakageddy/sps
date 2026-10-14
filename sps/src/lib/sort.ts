/**
 * Shared column sorting for the tables. One comparator so every table
 * agrees on the details nobody wants to re-decide per component: numbers
 * numerically, strings locale-aware, and null/undefined ("—" cells) LAST
 * in both directions — a missing value is never "the biggest".
 */
export function cmp(a: unknown, b: unknown): number {
	if (typeof a === "number" && typeof b === "number") return a - b;
	return String(a).localeCompare(String(b));
}

/**
 * Sorted copy of `rows` by `key(row)`; `key === null` keeps the incoming
 * order (the backend's or the page's ranking). Stable, so a tie keeps the
 * original ranking as the secondary order for free.
 */
export function sortRows<T>(
	rows: T[],
	key: ((row: T) => unknown) | null,
	descending: boolean,
): T[] {
	if (key === null) return rows;
	return rows.toSorted((a, b) => {
		const va = key(a);
		const vb = key(b);
		const na = va === null || va === undefined;
		const nb = vb === null || vb === undefined;
		if (na || nb) return na && nb ? 0 : na ? 1 : -1;
		const o = cmp(va, vb);
		return descending ? -o : o;
	});
}
