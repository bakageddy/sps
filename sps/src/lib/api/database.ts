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
