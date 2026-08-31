<script lang="ts">
  /**
   * "JVM vs Machine" — the two analyzers' totals on ONE time axis.
   *
   * Top chart, two lines, same unit (%):
   *   - "Machine Σ CPU": machine-level total from CPUMemStatistics dumps
   *   - "JVM Σ CPU":     sum of JVM thread usage from cpumonitoring dumps
   * The gap between the lines is itself a finding: machine hot while JVM
   * threads aren't = the problem lives outside the JVM (or in GC/native).
   * Clicking a point drills into the analyzer that owns that series.
   *
   * Dump-level side-by-side inspection lives on the Linked Dumps page.
   */
  import { goto } from "$app/navigation";
  import { cpuDumps } from "$lib/api/cpumonitoring";
  import { cpuMemTotalCpu, cpuMemTotalMemory } from "$lib/api/cpumemstats";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import LineChart, { type LineSeries } from "$lib/components/LineChart.svelte";
  import { cached } from "$lib/query-cache";

  // Series ids double as routing targets for point clicks.
  const PROCESSES = 1;
  const THREADS = 2;

  let errorMessage = $state<string | null>(null);
  let cpuCombined = $state<LineSeries[]>([]);
  let memorySeries = $state<LineSeries[]>([]);
  let maximized = $state<"cpu" | "memory" | null>(null);

  async function refresh() {
    const [procCpu, threadDumps, memory] = await Promise.allSettled([
      cached("cpumem_total_cpu", cpuMemTotalCpu),
      cached("cpu_dumps", cpuDumps),
      cached("cpumem_total_memory", cpuMemTotalMemory),
    ]);

    const combined: LineSeries[] = [];
    if (procCpu.status === "fulfilled") {
      combined.push({ id: PROCESSES, label: "Machine Σ CPU", points: procCpu.value });
    } else {
      errorMessage = String(procCpu.reason);
    }
    if (threadDumps.status === "fulfilled") {
      combined.push({
        id: THREADS,
        label: "JVM Σ CPU",
        points: threadDumps.value.map((d) => ({
          timestamp: d.timestamp,
          value: d.totalCpu,
        })),
      });
    } else {
      errorMessage = String(threadDumps.reason);
    }
    cpuCombined = combined;

    if (memory.status === "fulfilled") {
      memorySeries = [{ id: PROCESSES, label: "Total Memory", points: memory.value }];
    } else {
      errorMessage = String(memory.reason);
    }
  }

  $effect(() => {
    if (db.state.status === "open") {
      errorMessage = null;
      refresh();
    } else {
      cpuCombined = [];
      memorySeries = [];
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refresh();
  });

  // Each line drills into the analyzer it came from, at that moment.
  function drillIn(id: number, timestamp: number) {
    const target = id === THREADS ? "/cpumonitoring" : "/cpumemstats";
    goto(`${target}?t=${timestamp}`);
  }
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
        series={cpuCombined}
        unit="%"
        emptyText="No dumps yet — parse logs from the Ingest page."
        onpointselect={drillIn}
        expanded
        ontoggleexpand={() => (maximized = null)}
      />
    {:else if maximized === "memory"}
      <LineChart
        series={memorySeries}
        unit=" MB"
        emptyText="No CPUMemStatistics dumps yet."
        onpointselect={drillIn}
        expanded
        ontoggleexpand={() => (maximized = null)}
      />
    {:else}
      <SplitPane direction="column" initial={0.6}>
        {#snippet a()}
          <LineChart
            series={cpuCombined}
            unit="%"
            emptyText="No dumps yet — parse logs from the Ingest page."
            onpointselect={drillIn}
            ontoggleexpand={() => (maximized = "cpu")}
          />
        {/snippet}
        {#snippet b()}
          <LineChart
            series={memorySeries}
            unit=" MB"
            emptyText="No CPUMemStatistics dumps yet."
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
