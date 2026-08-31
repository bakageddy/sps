/**
 * Read-query cache. Valid because the data is IMMUTABLE between two events:
 * a completed ingest (ingest.generation) and a store swap (db.epoch).
 * Those two numbers form the validity token; when it changes, the whole
 * cache clears — invalidation is structural, never managed per-entry.
 *
 * We cache the PROMISE, not the value:
 *  - concurrent identical requests dedupe to one IPC call,
 *  - a rejected promise evicts itself, so failures are never cached.
 *
 * Bounded LRU (Map preserves insertion order; a get() re-inserts, so the
 * first key is always the least recently used).
 *
 * Not for the paged dump lists — the paging factory already holds its rows
 * and resets on the same tokens; a second cache is a second thing to be
 * wrong.
 */
import { db } from "$lib/database.svelte";
import { ingest } from "$lib/ingest.svelte";

const MAX_ENTRIES = 100;

const cache = new Map<string, Promise<unknown>>();
let token = "";

export function cached<T>(key: string, fetch: () => Promise<T>): Promise<T> {
  const current = `${ingest.generation}|${db.epoch}`;
  if (current !== token) {
    cache.clear();
    token = current;
  }

  const hit = cache.get(key);
  if (hit !== undefined) {
    // LRU touch: move to the back of the insertion order
    cache.delete(key);
    cache.set(key, hit);
    return hit as Promise<T>;
  }

  const promise = fetch().catch((error) => {
    cache.delete(key); // never cache a failure
    throw error;
  });
  cache.set(key, promise);
  if (cache.size > MAX_ENTRIES) {
    const oldest = cache.keys().next().value;
    if (oldest !== undefined) cache.delete(oldest);
  }
  return promise;
}
