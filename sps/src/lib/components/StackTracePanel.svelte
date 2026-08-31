<script lang="ts">
  /**
   * Shows the stack trace of one clicked sample.
   *
   * This component models async data the standard way: a DISCRIMINATED UNION
   * instead of separate `loading` / `error` / `data` booleans. With three
   * independent flags you can represent impossible states (loading AND error);
   * with a union, TypeScript narrows the fields you may touch in each branch.
   * The parent owns the fetch; we just render whatever state it hands us.
   */
  import type { StackFrame } from "$lib/api/cpumonitoring";

  export type TraceState =
    | { status: "idle" }
    | { status: "loading"; tid: number; timestamp: number }
    | { status: "error"; message: string }
    // frames: null = the sample was recorded without a trace (the backend's
    // Option::None), distinct from "not loaded yet" which is the states above.
    | { status: "ready"; tid: number; timestamp: number; frames: StackFrame[] | null };

  interface Props {
    trace: TraceState;
  }

  let { trace }: Props = $props();

  const timeFormat = new Intl.DateTimeFormat(undefined, {
    dateStyle: "short",
    timeStyle: "medium",
    hourCycle: "h23", // 24-hour clock
  });
</script>

<section class="panel">
  <header>
    <h2>Stack trace</h2>
    {#if trace.status === "ready" || trace.status === "loading"}
      <span class="context mono">
        tid {trace.tid} @ {timeFormat.format(trace.timestamp)}
      </span>
    {/if}
  </header>

  {#if trace.status === "idle"}
    <p class="hint">Click a point on the chart to inspect its stack trace.</p>
  {:else if trace.status === "loading"}
    <p class="hint">Loading…</p>
  {:else if trace.status === "error"}
    <p class="error">{trace.message}</p>
  {:else if trace.frames === null}
    <p class="hint">This sample was recorded without a stack trace.</p>
  {:else}
    <!-- Unkeyed each on purpose: frames have no identity of their own and
         the whole list is replaced wholesale on every fetch. (A key that
         isn't unique — like an undefined field — makes Svelte throw at
         render time: each_key_duplicate.) -->
    <ol class="frames">
      {#each trace.frames as frame}
        <li>
          <span class="method">{frame.method}</span>
          <span class="source">({frame.source})</span>
        </li>
      {/each}
    </ol>
  {/if}
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 12px;
    padding: 8px 12px;
    background: var(--bg-soft);
    border-bottom: 1px solid var(--hairline);
  }

  h2 {
    margin: 0;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-muted);
  }

  .context {
    font-size: 12px;
    color: var(--fg-muted);
  }

  .hint,
  .error {
    margin: 0;
    padding: 16px 12px;
    color: var(--fg-muted);
  }
  .error {
    color: var(--red);
  }

  .frames {
    /* semantically still an ordered list (frame order = the trace), just
       rendered without markers */
    list-style: none;
    margin: 0;
    padding: 8px 12px;
    /* long frames scroll horizontally inside the panel instead of wrapping */
    overflow: auto;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.7;
    text-align: left;
  }
  .frames li {
    white-space: nowrap;
  }

  .method {
    color: var(--fg);
  }
  .source {
    color: var(--fg-muted);
  }
  .mono {
    font-family: var(--font-mono);
  }
</style>
