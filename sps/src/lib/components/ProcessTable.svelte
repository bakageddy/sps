<script lang="ts" module>
  /** The table's VIEW type: one metric already chosen. The page derives
   * this from the API's unified ProcessRow (nullable cpu/memory) by
   * filtering on the non-null metric — DTO and view stay decoupled. */
  export interface UsageRow {
    pid: number;
    name: string;
    value: number;
    path: string | null;
  }
</script>

<script lang="ts">
  /**
   * One dump's process list (either the CPU or the Memory view — the
   * parent decides which rows and what the value column means via
   * `valueLabel`). Same grammar as ThreadTable: sortable, filterable,
   * row click = inspect.
   */
  interface Props {
    processes: UsageRow[];
    /** header for the value column, e.g. "CPU %" or "Mem MB" */
    valueLabel: string;
    /** pid of the inspected process, or null */
    selected: number | null;
    onselect: (pid: number) => void;
  }

  let { processes, valueLabel, selected, onselect }: Props = $props();

  type SortKey = "pid" | "name" | "value";
  let sortKey = $state<SortKey>("value");
  let sortDescending = $state(true);
  let filter = $state("");

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortDescending = !sortDescending;
    } else {
      sortKey = key;
      sortDescending = key !== "name";
    }
  }

  const visible = $derived.by(() => {
    const needle = filter.trim().toLowerCase();
    const filtered = needle
      ? processes.filter(
          (p) => p.name.toLowerCase().includes(needle) || String(p.pid).includes(needle),
        )
      : processes;

    return filtered.toSorted((a, b) => {
      const va = a[sortKey];
      const vb = b[sortKey];
      const order = va < vb ? -1 : va > vb ? 1 : 0;
      return sortDescending ? -order : order;
    });
  });
</script>

<div class="wrap">
  <input
    class="filter"
    type="search"
    placeholder="Filter by name or pid…"
    bind:value={filter}
  />

  <div class="scroller">
    <table>
      <thead>
        <tr>
          <th>
            <button class="sort" onclick={() => sortBy("name")}>
              Name
              {#if sortKey === "name"}<span class="arrow">{sortDescending ? "▼" : "▲"}</span>{/if}
            </button>
          </th>
          <th>
            <button class="sort" onclick={() => sortBy("pid")}>
              PID
              {#if sortKey === "pid"}<span class="arrow">{sortDescending ? "▼" : "▲"}</span>{/if}
            </button>
          </th>
          <th>
            <button class="sort" onclick={() => sortBy("value")}>
              {valueLabel}
              {#if sortKey === "value"}<span class="arrow">{sortDescending ? "▼" : "▲"}</span>{/if}
            </button>
          </th>
        </tr>
      </thead>
      <tbody>
        {#each visible as p (p.pid)}
          <!-- path goes in title: hover reveals it without a column that
               would dwarf every other one -->
          <tr
            class:selected={selected === p.pid}
            title={p.path ?? undefined}
            onclick={() => onselect(p.pid)}
          >
            <td class="name">{p.name}</td>
            <td class="num">{p.pid}</td>
            <td class="num">{p.value.toFixed(2)}</td>
          </tr>
        {:else}
          <tr>
            <td colspan="3" class="empty">
              {processes.length === 0 ? "Select a dump on the left." : "No match."}
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

  .name {
    max-width: 240px;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .empty {
    text-align: center;
    color: var(--fg-muted);
    padding: 24px;
    cursor: default;
  }
</style>
