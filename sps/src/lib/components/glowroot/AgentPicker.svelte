<script lang="ts">
	/**
	 * Central only: choose the agent (or rollup) whose traces to pull. Lists
	 * the agents that reported inside the window chosen in the step above, so
	 * a long-gone agent does not clutter the list. A rollup (id ends "::") is
	 * selectable — listing spans it and every trace keeps its own agent id —
	 * and can also be expanded to pick one child.
	 */
	import { untrack } from "svelte";
	import { glowrootAgents, type Agent } from "$lib/api/glowroot";
	import { form, ready, resolveWindow, selectAgent, session } from "$lib/glowroot.svelte";

	let top = $state<Agent[]>([]);
	/** rollup id → its children, once loaded */
	let children = $state<Record<string, Agent[]>>({});
	let open = $state<Record<string, boolean>>({});
	let loading = $state(false);
	let error = $state<string | null>(null);
	let filter = $state("");

	async function load() {
		const w = resolveWindow();
		if (typeof w === "string") {
			error = w;
			return;
		}
		loading = true;
		error = null;
		children = {};
		open = {};
		try {
			top = await glowrootAgents(null, w[0], w[1]);
			// restore the remembered pick once, without a click
			const remembered = top.find((a) => a.id === form.agentId.value);
			if (remembered && session.agent === null) void selectAgent(remembered);
		} catch (e) {
			top = [];
			error = String(e);
		} finally {
			loading = false;
		}
	}

	async function toggle(a: Agent) {
		if (open[a.id]) {
			open[a.id] = false;
			return;
		}
		open[a.id] = true;
		if (children[a.id]) return;
		const w = resolveWindow();
		if (typeof w === "string") return;
		try {
			children[a.id] = await glowrootAgents(a.id, w[0], w[1]);
		} catch (e) {
			error = String(e);
			open[a.id] = false;
		}
	}

	// (re)load when we become usable, the collector changes, or a preset is
	// picked; typing custom bounds waits for the Reload button
	$effect(() => {
		void session.collector?.url;
		void form.preset.value;
		if (!ready() || !session.collector?.central) return;
		untrack(() => void load());
	});

	const needle = $derived(filter.trim().toLowerCase());
	const shown = $derived(
		needle === "" ? top : top.filter((a) => a.display.toLowerCase().includes(needle) || a.id.toLowerCase().includes(needle)),
	);
</script>

<div class="bar">
	<input
		type="search"
		placeholder="Filter agents…"
		bind:value={filter}
		aria-label="Filter agents"
	/>
	<button type="button" onclick={() => void load()} disabled={loading}>
		{loading ? "Loading…" : "Reload"}
	</button>
</div>

{#if error}<p class="error" role="alert">{error}</p>{/if}

<ul class="list">
	{#each shown as a (a.id)}
		{@render row(a, 0)}
		{#if open[a.id]}
			{#each children[a.id] ?? [] as c (c.id)}
				{@render row(c, 1)}
			{:else}
				<li class="empty" style:padding-left="34px">Loading…</li>
			{/each}
		{/if}
	{:else}
		<li class="empty">
			{loading ? "Loading…" : top.length === 0 ? "No agent reported in this window." : "No match."}
		</li>
	{/each}
</ul>

{#snippet row(a: Agent, depth: number)}
	<li>
		<div class="item" class:selected={session.agent?.id === a.id} style:padding-left="{10 + depth * 24}px">
			{#if a.hasChildren}
				<button
					type="button"
					class="chev"
					onclick={() => void toggle(a)}
					aria-label={open[a.id] ? "Collapse" : "Expand"}
					aria-expanded={!!open[a.id]}>{open[a.id] ? "▾" : "▸"}</button
				>
			{:else}
				<span class="chev"></span>
			{/if}
			<button type="button" class="pick" onclick={() => void selectAgent(a)} title={a.id}>
				<span class="name">{a.display || a.id}</span>
				{#if a.rollup}<span class="tag">rollup</span>{/if}
			</button>
		</div>
	</li>
{/snippet}

<style>
	.bar {
		display: flex;
		gap: 8px;
		margin-bottom: 8px;
	}
	input {
		flex: 1;
		min-width: 0;
		padding: 6px 10px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		font-size: 13px;
	}
	.bar button {
		padding: 6px 14px;
		font-size: 12px;
		background: var(--bg-hard);
	}
	.bar button:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	.list {
		margin: 0;
		padding: 0;
		list-style: none;
		max-height: 260px;
		overflow-y: auto;
		background: var(--bg-soft);
		border-radius: var(--radius);
	}
	.item {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 2px 10px 2px 10px;
	}
	.item:hover {
		background: var(--bg-hover);
	}
	.item.selected {
		background: color-mix(in srgb, var(--accent) 18%, transparent);
	}
	.chev {
		width: 20px;
		flex: none;
		padding: 2px 0;
		background: none;
		color: var(--accent);
		font-size: 11px;
		text-align: center;
	}
	.pick {
		flex: 1;
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 6px 4px;
		background: none;
		font-size: 13px;
		text-align: left;
		min-width: 0;
	}
	.name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tag {
		padding: 0 7px;
		border-radius: 99px;
		background: var(--bg-hard);
		color: var(--purple);
		font-size: 10px;
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
	.empty {
		padding: 14px;
		font-size: 12px;
		color: var(--fg-muted);
	}
	.error {
		margin: 0 0 8px;
		font-size: 12px;
		color: var(--red);
		white-space: pre-wrap;
	}
</style>
