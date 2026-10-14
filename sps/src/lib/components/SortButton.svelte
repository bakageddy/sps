<script lang="ts">
	/**
	 * A column header that sorts. Inherits the header's font, colour and
	 * alignment so it drops into any existing header cell; the arrow is
	 * the only thing it adds. The parent owns the sort state.
	 */
	interface Props {
		label: string;
		/** this column is the current sort key */
		active: boolean;
		descending: boolean;
		onclick: () => void;
	}

	let { label, active, descending, onclick }: Props = $props();
</script>

<button class="sort" type="button" {onclick} title="Sort by {label}">
	{label}{#if active}<span class="arrow">{descending ? "▼" : "▲"}</span>{/if}
</button>

<style>
	.sort {
		padding: 0;
		background: none;
		border: none;
		border-radius: 0;
		font: inherit;
		color: inherit;
		text-align: inherit;
		cursor: pointer;
		white-space: nowrap;
	}
	.sort:hover {
		text-decoration: underline;
	}
	.arrow {
		margin-left: 3px;
		font-size: 9px;
		color: var(--accent);
	}
</style>
