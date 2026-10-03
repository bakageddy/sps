/**
 * Notes — the analyst's own words, stored IN the bundle's database so they
 * travel with the .duckdb file (and vanish with an in-memory one, like
 * everything else in it). The frontend keeps an optimistic copy and
 * writes through; these commands are the persistence, nothing more.
 *
 * Conventions as everywhere: Err serialized to String, camelCase JSON,
 * u64 ms timestamps.
 */

import { invoke } from "@tauri-apps/api/core";

export interface Note {
	/** ms epoch when the note was created — the PRIMARY KEY (one person,
	 *  one keystroke per note: two notes in the same millisecond can't
	 *  happen through the UI, and createdAt never changes) */
	createdAt: number;
	text: string;
	updatedAt: number;
	/** pathname + search of the page the note was taken on */
	route: string;
}

/**
 * ```rust
 * #[tauri::command]
 * async fn notes_list(state: ...) -> Result<Vec<Note>, String>
 * #[tauri::command]
 * async fn notes_upsert(note: Note, state: ...) -> Result<(), String>
 * #[tauri::command]
 * async fn notes_delete(created_at: u64, state: ...) -> Result<(), String>
 * ```
 * REQUIREMENTS:
 *  - Table (schema.sql):
 *      CREATE TABLE IF NOT EXISTS main.notes (
 *        created_at UBIGINT PRIMARY KEY,
 *        text STRING NOT NULL,
 *        updated_at UBIGINT NOT NULL,
 *        route STRING NOT NULL
 *      );
 *    It must survive re-ingest: parse_logs may truncate the log tables but
 *    NEVER this one — notes are about the bundle, not derived from it.
 *  - notes_list: every row, ordered by updated_at desc.
 *  - notes_upsert: `INSERT OR REPLACE INTO main.notes …` keyed on
 *    created_at (the frontend sends the whole row every time; last write
 *    wins). One row per
 *    Ctrl+S — a plain prepared statement, no appender.
 *  - notes_delete: DELETE WHERE created_at = $1; an unknown key is Ok(()).
 *  - `Note` derives Deserialize as well as Serialize (it is a command
 *    argument), with `rename_all = "camelCase"`.
 */
export function notesList(): Promise<Note[]> {
	return invoke("notes_list");
}
export function notesUpsert(note: Note): Promise<void> {
	return invoke("notes_upsert", { note });
}
export function notesDelete(createdAt: number): Promise<void> {
	return invoke("notes_delete", { createdAt });
}
