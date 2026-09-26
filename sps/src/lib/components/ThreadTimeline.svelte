<script lang="ts">
	/**
	 * One thread across every dump — the cpumonitoring LineChart's role for
	 * a categorical signal. One slot per dump (ORDINAL spacing: the three
	 * dumps of an incident would be crushed by the hours between incidents
	 * on a time axis), a marker colored by state, and a bar joining two
	 * consecutive markers when the thread is UNCHANGED between them — same
	 * state, waiting on the same thing, same lock owner — so an unbroken bar
	 * across dumps 1→3 reads as "stuck" at a glance, a broken one as "busy".
	 * Derived purely from the census rows; stacks are never compared. Click
	 * a marker to load that dump's stack in the panel below.
	 */
	import type {
		ThreadDumpPoint,
		ThreadDumpSummary,
	} from "$lib/api/threaddump";
	import { stateColor } from "$lib/threaddump";
	import { formatTimestamp } from "$lib/format";

	interface Props {
		/** every dump in the store — the slots, present or not */
		dumps: ThreadDumpSummary[];
		/** the thread's rows, only for dumps it appeared in */
		points: ThreadDumpPoint[];
		/** timestamp whose stack is shown below, or null */
		selected: number | null;
		onselect: (timestamp: number) => void;
		emptyText?: string;
	}

	let {
		dumps,
		points,
		selected,
		onselect,
		emptyText = "Click a thread to trace it across dumps.",
	}: Props = $props();

	const byTs = $derived(new Map(points.map((p) => [p.timestamp, p])));

	interface Slot {
		timestamp: number;
		point: ThreadDumpPoint | null;
		/** same state / wait target / lock owner as the PREVIOUS PRESENT slot */
		sameAsPrev: boolean;
	}

	const slots = $derived.by<Slot[]>(() => {
		let prev: ThreadDumpPoint | null = null;
		return dumps.map((d) => {
			const point = byTs.get(d.timestamp) ?? null;
			const sameAsPrev =
				point !== null &&
				prev !== null &&
				point.state === prev.state &&
				point.waitingOn === prev.waitingOn &&
				point.lockOwnerTid === prev.lockOwnerTid;
			if (point !== null) prev = point;
			return { timestamp: d.timestamp, point, sameAsPrev };
		});
	});

	// longest run of consecutive present slots with an unchanged stack
	const longestRun = $derived.by(() => {
		let best = { len: 0, state: null as ThreadDumpPoint["state"] | null };
		let len = 0;
		for (const s of slots) {
			if (s.point === null) {
				len = 0;
				continue;
			}
			len = s.sameAsPrev ? len + 1 : 1;
			if (len > best.len) best = { len, state: s.point.state };
		}
		return best;
	});

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
	const shortFormat = new Intl.DateTimeFormat(undefined, {
		hour: "2-digit",
		minute: "2-digit",
		second: "2-digit",
		hourCycle: "h23",
	});

	function title(s: Slot): string {
		const when = formatTimestamp(timeFormat, s.timestamp);
		if (s.point === null) return `${when} — thread not in this dump`;
		const p = s.point;
		return (
			`${when} — ${p.state}` +
			(p.waitingOn ? ` on ${p.waitingOn}` : "") +
			(p.lockOwnerTid !== null
				? ` · owned by ${p.lockOwnerName ?? "tid"} ${p.lockOwnerTid}`
				: "") +
			(s.sameAsPrev ? " · unchanged since previous dump" : "")
		);
	}
</script>

<div class="wrap">
	{#if points.length === 0}
		<p class="hint">{emptyText}</p>
	{:else}
		<div class="summary">
			<span
				>in <strong class="mono">{points.length}</strong> of
				<span class="mono">{dumps.length}</span> dumps</span
			>
			{#if longestRun.len > 1}
				<span class="muted">·</span>
				<span
					>longest unchanged run:
					<strong class="mono">{longestRun.len}</strong> dumps
					<span
						class="state"
						style:color={stateColor(longestRun.state!)}
						>{longestRun.state}</span
					></span
				>
			{/if}
			<span class="legend muted"
				>bar = same state · lock · owner as previous dump</span
			>
		</div>

		<div class="strip" role="group" aria-label="Thread state per dump">
			{#each slots as s (s.timestamp)}
				<button
					class="slot"
					class:absent={s.point === null}
					class:selected={selected === s.timestamp}
					disabled={s.point === null}
					title={title(s)}
					aria-pressed={selected === s.timestamp}
					onclick={() => onselect(s.timestamp)}
				>
					{#if s.sameAsPrev}
						<span class="link" style:background={stateColor(s.point!.state)}
						></span>
					{/if}
					<span
						class="marker"
						style:background={s.point === null
							? "transparent"
							: stateColor(s.point.state)}
					></span>
				</button>
			{/each}
		</div>

		<div class="axis">
			<span class="mono muted"
				>{formatTimestamp(shortFormat, slots[0].timestamp)}</span
			>
			<span class="mono muted"
				>{formatTimestamp(shortFormat, slots[slots.length - 1].timestamp)}</span
			>
		</div>
	{/if}
</div>

<style>
	.wrap {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
		padding: 8px 12px;
		gap: 6px;
	}
	.hint {
		margin: 0;
		padding: 8px 0;
		font-size: 12.5px;
		color: var(--fg-muted);
	}
	.summary {
		display: flex;
		align-items: baseline;
		gap: 8px;
		flex-wrap: wrap;
		font-size: 12px;
		color: var(--fg);
		flex-shrink: 0;
	}
	.legend {
		margin-left: auto;
		font-size: 11px;
	}
	.state {
		font-size: 10.5px;
		font-weight: 700;
		letter-spacing: 0.05em;
	}
	.mono {
		font-family: var(--font-mono);
	}
	.muted {
		color: var(--fg-muted);
	}

	/* one slot per dump; the strip scrolls horizontally when there are
	   hundreds of dumps rather than shrinking markers into illegibility */
	.strip {
		display: flex;
		align-items: center;
		flex: 1;
		min-height: 28px;
		overflow-x: auto;
		overflow-y: hidden;
		background: var(--bg-hard);
		border-radius: var(--radius);
		padding: 0 6px;
	}
	.slot {
		position: relative;
		flex: 0 0 18px;
		height: 100%;
		display: grid;
		place-items: center;
		padding: 0;
		background: none;
		border: none;
		border-radius: 0;
		cursor: pointer;
	}
	.slot:disabled {
		cursor: default;
	}
	.slot:hover:not(:disabled) .marker {
		transform: scale(1.35);
	}
	.marker {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		transition: transform 0.1s;
		z-index: 1;
	}
	.absent .marker {
		width: 3px;
		height: 3px;
		background: var(--fg-muted) !important;
		opacity: 0.35;
	}
	.selected .marker {
		outline: 2px solid var(--fg-strong);
		outline-offset: 2px;
	}
	/* the bar to the previous marker: same-stack continuity */
	.link {
		position: absolute;
		left: -9px; /* reach back to the previous slot's center */
		width: 18px;
		height: 3px;
		opacity: 0.55;
	}

	.axis {
		display: flex;
		justify-content: space-between;
		font-size: 10px;
		flex-shrink: 0;
	}
</style>
