/**
 * HealthMeter.html — the one file in a bundle that states the SERVER's
 * timezone. Every log line is server-local wall-clock time with no offset,
 * so without this the UI can only guess; with it, timestamps can be shown
 * exactly as the server saw them ("display timezone" on the Ingest page).
 *
 * Read-only lookup, not part of ingest: the frontend calls it once per
 * dropped path, right after the parse finishes, and keeps the answer.
 */

import { invoke } from "@tauri-apps/api/core";

export interface HealthMeterInfo {
	/** IANA zone name exactly as the file prints it, e.g. "Asia/Kolkata" */
	timezone: string;
	/** the Server Time cell's text minus the zone, e.g. "Sep 10, 2026 05:20 PM" */
	serverTime: string;
	/** the OS Locale row, e.g. "en_US.UTF-8", when present */
	osLocale: string | null;
}

/**
 * ```rust
 * #[tauri::command]
 * async fn healthmeter_info(path: String) -> Result<Option<HealthMeterInfo>, String>
 * ```
 * REQUIREMENTS: `path` is whatever the user dropped — a bundle directory
 * or a single log file; look for `HealthMeter.html` (case-insensitive) in
 * that directory, or in the file's parent. Parse (scraper is already a
 * dependency) the table row whose first cell is "Server Time"; the second
 * cell reads `Sep 10, 2026 05:20 PM  Asia/Kolkata` — the zone is the LAST
 * whitespace-separated token, the rest is serverTime. Same for the
 * "OS Locale" row → osLocale.
 * Ok(None) = no HealthMeter.html, or no Server Time row (a bundle without
 * one is normal, not an error). Err = I/O or HTML parse failure only.
 * Do NOT convert or validate the zone name — the frontend checks it
 * against Intl and falls back if the runtime doesn't know it.
 */
export function healthmeterInfo(path: string): Promise<HealthMeterInfo | null> {
	return invoke("healthmeter_info", { path });
}
