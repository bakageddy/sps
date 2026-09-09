/**
 * Shared stuck-query UI settings. A MODULE-LEVEL persisted() instance on
 * purpose: each persisted() call owns its own $state, so two components
 * calling persisted("stuckquery-slow-s", …) would NOT see each other's
 * changes — importing this single instance is what keeps the toolbar
 * control and every table in sync.
 */
import { persisted } from "$lib/persisted.svelte";

/** queries running longer than this (seconds) are flagged red */
export const slowThreshold = persisted("stuckquery-slow-s", 60);
