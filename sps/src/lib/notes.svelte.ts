/**
 * Quick notes — what you noticed while reading a bundle, kept beside the
 * analyzers. Ctrl+\ opens the editor over whatever page you're on; the
 * sidebar lists every note; each remembers the route (with its ?t=) it
 * was taken on, so a note is also a bookmark back into the evidence.
 *
 * Persistence is the DATABASE (api/notes.ts), not localStorage: notes
 * belong to the bundle and travel with its .duckdb file. Saving is
 * EXPLICIT — Ctrl+S in the editor — never on keystroke; this module holds
 * the saved copy and writes through on save/delete. With an in-memory
 * database they last exactly as long as it does, like everything else.
 */
import { notesList, notesUpsert, notesDelete, type Note } from "$lib/api/notes";
import { db, ensureOpen } from "$lib/database.svelte";

export type { Note };

export const notes = $state<{ list: Note[]; error: string | null }>({
	list: [],
	error: null,
});

/** the quick-note editor: open/closed, and which note (createdAt) it edits */
export const noteEditor = $state<{ open: boolean; id: number | null }>({
	open: false,
	id: null,
});

/** (re)load from the open database; clears when none is open */
export async function loadNotes(): Promise<void> {
	if (db.state.status !== "open") {
		notes.list = [];
		return;
	}
	try {
		notes.list = await notesList();
		notes.error = null;
	} catch (e) {
		notes.error = String(e);
	}
}

export function openQuickNote(id: number | null = null): void {
	noteEditor.id = id;
	noteEditor.open = true;
}
export function closeQuickNote(): void {
	noteEditor.open = false;
	noteEditor.id = null;
}
export function toggleQuickNote(): void {
	if (noteEditor.open) closeQuickNote();
	else openQuickNote(null);
}

export function getNote(id: number): Note | undefined {
	return notes.list.find((n) => n.createdAt === id);
}

/**
 * Save: create (id null) or update. Writes to the database FIRST and only
 * then updates the in-memory list, so what the sidebar shows is what the
 * database has. Throws on failure (the editor shows it); returns the key.
 */
export async function saveNote(id: number | null, text: string, route: string): Promise<number> {
	const now = Date.now();
	let note: Note;
	const i = id === null ? -1 : notes.list.findIndex((n) => n.createdAt === id);
	if (i >= 0) note = { ...notes.list[i], text, updatedAt: now };
	else {
		// createdAt is the key: bump past the newest note if two land in one ms
		const newest = notes.list.reduce((m, n) => Math.max(m, n.createdAt), 0);
		note = { createdAt: Math.max(now, newest + 1), text, updatedAt: now, route };
	}
	await ensureOpen();
	await notesUpsert(note);
	notes.error = null;
	if (i >= 0) notes.list[i] = note;
	else notes.list = [note, ...notes.list];
	return note.createdAt;
}

export async function deleteNote(id: number): Promise<void> {
	await notesDelete(id);
	notes.error = null;
	notes.list = notes.list.filter((n) => n.createdAt !== id);
}

/** first non-empty line, for lists */
export function noteTitle(n: Note, max = 48): string {
	const line = n.text.split("\n").find((l) => l.trim() !== "") ?? "";
	const t = line.trim();
	return t.length > max ? t.slice(0, max - 1) + "…" : t || "(empty)";
}

/** every note as one Markdown document, newest first */
export function notesAsMarkdown(fmt: (ms: number) => string): string {
	return notes.list
		.toSorted((a, b) => b.updatedAt - a.updatedAt)
		.map((n) => `## ${fmt(n.createdAt)} · ${n.route}\n\n${n.text.trim()}\n`)
		.join("\n");
}
