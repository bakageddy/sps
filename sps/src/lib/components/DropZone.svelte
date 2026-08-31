<script lang="ts">
  /**
   * Drag-and-drop target for log files/folders.
   *
   * Why not HTML5 drag events? A webview's DOM drop event only exposes File
   * objects, never filesystem PATHS — the browser sandbox hides them. Tauri
   * bypasses this: the Rust side watches native window drops and streams
   * them to us via onDragDropEvent, paths included. (Requires the window's
   * `dragDropEnabled: true`, which is the default.)
   *
   * The $effect here shows the subscribe/cleanup pattern: the effect body
   * runs on mount and RETURNS a teardown function that Svelte calls when the
   * component unmounts — without it, navigating away and back would stack up
   * duplicate listeners. This is the classic legitimate $effect use:
   * syncing with an external event source.
   */
  import { getCurrentWebview } from "@tauri-apps/api/webview";
  import { open } from "@tauri-apps/plugin-dialog";
  import { ingest, parse } from "$lib/ingest.svelte";

  let hovering = $state(false);
  let pickError = $state<string | null>(null);

  const parsing = $derived(ingest.state.status === "parsing");

  // The report renders DURING parsing (counts tick up live as file events
  // arrive) and stays after. One derived shape serves both states.
  const report = $derived.by(() => {
    if (ingest.state.status !== "parsing" && ingest.state.status !== "done") return null;
    return {
      kinds: Object.entries(ingest.state.kinds).toSorted(([a], [b]) => a.localeCompare(b)),
      problems: ingest.state.problems,
      settled: ingest.state.status === "done",
    };
  });

  /**
   * Native picker via tauri-plugin-dialog. `open()` resolves to the chosen
   * absolute path, or null if the user cancelled — cancellation is a normal
   * outcome, not an error, so it just does nothing.
   *
   * Rust side needed (dinesh):
   *   cargo add tauri-plugin-dialog
   *   .plugin(tauri_plugin_dialog::init()) in the Builder
   *   add "dialog:default" to src-tauri/capabilities/*.json permissions
   */
  async function pick() {
    pickError = null;
    try {
      const path = await open({
        directory: true,
        multiple: false,
        title: "Choose a log bundle folder",
      });
      if (path !== null) await parse(path);
    } catch (e) {
      pickError = String(e);
    }
  }

  $effect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === "enter" || event.payload.type === "over") {
        hovering = true;
      } else if (event.payload.type === "leave") {
        hovering = false;
      } else if (event.payload.type === "drop") {
        hovering = false;
        void parseAll(event.payload.paths);
      }
    });
    // onDragDropEvent is async and resolves to the unlisten function,
    // so the teardown has to await it before calling.
    return () => {
      unlisten.then((fn) => fn());
    };
  });

  async function parseAll(paths: string[]) {
    // Sequential on purpose: parses share the store and one report is
    // shown at a time; dropping a single folder is the main use case.
    for (const p of paths) {
      await parse(p);
    }
  }
</script>

<div class="zone" class:hovering class:parsing>
  {#if parsing}
    <p class="big">Parsing…</p>
    <p class="sub">results fill in below as files complete</p>
  {:else}
    <p class="big">Drop a log file or bundle folder here</p>
    <p class="sub">anything recognized gets parsed into the database</p>
    <div class="pickers">
      <button onclick={pick}>Choose folder…</button>
    </div>
    {#if pickError}
      <p class="pick-error" role="alert">{pickError}</p>
    {/if}
  {/if}
</div>

{#if report}
  <div class="report">
    {#each report.kinds as [kind, counts] (kind)}
      <div class="kind">
        <span class="mono">{kind}</span>
        <span class="count">{counts.entries} entries</span>
        {#if counts.errors > 0}
          <span class="rejected">{counts.errors} rejected</span>
        {/if}
      </div>
    {:else}
      {#if report.settled}
        <p class="none">Nothing recognized in that path.</p>
      {/if}
    {/each}
    {#each report.problems as problem, i (i)}
      <p class="error">{problem}</p>
    {/each}
  </div>
{:else if ingest.state.status === "error"}
  <p class="error" role="alert">{ingest.state.message}</p>
{/if}

<style>
  .zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 48px 24px;
    border: 2px dashed var(--border);
    border-radius: calc(var(--radius) * 2);
    text-align: center;
    transition: border-color 0.15s, background-color 0.15s;
  }
  .zone.hovering {
    background: var(--bg-hover);
    background: color-mix(in srgb, var(--accent) 8%, transparent);
  }
  .zone.parsing {
    border-style: solid;
  }

  .big {
    margin: 0;
    font-size: 15px;
    color: var(--fg-strong);
  }
  .sub {
    margin: 0;
    font-size: 12px;
    color: var(--fg-muted);
  }

  .pickers {
    display: flex;
    gap: 8px;
    margin-top: 14px;
  }
  .pickers button {
    padding: 5px 14px;
    font-size: 12px;
    background: var(--bg-soft);
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
  }
  .pickers button:hover {
    background: var(--bg-hover);
  }

  .pick-error {
    margin: 10px 0 0;
    font-size: 12px;
    color: var(--red);
  }

  .report {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 16px;
  }
  .kind {
    display: flex;
    gap: 12px;
    align-items: baseline;
    padding: 8px 12px;
    background: var(--bg-soft);
    border: none;
    border-radius: var(--radius);
    font-size: 13px;
  }
  .count {
    color: var(--green);
  }
  .rejected {
    color: var(--yellow);
    font-size: 12px;
  }
  .none {
    color: var(--fg-muted);
    font-size: 13px;
  }

  .error {
    margin-top: 16px;
    color: var(--red);
    font-size: 13px;
  }

  .mono {
    font-family: var(--font-mono);
  }
</style>
