<script lang="ts">
	/**
	 * Hold episodes across dumps — the connection-dump equivalent of the
	 * long-runners table. The backend only returns holds seen in more than
	 * one dump (dumpCount > 1 guaranteed), so every row here already
	 * survived at least one dump interval. The owner frame is derived here
	 * (see the ConnDumpHolder contract note); rows are sorted by duration
	 * descending locally.
	 */
	import type { ConnDumpHolder } from "$lib/api/connectiondump";
	import { appFrame } from "$lib/connectiondump";
	import { formatDuration, formatTimestamp } from "$lib/format";

	interface Props {
		rows: ConnDumpHolder[];
		/** held-for (ms) past which a row flags red */
		slowThresholdMs: number;
	}

	let { rows, slowThresholdMs }: Props = $props();

	const rowKey = (r: ConnDumpHolder) => `${r.id}:${r.startTime}`;

	const sorted = $derived([...rows].sort((a, b) => b.duration - a.duration));

	let expanded = $state<string | null>(null);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
</script>

<div class="table">
	<div class="head">
		<span>Owner</span>
		<span class="col-num">Tid</span>
		<span class="col-num">Held for</span>
		<span class="col-num">Dumps</span>
		<span>Acquired</span>
	</div>

	<div class="rows">
		{#each sorted as row (rowKey(row))}
			<button
				class="row"
				class:slow={row.duration > slowThresholdMs}
				onclick={() =>
					(expanded = expanded === rowKey(row) ? null : rowKey(row))}
			>
				<span class="mono ellipsis"
					>{appFrame(row.stackTrace) ?? "—"}</span
				>
				<span class="col-num mono">{row.id}</span>
				<span class="col-num mono">{formatDuration(row.duration)}</span>
				<span class="col-num mono">{row.dumpCount}</span>
				<span class="mono">{formatTimestamp(timeFormat, row.startTime)}</span
				>
			</button>
			{#if expanded === rowKey(row)}
				<div class="expand">
					<dl>
						<dt>Acquired</dt>
						<dd class="mono">
							{formatTimestamp(timeFormat, row.startTime)}
						</dd>
						<dt>Owner</dt>
						<dd class="mono">{appFrame(row.stackTrace) ?? "—"}</dd>
					</dl>
					<ol class="stack">
						{#each row.stackTrace as frame}
							<li class="mono">{frame}</li>
						{/each}
					</ol>
				</div>
			{/if}
		{:else}
			<p class="empty">
				No continuing holds — nothing survived across dumps in range.
			</p>
		{/each}
	</div>
</div>

<style>
	.table {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		font-size: 12.5px;
	}

	.head,
	.row {
		display: grid;
		grid-template-columns: minmax(200px, 1fr) 52px 90px 56px 176px;
		gap: 10px;
		align-items: center;
		padding: 0 10px;
	}

	.head {
		flex-shrink: 0;
		padding-top: 8px;
		padding-bottom: 8px;
		background: var(--bg-soft);
		font-weight: 600;
		color: var(--fg-strong);
		position: sticky;
		top: 0;
		z-index: 1;
	}

	.rows {
		overflow: auto;
		flex: 1;
	}

	.row {
		width: 100%;
		text-align: left;
		padding-top: 4px;
		padding-bottom: 4px;
		border-top: 1px solid var(--hairline);
		border-radius: 0;
		color: var(--fg);
	}
	.row:hover {
		background: var(--bg-hover);
	}
	.row.slow {
		color: var(--red);
	}

	.col-num {
		text-align: right;
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
	.ellipsis {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.expand {
		padding: 8px 12px;
		border-top: 1px solid var(--hairline);
		background: var(--bg-hard);
	}
	.expand dl {
		display: grid;
		grid-template-columns: auto 1fr;
		gap: 3px 12px;
		margin: 0;
		font-size: 12px;
	}
	.expand dt {
		color: var(--fg-muted);
	}
	.expand dd {
		margin: 0;
		overflow-wrap: anywhere;
	}

	.stack {
		margin: 8px 0 0;
		padding: 6px 0 6px 28px;
		max-height: 240px;
		overflow: auto;
		border-top: 1px solid var(--hairline);
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.stack li {
		overflow-wrap: anywhere;
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
