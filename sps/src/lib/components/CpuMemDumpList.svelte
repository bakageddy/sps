<script lang="ts">
  /** CPUMemStatistics dump list — same locally-sorted table grammar as the
   * cpumonitoring DumpList, different columns. */
  import type { CpuMemDumpSummary } from "$lib/api/cpumemstats";

  interface Props {
    dumps: CpuMemDumpSummary[];
    selected: number | null;
    onselect: (timestamp: number) => void;
  }

  let { dumps, selected, onselect }: Props = $props();

  type SortKey = "timestamp" | "totalCpu" | "totalMemory";
  let sortKey = $state<SortKey>("timestamp");
  let sortDescending = $state(false);

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortDescending = !sortDescending;
    } else {
      sortKey = key;
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
    { key: "totalCpu", label: "CPU %" },
    { key: "totalMemory", label: "Mem MB" },
  ];

  const dumpFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "medium",
    timeStyle: "medium",
    hourCycle: "h23",
  });

  // Selection can arrive from outside (cross-analyzer link, chart click) —
  // bring the row into view. Runs after the DOM updated.
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
            <td class="mono num cpu">{dump.totalCpu.toFixed(1)}</td>
            <td class="mono num mem">{dump.totalMemory.toFixed(0)}</td>
          </tr>
        {:else}
          <tr>
            <td colspan="3" class="empty">
              No dumps yet — parse a CPUMemStatistics log from the Ingest page.
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
  .cpu {
    color: var(--yellow);
  }
  .mem {
    color: var(--aqua);
  }

  .empty {
    text-align: center;
    color: var(--fg-muted);
    padding: 24px;
    cursor: default;
  }
</style>
