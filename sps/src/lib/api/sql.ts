/**
 * SQL console — run READ-ONLY queries against the open DuckDB and export
 * results without the rows crossing IPC.
 *
 * Trust model: the frontend is a text box; it trusts NOTHING about the
 * statement. Everything that makes this safe (SELECT-only, one statement,
 * row cap) is enforced in the backend — the requirements below are the
 * spec, the UI merely displays outcomes.
 *
 * Conventions as everywhere: Err serialized to String (rejected promise —
 * here the DuckDB error text VERBATIM, the console shows it under the
 * editor), camelCase JSON.
 */

import { invoke } from "@tauri-apps/api/core";

/** a schema column (from DuckDB's own catalog, never from the user's query) */
export interface SqlColumn {
	name: string;
	/** DuckDB type text, e.g. "UBIGINT" or the full
	 *  "ENUM('High CPU', 'No ManagedConnections', ...)" definition */
	type: string;
	nullable: boolean;
	/** the column's default expression exactly as DuckDB prints it, or
	 *  null when the column has none (named defaultValue, not default:
	 *  a reserved word, awkward to destructure) */
	defaultValue: string | null;
}

export interface SqlResult {
	/** result column NAMES, from the executed statement's metadata */
	columns: string[];
	/** one entry per column, in `columns` order; NULL → null */
	rows: (string | null)[][];
	/** backend-measured execution time, ms */
	elapsedMs: number;
}

/**
 * ```rust
 * #[tauri::command]
 * async fn sql_query(sql: String, limit: u64, offset: u64, state: ...)
 *     -> Result<Option<SqlResult>, String>
 * ```
 * REQUIREMENTS:
 *  - SELECT-ONLY, ONE STATEMENT. Reject anything else with an Err that
 *    says so ("only a single SELECT statement is allowed"). Enforce it on
 *    the parsed statement, not on the text — a text prefix check passes
 *    `SELECT 1; DROP TABLE x`. Two in-database options: DuckDB's
 *    `json_serialize_sql($1)` only serializes SELECT statements (errors on
 *    anything else) and returns a `statements` array whose length must be
 *    exactly 1; or duckdb-rs's prepare, which refuses multi-statement
 *    strings, plus a statement-type check (unverified on my side — check
 *    the crate). Either way the gate runs BEFORE execution.
 *  - CAP + PAGE, NO PROBE: execute `SELECT * FROM (<sql>) AS q LIMIT
 *    $limit OFFSET $offset` — plain limit, not limit+1; nothing to drop.
 *    Ok(None) when that returns zero rows — covers BOTH "the query
 *    genuinely matched nothing" and "offset is past the last row"; the
 *    frontend renders the same "No rows" either way, so collapsing them
 *    is fine. Ok(Some(result)) otherwise, 1..=limit rows. The frontend
 *    picks limit from {100, 1000, 10000} and infers "maybe another page"
 *    from `rows.length === limit` (a short page is definitely the last
 *    one) — no hasMore field to compute.
 *  - NO DESCRIBE / EXPLAIN of the user's statement — the statement runs
 *    exactly once. `columns` = the executed statement's column names
 *    (statement metadata after execution); no types.
 *  - CELLS AS TEXT: every value stringified in Rust from the row value
 *    (NULL → null; numbers/bools/timestamps in their DuckDB text form;
 *    ENUM by label; LIST/STRUCT in DuckDB's literal syntax or JSON —
 *    your call, the console only displays and copies them).
 *  - elapsedMs measured around the execution only (not the gate); absent
 *    on Ok(None) — there is no result to attach it to.
 *  - Err text = DuckDB's own message (Parser/Binder/Catalog errors carry
 *    the position and the candidate names — the console shows them raw).
 */
export function sqlQuery(
	sql: string,
	limit: number,
	offset: number,
): Promise<SqlResult | null> {
	return invoke("sql_query", { sql, limit, offset });
}

export interface SqlTable {
	name: string;
	columns: SqlColumn[];
	/** COUNT(*) — DuckDB answers this from metadata, it is cheap */
	rows: number;
}

/**
 * ```rust
 * #[tauri::command]
 * async fn sql_schema(state: ...) -> Result<Vec<SqlTable>, String>
 * ```
 * REQUIREMENTS: every table in schema `main`, with its row count
 * (`SELECT COUNT(*)` per table, or duckdb_tables().estimated_size if
 * approximate is fine). Columns and types come from DuckDB's own catalog,
 * NOT information_schema (verified 2026-10-03):
 *
 *   SELECT table_name, column_index, column_name, data_type,
 *          is_nullable, column_default
 *   FROM duckdb_columns() AS col
 *   JOIN duckdb_types() AS t
 *     ON col.database_name = t.database_name
 *    AND col.schema_name = t.schema_name
 *    AND col.data_type_id = t.type_oid
 *   WHERE col.schema_name = 'main'
 *   ORDER BY table_name, column_index;
 *
 * This resolves an ENUM column to its FULL definition — e.g.
 * `ENUM('High CPU', 'No ManagedConnections', 'High Memory Consumption',
 * 'URL invocation')` — not the bare word "ENUM" and not just the type's
 * catalog name; that's `type` verbatim. `is_nullable` -> `nullable`,
 * `column_default` -> `defaultValue` (NULL -> null; it is the DEFAULT
 * expression text, e.g. "0" or "now()", not a sentinel). Empty Vec only
 * if the schema was never applied.
 */
export function sqlSchema(): Promise<SqlTable[]> {
	return invoke("sql_schema");
}

/**
 * ```rust
 * #[tauri::command]
 * async fn sql_export_csv(sql: String, path: String, state: ...)
 *     -> Result<u64, String>
 * ```
 * REQUIREMENTS: same SELECT-only/single-statement gate as sql_query, then
 * `COPY (<sql>) TO '<path>' (FORMAT CSV, HEADER)` — the FULL result, no
 * cap, and no rows cross IPC. Return the row count COPY reports. `path`
 * comes from the native save dialog (tauri-plugin-dialog `save()`, already
 * permitted by `dialog:default` in the capability); overwrite is the user's call
 * (the dialog already asked). Err = DuckDB's message, including I/O
 * failures on the path.
 */
export function sqlExportCsv(sql: string, path: string): Promise<number> {
	return invoke("sql_export_csv", { sql, path });
}
