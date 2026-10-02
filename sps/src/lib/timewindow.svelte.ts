/**
 * Global time window — one [from, to] (ms epoch) the whole app respects.
 *
 * Set by sweeping the TimeStrip under the top of every analyzer; null =
 * the full bundle. Lists apply it through applyWindow() inside the shared
 * list components, so every page narrows at once without each one
 * plumbing a prop. Charts with their own zoom (pool chart, stuck-thread
 * overview) take it as their domain/initial view.
 */
import { persisted } from "$lib/persisted.svelte";

export const timeWindow = persisted<[number, number] | null>("time-window", null);

export function inWindow(ts: number): boolean {
	const w = timeWindow.value;
	return w === null || (ts >= w[0] && ts <= w[1]);
}

/** rows whose timestamp falls in the window (all of them when unset) */
export function applyWindow<T extends { timestamp: number }>(rows: T[]): T[] {
	const w = timeWindow.value;
	if (w === null) return rows;
	return rows.filter((r) => r.timestamp >= w[0] && r.timestamp <= w[1]);
}

/** [first, last] timestamp across every log kind — set by the TimeStrip
 *  once the lanes load; null before that. Bare "HH:MM" inputs take their
 *  date from here. */
export const bundleDomain = $state<{ value: [number, number] | null }>({ value: null });
