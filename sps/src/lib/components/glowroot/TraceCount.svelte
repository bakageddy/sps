<script lang="ts">
	/**
	 * "How many traces match?" — asks the collector with the current form
	 * values before anything heavy starts, so a 7-day window on a busy
	 * Central is not a surprise.
	 */
	import { count, form, session } from "$lib/glowroot.svelte";

	const num = new Intl.NumberFormat();

	/** slow and error listings can contain the same trace, so the sum is an upper bound */
	const total = $derived(session.counts?.reduce((n, c) => n + c.slow + c.error, 0) ?? null);
	const cap = $derived(Math.max(1, Math.floor(Number(form.maxTraces.value)) || 1));
</script>

<div class="count">
	<button type="button" onclick={() => void count()} disabled={session.busy === "count" || session.layout === null}>
		<!-- both labels share one grid cell so the button keeps the width of the longer one -->
		<span class="label" class:off={session.busy === "count"}>Count matching traces</span>
		<span class="label" class:off={session.busy !== "count"}>Counting…</span>
	</button>
	{#if session.counts}
		<table>
			<thead>
				<tr><th>Type</th><th class="n">Slow</th><th class="n">Error</th></tr>
			</thead>
			<tbody>
				{#each session.counts as c (c.transactionType)}
					<tr>
						<td>{c.transactionType}</td>
						<td class="n mono">{num.format(c.slow)}</td>
						<td class="n mono">{num.format(c.error)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
		{#if total !== null}
			<span class="summary">
				up to <b class="mono">{num.format(Math.min(total, cap))}</b> to fetch
				{#if total > cap}<span class="warn">(cap {num.format(cap)} of {num.format(total)})</span>{/if}
				<span class="muted">· slow and error can overlap</span>
			</span>
		{/if}
	{/if}
	{#if session.countError}<span class="error" role="alert">{session.countError}</span>{/if}
</div>

<style>
	.mono {
		font-family: var(--font-mono);
	}
	.muted {
		color: var(--fg-muted);
	}
	.warn {
		color: var(--alert);
	}
	.error {
		color: var(--red);
		font-size: 12px;
		white-space: pre-wrap;
	}
	.count {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		gap: 12px 16px;
	}
	button {
		display: inline-grid;
		padding: 6px 14px;
		font-size: 12px;
		background: var(--bg-hard);
	}
	.label {
		grid-area: 1 / 1;
		white-space: nowrap;
	}
	.label.off {
		visibility: hidden;
	}
	button:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	table {
		border-collapse: collapse;
		font-size: 12px;
	}
	th,
	td {
		padding: 3px 12px 3px 0;
		text-align: left;
	}
	th {
		color: var(--fg-muted);
		font-weight: 400;
	}
	.n {
		text-align: right;
	}
	.summary {
		align-self: center;
		font-size: 12px;
	}
</style>
