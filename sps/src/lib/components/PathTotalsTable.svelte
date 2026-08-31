<script lang="ts" module>
  /** One executable's rollup within the selected dump, for the ACTIVE
   * metric — the page's global CPU|Memory toggle drives which. */
  export interface PathTotal {
    /** executable path, falling back to process name when the log had none */
    label: string;
    /** true when label is a real path; false when it's a name fallback */
    byPath: boolean;
    value: number;
  }
</script>

<script lang="ts">
  /**
   * Per-path rollup of the selected dump (all postgres.exe rows → one row),
   * following the page's global metric toggle like every other pane.
   * Clicking a row plots that executable's aggregate series in the chart.
   */
  interface Props {
    totals: PathTotal[];
    /** header for the value column, e.g. "Σ CPU %" or "Σ Mem MB" */
    valueLabel: string;
    /** label of the plotted rollup row, or null */
    selected: string | null;
    onselect: (row: PathTotal) => void;
  }

  let { totals, valueLabel, selected, onselect }: Props = $props();

  type SortKey = "label" | "value";
  let sortKey = $state<SortKey>("value");
  let sortDescending = $state(true);

  function sortBy(key: SortKey) {
    if (sortKey === key) {
      sortDescending = !sortDescending;
    } else {
      sortKey = key;
      sortDescending = key !== "label";
    }
  }

  const visible = $derived(
    totals.toSorted((a, b) => {
      const va = a[sortKey];
      const vb = b[sortKey];
      const order = va < vb ? -1 : va > vb ? 1 : 0;
      return sortDescending ? -order : order;
    }),
  );
</script>

<div class="wrap">
  <div class="scroller">
    <table>
      <thead>
        <tr>
          <th>
            <button class="sort" onclick={() => sortBy("label")}>
              Path
              {#if sortKey === "label"}<span class="arrow">{sortDescending ? "▼" : "▲"}</span>{/if}
            </button>
          </th>
          <th class="value-col">
            <button class="sort" onclick={() => sortBy("value")}>
              {valueLabel}
              {#if sortKey === "value"}<span class="arrow">{sortDescending ? "▼" : "▲"}</span>{/if}
            </button>
          </th>
        </tr>
      </thead>
      <tbody>
        {#each visible as row (row.label)}
          <tr
            class:selected={selected === row.label}
            title={row.label}
            onclick={() => onselect(row)}
          >
            <td class="label">{row.label}</td>
            <td class="num">{row.value.toFixed(2)}</td>
          </tr>
        {:else}
          <tr>
            <td colspan="2" class="empty">Select a dump to see per-path totals.</td>
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
    /* fixed layout: column widths come from the header row, so a long
       path can never widen its column and shove others off-pane —
       overflow stays INSIDE the label cell where the ellipsis handles it */
    table-layout: fixed;
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
  th.value-col {
    width: 120px;
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

  .label {
    overflow: hidden;
    text-overflow: ellipsis;
    direction: rtl; /* ellipsis on the LEFT: the filename end of a path
                       matters more than the C:\ or /usr prefix */
    text-align: left;
  }

  .num {
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .empty {
    text-align: center;
    color: var(--fg-muted);
    padding: 24px;
  }
</style>
