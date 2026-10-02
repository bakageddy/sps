<script lang="ts">
	/**
	 * Tables and columns of the open database, for the SQL console: expand
	 * a table to see its columns; click a name to drop it into the editor.
	 */
	import type { SqlTable } from "$lib/api/sql";

	interface Props {
		tables: SqlTable[];
		/** insert text at the editor's caret */
		oninsert: (text: string) => void;
	}

	let { tables, oninsert }: Props = $props();

	let open = $state<Set<string>>(new Set());
	function toggle(name: string) {
		const next = new Set(open);
		if (next.has(name)) next.delete(name);
		else next.add(name);
		open = next;
	}

	let filter = $state("");
	const visible = $derived.by(() => {
		const q = filter.trim().toLowerCase();
		if (q === "") return tables;
		return tables.filter(
			(t) =>
				t.name.toLowerCase().includes(q) ||
				t.columns.some((c) => c.name.toLowerCase().includes(q)),
		);
	});

	const countFormat = new Intl.NumberFormat();
</script>

<div class="browser">
	<div class="tools">
		<input type="search" placeholder="filter tables / columns" bind:value={filter} />
	</div>
	<div class="list">
		{#each visible as t (t.name)}
			<div class="table">
				<div class="table-row">
					<button
						class="disclosure"
						onclick={() => toggle(t.name)}
						aria-expanded={open.has(t.name)}
						aria-label="Toggle columns of {t.name}"
						>{open.has(t.name) ? "▾" : "▸"}</button
					>
					<button
						class="name mono"
						onclick={() => oninsert(t.name)}
						title="Insert table name">{t.name}</button
					>
					<span class="count mono">{countFormat.format(t.rows)}</span>
				</div>
				{#if open.has(t.name)}
					<ul>
						{#each t.columns as c (c.name)}
							<li>
								<button
									class="col mono"
									onclick={() => oninsert(c.name)}
									title="Insert column name">{c.name}</button
								>
								<span class="type mono">{c.type}</span>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		{:else}
			<p class="empty">
				{tables.length === 0 ? "No tables — open a database." : "No match."}
			</p>
		{/each}
	</div>
</div>

<style>
	.browser {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.tools {
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.tools input {
		width: 100%;
		box-sizing: border-box;
		padding: 3px 8px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		color: var(--fg);
		font-size: 12px;
	}
	.list {
		overflow: auto;
		flex: 1;
		padding: 4px 0;
	}

	.table-row {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 0 6px;
	}
	.disclosure {
		width: 18px;
		padding: 0;
		color: var(--fg-muted);
		font-size: 11px;
	}
	.name {
		flex: 1;
		min-width: 0;
		padding: 3px 4px;
		text-align: left;
		color: var(--fg);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		border-radius: var(--radius);
	}
	.name:hover,
	.col:hover {
		background: var(--bg-hover);
		color: var(--accent);
	}
	.count {
		font-size: 10.5px;
		color: var(--fg-muted);
		padding-right: 4px;
	}

	ul {
		list-style: none;
		margin: 0 0 4px;
		padding: 0 6px 0 28px;
	}
	li {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.col {
		flex: 1;
		min-width: 0;
		padding: 1px 4px;
		text-align: left;
		color: var(--fg);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		border-radius: var(--radius);
	}
	.type {
		font-size: 10px;
		color: var(--fg-muted);
		white-space: nowrap;
	}
	.mono {
		font-family: var(--font-mono);
		font-size: 12px;
	}
	.empty {
		padding: 24px 12px;
		text-align: center;
		color: var(--fg-muted);
		font-size: 12.5px;
	}
</style>
