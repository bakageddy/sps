<script lang="ts">
  /**
   * The database picker in the topbar.
   *
   * Note what this component does NOT own: the connection state. That lives
   * in $lib/database.svelte.ts so every page can react to it. This component
   * only owns the text being typed (`path`) — state that nobody else cares
   * about stays local, shared state goes in a module. Same rule as the
   * thread table's sort order.
   */
  import { db, open } from "$lib/database.svelte";

  let path = $state("");

  function onsubmit(event: SubmitEvent) {
    event.preventDefault();
    open(path);
  }

  // Show only the filename in the status chip; the full path is in `title`.
  const basename = (p: string) => p.split("/").at(-1) ?? p;
</script>

<form class="control" {onsubmit}>
  <input
    type="text"
    placeholder="database path (empty = in-memory)"
    bind:value={path}
    spellcheck="false"
    disabled={db.state.status === "opening"}
  />
  <button type="submit" disabled={db.state.status === "opening"}>
    {db.state.status === "opening" ? "Opening…" : "Open"}
  </button>

  {#if db.state.status === "open"}
    <span
      class="chip open"
      title={db.state.info.path ?? "in-memory database"}
    >
      ● {db.state.info.path ? basename(db.state.info.path) : "in-memory"}
    </span>
  {:else if db.state.status === "error"}
    <span class="chip error" title={db.state.message}>● failed</span>
  {:else if db.state.status === "closed"}
    <!-- Not a blocker: parsing lazily opens an in-memory db (ensureOpen) -->
    <span class="chip closed">○ in-memory on parse</span>
  {/if}
</form>

<style>
  /* Stacked for the sidebar footer; the grid puts button and status chip
     on one row under the full-width input. */
  .control {
    display: grid;
    grid-template-columns: auto 1fr;
    gap: 6px 8px;
    align-items: center;
  }

  input {
    grid-column: 1 / -1;
    width: 100%;
    padding: 3px 8px;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg);
    border: none;
    border-radius: var(--radius);
  }

  button {
    padding: 3px 10px;
    font-size: 12px;
    background: var(--bg-soft);
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
  }
  button:hover:not(:disabled) {
    background: var(--bg-hover);
  }
  button:disabled {
    opacity: 0.5;
  }

  .chip {
    font-size: 12px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    min-width: 0; /* allow the grid cell to shrink so ellipsis can kick in */
  }
  .chip.open {
    color: var(--green);
  }
  .chip.error {
    color: var(--red);
  }
  .chip.closed {
    color: var(--fg-muted);
  }
</style>
