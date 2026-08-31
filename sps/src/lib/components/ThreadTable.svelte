<script lang="ts">
  /**
   * The threads of ONE dump: sortable, filterable, one row selectable.
   * Clicking a row is "inspect this thread" — the page fetches its stack
   * trace (for this dump) and its usage history (for the chart).
   *
   * Selection lives in the parent; sort/filter affect nobody else, so they
   * stay local $state ("state up, events down").
   */
  import type { DumpThread } from "$lib/api/cpumonitoring";

  interface Props {
    threads: DumpThread[];
    /** tid of the inspected thread, or null */
    selected: number | null;
    onselect: (tid: number) => void;
  }

  let { threads, selected, onselect }: Props = $props();

  type SortKey = "tid" | "name" | "state" | "cpu";
  let sortKey = $state<SortKey>("cpu");
  let sortDescending = $state(true);
  let filter = $state("");

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortDescending = !sortDescending; // second click flips direction
    } else {
      sortKey = key;
      sortDescending = key !== "name" && key !== "state";
    }
  }

  const visible = $derived.by(() => {
    const needle = filter.trim().toLowerCase();
    const filtered = needle
      ? threads.filter(
          (t) =>
            (t.name ?? "").toLowerCase().includes(needle) ||
            String(t.tid).includes(needle),
        )
      : threads;

    // toSorted() = sorted copy; never mutate props, the parent owns them.
    return filtered.toSorted((a, b) => {
      const va = a[sortKey] ?? "";
      const vb = b[sortKey] ?? "";
      const order = va < vb ? -1 : va > vb ? 1 : 0;
      return sortDescending ? -order : order;
    });
  });

  const columns: { key: SortKey; label: string }[] = [
    { key: "tid", label: "TID" },
    { key: "name", label: "Name" },
    { key: "state", label: "State" },
    { key: "cpu", label: "CPU %" },
  ];

  const stateColor: Record<string, string> = {
    RUNNABLE: "var(--green)",
    BLOCKED: "var(--red)",
    WAITING: "var(--blue)",
    TIMED_WAITING: "var(--yellow)",
    NEW: "var(--aqua)",
    TERMINATED: "var(--fg-muted)",
  };
</script>

<div class="wrap">
  <input
    class="filter"
    type="search"
    placeholder="Filter by name or tid…"
    bind:value={filter}
  />

  <div class="scroller">
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
        {#each visible as t (t.tid)}
          <tr
            class:selected={selected === t.tid}
            onclick={() => onselect(t.tid)}
          >
            <td>{t.tid}</td>
            <td class="name" title={t.name ?? undefined}>{t.name ?? "—"}</td>
            <td>
              <span class="state" style:color={stateColor[t.state] ?? "var(--fg)"}>
                {t.state}
              </span>
            </td>
            <td class="num">{t.cpu.toFixed(2)}</td>
          </tr>
        {:else}
          <!-- {:else} on an {#each}: rendered when the list is empty -->
          <tr>
            <td colspan="4" class="empty">
              {threads.length === 0 ? "Select a dump on the left." : "No match."}
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

  .filter {
    margin: 8px;
    padding: 6px 10px;
    background: var(--bg-hard);
    border: none;
    border-radius: var(--radius);
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
    position: sticky; /* header stays visible while the body scrolls */
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

  .name {
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .num {
    text-align: right;
    /* tabular-nums: every digit gets the same width, so numbers align in
       columns even in a proportional font — the mono look without mono */
    font-variant-numeric: tabular-nums;
  }
  .state {
    font-size: 12px;
    font-weight: 600;
  }
  .empty {
    text-align: center;
    color: var(--fg-muted);
    padding: 24px;
    cursor: default;
  }
</style>
