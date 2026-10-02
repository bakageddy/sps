<script lang="ts">
	/**
	 * sp_who2 for one running-query snapshot: every SQL Server session,
	 * not just the ones running a statement (that's sp_WhoIsActive, in
	 * MssqlQueryTable). Useful for the sleeping-but-blocking session that
	 * never shows up as a running query. Sortable like its siblings.
	 */
	import type { SpWho2Row } from "$lib/api/runningquery";
	import { formatDuration, formatTimestamp } from "$lib/format";

	interface Props {
		rows: SpWho2Row[];
	}

	let { rows }: Props = $props();

	type SortKey = "spid" | "status" | "blockedBy" | "cpu" | "diskio" | "lastbatch";
	let sortKey = $state<SortKey>("cpu");
	let sortDescending = $state(true);

	function sortBy(key: SortKey) {
		if (sortKey === key) sortDescending = !sortDescending;
		else {
			sortKey = key;
			sortDescending = key !== "spid" && key !== "status";
		}
	}

	function sortValue(r: SpWho2Row): string | number {
		switch (sortKey) {
			case "spid":
				return r.spid;
			case "status":
				return r.status.toLowerCase();
			case "blockedBy":
				return r.blockedBy ?? 0;
			case "cpu":
				return r.cputime;
			case "diskio":
				return r.diskio;
			case "lastbatch":
				return r.lastbatch;
		}
	}

	const sorted = $derived(
		rows.toSorted((a, b) => {
			const va = sortValue(a);
			const vb = sortValue(b);
			const order = va < vb ? -1 : va > vb ? 1 : 0;
			return sortDescending ? -order : order;
		}),
	);

	const blocked = (r: SpWho2Row) => r.blockedBy !== null && r.blockedBy !== 0;

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		month: "short",
		day: "2-digit",
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
		hourCycle: "h23",
	});

	const columns: { key: SortKey; label: string; class: string }[] = [
		{ key: "spid", label: "SPID", class: "col-num" },
		{ key: "status", label: "Status", class: "col-status" },
		{ key: "blockedBy", label: "Blk by", class: "col-num" },
	];
	const tailColumns: { key: SortKey; label: string; class: string }[] = [
		{ key: "cpu", label: "CPU", class: "col-num" },
		{ key: "diskio", label: "Disk IO", class: "col-num" },
		{ key: "lastbatch", label: "Last batch", class: "col-when" },
	];
</script>

<div class="table">
	<div class="head">
		{#each columns as col (col.key)}
			<button class="sort {col.class}" onclick={() => sortBy(col.key)}>
				{col.label}
				{#if sortKey === col.key}<span class="arrow"
						>{sortDescending ? "▼" : "▲"}</span
					>{/if}
			</button>
		{/each}
		<span class="sort col-text">Login</span>
		<span class="sort col-text">DB</span>
		<span class="sort col-cmd">Command</span>
		{#each tailColumns as col (col.key)}
			<button class="sort {col.class}" onclick={() => sortBy(col.key)}>
				{col.label}
				{#if sortKey === col.key}<span class="arrow"
						>{sortDescending ? "▼" : "▲"}</span
					>{/if}
			</button>
		{/each}
	</div>

	<div class="rows">
		{#each sorted as r (r.spid + ":" + r.requestId)}
			<div class="row" class:blocked={blocked(r)}>
				<span class="col-num mono">{r.spid}</span>
				<span class="col-status"
					><span class="badge {r.status.toLowerCase()}">{r.status}</span></span
				>
				<span class="col-num mono" class:bad={blocked(r)}
					>{blocked(r) ? r.blockedBy : "—"}</span
				>
				<span
					class="col-text mono"
					title={[r.hostname, r.programName].filter(Boolean).join(" · ") ||
						undefined}>{r.login || "—"}</span
				>
				<span class="col-text mono">{r.dbname || "—"}</span>
				<span class="col-cmd mono">{r.command}</span>
				<span class="col-num mono">{formatDuration(r.cputime)}</span>
				<span class="col-num mono">{r.diskio}</span>
				<span class="col-when mono"
					>{formatTimestamp(timeFormat, r.lastbatch)}</span
				>
			</div>
		{:else}
			<p class="empty">No sp_who2 rows in this snapshot.</p>
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
		grid-template-columns: 56px 100px 60px 140px 120px 1fr 72px 72px 150px;
		gap: 10px;
		align-items: center;
		padding: 0 10px;
	}

	.head {
		flex-shrink: 0;
		background: var(--bg-soft);
		position: sticky;
		top: 0;
		z-index: 1;
	}

	.sort {
		padding: 8px 0;
		text-align: left;
		font-weight: 600;
		color: var(--fg-strong);
		white-space: nowrap;
		border-radius: 0;
	}
	button.sort:hover {
		color: var(--accent);
	}
	.sort.col-num {
		text-align: right;
	}
	.arrow {
		font-size: 9px;
		color: var(--accent);
	}

	.rows {
		overflow: auto;
		flex: 1;
	}

	.row {
		padding-top: 4px;
		padding-bottom: 4px;
		border-top: 1px solid var(--hairline);
		color: var(--fg);
	}
	.row:hover {
		background: var(--bg-hover);
	}
	/* attention is reserved for sessions someone is waiting on/behind */
	.row.blocked {
		background: color-mix(in srgb, var(--alert) 8%, transparent);
	}
	.bad {
		color: var(--alert);
		font-weight: 600;
	}

	.col-num {
		text-align: right;
	}
	.col-text,
	.col-cmd {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.col-when {
		white-space: nowrap;
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}

	.badge {
		padding: 0 8px;
		border-radius: 999px;
		font-size: 11px;
		font-weight: 600;
		background: var(--bg-hard);
		color: var(--fg-muted);
	}
	.badge.suspended {
		background: color-mix(in srgb, var(--yellow) 18%, transparent);
		color: var(--yellow);
	}
	.badge.runnable {
		background: color-mix(in srgb, var(--green) 18%, transparent);
		color: var(--green);
	}

	.empty {
		padding: 24px;
		text-align: center;
		color: var(--fg-muted);
	}
</style>
