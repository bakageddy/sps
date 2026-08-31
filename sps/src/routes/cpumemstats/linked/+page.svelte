<script lang="ts">
  /**
   * Linked Dumps — both analyzers' dump lists side by side, linked by time.
   *
   * Clicking a dump in EITHER list also highlights (and loads) the closest
   * dump in the other list — but only when it lies within the link
   * threshold (seconds, user-tunable, persisted). Outside the threshold
   * the other side clears rather than pretending an association exists.
   *
   *   left:  machine dumps over JVM dumps (both fully visible)
   *   middle: linked JVM dump's threads + stack traces
   *   right:  linked machine dump's processes (CPU|Memory toggle)
   */
  import { MediaQuery } from "svelte/reactivity";
  import {
    cpuDumps,
    cpuDumpThreads,
    cpuStacktrace,
    type DumpSummary,
    type DumpThread,
  } from "$lib/api/cpumonitoring";
  import {
    cpuMemDumps,
    cpuMemCpuProcesses,
    cpuMemMemoryProcesses,
    type CpuMemDumpSummary,
    type ProcessUsage,
  } from "$lib/api/cpumemstats";
  import { db } from "$lib/database.svelte";
  import { ingest } from "$lib/ingest.svelte";
  import { cached } from "$lib/query-cache";
  import { nearestByTimestamp } from "$lib/nearest";
  import { persisted } from "$lib/persisted.svelte";
  import SplitPane from "$lib/components/SplitPane.svelte";
  import DumpList from "$lib/components/DumpList.svelte";
  import CpuMemDumpList from "$lib/components/CpuMemDumpList.svelte";
  import ThreadTable from "$lib/components/ThreadTable.svelte";
  import StackTracePanel, {
    type TraceState,
  } from "$lib/components/StackTracePanel.svelte";
  import ProcessTable, { type UsageRow } from "$lib/components/ProcessTable.svelte";

  const Metric = { Cpu: "cpu", Memory: "memory" } as const;
  type Metric = (typeof Metric)[keyof typeof Metric];

  // --- state -------------------------------------------------------------
  let errorMessage = $state<string | null>(null);

  let machineDumps = $state<CpuMemDumpSummary[]>([]);
  let jvmDumps = $state<DumpSummary[]>([]);
  let selectedMachine = $state<number | null>(null);
  let selectedJvm = $state<number | null>(null);

  /** max distance (seconds) for two dumps to count as "the same moment" */
  const threshold = persisted("linked-threshold-s", 30);

  let threads = $state<DumpThread[]>([]);
  let selectedTid = $state<number | null>(null);
  let trace = $state<TraceState>({ status: "idle" });

  let metric = $state<Metric>(Metric.Cpu);
  let cpuProcesses = $state<ProcessUsage[]>([]);
  let memoryProcesses = $state<ProcessUsage[]>([]);

  const portrait = new MediaQuery("(orientation: portrait)");

  const activeProcesses = $derived<UsageRow[]>(
    metric === Metric.Cpu ? cpuProcesses : memoryProcesses,
  );
  const valueLabel = $derived(metric === Metric.Cpu ? "CPU %" : "Mem MB");

  /** seconds between the two linked dumps, when both are selected */
  const linkDelta = $derived(
    selectedMachine !== null && selectedJvm !== null
      ? (selectedJvm - selectedMachine) / 1000
      : null,
  );

  // --- lifecycle -----------------------------------------------------------
  function resetSelection() {
    selectedMachine = null;
    selectedJvm = null;
    threads = [];
    selectedTid = null;
    trace = { status: "idle" };
    cpuProcesses = [];
    memoryProcesses = [];
  }

  async function refresh() {
    const [machine, jvm] = await Promise.allSettled([
      cached("cpumem_dumps", cpuMemDumps),
      cached("cpu_dumps", cpuDumps),
    ]);
    if (machine.status === "fulfilled") machineDumps = machine.value;
    else errorMessage = String(machine.reason);
    if (jvm.status === "fulfilled") jvmDumps = jvm.value;
    else errorMessage = String(jvm.reason);
  }

  $effect(() => {
    if (db.state.status === "open") {
      resetSelection();
      errorMessage = null;
      refresh();
    } else {
      machineDumps = [];
      jvmDumps = [];
      resetSelection();
    }
  });

  $effect(() => {
    if (ingest.generation === 0) return;
    refresh();
  });

  // --- linking ---------------------------------------------------------
  /** nearest timestamp in `rows`, but only within the threshold */
  function associate(target: number, rows: { timestamp: number }[]): number | null {
    const nearest = nearestByTimestamp(rows, target);
    if (nearest === null) return null;
    return Math.abs(nearest.timestamp - target) <= threshold.value * 1000
      ? nearest.timestamp
      : null;
  }

  function loadProcesses(timestamp: number | null) {
    if (timestamp === null) {
      cpuProcesses = [];
      memoryProcesses = [];
      return;
    }
    cached(`cpumem_cpu_processes:${timestamp}`, () => cpuMemCpuProcesses(timestamp))
      .then((rows) => void (cpuProcesses = rows))
      .catch((e) => void (errorMessage = String(e)));
    cached(`cpumem_mem_processes:${timestamp}`, () => cpuMemMemoryProcesses(timestamp))
      .then((rows) => void (memoryProcesses = rows))
      .catch((e) => void (errorMessage = String(e)));
  }

  function loadThreads(timestamp: number | null) {
    selectedTid = null;
    trace = { status: "idle" };
    if (timestamp === null) {
      threads = [];
      return;
    }
    cached(`cpu_dump_threads:${timestamp}`, () => cpuDumpThreads(timestamp))
      .then((rows) => void (threads = rows))
      .catch((e) => void (errorMessage = String(e)));
  }

  // Symmetric: clicking either list selects there and links the other side.
  function onselectmachine(timestamp: number) {
    selectedMachine = timestamp;
    loadProcesses(timestamp);
    selectedJvm = associate(timestamp, jvmDumps);
    loadThreads(selectedJvm);
  }

  function onselectjvm(timestamp: number) {
    selectedJvm = timestamp;
    loadThreads(timestamp);
    selectedMachine = associate(timestamp, machineDumps);
    loadProcesses(selectedMachine);
  }

  async function onselectthread(tid: number) {
    if (selectedJvm === null) return;
    const jvmTs = selectedJvm;
    selectedTid = tid;
    trace = { status: "loading", tid, timestamp: jvmTs };
    try {
      const frames = await cached(`cpu_stacktrace:${tid}:${jvmTs}`, () =>
        cpuStacktrace(tid, jvmTs),
      );
      trace = { status: "ready", tid, timestamp: jvmTs, frames };
    } catch (e) {
      trace = { status: "error", message: String(e) };
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

  <div class="toolbar">
    <label class="threshold">
      link threshold
      <input
        type="number"
        min="1"
        max="600"
        bind:value={threshold.value}
        title="Max seconds between dumps to link them (applies to the next click)"
      />
      s
    </label>
    {#if linkDelta !== null}
      <span class="delta mono">
        linked · Δ {linkDelta >= 0 ? "+" : ""}{linkDelta.toFixed(1)}s
      </span>
    {:else if selectedMachine !== null || selectedJvm !== null}
      <span class="delta muted">no dump within {threshold.value}s on the other side</span>
    {/if}
  </div>

  <div class="content">
    <SplitPane direction={portrait.current ? "column" : "row"} initial={0.28}>
      {#snippet a()}
        <SplitPane direction="column" initial={0.5}>
          {#snippet a()}
            <div class="pane-block">
              <h3>Machine dumps</h3>
              <CpuMemDumpList
                dumps={machineDumps}
                selected={selectedMachine}
                onselect={onselectmachine}
              />
            </div>
          {/snippet}
          {#snippet b()}
            <div class="pane-block">
              <h3>JVM dumps</h3>
              <DumpList dumps={jvmDumps} selected={selectedJvm} onselect={onselectjvm} />
            </div>
          {/snippet}
        </SplitPane>
      {/snippet}
      {#snippet b()}
        <SplitPane direction={portrait.current ? "column" : "row"} initial={0.5}>
          {#snippet a()}
            <div class="pane-block">
              <h3>JVM threads</h3>
              <SplitPane direction="column" initial={0.6}>
                {#snippet a()}
                  <ThreadTable {threads} selected={selectedTid} onselect={onselectthread} />
                {/snippet}
                {#snippet b()}
                  <StackTracePanel {trace} />
                {/snippet}
              </SplitPane>
            </div>
          {/snippet}
          {#snippet b()}
            <div class="pane-block">
              <h3>
                Processes
                <span class="toggle" role="group" aria-label="Metric">
                  <button
                    class:active={metric === Metric.Cpu}
                    onclick={() => (metric = Metric.Cpu)}
                  >CPU</button>
                  <button
                    class:active={metric === Metric.Memory}
                    onclick={() => (metric = Metric.Memory)}
                  >Memory</button>
                </span>
              </h3>
              <ProcessTable
                processes={activeProcesses}
                {valueLabel}
                selected={null}
                onselect={() => {}}
              />
            </div>
          {/snippet}
        </SplitPane>
      {/snippet}
    </SplitPane>
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
    align-items: center;
    gap: 16px;
    padding: 6px 12px;
    border-bottom: 1px solid var(--hairline);
    flex-shrink: 0;
  }

  .threshold {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 12px;
    color: var(--fg-muted);
  }
  .threshold input {
    width: 64px;
    padding: 3px 6px;
    font-family: var(--font-mono);
    font-size: 12px;
    background: var(--bg-hard);
    border: none;
    border-radius: var(--radius);
    text-align: right;
  }

  .delta {
    font-size: 12px;
    color: var(--green);
  }
  .delta.muted {
    color: var(--fg-muted);
  }
  .mono {
    font-family: var(--font-mono);
  }

  .content {
    flex: 1;
    min-height: 0;
  }

  .pane-block {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }
  .pane-block h3 {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 0;
    padding: 6px 10px;
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-muted);
    background: var(--bg-hard);
    flex-shrink: 0;
  }

  /* Segmented pill: an inset track with a filled active segment. */
  .toggle {
    display: flex;
    gap: 2px;
    margin-left: auto;
    padding: 2px;
    background: var(--bg-soft);
    border-radius: 999px;
  }
  .toggle button {
    padding: 2px 10px;
    font-size: 11px;
    border-radius: 999px;
    color: var(--fg-muted);
    text-transform: none;
    letter-spacing: normal;
  }
  .toggle button:hover {
    color: var(--fg);
  }
  .toggle button.active {
    background: var(--accent);
    color: var(--bg-hard);
    font-weight: 600;
  }
</style>
