<script lang="ts">
	/**
	 * Glowroot ingest. Collector first (protocol, host, port — it connects
	 * by itself once typing pauses), then login, time window and agent, with
	 * every other knob folded under "Advanced".
	 *
	 * State lives in lib/glowroot.svelte.ts so a run survives leaving the page.
	 */
	import Connect from "$lib/components/glowroot/Connect.svelte";
	import RangeControl from "$lib/components/glowroot/RangeControl.svelte";
	import AgentPicker from "$lib/components/glowroot/AgentPicker.svelte";
	import OptionsForm from "$lib/components/glowroot/OptionsForm.svelte";
	import TraceCount from "$lib/components/glowroot/TraceCount.svelte";
	import IngestProgress from "$lib/components/glowroot/IngestProgress.svelte";
	import RunHistory from "$lib/components/glowroot/RunHistory.svelte";
	import { filterCount, form, ready, session } from "$lib/glowroot.svelte";

	const central = $derived(session.collector?.central === true);
	const connected = $derived(ready());
	/** the run needs a loaded agent layout: embedded has it at once, Central after a pick */
	const configurable = $derived(connected && session.layout !== null);

	const num = new Intl.NumberFormat();
	/** what Advanced currently holds, readable while it is folded */
	const summary = $derived.by(() => {
		const parts: string[] = [];
		if (form.types.value.length > 0) parts.push(form.types.value.join(" + "));
		const fam = [form.slow.value && "slow", form.error.value && "error"].filter(Boolean);
		if (fam.length > 0) parts.push(fam.join(" + "));
		const lo = form.minSeconds.value.trim();
		const hi = form.maxSeconds.value.trim();
		if (lo !== "" || hi !== "") parts.push(`${lo || "0"}s – ${hi === "" ? "∞" : `${hi}s`}`);
		const nf = filterCount();
		if (nf > 0) parts.push(`${nf} filter${nf === 1 ? "" : "s"}`);
		parts.push(`≤ ${num.format(Number(form.maxTraces.value) || 0)} traces`);
		parts.push(`${form.workers.value} workers`);
		return parts.join(" · ");
	});
</script>

<div class="scroll">
	<div class="page">
		<h1>Glowroot</h1>
		<p class="lede">
			Pull Web and Schedule transaction traces from an embedded agent or Glowroot Central into the
			open database. Plain HTTP; nothing is sent anywhere but your collector.
		</p>

		<section>
			<h2>Collector</h2>
			<Connect />
		</section>

		{#if connected}
			<section>
				<h2>Time window</h2>
				<RangeControl />
			</section>

			{#if central}
				<section>
					<h2>Agent</h2>
					<AgentPicker />
				</section>
			{/if}
		{/if}

		{#if configurable}
			<details class="advanced" bind:open={form.advancedOpen.value}>
				<summary>
					<span class="chev" aria-hidden="true">{form.advancedOpen.value ? "▾" : "▸"}</span>
					Advanced
					{#if !form.advancedOpen.value}<span class="sum">{summary}</span>{/if}
				</summary>
				<OptionsForm />
			</details>

			<section>
				<h2>Run</h2>
				<div class="count"><TraceCount /></div>
				<IngestProgress />
			</section>
		{:else if connected && central}
			<p class="muted">Pick an agent to see the options.</p>
		{/if}

		<h2 class="past">Past runs</h2>
		<RunHistory />
	</div>
</div>

<style>
	.scroll {
		height: 100%;
		overflow-y: auto;
	}
	.page {
		max-width: 820px;
		margin: 0 auto;
		padding: 32px 24px 56px;
	}
	h1 {
		margin: 0 0 6px;
		font-size: 15px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.07em;
		color: var(--accent);
	}
	.lede {
		margin: 0 0 8px;
		font-size: 13px;
		color: var(--fg-muted);
	}
	section {
		margin-top: 26px;
	}
	h2 {
		margin: 0 0 10px;
		font-size: 13px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--fg-muted);
	}
	h2.past {
		margin-top: 36px;
	}
	.count {
		margin-bottom: 14px;
	}
	.muted {
		margin-top: 20px;
		font-size: 12px;
		color: var(--fg-muted);
	}

	/* folds like the Logs hub's: our own chevron, summary in the heading style */
	.advanced {
		margin-top: 26px;
	}
	.advanced summary {
		display: flex;
		align-items: baseline;
		gap: 8px;
		margin: 0 0 12px;
		font-size: 13px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--fg-muted);
		cursor: pointer;
		list-style: none;
		user-select: none;
	}
	.advanced summary::-webkit-details-marker {
		display: none;
	}
	.advanced summary:hover {
		color: var(--fg);
	}
	.chev {
		width: 10px;
		font-size: 11px;
		color: var(--accent);
	}
	.sum {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		text-transform: none;
		letter-spacing: 0;
		font-size: 12px;
		opacity: 0.8;
	}
</style>
