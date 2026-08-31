/**
 * Log ingestion — kind-agnostic on purpose.
 *
 * The topbar/DropZone hands ONE path (a log file or a bundle directory) to
 * the backend, which decides what's inside. Progress and results do NOT
 * come back through this command: parsing reports via the events in
 * ./ingest-events.ts, so the command only validates and kicks the work off.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * Validate the path and start parsing it in the background. Resolves as
 * soon as parsing has STARTED (Err = validation failed: missing path,
 * no database, ...). Everything after that arrives as ingest:* events.
 *
 * ```rust
 * #[tauri::command]
 * async fn parse_logs(
 *     path: String,
 *     app: tauri::AppHandle,
 *     state: tauri::State<'_, Mutex<AppState>>,
 * ) -> Result<(), String>
 * ```
 * REQUIREMENTS: resolves once parsing has STARTED (Err = validation only);
 * must not block the command or the UI for the duration of the parse;
 * ingest:finished fires exactly once, after ALL parsers are done.
 */
export function parseLogs(path: string): Promise<void> {
  return invoke("parse_logs", { path });
}
