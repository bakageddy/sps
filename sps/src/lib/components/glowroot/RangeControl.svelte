<script lang="ts">
	/**
	 * Time window for the run: a preset ("the last N") or explicit bounds typed
	 * in the display timezone — same text formats as TimeRangePicker. Shows the
	 * window it currently resolves to so "last 4h" is never ambiguous.
	 */
	import { PRESETS, form, resolveWindow } from "$lib/glowroot.svelte";
	import { formatTimestamp } from "$lib/format";

	const fmt = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	// a preset's window slides with the clock: recompute when asked, not on a timer
	let tick = $state(0);
	const resolved = $derived.by(() => {
		void tick;
		void form.preset.value;
		void form.customFrom.value;
		void form.customTo.value;
		return resolveWindow();
	});

	function pick(id: (typeof PRESETS)[number]["id"] | "custom") {
		form.preset.value = id;
		tick += 1;
		if (id === "custom" && form.customFrom.value === "" && form.customTo.value === "") {
			// seed with the last hour so the fields are never both empty
			const now = Date.now();
			const pad = (n: number) => String(n).padStart(2, "0");
			const f = (ms: number) => {
				const d = new Date(ms);
				return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`;
			};
			form.customFrom.value = f(now - 3_600_000);
			form.customTo.value = f(now);
		}
	}
</script>

<div class="chips" role="group" aria-label="Time range">
	{#each PRESETS as p (p.id)}
		<button
			type="button"
			class:on={form.preset.value === p.id}
			onclick={() => pick(p.id)}
			title="The last {p.label}, ending now">{p.label}</button
		>
	{/each}
	<button type="button" class:on={form.preset.value === "custom"} onclick={() => pick("custom")}>
		Custom
	</button>
</div>

{#if form.preset.value === "custom"}
	<div class="custom">
		<label>
			<span>From</span>
			<input
				class="mono"
				type="text"
				placeholder="2026-11-07 09:30"
				bind:value={form.customFrom.value}
				spellcheck="false"
			/>
		</label>
		<label>
			<span>To</span>
			<input
				class="mono"
				type="text"
				placeholder="2026-11-07 17:00"
				bind:value={form.customTo.value}
				spellcheck="false"
			/>
		</label>
	</div>
{/if}

<p class="resolved mono" class:bad={typeof resolved === "string"}>
	{#if typeof resolved === "string"}
		{resolved}
	{:else}
		{formatTimestamp(fmt, resolved[0])} → {formatTimestamp(fmt, resolved[1])}
		{#if form.preset.value !== "custom"}
			<button type="button" class="refresh" onclick={() => (tick += 1)} title="Slide the window to now"
				>↻</button
			>
		{/if}
	{/if}
</p>

<style>
	.mono {
		font-family: var(--font-mono);
	}
	.chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}
	.chips button {
		padding: 4px 12px;
		font-size: 12px;
		background: var(--bg-hard);
	}
	.chips button:hover {
		background: var(--bg-hover);
	}
	.chips button.on {
		background: var(--accent);
		color: var(--bg);
		font-weight: 700;
	}
	.custom {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		margin-top: 10px;
	}
	label {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	input {
		width: 170px;
		padding: 5px 8px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		font-size: 12px;
	}
	.resolved {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.resolved.bad {
		color: var(--red);
	}
	.refresh {
		margin-left: 6px;
		padding: 0 6px;
		background: none;
		color: var(--accent);
		font-size: 13px;
	}
</style>
