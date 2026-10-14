/**
 * Index-fragmentation queries — the contract between this frontend and the
 * Rust side. Each function wraps one Tauri command; the comment above it is
 * the signature to implement in src-tauri. Conventions (same as the rest):
 *
 *   - Err type serialized to String (`.map_err(|e| e.to_string())`) — it
 *     arrives as a rejected promise, never in the return type.
 *   - Response structs derive Serialize with `#[serde(rename_all = "camelCase")]`.
 *   - Timestamps are u64 ms since epoch.
 *   - Store lives behind AppDb (see api/database.ts); commands error when
 *     no database is open.
 *
 * Source table: `main.fragmentationratio`
 *   (timestamp UBIGINT, schema VARCHAR, table VARCHAR, index VARCHAR,
 *    ratio DOUBLE, page_count UBIGINT)
 * One capture = every index of the database at one timestamp; a bundle
 * holds a handful of captures. `ratio` is SQL Server's
 * avg_fragmentation_in_percent, i.e. 0–100, NOT 0–1.
 *
 * Navigation is SNAPSHOT-CENTRIC like the other analyzers: list captures,
 * drill into one capture's indices, then one index's ratio across captures
 * (is it growing, or was it always like this?).
 */

import { invoke } from "@tauri-apps/api/core";

// ---------------------------------------------------------------------------
// Types (mirror these as Rust structs)
// ---------------------------------------------------------------------------

/** One row per capture — feeds the snapshot list. */
export interface FragSnapshot {
	/** ms epoch of the capture header; identifies the capture everywhere */
	timestamp: number;
	/** number of index rows in this capture */
	indices: number;
	/** highest ratio in the capture (0–100) */
	maxRatio: number;
	/** sum of page_count over the capture — the database's indexed size */
	totalPages: number;
}

/** One index within ONE capture. */
export interface FragIndex {
	schema: string;
	table: string;
	index: string;
	/** 0–100 */
	ratio: number;
	pageCount: number;
}

/** One index that crossed BOTH thresholds in at least one capture. */
export interface FragFlag {
	schema: string;
	table: string;
	index: string;
	/** captures that contain this index at all */
	captures: number;
	/** captures where ratio >= min_ratio AND page_count >= min_pages */
	flaggedCaptures: number;
	/** ms epoch of the first / last capture where it was flagged */
	firstFlagged: number;
	lastFlagged: number;
	/** reading at the NEWEST capture that contains the index */
	latestRatio: number;
	latestPages: number;
	/** highest ratio over all captures */
	maxRatio: number;
}

/** One index's reading at one capture — the trend series. */
export interface FragPoint {
	/** ms epoch of the capture */
	timestamp: number;
	ratio: number;
	pageCount: number;
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

/**
 * Every capture in the store, ordered by timestamp ascending.
 *
 * ```rust
 * #[tauri::command]
 * async fn fragmentation_snapshots(
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<FragSnapshot>, String>
 * ```
 */
export function fragmentationSnapshots(): Promise<FragSnapshot[]> {
	return invoke("fragmentation_snapshots");
}

/**
 * All index rows of ONE capture, ordered by ratio descending (the table
 * re-sorts client-side; this is just a sensible arrival order).
 *
 * ```rust
 * #[tauri::command]
 * async fn fragmentation_indices(
 *     timestamp: u64,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<FragIndex>, String>
 * ```
 */
export function fragmentationIndices(timestamp: number): Promise<FragIndex[]> {
	return invoke("fragmentation_indices", { timestamp });
}

/**
 * One index's ratio and size across EVERY capture, ordered by timestamp
 * ascending — the chart needs the order; don't rely on insertion order.
 *
 * ```rust
 * #[tauri::command]
 * async fn fragmentation_index_series(
 *     schema: String,
 *     table: String,
 *     index: String,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<FragPoint>, String>
 * ```
 */
export function fragmentationIndexSeries(
	schema: string,
	table: string,
	index: string,
): Promise<FragPoint[]> {
	return invoke("fragmentation_index_series", { schema, table, index });
}

/**
 * Indices that need attention ACROSS captures: one row per index that was
 * flagged (ratio >= min_ratio AND page_count >= min_pages) in at least one
 * capture, with how persistently. Thresholds are arguments — the user
 * tunes them in the toolbar — never constants on the Rust side.
 *
 * ```rust
 * #[tauri::command]
 * async fn fragmentation_flagged(
 *     min_pages: u64,
 *     min_ratio: f64,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Vec<FragFlag>, String>
 * ```
 */
export function fragmentationFlagged(
	minPages: number,
	minRatio: number,
): Promise<FragFlag[]> {
	return invoke("fragmentation_flagged", { minPages, minRatio });
}
