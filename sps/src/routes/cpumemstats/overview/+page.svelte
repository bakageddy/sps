<script lang="ts">
  /**
   * CPUMemStatistics overview — the machine-level picture: overall total
   * CPU % and total memory across every dump, as two stacked charts.
   * No drill-down here; this page answers "when was the box under load?" —
   * then you take that timestamp to the main cpumemstats page.
   *
   * Same reactive skeleton as the analyzer pages: fetch on database open,
   * refetch on ingest generation.
   */
  import {
    cpuMemTotalCpu,
    cpuMemTotalMemory,
  } from "$lib/api/cpumemstats";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import { cached } from "$lib/query-cache";
  import { goto } from "$app/navigation";

  // Overview is the "find the incident window" view — clicking a point
  // drills into the analyzer at that moment via the same ?t= link.
  function drillIn(_id: number, timestamp: number) {
    goto(`/cpumemstats?t=${timestamp}`);
  }
  import LineChart, { type LineSeries } from "$lib/components/LineChart.svelte";

  let errorMessage = $state<string | null>(null);
  let cpuSeries = $state<LineSeries[]>([]);
  let memorySeries = $state<LineSeries[]>([]);
  let maximized = $state<"cpu" | "memory" | null>(null);

  async function refresh() {
    const [cpu, memory] = await Promise.allSettled([
      cached("cpumem_total_cpu", cpuMemTotalCpu),
      cached("cpumem_total_memory", cpuMemTotalMemory),
    ]);
    if (cpu.status === "fulfilled") {
      cpuSeries = [{ id: 0, label: "Total CPU", points: cpu.value }];
    } else {
      errorMessage = String(cpu.reason);
    }
    if (memory.status === "fulfilled") {
      memorySeries = [{ id: 0, label: "Total Memory", points: memory.value }];
    } else {
      errorMessage = String(memory.reason);
    }
  }

  $effect(() => {
    if (db.state.status === "open") {
      errorMessage = null;
      refresh();
    } else {
      cpuSeries = [];
      memorySeries = [];
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refresh();
  });
</script>

<div class="page">
  {#if errorMessage}
    <div class="error-bar" role="alert">
      {errorMessage}
      <button onclick={() => (errorMessage = null)} aria-label="Dismiss">✕</button>
    </div>
  {/if}

  <div class="content">
    {#if maximized === "cpu"}
      <LineChart
        series={cpuSeries}
        unit="%"
        emptyText="No CPUMemStatistics dumps yet — parse a log from the Ingest page."
        onpointselect={drillIn}
        expanded
        ontoggleexpand={() => (maximized = null)}
      />
    {:else if maximized === "memory"}
      <LineChart
        series={memorySeries}
        unit=" MB"
        emptyText="No CPUMemStatistics dumps yet — parse a log from the Ingest page."
        onpointselect={drillIn}
        expanded
        ontoggleexpand={() => (maximized = null)}
      />
    {:else}
      <SplitPane direction="column" initial={0.5}>
        {#snippet a()}
          <LineChart
            series={cpuSeries}
            unit="%"
            emptyText="No CPUMemStatistics dumps yet — parse a log from the Ingest page."
            onpointselect={drillIn}
            ontoggleexpand={() => (maximized = "cpu")}
          />
        {/snippet}
        {#snippet b()}
          <LineChart
            series={memorySeries}
            unit=" MB"
            emptyText="No CPUMemStatistics dumps yet — parse a log from the Ingest page."
            onpointselect={drillIn}
            ontoggleexpand={() => (maximized = "memory")}
          />
        {/snippet}
      </SplitPane>
    {/if}
  </div>
</div>

<style>
  .page {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .error-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 12px;
    font-size: 12px;
    color: var(--red);
    background: color-mix(in srgb, var(--red) 12%, transparent);
    border-bottom: 1px solid var(--hairline);
  }
  .error-bar button {
    background: none;
    border: none;
    cursor: pointer;
    color: inherit;
  }

  .content {
    flex: 1;
    min-height: 0;
    padding: 8px;
  }
</style>
