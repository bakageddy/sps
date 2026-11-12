<script lang="ts">
	/**
	 * Past runs from the database, newest first, sortable. Refetches when a
	 * run starts or finishes, and when a different database is opened.
	 */
	import { glowrootRuns, type IngestRun } from "$lib/api/glowroot";
	import { db } from "$lib/database.svelte";
	import { describeFilters, history } from "$lib/glowroot.svelte";
	import { formatDuration, formatTimestamp } from "$lib/format";
	import { sortRows } from "$lib/sort";
	import SortButton from "$lib/components/SortButton.svelte";

	let runs = $state<IngestRun[]>([]);
	let error = $state<string | null>(null);

	// a stale response must not overwrite a newer one
	let seq = 0;
	$effect(() => {
		void history.version;
		void db.epoch;
		if (db.state.status !== "open") {
			runs = [];
			return;
		}
		const mine = ++seq;
		glowrootRuns()
			.then((r) => {
				if (mine !== seq) return;
				runs = r;
				error = null;
			})
			.catch((e) => {
				if (mine === seq) error = String(e);
			});
	});

	type SortKey = "startedAt" | "agentRollupId" | "outcome" | "stored" | "failed" | "duration" | "window";
	let sortKey = $state<SortKey | null>(null); // null keeps the backend's newest-first
	let sortDescending = $state(true);
	function sortBy(key: SortKey) {
		if (sortKey === key) sortDescending = !sortDescending;
		else {
			sortKey = key;
			sortDescending = key !== "agentRollupId" && key !== "outcome";
		}
	}
	const key = (r: IngestRun, k: SortKey): unknown =>
		k === "duration"
			? r.finishedAt === null ? null : r.finishedAt - r.startedAt
			: k === "window"
				? r.options.to - r.options.from
				: r[k];
	const sorted = $derived(
		sortRows(runs, sortKey === null ? null : (r) => key(r, sortKey!), sortDescending),
	);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "short",
		hourCycle: "h23",
	});
	const num = new Intl.NumberFormat();
	const head = (k: SortKey) => ({
		active: sortKey === k,
		descending: sortDescending,
		onclick: () => sortBy(k),
	});

	function band(r: IngestRun): string {
		const lo = r.options.durationMillisLow;
		const hi = r.options.durationMillisHigh;
		if (lo === 0 && hi === null) return "any";
		return `${lo / 1000}s – ${hi === null ? "∞" : `${hi / 1000}s`}`;
	}
</script>

{#if error}<p class="error" role="alert">{error}</p>{/if}

{#if runs.length === 0}
	<p class="muted">No runs yet.</p>
{:else}
	<div class="scroller">
		<table>
			<thead>
				<tr>
					<th><SortButton label="Started" {...head("startedAt")} /></th>
					<th><SortButton label="Agent" {...head("agentRollupId")} /></th>
					<th><SortButton label="Window" {...head("window")} /></th>
					<th>Types</th>
					<th>Duration band</th>
					<th>Filters</th>
					<th><SortButton label="Outcome" {...head("outcome")} /></th>
					<th class="n"><SortButton label="Stored" {...head("stored")} /></th>
					<th class="n"><SortButton label="Failed" {...head("failed")} /></th>
					<th class="n"><SortButton label="Took" {...head("duration")} /></th>
				</tr>
			</thead>
			<tbody>
				{#each sorted as r (r.id)}
					<tr title={r.collectorUrl}>
						<td class="mono">{formatTimestamp(timeFormat, r.startedAt)}</td>
						<td class="agent">{r.agentRollupId || "embedded"}</td>
						<td class="mono">{formatDuration(r.options.to - r.options.from)}</td>
						<td>{r.options.transactionTypes.join(", ")}</td>
						<td class="mono">{band(r)}</td>
						<td class="filt" title={describeFilters(r.options).join("\n")}>
							{describeFilters(r.options).join(" · ") || "—"}
						</td>
						<td><span class="outcome {r.outcome}">{r.outcome}</span></td>
						<td class="n mono">{num.format(r.stored)}<span class="dim"> / {num.format(r.total)}</span></td>
						<td class="n mono" class:bad={r.failed > 0}>{num.format(r.failed)}</td>
						<td class="n mono">{r.finishedAt === null ? "—" : formatDuration(r.finishedAt - r.startedAt)}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}

<style>
	.mono {
		font-family: var(--font-mono);
	}
	.muted,
	.dim {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.error {
		margin: 0 0 8px;
		font-size: 12px;
		color: var(--red);
	}
	.scroller {
		overflow-x: auto;
		background: var(--bg-soft);
		border-radius: var(--radius);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12px;
	}
	th {
		padding: 8px 12px;
		text-align: left;
		font-weight: 400;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	td {
		padding: 6px 12px;
		white-space: nowrap;
	}
	tbody tr:hover {
		background: var(--bg-hover);
	}
	.n {
		text-align: right;
	}
	.filt {
		max-width: 260px;
		overflow: hidden;
		text-overflow: ellipsis;
		color: var(--fg-muted);
	}
	.agent {
		max-width: 220px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.bad {
		color: var(--red);
	}
	.outcome {
		text-transform: uppercase;
		letter-spacing: 0.06em;
		font-size: 11px;
		font-weight: 700;
	}
	.outcome.completed {
		color: var(--green);
	}
	.outcome.cancelled {
		color: var(--alert);
	}
	.outcome.failed {
		color: var(--red);
	}
	.outcome.running {
		color: var(--accent);
	}
</style>
