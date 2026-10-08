/**
 * HealthMeter — just the "Server Time" cell, verbatim, nothing interpreted.
 *
 * The backend's job stops at "here is the text DuckDB has"; zone extraction
 * (pulling a token like "IST" or "Asia/Calcutta" out of it, validating it
 * against Intl) lives entirely in the frontend (lib/timezone.svelte.ts
 * extractZone/isValidZone) — one place that owns "how do we interpret this
 * string", not split across Rust and TS.
 */

import { invoke } from "@tauri-apps/api/core";

/**
 * ```rust
 * #[tauri::command]
 * async fn healthmeter_info(state: ...) -> Result<Option<String>, String>
 * ```
 * REQUIREMENTS: `SELECT val FROM main.healthmeter WHERE key = 'Server Time'
 * LIMIT 1` — the row text, unmodified (no IANA/offset parsing, no format
 * validation). The table has no column identifying which capture a row
 * belongs to, so absent an ORDER BY, which row comes back is whichever one
 * DuckDB happens to return — that's a known, accepted gap for now, not a
 * bug in this command. Ok(None) when the table is empty or has no such key
 * (the bundle had no HealthMeter data — normal, not an error). Err is for
 * actual query/database failures only.
 */
export function healthmeterInfo(): Promise<string | null> {
	return invoke("healthmeter_info");
}
