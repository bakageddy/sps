/**
 * Shared index-fragmentation thresholds. MODULE-LEVEL persisted()
 * instances on purpose (same reason as stuckquery-settings): the toolbar
 * inputs, the per-capture table's row markers, the summary counts and the
 * cross-capture "flagged" query all read these two numbers, so they can
 * never disagree about what "flagged" means.
 *
 *   flagged  = pageCount >= minPages AND ratio >= minRatio
 *
 * Defaults follow SQL Server's maintenance guidance (rebuild at 30 %) with
 * a 200-page floor: fragmentation on something smaller costs nothing.
 */
import { persisted } from "$lib/persisted.svelte";

/** indices smaller than this (pages) are never flagged */
export const minPages = persisted("fragmentation-min-pages", 200);
/** fragmentation (0–100) at or above which an index is flagged */
export const minRatio = persisted("fragmentation-min-ratio", 30);
