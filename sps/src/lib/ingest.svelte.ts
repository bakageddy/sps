/**
 * App-wide ingestion state — now EVENT-DRIVEN: the backend emits ingest:*
 * events (see api/ingest-events.ts) and this module folds them into
 * reactive state. The frontend is the aggregation point: every event is an
 * additive delta, so folding is just summing — no ordering assumptions.
 *
 * `generation` still bumps once per completed run; pages watch it to
 * refetch, exactly as before — nothing outside this module changed.
 *
 * The listeners are registered once at module init and never torn down:
 * this module lives as long as the app, so the usual unlisten-on-cleanup
 * dance doesn't apply. (Events emitted before init would be lost, but
 * parsing can only be triggered after the UI is up.)
 */
import { listen } from "@tauri-apps/api/event";
import { parseLogs } from "$lib/api/logs";
import { ensureOpen } from "$lib/database.svelte";
import {
  IngestEvent,
  type IngestStarted,
  type IngestFile,
  type IngestError,
  type IngestFinished,
} from "$lib/api/ingest-events";

export interface KindCounts {
  entries: number;
  errors: number;
}

export type IngestState =
  | { status: "idle" }
  | { status: "parsing"; kinds: Record<string, KindCounts>; problems: string[] }
  | { status: "done"; kinds: Record<string, KindCounts>; problems: string[] }
  /** the command itself failed (bad path, no database) — nothing ran */
  | { status: "error"; message: string };

export const ingest = $state<{ state: IngestState; generation: number }>({
  state: { status: "idle" },
  generation: 0,
});

// Dropping several paths starts several concurrent runs; counts merge into
// one report and we're "done" only when the last run finishes.
let activeRuns = 0;

function beginParsing(): void {
  if (ingest.state.status !== "parsing") {
    ingest.state = { status: "parsing", kinds: {}, problems: [] };
  }
}

listen<IngestStarted>(IngestEvent.Started, () => {
  activeRuns += 1;
  beginParsing();
});

listen<IngestFile>(IngestEvent.File, (event) => {
  if (ingest.state.status !== "parsing") return; // stray/late event
  const { kind, entries, errors } = event.payload;
  // ??= : create the bucket on a kind's first file. $state proxies are
  // deeply reactive, so mutating the record updates the UI.
  const bucket = (ingest.state.kinds[kind] ??= { entries: 0, errors: 0 });
  bucket.entries += entries;
  bucket.errors += errors;
});

listen<IngestError>(IngestEvent.Error, (event) => {
  if (ingest.state.status !== "parsing") return;
  const { file, message } = event.payload;
  ingest.state.problems.push(file ? `${file}: ${message}` : message);
});

listen<IngestFinished>(IngestEvent.Finished, () => {
  activeRuns = Math.max(0, activeRuns - 1);
  if (activeRuns > 0) return;
  if (ingest.state.status === "parsing") {
    ingest.state = {
      status: "done",
      kinds: ingest.state.kinds,
      problems: ingest.state.problems,
    };
  }
  ingest.generation += 1; // pages refetch
});

export async function parse(path: string): Promise<void> {
  try {
    // Opening a database explicitly is optional — default to in-memory.
    await ensureOpen();
    beginParsing(); // optimistic; ingest:started confirms it
    await parseLogs(path); // resolves once parsing has STARTED
  } catch (e) {
    ingest.state = { status: "error", message: String(e) };
  }
}
