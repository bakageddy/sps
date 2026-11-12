<script lang="ts">
	/**
	 * The advanced options: what to pull and how hard to pull it. Everything here is a
	 * persisted form value (lib/glowroot.svelte.ts), so the choices survive a
	 * restart.
	 */
	import { untrack } from "svelte";
	import { COMPARATORS, type Comparator } from "$lib/api/glowroot";
	import { MAX_WORKERS, form, session } from "$lib/glowroot.svelte";

	const comparatorLabel: Record<Comparator, string> = {
		begins: "begins with",
		equals: "equals",
		ends: "ends with",
		contains: "contains",
		"not-contains": "doesn't contain",
	};

	/** attribute names offered for the selected types, deduplicated */
	const attributeNames = $derived.by(() => {
		const all = layout?.traceAttributeNames ?? {};
		return [...new Set(form.types.value.flatMap((t) => all[t] ?? []))].toSorted();
	});

	// a count taken under other filters would mislead: forget it on any change
	$effect(() => {
		void [
			form.slow.value, form.error.value, form.minSeconds.value, form.maxSeconds.value,
			form.transactionName.value, form.headline.value, form.headlineCmp.value,
			form.user.value, form.userCmp.value, form.errorMessage.value, form.errorMessageCmp.value,
			form.attributeName.value, form.attributeValue.value, form.attributeCmp.value,
		];
		untrack(() => (session.counts = null));
	});

	const layout = $derived(session.layout);
	const perm = $derived(layout?.permissions);

	function toggleType(t: string) {
		form.types.value = form.types.value.includes(t)
			? form.types.value.filter((x) => x !== t)
			: [...form.types.value, t];
		session.counts = null;
	}
</script>

