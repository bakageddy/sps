<script lang="ts">
  /**
   * The dumps as a sortable table — same interaction grammar as
   * ThreadTable (sticky header, click a column to sort, click a row to
   * drill in). The full list is loaded (and cached — see lib/query-cache),
   * so sorting is local.
   */
  import type { DumpSummary } from "$lib/api/cpumonitoring";

  interface Props {
    dumps: DumpSummary[];
    /** timestamp of the selected dump, or null */
    selected: number | null;
    onselect: (timestamp: number) => void;
  }

  let { dumps, selected, onselect }: Props = $props();

  type SortKey = "timestamp" | "threads" | "totalCpu" | "maxCpu";
  let sortKey = $state<SortKey>("timestamp");
  let sortDescending = $state(false);

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortDescending = !sortDescending;
    } else {
      sortKey = key;
      // Chronological reads oldest-first; metrics read biggest-first.
      sortDescending = key !== "timestamp";
    }
  }

  const visible = $derived(
    dumps.toSorted((a, b) =>
      sortDescending ? b[sortKey] - a[sortKey] : a[sortKey] - b[sortKey],
    ),
  );

  const columns: { key: SortKey; label: string }[] = [
    { key: "timestamp", label: "Dump" },
    { key: "threads", label: "Threads" },
    { key: "totalCpu", label: "Σ %" },
    { key: "maxCpu", label: "Max %" },
  ];

  const dumpFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });

  // When selection changes from OUTSIDE (cross-analyzer link), the row may
  // be far down the list — bring it into view. $effect runs after the DOM
  // updated, so the .selected class is already applied when we query it.
  let scroller = $state<HTMLDivElement>();
  $effect(() => {
    void selected;
    scroller?.querySelector("tr.selected")?.scrollIntoView({ block: "nearest" });
  });
</script>

<div class="wrap">
  <div class="scroller" bind:this={scroller}>
    <table>
      <thead>
        <tr>
          {#each columns as col (col.key)}
            <th>
              <button class="sort" onclick={() => sortBy(col.key)}>
                {col.label}
                {#if sortKey === col.key}
                  <span class="arrow">{sortDescending ? "▼" : "▲"}</span>
                {/if}
              </button>
            </th>
          {/each}
        </tr>
      </thead>
      <tbody>
        {#each visible as dump (dump.timestamp)}
          <tr
            class:selected={dump.timestamp === selected}
            onclick={() => onselect(dump.timestamp)}
          >
            <td class="mono when">{dumpFormat.format(dump.timestamp)}</td>
            <td class="mono num">{dump.threads}</td>
            <td class="mono num total">{dump.totalCpu.toFixed(1)}</td>
            <td class="mono num max">{dump.maxCpu.toFixed(1)}</td>
          </tr>
        {:else}
          <tr>
            <td colspan="4" class="empty">
              No dumps yet — parse a cpumonitoring log from the Ingest page.
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  </div>
</div>

<style>
  .wrap {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .scroller {
    overflow: auto;
    flex: 1;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  thead {
    position: sticky;
    top: 0;
    background: var(--bg-soft);
  }

  th {
    text-align: left;
    padding: 0;
  }

  .sort {
    width: 100%;
    padding: 8px 10px;
    text-align: left;
    background: none;
    border: none;
    cursor: pointer;
    font-weight: 600;
    color: var(--fg-strong);
    white-space: nowrap;
  }
  .sort:hover {
    background: var(--bg-hover);
  }
  .arrow {
    font-size: 9px;
    color: var(--accent);
  }

  td {
    padding: 5px 10px;
    border-top: 1px solid var(--hairline);
    white-space: nowrap;
  }

  tbody tr {
    cursor: pointer;
  }
  tbody tr:hover {
    background: var(--bg-hover);
  }
  tr.selected {
    background: color-mix(in srgb, var(--accent) 12%, transparent);
  }

  .when {
    color: var(--fg-strong);
  }
  .mono {
    font-family: var(--font-mono);
  }
  .num {
    text-align: right;
  }
  .total {
    color: var(--aqua);
  }
  .max {
    color: var(--yellow);
  }

  .empty {
    text-align: center;
    color: var(--fg-muted);
    padding: 24px;
    cursor: default;
  }
</style>
