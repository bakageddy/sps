<script lang="ts">
	/**
	 * Thread dumps as a sortable table — DumpList's interaction grammar
	 * (sticky header, click a column to sort, click a row to drill in), with
	 * the state distribution as the metrics: a dump with many BLOCKED
	 * threads is the one to open first.
	 */
	import type { ThreadDumpSummary } from "$lib/api/threaddump";
	import { formatTimestamp } from "$lib/format";

	interface Props {
		dumps: ThreadDumpSummary[];
		/** timestamp of the selected dump, or null */
		selected: number | null;
		onselect: (timestamp: number) => void;
	}

	let { dumps, selected, onselect }: Props = $props();

	type SortKey = "timestamp" | "threads" | "blocked" | "waiting" | "runnable";
	let sortKey = $state<SortKey>("timestamp");
	let sortDescending = $state(false);

	function sortBy(key: SortKey) {
		if (sortKey === key) {
			sortDescending = !sortDescending;
		} else {
			sortKey = key;
			// chronological reads oldest-first; counts read biggest-first
			sortDescending = key !== "timestamp";
		}
	}

	const visible = $derived(
		dumps.toSorted((a, b) =>
			sortDescending ? b[sortKey] - a[sortKey] : a[sortKey] - b[sortKey],
		),
	);

	const columns: { key: SortKey; label: string }[] = [
		{ key: "timestamp", label: "Dump" },
		{ key: "threads", label: "Threads" },
		{ key: "blocked", label: "Blocked" },
		{ key: "waiting", label: "Waiting" },
		{ key: "runnable", label: "Runnable" },
	];

	const dumpFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	// bring an externally-selected row (cross-analyzer link) into view
	let scroller = $state<HTMLDivElement>();
	$effect(() => {
		void selected;
		scroller
			?.querySelector("tr.selected")
			?.scrollIntoView({ block: "nearest" });
	});
</script>

<div class="wrap">
	<div class="scroller" bind:this={scroller}>
		<table>
			<thead>
				<tr>
					{#each columns as col (col.key)}
						<th>
							<button class="sort" onclick={() => sortBy(col.key)}>
								{col.label}
								{#if sortKey === col.key}
									<span class="arrow"
										>{sortDescending ? "▼" : "▲"}</span
									>
								{/if}
							</button>
						</th>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each visible as dump (dump.timestamp)}
					<tr
						class:selected={dump.timestamp === selected}
						onclick={() => onselect(dump.timestamp)}
					>
						<td class="mono when"
							>{formatTimestamp(dumpFormat, dump.timestamp)}</td
						>
						<td class="mono num">{dump.threads}</td>
						<td class="mono num" class:bad={dump.blocked > 0}
							>{dump.blocked}</td
						>
						<td class="mono num warn">{dump.waiting}</td>
						<td class="mono num">{dump.runnable}</td>
					</tr>
				{:else}
					<tr>
						<td colspan="5" class="empty">
							No thread dumps yet — parse a bundle with thread dumps
							from the Ingest page.
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
	.when {
		color: var(--fg-strong);
	}
	.mono {
		font-family: var(--font-mono);
	}
	.num {
		text-align: right;
	}
	.bad {
		color: var(--red);
		font-weight: 600;
	}
	.warn {
		color: var(--yellow);
	}
	.empty {
		text-align: center;
		color: var(--fg-muted);
		padding: 24px;
		cursor: default;
	}
</style>
