<script lang="ts">
	/**
	 * Step 4: start, watch, cancel. The numbers come from glowroot:* events
	 * folded in lib/glowroot.svelte.ts; rate and ETA are derived here from the
	 * last ~15 s of progress samples so a stall shows up as a falling rate
	 * instead of a frozen estimate.
	 */
	import {
		cancel,
		etaSeconds,
		rate,
		ready,
		run,
		running,
		session,
		start,
	} from "$lib/glowroot.svelte";
	import { formatDuration } from "$lib/format";

	const num = new Intl.NumberFormat();

	// elapsed ticks on its own clock; the events only arrive a few times a second
	let now = $state(Date.now());
	$effect(() => {
		if (!running()) return;
		now = Date.now();
		const id = setInterval(() => (now = Date.now()), 500);
		return () => clearInterval(id);
	});

	const elapsedMs = $derived(
		run.result ? run.result.elapsedMs : run.startedAt === null ? 0 : Math.max(0, now - run.startedAt),
	);
	const pct = $derived(run.total > 0 ? Math.min(100, (run.fetched / run.total) * 100) : 0);
	const perSecond = $derived(rate());
	const eta = $derived(etaSeconds());

	const phaseLabel: Record<typeof run.phase, string> = {
		idle: "",
		starting: "Starting…",
		listing: "Listing traces",
		fetching: "Fetching traces",
		finished: "",
	};

	const canStart = $derived(ready() && session.layout !== null && !running());
</script>

