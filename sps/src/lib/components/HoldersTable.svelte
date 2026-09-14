<script lang="ts">
	/**
	 * Hold episodes across dumps — the connection-dump equivalent of the
	 * long-runners table. A hold that survives from one dump to the next
	 * (dumps > 1) or grows past the threshold IS the incident; single-dump
	 * short holds are just a busy pool. Fixed ranking comes pre-sorted from
	 * the backend (maxDuration desc).
	 */
	import type { ConnDumpHolder } from "$lib/api/connectiondump";
	import { threadLabel } from "$lib/connectiondump";
	import { formatDuration, formatTimestamp } from "$lib/format";

	interface Props {
		rows: ConnDumpHolder[];
		/** held-for (ms) past which a row flags red */
		slowThresholdMs: number;
		/** jump to the dump this hold was last seen in */
		onjump?: (row: ConnDumpHolder) => void;
	}

	let { rows, slowThresholdMs, onjump }: Props = $props();

	const rowKey = (r: ConnDumpHolder) => `${r.id}:${r.startTime}`;

	let expanded = $state<string | null>(null);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
</script>

<div class="table">
	<div class="head">
		<span>Thread</span>
		<span class="col-num">Tid</span>
		<span class="col-num">Held up to</span>
		<span class="col-num">Dumps</span>
		<span>Acquired</span>
		<span>Owner</span>
	</div>

	<div class="rows">
		{#each rows as row (rowKey(row))}
			<button
				class="row"
				class:slow={row.maxDuration > slowThresholdMs}
				onclick={() =>
					(expanded = expanded === rowKey(row) ? null : rowKey(row))}
			>
				<span class="mono ellipsis" title={row.threadName}
					>{threadLabel(row.threadName)}</span
				>
				<span class="col-num mono">{row.id}</span>
				<span class="col-num mono">{formatDuration(row.maxDuration)}</span>
				<span class="col-num mono" class:multi={row.dumpCount > 1}
					>{row.dumpCount}</span
				>
				<span class="mono">{formatTimestamp(timeFormat, row.startTime)}</span
				>
				<span class="mono ellipsis muted">{row.appFrame ?? "—"}</span>
			</button>
			{#if expanded === rowKey(row)}
				<div class="expand">
					<dl>
						<dt>Thread</dt>
						<dd class="mono">{row.threadName}</dd>
						<dt>Acquired</dt>
						<dd class="mono">
							{formatTimestamp(timeFormat, row.startTime)}
						</dd>
						<dt>Last seen</dt>
						<dd class="mono">
							{formatTimestamp(timeFormat, row.lastSeen)}
						</dd>
						<dt>Owner</dt>
						<dd class="mono">{row.appFrame ?? "—"}</dd>
					</dl>
					{#if onjump}
						<button class="jump" onclick={() => onjump(row)}>
							view last dump →
						</button>
					{/if}
				</div>
			{/if}
		{:else}
			<p class="empty">No holds recorded — no trace dumps in range.</p>
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
		grid-template-columns: minmax(130px, 1fr) 52px 90px 56px 176px 1.5fr;
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
	.muted {
		color: var(--fg-muted);
	}
	.row.slow .muted {
		color: inherit;
	}
	.multi {
		font-weight: 700;
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

	.jump {
		margin-top: 8px;
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.jump:hover {
		background: var(--bg-hover);
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
