/**
 * Quick notes — what you noticed while reading a bundle, kept beside the
 * analyzers. Ctrl+\ opens the editor over whatever page you're on; the
 * sidebar lists every note; each remembers the route (with its ?t=) it
 * was taken on, so a note is also a bookmark back into the evidence.
 *
 * Persistence is the DATABASE (api/notes.ts), not localStorage: notes
 * belong to the bundle and travel with its .duckdb file. This module holds
 * the optimistic in-memory copy — edits apply here instantly and are
 * written through, debounced while typing, flushed when the editor closes.
 * With an in-memory database they last exactly as long as it does, like
 * everything else in it.
 */
import { notesList, notesUpsert, notesDelete, type Note } from "$lib/api/notes";
import { db, ensureOpen } from "$lib/database.svelte";

export type { Note };

export const notes = $state<{ list: Note[]; error: string | null }>({
	list: [],
	error: null,
});

/** the quick-note editor: open/closed, and which note it is editing */
export const noteEditor = $state<{ open: boolean; id: number | null }>({
	open: false,
	id: null,
});

// --- persistence -----------------------------------------------------------
const WRITE_DELAY_MS = 400;
const pending = new Map<number, ReturnType<typeof setTimeout>>();

function fail(e: unknown) {
	notes.error = String(e);
}

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
		fail(e);
	}
}

function scheduleWrite(note: Note) {
	const prev = pending.get(note.createdAt);
	if (prev !== undefined) clearTimeout(prev);
	pending.set(
		note.createdAt,
		setTimeout(() => {
			pending.delete(note.createdAt);
			writeNow(note);
		}, WRITE_DELAY_MS),
	);
}

async function writeNow(note: Note) {
	try {
		await ensureOpen();
		await notesUpsert(note);
		notes.error = null;
	} catch (e) {
		fail(e);
	}
}

/** push every debounced write now (editor closing, app blur) */
export function flushNotes(): void {
	for (const [id, timer] of pending) {
		clearTimeout(timer);
		pending.delete(id);
		const note = notes.list.find((n) => n.createdAt === id);
		if (note) void writeNow(note);
	}
}

// --- editor state ------------------------------------------------------------
export function openQuickNote(id: number | null = null): void {
	noteEditor.id = id;
	noteEditor.open = true;
}
export function closeQuickNote(): void {
	flushNotes();
	noteEditor.open = false;
	noteEditor.id = null;
}
export function toggleQuickNote(): void {
	if (noteEditor.open) closeQuickNote();
	else openQuickNote(null);
}

// --- edits (optimistic, written through) -------------------------------------
export function getNote(id: number): Note | undefined {
	return notes.list.find((n) => n.createdAt === id);
}

/** create (id null) or update; returns the note's key (createdAt) */
export function upsertNote(id: number | null, text: string, route: string): number {
	const now = Date.now();
	if (id !== null) {
		const i = notes.list.findIndex((n) => n.createdAt === id);
		if (i >= 0) {
			const next = { ...notes.list[i], text, updatedAt: now };
			notes.list[i] = next;
			scheduleWrite(next);
			return id;
		}
	}
	// createdAt is the key: bump past the newest note if two land in one ms
	const newest = notes.list.reduce((m, n) => Math.max(m, n.createdAt), 0);
	const createdAt = Math.max(now, newest + 1);
	const fresh: Note = { createdAt, text, updatedAt: now, route };
	notes.list = [fresh, ...notes.list];
	scheduleWrite(fresh);
	return createdAt;
}

export function deleteNote(id: number): void {
	const timer = pending.get(id);
	if (timer !== undefined) {
		clearTimeout(timer);
		pending.delete(id);
	}
	notes.list = notes.list.filter((n) => n.createdAt !== id);
	notesDelete(id).catch(fail);
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
