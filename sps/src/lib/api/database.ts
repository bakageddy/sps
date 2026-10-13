/**
 * Database lifecycle — which DuckDB file the Store is backed by.
 * The user picks the location at runtime; empty/None = in-memory.
 */

import { invoke } from "@tauri-apps/api/core";

export interface DatabaseInfo {
	/** Absolute path of the backing file; null when running in-memory. */
	path: string | null;
}

/**
 * Open (or create) a database at `path`, replacing any currently open one.
 * `null` path = in-memory database.
 *
 * ```rust
 * #[tauri::command]
 * fn open_database(
 *     path: Option<String>,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<DatabaseInfo, String>
 * ```
 * REQUIREMENT: replaces any currently open database.
 */
export function openDatabase(path: string | null): Promise<DatabaseInfo> {
	return invoke("open_database", { path });
}

/**
 * What's currently open, if anything. The frontend calls this once on
 * startup so a webview reload (dev-mode hot reload!) re-syncs with the
 * backend instead of showing "no database" while one is actually open.
 *
 * ```rust
 * #[tauri::command]
 * fn database_info(
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<Option<DatabaseInfo>, String>
 * ```
 */
export function databaseInfo(): Promise<DatabaseInfo | null> {
	return invoke("database_info");
}

/**
 * Persist the live (in-memory) database to a file. Copies everything —
 * every table, the ENUM types, `notes` — and leaves the in-memory
 * connection as the live one; the FRONTEND then calls open_database(path)
 * to switch over, so from that point every write lands on disk.
 *
 * ```rust
 * #[tauri::command]
 * fn save_database(
 *     path: String,
 *     state: tauri::State<'_, AppDb>,
 * ) -> Result<(), String>
 * ```
 * REQUIREMENTS: on the live connection,
 *   ATTACH '<path>' AS target;
 *   COPY FROM DATABASE memory TO target;
 *   DETACH target;
 * (`memory` is DuckDB's name for the default in-memory catalog; COPY FROM
 * DATABASE needs DuckDB ≥ 0.10 and an EMPTY target — ATTACH on a path that
 * doesn't exist yet creates it fresh, which is the intended case.) Does NOT
 * reopen or switch the store. Err = the path already exists with content,
 * the live database is already file-backed, or DuckDB failed — verbatim.
 */
export function saveDatabaseAs(path: string): Promise<void> {
	return invoke("save_database", { path });
}
