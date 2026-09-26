/**
 * Ingestion event contract — the backend EMITS these while parsing; the
 * frontend aggregates them (see lib/ingest.svelte.ts). This replaces the
 * old ParseReport return value: each parser thread reports for itself and
 * nothing in Rust has to join/merge results.
 *
 * Design rule the whole scheme rests on: every event is an ADDITIVE,
 * ORDER-INDEPENDENT fact. Parser threads interleave arbitrarily; the
 * frontend just folds. Never emit "the totals so far" — emit deltas.
 *
 * Granularity: one `file` event per parsed file (NOT per entry — events
 * are serialized IPC; thousands per second is waste).
 *
 * REQUIREMENTS: payloads must serialize to the shapes below (event names
 * exact); `ingest:file` fires once per file a parser has FINISHED with
 * (success or not — a failed file additionally gets an `ingest:error`);
 * `ingest:finished` fires exactly once per run, only after every parser
 * has finished.
 *
 * Rust side: `app.emit("ingest:file", IngestFile { .. })` — `emit` lives on
 * the `tauri::Emitter` trait in v2, so `use tauri::Emitter;` is required.
 * Payload structs derive `Serialize + Clone`.
 */

export const IngestEvent = {
	Started: "ingest:started",
	File: "ingest:file",
	Error: "ingest:error",
	Finished: "ingest:finished",
} as const;

export interface IngestStarted {
	path: string;
}

/** One file done — the frontend counts these per kind. No entry/error
 *  counts on purpose: parsers bulk-append and don't know row counts cheaply,
 *  and the analyzers show the real data moments later anyway. */
export interface IngestFile {
	/** log kind, e.g. "cpumonitoring", "cpumemstats" */
	kind: string;
	/** the file just finished */
	file: string;
}

/** A non-fatal problem (unreadable file, invalid UTF-8, append failure). */
export interface IngestError {
	file: string | null;
	message: string;
}

/** `ingest:finished` is emitted with `()` — its payload arrives as null. */
export type IngestFinished = null;
