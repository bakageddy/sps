<script lang="ts">
	/**
	 * One dump's census: every thread with its state badge and lock
	 * relationship. The owner column is a link — clicking "owned by tid 47"
	 * selects thread 47, so you follow a blocking chain one hop at a time
	 * without leaving the table. Filter matches name or tid.
	 */
	import type { ThreadDumpThread } from "$lib/api/threaddump";
	import { stateColor } from "$lib/threaddump";

	interface Props {
		threads: ThreadDumpThread[];
		/** tid of the inspected thread, or null */
		selected: number | null;
		onselect: (tid: number) => void;
	}

	let { threads, selected, onselect }: Props = $props();

	let filter = $state("");
	const visible = $derived.by(() => {
		const q = filter.trim().toLowerCase();
		if (q === "") return threads;
		return threads.filter(
			(t) => t.name.toLowerCase().includes(q) || String(t.tid).includes(q),
		);
	});

	// bring an externally-selected row into view (owner-link jumps)
	let scroller = $state<HTMLDivElement>();
	$effect(() => {
		void selected;
		scroller
			?.querySelector("tr.selected")
			?.scrollIntoView({ block: "nearest" });
	});
</script>

<div class="wrap">
	<div class="tools">
		<input
			type="search"
			placeholder="filter by name or tid"
			bind:value={filter}
		/>
		<span class="count mono">{visible.length} / {threads.length}</span>
	</div>
	<div class="scroller" bind:this={scroller}>
		<table>
			<thead>
				<tr>
					<th class="num">Tid</th>
					<th>Name</th>
					<th>State</th>
					<th>Waiting on</th>
					<th>Owner</th>
				</tr>
			</thead>
			<tbody>
				{#each visible as t (t.tid)}
					<tr
						class:selected={selected === t.tid}
						class:blocked={t.state === "BLOCKED"}
						onclick={() => onselect(t.tid)}
					>
						<td class="mono num">{t.tid}</td>
						<td class="mono name" title={t.name}>{t.name}</td>
						<td>
							<span class="state" style:color={stateColor(t.state)}
								>{t.state}</span
							>
						</td>
						<td class="mono obj" title={t.lock ?? t.waitingOn ?? undefined}
							>{t.waitingOn ?? "—"}</td
						>
						<td class="mono">
							{#if t.lockOwnerTid !== null}
								<button
									class="owner"
									onclick={(e) => {
										e.stopPropagation();
										onselect(t.lockOwnerTid!);
									}}
									title="Select the owner thread"
									>{t.lockOwnerName ?? "tid"} · {t.lockOwnerTid}</button
								>
							{:else}
								—
							{/if}
						</td>
					</tr>
				{:else}
					<tr>
						<td colspan="5" class="empty">
							{threads.length === 0
								? "Select a dump to list its threads."
								: "No thread matches the filter."}
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
	.tools {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.tools input {
		flex: 1;
		padding: 3px 8px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		color: var(--fg);
		font-size: 12px;
	}
	.count {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.scroller {
		overflow: auto;
		flex: 1;
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 12.5px;
	}
	thead {
		position: sticky;
		top: 0;
		background: var(--bg-soft);
		z-index: 1;
	}
	th {
		text-align: left;
		padding: 8px 10px;
		font-weight: 600;
		color: var(--fg-strong);
		white-space: nowrap;
	}
	td {
		padding: 4px 10px;
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
	tr.blocked .name {
		color: var(--red);
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
	.num {
		text-align: right;
	}
	.name,
	.obj {
		max-width: 260px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.state {
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}
	.owner {
		padding: 0 6px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font: inherit;
		font-size: 11.5px;
	}
	.owner:hover {
		background: var(--bg-hover);
	}
	.empty {
		text-align: center;
		color: var(--fg-muted);
		padding: 24px;
		cursor: default;
	}
</style>
