/**
 * App-wide database connection state.
 *
 * Same pattern as theme.svelte.ts — a reactive module — but here the state
 * is a discriminated union, because "is a database open?" has four distinct
 * answers and components render differently for each. Any component can
 * `import { db }` and react; only this module talks to the backend about it.
 */
import {
	openDatabase,
	databaseInfo,
	saveDatabaseAs,
	type DatabaseInfo,
} from "$lib/api/database";

export type DbState =
	| { status: "closed" }
	| { status: "opening" }
	| { status: "open"; info: DatabaseInfo }
	| { status: "error"; message: string };

/** `epoch` increments on every successful open/adopt — it identifies the
 * live store INSTANCE, not the path (reopening in-memory is a fresh empty
 * database at the same "path"). Cache invalidation keys off it. */
export const db = $state<{ state: DbState; epoch: number }>({
	state: { status: "closed" },
	epoch: 0,
});

/**
 * Open/create a database; empty or whitespace path means in-memory.
 * Returns the resulting state so callers can check the outcome directly —
 * re-reading db.state after the await runs into TypeScript keeping stale
 * narrowing on it (it can't know this function mutates the object).
 */
export async function open(path: string): Promise<DbState> {
	db.state = { status: "opening" };
	try {
		const trimmed = path.trim();
		const info = await openDatabase(trimmed === "" ? null : trimmed);
		db.state = { status: "open", info };
		db.epoch += 1;
	} catch (e) {
		db.state = { status: "error", message: String(e) };
	}
	return db.state;
}

/**
 * Persist the in-memory database to `path`, then switch the live
 * connection over to that file (so later ingests and notes land on disk).
 * Two steps on purpose — the backend only copies; this module owns the
 * reopen, same as every other connection-state change.
 *
 * THROWS if the copy fails, leaving db.state untouched: the in-memory
 * database is still perfectly good, and a failed save must not paint the
 * working connection as broken. A failed reopen reports through db.state
 * like any open() does.
 */
export async function saveAs(path: string): Promise<DbState> {
	await saveDatabaseAs(path);
	return open(path);
}

/**
 * Opening a database explicitly is OPTIONAL: pages that need one call this
 * before their first data command, and it lazily opens an in-memory
 * database if the user never touched the topbar control.
 */
export async function ensureOpen(): Promise<void> {
	if (db.state.status === "open") return;
	const state = await open("");
	if (state.status !== "open") {
		const reason = state.status === "error" ? state.message : "unknown";
		throw new Error(`could not open in-memory database: ${reason}`);
	}
}

/** Called once at app start: adopt whatever the backend already has open. */
export async function sync(): Promise<void> {
	try {
		const info = await databaseInfo();
		if (info) {
			db.state = { status: "open", info };
			db.epoch += 1;
		}
	} catch (e) {
		db.state = { status: "error", message: String(e) };
		// Command not implemented yet — stay "closed", the UI still works.
	}
}