{#if layout === null}
	<p class="muted">{session.busy === "layout" ? "Loading the agent…" : "Pick an agent above."}</p>
{:else}
	{#if perm && !perm.traces && !perm.errorTraces}
		<p class="warn">This login cannot read traces on this agent — the run will find nothing.</p>
	{/if}

	<div class="grid">
		<div class="field wide">
			<span class="label">Transaction types</span>
			<div class="checks">
				{#each layout.transactionTypes as t (t)}
					<label class="check">
						<input type="checkbox" checked={form.types.value.includes(t)} onchange={() => toggleType(t)} />
						{t}
					</label>
				{:else}
					<span class="muted">this agent has no transaction types in storage</span>
				{/each}
			</div>
		</div>

		<div class="field wide">
			<span class="label">Traces</span>
			<div class="checks">
				<label class="check" title="traces over the slow threshold">
					<input type="checkbox" bind:checked={form.slow.value} disabled={perm ? !perm.traces : false} /> slow
				</label>
				<label class="check" title="traces that ended in an error">
					<input type="checkbox" bind:checked={form.error.value} disabled={perm ? !perm.errorTraces : false} /> error
				</label>
			</div>
		</div>

		<label class="field">
			<span class="label">Duration from (s)</span>
			<input class="mono" type="text" inputmode="decimal" placeholder="0" bind:value={form.minSeconds.value} />
		</label>
		<label class="field">
			<span class="label">Duration to (s)</span>
			<input class="mono" type="text" inputmode="decimal" placeholder="no limit" bind:value={form.maxSeconds.value} />
		</label>

		<label class="field wide">
			<span class="label">Transaction name <span class="muted">(exact, optional)</span></span>
			<input class="mono" type="text" placeholder="all names" bind:value={form.transactionName.value} spellcheck="false" />
		</label>

		<div class="field wide">
			<span class="label">Filters <span class="muted">(empty = off, all must match)</span></span>
			<div class="filters">
				{@render textRow("Headline", form.headlineCmp, form.headline, "e.g. /api/v3/requests")}
				{@render textRow("User", form.userCmp, form.user, "e.g. admin@example.com")}
				{@render textRow("Error message", form.errorMessageCmp, form.errorMessage, "error traces only")}

				<span class="row-label">Attribute</span>
				<input
					class="mono attr-name"
					type="text"
					list="glowroot-attr-names"
					placeholder="name"
					bind:value={form.attributeName.value}
					spellcheck="false"
					aria-label="Attribute name"
				/>
				<datalist id="glowroot-attr-names">
					{#each attributeNames as n (n)}<option value={n}></option>{/each}
				</datalist>
				<select bind:value={form.attributeCmp.value} aria-label="Attribute comparator">
					{#each COMPARATORS as c (c)}<option value={c}>{comparatorLabel[c]}</option>{/each}
				</select>
				<input
					class="mono"
					type="text"
					placeholder="value"
					bind:value={form.attributeValue.value}
					spellcheck="false"
					aria-label="Attribute value"
				/>
			</div>
			{#if form.errorMessage.value.trim() !== "" && form.slow.value}
				<span class="note">The error-message filter only narrows error traces; slow traces are still fetched unfiltered.</span>
			{/if}
		</div>

		<label class="field">
			<span class="label">Max traces</span>
			<input class="mono" type="number" min="1" step="1000" bind:value={form.maxTraces.value} />
			<span class="note">newest first when it bites</span>
		</label>

		<div class="field">
			<span class="label">Workers <span class="mono">{form.workers.value}</span></span>
			<input type="range" min="1" max={MAX_WORKERS} bind:value={form.workers.value} />
			<span class="note">one connection each; Central is easier on itself at 4–8</span>
		</div>

		<div class="field wide">
			<span class="label">Fetch besides the header</span>
			<div class="checks">
				<label class="check">
					<input type="checkbox" bind:checked={form.entries.value} disabled={perm ? !perm.trace : false} /> entries
				</label>
				<label class="check">
					<input type="checkbox" bind:checked={form.queries.value} disabled={perm ? !perm.queries : false} /> queries
				</label>
				<label class="check">
					<input type="checkbox" bind:checked={form.profiles.value} disabled={perm ? !perm.threadProfile : false} /> profiles
				</label>
				<label class="check" title="drop traces already stored for this agent before the cap applies">
					<input type="checkbox" bind:checked={form.skipExisting.value} /> skip traces already stored
				</label>
			</div>
		</div>
	</div>
{/if}

{#snippet textRow(
	label: string,
	cmp: { value: Comparator },
	text: { value: string },
	placeholder: string,
)}
	<span class="row-label">{label}</span>
	<select bind:value={cmp.value} aria-label="{label} comparator">
		{#each COMPARATORS as c (c)}<option value={c}>{comparatorLabel[c]}</option>{/each}
	</select>
	<input
		class="mono text-span"
		type="text"
		{placeholder}
		bind:value={text.value}
		spellcheck="false"
		aria-label="{label} text"
	/>
{/snippet}

<style>
	.mono {
		font-family: var(--font-mono);
	}
	.muted {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.warn {
		color: var(--alert);
		font-size: 12px;
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(2, minmax(0, 1fr));
		gap: 14px 20px;
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: 5px;
		min-width: 0;
	}
	.field.wide {
		grid-column: 1 / -1;
	}
	.label {
		font-size: 11px;
		text-transform: uppercase;
		letter-spacing: 0.05em;
		color: var(--fg-muted);
	}
	.note {
		font-size: 11px;
		color: var(--fg-muted);
	}
	input[type="text"],
	input[type="number"] {
		padding: 6px 10px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		font-size: 13px;
	}
	input[type="range"] {
		accent-color: var(--accent);
	}
	.checks {
		display: flex;
		flex-wrap: wrap;
		gap: 6px 18px;
	}
	.check {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		cursor: pointer;
	}
	.check input {
		accent-color: var(--accent);
	}
	.filters {
		display: grid;
		grid-template-columns: max-content max-content max-content minmax(0, 1fr);
		align-items: center;
		gap: 6px 10px;
	}
	/* simple rows are label, comparator, text (the text spans the last two
	   columns); the attribute row uses all four: label, name, comparator, value */
	.filters :global(.text-span) {
		grid-column: span 2;
	}
	.attr-name {
		width: 150px;
	}
	.row-label {
		font-size: 12px;
		color: var(--fg-muted);
	}
	select {
		padding: 6px 10px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		font-size: 12px;
		cursor: pointer;
	}
	@media (max-width: 640px) {
		.grid {
			grid-template-columns: 1fr;
		}
	}
</style>
