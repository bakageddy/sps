<script lang="ts">
	/**
	 * Result grid for the SQL console: dynamic columns, virtualized rows
	 * (a 10k-row page costs the same as 30), horizontal scroll for wide
	 * results. Cells are text (the backend stringifies), NULL is rendered
	 * distinctly from the empty string — the difference matters in SQL.
	 */
	import type { SqlResult } from "$lib/api/sql";
	import { virtualWindow } from "$lib/virtual";
	import { copyText } from "$lib/clipboard";

	interface Props {
		result: SqlResult;
		/** row number of the first row (paging offset) */
		offset?: number;
	}

	let { result, offset = 0 }: Props = $props();

	const ROW = 28;
	const HEADER = 34;
	let viewport = $state(0);
	let scrollTop = $state(0);
	const win = $derived(
		virtualWindow(scrollTop - HEADER, viewport, ROW, result.rows.length),
	);
	const slice = $derived(result.rows.slice(win.start, win.end));

	// fixed column width: max-content would re-measure per virtualized
	// slice and make the columns jitter while scrolling
	const template = $derived(
		`56px repeat(${result.columns.length}, 200px)`,
	);

	let copied = $state<string | null>(null);
	async function copyCell(value: string | null, key: string) {
		if (value === null) return;
		if (await copyText(value)) {
			copied = key;
			setTimeout(() => (copied = null), 900);
		}
	}
</script>

<div
	class="grid"
	bind:clientHeight={viewport}
	onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
>
	<div class="sheet" style:min-width="calc(56px + {result.columns.length} * 200px)">
		<div class="head" style:grid-template-columns={template}>
			<span class="cell num">#</span>
			{#each result.columns as col, i (i)}
				<span class="cell mono" title={col}>{col}</span>
			{/each}
		</div>
		<div class="spacer" style:height="{win.padTop}px"></div>
		{#each slice as row, i}
			{@const n = win.start + i}
			<div class="row" style:grid-template-columns={template}>
				<span class="cell num mono">{offset + n + 1}</span>
				{#each row as value, c}
					<!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
					<span
						class="cell mono"
						class:null={value === null}
						class:copied={copied === `${n}:${c}`}
						title={value === null ? "NULL" : value}
						onclick={() => copyCell(value, `${n}:${c}`)}
						>{value === null ? "NULL" : value}</span
					>
				{/each}
			</div>
		{:else}
			<p class="empty">No rows.</p>
		{/each}
		<div class="spacer" style:height="{win.padBottom}px"></div>
	</div>
</div>

<style>
	.grid {
		height: 100%;
		overflow: auto;
		font-size: 12.5px;
	}
	.sheet {
		display: flex;
		flex-direction: column;
	}

	.head,
	.row {
		display: grid;
		column-gap: 0;
	}
	.head {
		position: sticky;
		top: 0;
		z-index: 1;
		height: 34px; /* = HEADER */
		background: var(--bg-soft);
	}
	.head .cell {
		line-height: 34px;
		font-weight: 600;
		color: var(--fg-strong);
	}

	.row {
		height: 28px; /* = ROW; virtualization needs it exact */
		border-top: 1px solid var(--hairline);
	}
	.row:hover {
		background: var(--bg-hover);
	}

	.cell {
		box-sizing: border-box;
		padding: 0 10px;
		line-height: 28px;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		border-right: 1px solid var(--hairline);
	}
	.row .cell {
		cursor: copy;
	}
	.cell.num {
		text-align: right;
		color: var(--fg-muted);
		background: var(--bg-soft);
		position: sticky;
		left: 0;
	}
	.cell.null {
		color: var(--fg-muted);
		font-style: italic;
	}
	.cell.copied {
		background: color-mix(in srgb, var(--green) 18%, transparent);
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
