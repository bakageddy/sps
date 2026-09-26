<script lang="ts">
	/**
	 * The incident census: every thread of the incident's threaddump
	 * (the spine) decorated by tid with what the other logs knew — cpu %
	 * from cpumonitoring, held-for from connectiondump. "—" means that log
	 * had no row for the thread (it only samples hot threads / holders),
	 * which is normal. Owner is a link, as in the plain census.
	 */
	import type { IncidentThread } from "$lib/connectiondump";
	import { stateColor } from "$lib/threaddump";
	import { formatDuration } from "$lib/format";
	import { virtualWindow } from "$lib/virtual";

	interface Props {
		threads: IncidentThread[];
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

	// Virtualized rows: a dump has thousands of threads and the incident
	// page mounts this table on every signal click. ROW/HEADER must match
	// the fixed heights in the CSS below.
	const ROW = 28;
	const HEADER = 32;
	let scroller = $state<HTMLDivElement>();
	let viewport = $state(0);
	let scrollTop = $state(0);
	const win = $derived(
		virtualWindow(scrollTop - HEADER, viewport, ROW, visible.length),
	);
	const slice = $derived(visible.slice(win.start, win.end));

	// bring an externally-selected row into view (owner-link jumps) — by
	// index, since the row may not exist in the DOM yet
	$effect(() => {
		if (!scroller || selected === null) return;
		const idx = visible.findIndex((t) => t.tid === selected);
		if (idx < 0) return;
		const top = HEADER + idx * ROW;
		const cur = scroller.scrollTop;
		if (top - HEADER < cur) scroller.scrollTop = top - HEADER;
		else if (top + ROW > cur + viewport)
			scroller.scrollTop = top + ROW - viewport;
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
	<div
		class="scroller"
		bind:this={scroller}
		bind:clientHeight={viewport}
		onscroll={(e) => (scrollTop = (e.currentTarget as HTMLDivElement).scrollTop)}
	>
		<table>
			<thead>
				<tr>
					<th class="num">Tid</th>
					<th>Name</th>
					<th>State</th>
					<th class="num">CPU %</th>
					<th class="num">Held</th>
					<th>Owner</th>
				</tr>
			</thead>
			<tbody>
				<tr class="spacer" style:height="{win.padTop}px"><td colspan="6"></td></tr>
				{#each slice as t (t.tid)}
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
						<td class="mono num cpu" class:hot={(t.cpu ?? 0) >= 50}
							>{t.cpu === null ? "—" : t.cpu.toFixed(1)}</td
						>
						<td class="mono num held"
							>{t.heldFor === null ? "—" : formatDuration(t.heldFor)}</td
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
						<td colspan="6" class="empty">
							{threads.length === 0
								? "No thread dump within the incident window."
								: "No thread matches the filter."}
						</td>
					</tr>
				{/each}
				<tr class="spacer" style:height="{win.padBottom}px"><td colspan="6"></td></tr>
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
		height: 32px; /* = HEADER in the script */
		box-sizing: border-box;
		padding: 0 10px;
		font-weight: 600;
		color: var(--fg-strong);
		white-space: nowrap;
	}
	td {
		height: 28px; /* = ROW in the script; virtualization needs it exact */
		box-sizing: border-box;
		padding: 0 10px;
		border-top: 1px solid var(--hairline);
		white-space: nowrap;
		overflow: hidden;
	}
	.spacer td {
		height: auto;
		padding: 0;
		border: 0;
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
		color: var(--alert);
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
	.num {
		text-align: right;
	}
	.name {
		max-width: 260px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.state {
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}
	.cpu.hot {
		color: var(--yellow);
		font-weight: 600;
	}
	.held {
		color: var(--fg-muted);
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
		height: auto;
		text-align: center;
		color: var(--fg-muted);
		padding: 24px;
		cursor: default;
	}
</style>