<div class="actions">
	{#if running()}
		<button type="button" class="danger" onclick={() => void cancel()} disabled={run.id === null}>
			Cancel
		</button>
	{:else}
		<button type="button" class="primary" onclick={() => void start()} disabled={!canStart}>
			{run.phase === "finished" ? "Run again" : "Start ingest"}
		</button>
	{/if}
	{#if run.startError}<span class="error" role="alert">{run.startError}</span>{/if}
</div>

{#if running()}
	<div class="panel">
		<div class="phase">
			<span>{phaseLabel[run.phase]}</span>
			{#if run.phase === "listing"}
				<span class="mono dim">{num.format(run.found)} found so far</span>
			{:else if run.phase === "fetching"}
				<span class="mono dim">
					{num.format(run.fetched)} / {num.format(run.total)}
				</span>
			{/if}
			<span class="spacer"></span>
			<span class="mono dim">{formatDuration(Math.round(elapsedMs))}</span>
		</div>

		<div
			class="track"
			role="progressbar"
			aria-label="Ingest progress"
			aria-valuemin="0"
			aria-valuemax={run.total || undefined}
			aria-valuenow={run.phase === "fetching" ? run.fetched : undefined}
		>
			<div class="fill" class:indeterminate={run.phase !== "fetching"} style:width="{run.phase === 'fetching' ? pct : 100}%"></div>
		</div>

		<dl class="stats">
			<div><dt>stored</dt><dd class="mono">{num.format(run.stored)}</dd></div>
			<div><dt>failed</dt><dd class="mono" class:bad={run.failed > 0}>{num.format(run.failed)}</dd></div>
			<div><dt>in flight</dt><dd class="mono">{run.inFlight}</dd></div>
			<div><dt>rate</dt><dd class="mono">{perSecond === null ? "—" : `${perSecond.toFixed(1)} /s`}</dd></div>
			<div><dt>left</dt><dd class="mono">{eta === null ? "—" : formatDuration(Math.round(eta * 1000))}</dd></div>
			{#if run.skippedExisting > 0}
				<div><dt>skipped</dt><dd class="mono">{num.format(run.skippedExisting)}</dd></div>
			{/if}
		</dl>
		{#if run.capped}
			<p class="note">Capped: the newest {num.format(run.total)} traces are being fetched.</p>
		{/if}
	</div>
{/if}

{#if run.phase === "finished" && run.result}
	{@const r = run.result}
	<div class="panel result {r.outcome}">
		<div class="phase">
			<span class="outcome">{r.outcome}</span>
			<span class="spacer"></span>
			<span class="mono dim">{formatDuration(Math.round(r.elapsedMs))}</span>
		</div>
		<dl class="stats">
			<div><dt>stored</dt><dd class="mono">{num.format(r.stored)}</dd></div>
			<div><dt>failed</dt><dd class="mono" class:bad={r.failed > 0}>{num.format(r.failed)}</dd></div>
			<div><dt>planned</dt><dd class="mono">{num.format(r.total)}</dd></div>
		</dl>
		{#if r.message}<p class="error">{r.message}</p>{/if}
		{#if r.stored > 0}
			<p class="note">
				Traces are in the database. Look at them in the <a href="/sql">SQL Console</a> — a trace browser is next.
			</p>
		{/if}
	</div>
{/if}

{#if run.problems.length > 0}
	<details class="problems">
		<summary>
			{run.problems.length}{run.problems.length >= 200 ? "+" : ""} problem{run.problems.length === 1 ? "" : "s"}
		</summary>
		<ul>
			{#each run.problems as p, i (i)}
				<li>
					{#if p.traceId}<span class="mono dim">{p.traceId}</span>{/if}
					{p.message}
				</li>
			{/each}
		</ul>
	</details>
{/if}

<style>
	.mono {
		font-family: var(--font-mono);
	}
	.dim {
		color: var(--fg-muted);
	}
	.actions {
		display: flex;
		align-items: center;
		gap: 12px;
		flex-wrap: wrap;
	}
	button {
		padding: 7px 18px;
		font-size: 12px;
		font-weight: 700;
	}
	button:disabled {
		opacity: 0.45;
		cursor: default;
	}
	.primary {
		background: var(--accent);
		color: var(--bg);
	}
	.primary:hover:not(:disabled) {
		opacity: 0.85;
	}
	.danger {
		background: color-mix(in srgb, var(--red) 25%, var(--bg-hard));
		color: var(--fg-strong);
	}
	.danger:hover:not(:disabled) {
		background: color-mix(in srgb, var(--red) 40%, var(--bg-hard));
	}
	.error {
		color: var(--red);
		font-size: 12px;
		white-space: pre-wrap;
		margin: 0;
	}

	.panel {
		margin-top: 14px;
		padding: 12px 14px;
		background: var(--bg-soft);
		border-radius: var(--radius);
	}
	.phase {
		display: flex;
		align-items: baseline;
		gap: 12px;
		font-size: 13px;
		color: var(--fg-strong);
	}
	.spacer {
		flex: 1;
	}
	.track {
		height: 8px;
		margin: 10px 0 12px;
		background: var(--bg-hard);
		border-radius: 99px;
		overflow: hidden;
	}
	.fill {
		height: 100%;
		background: var(--accent);
		border-radius: 99px;
		transition: width 0.25s ease;
	}
	.fill.indeterminate {
		background: repeating-linear-gradient(
			-45deg,
			var(--accent) 0 10px,
			color-mix(in srgb, var(--accent) 55%, var(--bg-hard)) 10px 20px
		);
		background-size: 28px 28px;
		animation: slide 0.8s linear infinite;
	}
	@keyframes slide {
		to {
			background-position: 28px 0;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.fill.indeterminate {
			animation: none;
		}
		.fill {
			transition: none;
		}
	}
	.stats {
		display: flex;
		flex-wrap: wrap;
		gap: 8px 24px;
		margin: 0;
	}
	.stats div {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	dt {
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		color: var(--fg-muted);
	}
	dd {
		margin: 0;
		font-size: 14px;
		color: var(--fg-strong);
	}
	dd.bad {
		color: var(--red);
	}
	.note {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.outcome {
		text-transform: uppercase;
		letter-spacing: 0.07em;
		font-weight: 700;
	}
	.result.completed .outcome {
		color: var(--green);
	}
	.result.cancelled .outcome {
		color: var(--alert);
	}
	.result.failed .outcome {
		color: var(--red);
	}

	.problems {
		margin-top: 12px;
		font-size: 12px;
	}
	.problems summary {
		cursor: pointer;
		color: var(--alert);
	}
	.problems ul {
		margin: 6px 0 0;
		padding: 0;
		list-style: none;
		max-height: 200px;
		overflow-y: auto;
		background: var(--bg-soft);
		border-radius: var(--radius);
	}
	.problems li {
		padding: 4px 10px;
		color: var(--fg);
		word-break: break-word;
	}
</style>
