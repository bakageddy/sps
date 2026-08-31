<script lang="ts">
  /**
   * CPUMemStatistics analyzer:
   *
   *   [ CPU | Memory ]  ← one GLOBAL metric toggle for the whole page
   *   dumps (left) → selected dump's processes (middle, active metric)
   *   → right column: clicked process's history chart (active metric)
   *                   over a per-path rollup of the selected dump
   *
   * Both metrics are fetched eagerly (dump click grabs both lists, process
   * click grabs both series), so flipping the toggle re-renders everything
   * locally with zero IPC.
   */
  import { MediaQuery } from "svelte/reactivity";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { nearestByTimestamp } from "$lib/nearest";
  import {
    cpuMemDumps,
    cpuMemCpuProcesses,
    cpuMemMemoryProcesses,
    cpuMemSeries,
    cpuMemPathSeries,
    type CpuMemDumpSummary,
    type ProcessUsage,
    type ProcessSeries,
  } from "$lib/api/cpumemstats";
  import { cached } from "$lib/query-cache";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import CpuMemDumpList from "$lib/components/CpuMemDumpList.svelte";
  import ProcessTable, { type UsageRow } from "$lib/components/ProcessTable.svelte";
  import PathTotalsTable, { type PathTotal } from "$lib/components/PathTotalsTable.svelte";
  import LineChart, { type LineSeries } from "$lib/components/LineChart.svelte";

  // Same const-object enum pattern as the theme's Mode/Colorscheme.
  const Metric = { Cpu: "cpu", Memory: "memory" } as const;
  type Metric = (typeof Metric)[keyof typeof Metric];

  // --- state -----------------------------------------------------------
  let errorMessage = $state<string | null>(null);

  let dumps = $state<CpuMemDumpSummary[]>([]);
  let selectedDump = $state<number | null>(null);
  let metric = $state<Metric>(Metric.Cpu);
  let cpuProcesses = $state<ProcessUsage[]>([]);
  let memoryProcesses = $state<ProcessUsage[]>([]);
  // The chart plots ONE thing at a time: a process (pid) or a rollup row
  // (path) — the two selections are mutually exclusive.
  let selectedPid = $state<number | null>(null);
  let selectedPath = $state<string | null>(null);
  let selectedLabel = $state<string>("");
  let series = $state<ProcessSeries | null>(null);
  let chartMaximized = $state(false);

  const portrait = new MediaQuery("(orientation: portrait)");

  // --- metric-driven views (all local; the toggle costs no IPC) ---------
  const activeProcesses = $derived<UsageRow[]>(
    metric === Metric.Cpu ? cpuProcesses : memoryProcesses,
  );
  const valueLabel = $derived(metric === Metric.Cpu ? "CPU %" : "Mem MB");
  const chartUnit = $derived(metric === Metric.Cpu ? "%" : " MB");

  const chartSeries = $derived<LineSeries[]>(
    series === null
      ? []
      : [
          {
            id: selectedPid ?? 0,
            label: selectedLabel,
            points: metric === Metric.Cpu ? series.cpu : series.memory,
          },
        ],
  );

  // Per-path rollup of the selected dump, following the global toggle:
  // group the ACTIVE metric's list by path (name when the log had none),
  // summing values.
  const pathTotals = $derived.by<PathTotal[]>(() => {
    const groups = new Map<string, { byPath: boolean; value: number }>();
    for (const row of activeProcesses) {
      const label = row.path ?? row.name;
      const group = groups.get(label);
      if (group) group.value += row.value;
      else groups.set(label, { byPath: row.path !== null, value: row.value });
    }
    return [...groups.entries()].map(([label, g]) => ({ label, ...g }));
  });

  // --- reactions to app-level state -------------------------------------
  function resetBelow(level: "dump" | "process") {
    if (level === "dump") {
      selectedDump = null;
      cpuProcesses = [];
      memoryProcesses = [];
    }
    selectedPid = null;
    selectedPath = null;
    selectedLabel = "";
    series = null;
  }

  async function refreshDumps() {
    dumps = await cached("cpumem_dumps", cpuMemDumps);
  }

  $effect(() => {
    if (db.state.status === "open") {
      resetBelow("dump");
      errorMessage = null;
      refreshDumps().catch((e) => (errorMessage = String(e)));
    } else {
      dumps = [];
      resetBelow("dump");
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refreshDumps().catch((e) => (errorMessage = String(e)));
  });

  // Cross-analyzer link (see cpumonitoring's twin): wait for dumps,
  // select the nearest, consume once.
  let linkConsumed = $state(false);
  $effect(() => {
    if (linkConsumed) return;
    const raw = page.url.searchParams.get("t");
    if (raw === null) {
      linkConsumed = true;
      return;
    }
    const target = Number(raw);
    if (!Number.isFinite(target) || dumps.length === 0) return; // wait for data
    const nearest = nearestByTimestamp(dumps, target);
    if (nearest === null) return;
    linkConsumed = true;
    onselectdump(nearest.timestamp);
  });

  // --- drill-down actions ------------------------------------------------
  // preserveChart: a chart-point click jumps dumps to inspect that moment —
  // clearing the plotted series would destroy the very thing being clicked.
  async function onselectdump(timestamp: number, preserveChart = false) {
    selectedDump = timestamp;
    if (!preserveChart) resetBelow("process");
    const [cpu, memory] = await Promise.allSettled([
      cached(`cpumem_cpu_processes:${timestamp}`, () => cpuMemCpuProcesses(timestamp)),
      cached(`cpumem_mem_processes:${timestamp}`, () => cpuMemMemoryProcesses(timestamp)),
    ]);
    if (cpu.status === "fulfilled") cpuProcesses = cpu.value;
    else errorMessage = String(cpu.reason);
    if (memory.status === "fulfilled") memoryProcesses = memory.value;
    else errorMessage = String(memory.reason);
  }

  async function onselectprocess(pid: number) {
    selectedPid = pid;
    selectedPath = null;
    const name =
      cpuProcesses.find((p) => p.pid === pid)?.name ??
      memoryProcesses.find((p) => p.pid === pid)?.name ??
      String(pid);
    selectedLabel = `${name} (${pid})`;
    try {
      series = await cached(`cpumem_series:${pid}`, () => cpuMemSeries(pid));
    } catch (e) {
      errorMessage = String(e);
    }
  }

  async function onselectpath(row: PathTotal) {
    selectedPath = row.label;
    selectedPid = null;
    // Legend shows the executable, not the whole C:\... prefix.
    selectedLabel = `Σ ${row.label.split(/[\\/]/).at(-1) ?? row.label}`;
    const path = row.byPath ? row.label : null;
    const name = row.byPath ? null : row.label;
    try {
      series = await cached(`cpumem_path_series:${row.byPath}:${row.label}`, () =>
        cpuMemPathSeries(path, name),
      );
    } catch (e) {
      errorMessage = String(e);
    }
  }
</script>

<div class="page">
  {#if errorMessage}
    <div class="error-bar" role="alert">
      {errorMessage}
      <button onclick={() => (errorMessage = null)} aria-label="Dismiss">✕</button>
    </div>
  {/if}

  <!-- Global metric toggle: drives the process table, the chart, and the
       units — everything except the dump list and rollup, which show both. -->
  <div class="toolbar">
    <div class="toggle" role="group" aria-label="Metric">
      <button
        class:active={metric === Metric.Cpu}
        onclick={() => (metric = Metric.Cpu)}
      >CPU</button>
      <button
        class:active={metric === Metric.Memory}
        onclick={() => (metric = Metric.Memory)}
      >Memory</button>
    </div>

    {#if selectedDump !== null}
      <!-- Cross-analyzer jump: the URL is the message bus between pages
           that share no state — cpumonitoring reads ?t= and selects its
           nearest dump. -->
      <button
        class="cross-link"
        onclick={() => goto(`/cpumonitoring?t=${selectedDump}`)}
        title="Open CPU Monitoring at the nearest thread dump"
      >threads at this time →</button>
    {/if}
  </div>

  <div class="content">
    {#if chartMaximized}
      <LineChart
        series={chartSeries}
        unit={chartUnit}
        emptyText="Click a process to plot its {metric} usage across dumps."
        onpointselect={(_id, timestamp) => {
          const nearest = nearestByTimestamp(dumps, timestamp);
          if (nearest) onselectdump(nearest.timestamp, true);
        }}
        expanded
        ontoggleexpand={() => (chartMaximized = false)}
      />
    {:else}
    <SplitPane direction={portrait.current ? "column" : "row"} initial={0.24}>
      {#snippet a()}
        <CpuMemDumpList {dumps} selected={selectedDump} onselect={onselectdump} />
      {/snippet}
      {#snippet b()}
        <SplitPane direction={portrait.current ? "column" : "row"} initial={0.5}>
          {#snippet a()}
            <ProcessTable
              processes={activeProcesses}
              {valueLabel}
              selected={selectedPid}
              onselect={onselectprocess}
            />
          {/snippet}
          {#snippet b()}
            <SplitPane direction="column" initial={0.5}>
              {#snippet a()}
                <LineChart
                  series={chartSeries}
                  unit={chartUnit}
                  emptyText="Click a process to plot its {metric} usage across dumps."
                  onpointselect={(_id, timestamp) => {
                    const nearest = nearestByTimestamp(dumps, timestamp);
                    if (nearest) onselectdump(nearest.timestamp, true);
                  }}
                  ontoggleexpand={() => (chartMaximized = true)}
                />
              {/snippet}
              {#snippet b()}
                <PathTotalsTable
                  totals={pathTotals}
                  valueLabel="Σ {valueLabel}"
                  selected={selectedPath}
                  onselect={onselectpath}
                />
              {/snippet}
            </SplitPane>
          {/snippet}
        </SplitPane>
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

  .toolbar {
    display: flex;
    padding: 8px 12px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  /* Segmented pill: an inset track with a filled active segment. */
  .toggle {
    display: flex;
    gap: 2px;
    padding: 3px;
    background: var(--bg-hard);
    border-radius: 999px;
  }
  .toggle button {
    padding: 3px 14px;
    font-size: 12px;
    border-radius: 999px;
    color: var(--fg-muted);
  }
  .toggle button:hover {
    color: var(--fg);
  }
  .toggle button.active {
    background: var(--accent);
    color: var(--bg-hard);
    font-weight: 600;
  }

  .cross-link {
    margin-left: auto;
    padding: 4px 12px;
    font-size: 12px;
    background: none;
    border: none;
    border-radius: var(--radius);
    color: var(--fg-muted);
    cursor: pointer;
  }
  .cross-link:hover {
    background: var(--bg-hover);
    color: var(--fg);
  }

  .content {
    flex: 1;
    min-height: 0;
  }
</style>
