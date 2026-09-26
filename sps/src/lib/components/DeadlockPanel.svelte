<script lang="ts">
	/**
	 * Lock analysis for one dump, derived entirely from the census rows'
	 * (tid, lockOwnerTid) edges — no backend command:
	 *
	 *  - Deadlocks: cycles in the wait-for graph, each shown as its ring.
	 *  - Hot locks: owners with the most threads stuck behind them
	 *    (transitively) — the convoy view; nine out of ten lock incidents
	 *    are one owner, and this names it.
	 *
	 * Clicking any thread selects it in the census table.
	 */
	import type { ThreadDumpThread } from "$lib/api/threaddump";
	import { blockedBehind, findDeadlocks } from "$lib/threaddump";

	interface Props {
		threads: ThreadDumpThread[];
		onselect: (tid: number) => void;
	}

	let { threads, onselect }: Props = $props();

	const cycles = $derived(findDeadlocks(threads));

	// every thread that owns a lock someone waits on, ranked by how many
	// are stuck behind it
	const hotLocks = $derived.by(() => {
		const byTid = new Map(threads.map((t) => [t.tid, t]));
		const owners = new Set<number>();
		for (const t of threads)
			if (t.lockOwnerTid !== null) owners.add(t.lockOwnerTid);
		return [...owners]
			.map((tid) => ({
				owner: byTid.get(tid) ?? null,
				tid,
				name:
					byTid.get(tid)?.name ??
					threads.find((t) => t.lockOwnerTid === tid)?.lockOwnerName ??
					"?",
				behind: blockedBehind(threads, tid),
			}))
			.toSorted((a, b) => b.behind - a.behind)
			.slice(0, 12);
	});
</script>

<div class="panel">
	<section>
		<h3>
			Deadlocks
			<span class="count mono">{cycles.length}</span>
		</h3>
		{#each cycles as cycle, i (i)}
			<div class="ring">
				<div class="ring-head">
					<span class="crown">deadlock</span>
					<span class="muted">{cycle.length} threads in a cycle</span>
				</div>
				{#each cycle as t, j (t.tid)}
					<button class="member" onclick={() => onselect(t.tid)}>
						<span class="mono tid">{t.tid}</span>
						<span class="mono name">{t.name}</span>
						<span class="mono muted obj"
							>waits on {t.waitingOn ?? t.lock ?? "?"}</span
						>
						<span class="arrow"
							>→ owned by {cycle[(j + 1) % cycle.length].tid}</span
						>
					</button>
				{/each}
			</div>
		{:else}
			<p class="empty">No wait-for cycles in this dump.</p>
		{/each}
	</section>

	<section>
		<h3>
			Hot locks
			<span class="muted small">threads stuck behind an owner</span>
		</h3>
		{#each hotLocks as h (h.tid)}
			<button class="member" onclick={() => onselect(h.tid)}>
				<span class="mono tid">{h.tid}</span>
				<span class="mono name">{h.name}</span>
				<span class="mono muted">{h.owner?.state ?? "not in dump"}</span>
				<span class="behind mono" class:bad={h.behind >= 5}
					>{h.behind} behind</span
				>
			</button>
		{:else}
			<p class="empty">No thread is waiting on another's lock.</p>
		{/each}
	</section>
</div>

<style>
	.panel {
		overflow: auto;
		height: 100%;
		font-size: 12.5px;
	}
	section {
		padding: 4px 0 8px;
		border-bottom: 1px solid var(--hairline);
	}
	h3 {
		display: flex;
		align-items: baseline;
		gap: 10px;
		margin: 0;
		padding: 8px 12px 4px;
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.07em;
		color: var(--fg-muted);
	}
	.count {
		color: var(--fg-strong);
	}
	.small {
		text-transform: none;
		letter-spacing: 0;
		font-weight: 400;
	}
	.ring {
		margin: 4px 12px 8px;
		border-radius: var(--radius);
		background: color-mix(in srgb, var(--alert) 8%, transparent);
	}
	.ring-head {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 10px 2px;
	}
	.crown {
		padding: 0 8px;
		border-radius: 999px;
		font-size: 10.5px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		background: color-mix(in srgb, var(--alert) 20%, transparent);
		color: var(--alert);
	}
	.member {
		display: grid;
		grid-template-columns: 56px minmax(120px, 1fr) minmax(100px, 1.2fr) auto;
		gap: 10px;
		align-items: center;
		width: 100%;
		padding: 4px 12px;
		text-align: left;
		border-radius: 0;
		color: var(--fg);
	}
	.member:hover {
		background: var(--bg-hover);
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
	.tid {
		text-align: right;
	}
	.name,
	.obj {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.muted {
		color: var(--fg-muted);
	}
	.arrow {
		font-size: 11.5px;
		color: var(--accent);
		white-space: nowrap;
	}
	.behind {
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.behind.bad {
		color: var(--alert);
		font-weight: 600;
	}
	.empty {
		padding: 8px 12px 12px;
		color: var(--fg-muted);
	}
</style>
