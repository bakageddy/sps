<script lang="ts">
  /**
   * Per-path rollup of stuck episodes — answers "one bad endpoint or
   * everything?" in a glance. Sorted worst-first by total stuck time
   * (fixed sort on purpose: this view is a ranking, not a browser).
   * Clicking a row drills down: the page filters episodes to that path.
   */
  import type { PathRollup } from "$lib/stuckthread";
  import { formatDuration } from "$lib/format";

  interface Props {
    rollups: PathRollup[];
    onpick: (path: string) => void;
  }

  let { rollups, onpick }: Props = $props();
</script>

<div class="table">
  <div class="head">
    <span class="col-path">Path</span>
    <span class="col-num">Episodes</span>
    <span class="col-num">Stuck</span>
    <span class="col-num">Max</span>
    <span class="col-num">Total</span>
  </div>

  <div class="rows">
    {#each rollups as r}
      <button class="row" onclick={() => onpick(r.path)} title="Filter episodes to {r.path}">
        <span class="col-path">{r.path}</span>
        <span class="col-num mono">{r.episodes}</span>
        <span class="col-num mono" class:bad={r.open > 0}>{r.open}</span>
        <span class="col-num mono">{formatDuration(r.maxMs)}</span>
        <span class="col-num mono total">{formatDuration(r.totalMs)}</span>
      </button>
    {:else}
      <p class="empty">No episodes to roll up.</p>
    {/each}
  </div>
</div>

<style>
  .table {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-size: 12.5px;
  }

  .head,
  .row {
    display: grid;
    grid-template-columns: minmax(150px, 1fr) 76px 60px 90px 100px;
    gap: 10px;
    align-items: center;
    padding: 0 10px;
  }

  .head {
    flex-shrink: 0;
    padding-top: 8px;
    padding-bottom: 8px;
    background: var(--bg-soft);
    font-weight: 600;
    color: var(--fg-strong);
    position: sticky;
    top: 0;
    z-index: 1;
  }

  .rows {
    overflow: auto;
    flex: 1;
  }

  .row {
    width: 100%;
    text-align: left;
    padding-top: 4px;
    padding-bottom: 4px;
    border-top: 1px solid var(--hairline);
    border-radius: 0;
    color: var(--fg);
  }
  .row:hover {
    background: var(--bg-hover);
  }

  .col-path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .col-num {
    text-align: right;
  }
  .mono {
    font-family: var(--font-mono);
    font-size: 12px;
  }
  .total {
    color: var(--yellow);
  }
  .bad {
    color: var(--red);
    font-weight: 600;
  }

  .empty {
    padding: 24px;
    text-align: center;
    color: var(--fg-muted);
  }
</style>
