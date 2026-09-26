<script lang="ts">
	/**
	 * Traces of one connection dump: who holds a pooled connection right now,
	 * ranked by held-for (fixed sort — this is a ranking). Expanding a row
	 * shows the full acquisition stack.
	 */
	import type { ConnDumpTrace } from "$lib/api/connectiondump";
	import { appFrame, threadLabel } from "$lib/connectiondump";
	import { copyText } from "$lib/clipboard";
	import { formatDuration, formatTimestamp } from "$lib/format";

	interface Props {
		traces: ConnDumpTrace[];
		/** held-for (ms) past which a row flags red */
		slowThresholdMs: number;
	}

	let { traces, slowThresholdMs }: Props = $props();

	// identity: holder thread + acquisition instant (see api/connectiondump)
	const rowKey = (t: ConnDumpTrace) => `${t.id}:${t.startTime}`;

	let expanded = $state<string | null>(null);
	let copied = $state<string | null>(null);

	const sorted = $derived(traces.toSorted((a, b) => b.duration - a.duration));

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	async function copyStack(t: ConnDumpTrace) {
		if (await copyText(t.stackTrace.join("\n"))) {
			copied = rowKey(t);
			setTimeout(() => (copied = null), 1200);
		}
	}
</script>

<div class="table">
	<div class="head">
		<span>Thread</span>
		<span class="col-num">Tid</span>
		<span class="col-num">Held</span>
		<span>Since</span>
		<span>Owner</span>
	</div>

	<div class="rows">
		{#each sorted as t (rowKey(t))}
			<button
				class="row"
				class:slow={t.duration > slowThresholdMs}
				onclick={() =>
					(expanded = expanded === rowKey(t) ? null : rowKey(t))}
			>
				<span class="mono ellipsis" title={t.threadName}
					>{threadLabel(t.threadName)}</span
				>
				<span class="col-num mono">{t.id}</span>
				<span class="col-num mono">{formatDuration(t.duration)}</span>
				<span class="mono">{formatTimestamp(timeFormat, t.startTime)}</span>
				<span class="mono ellipsis muted"
					>{appFrame(t.stackTrace) ?? "—"}</span
				>
			</button>
			{#if expanded === rowKey(t)}
				<div class="expand">
					{#if t.invokedBy !== null}
						<div class="invoked mono">invoked by {t.invokedBy}</div>
					{/if}
					<pre>{t.stackTrace.join("\n")}</pre>
					<button class="copy" onclick={() => copyStack(t)}>
						{copied === rowKey(t) ? "copied" : "copy stack"}
					</button>
				</div>
			{/if}
		{:else}
			<p class="empty">No traces in this dump.</p>
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
		grid-template-columns: minmax(140px, 1.1fr) 52px 84px 176px 1.6fr;
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
		color: var(--alert);
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

	.expand {
		padding: 8px 12px;
		border-top: 1px solid var(--hairline);
		background: var(--bg-hard);
	}
	.invoked {
		margin-bottom: 6px;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.expand pre {
		/* explicit: <pre> otherwise falls back to the UA monospace */
		font-family: var(--font-mono);
		margin: 0;
		padding: 8px;
		background: var(--bg);
		border-radius: var(--radius);
		font-size: 11.5px;
		white-space: pre-wrap;
		overflow-wrap: anywhere;
		max-height: 280px;
		overflow: auto;
	}
	.copy {
		margin-top: 8px;
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.copy:hover {
		background: var(--bg-hover);
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
