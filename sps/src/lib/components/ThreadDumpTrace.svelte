<script lang="ts" module>
	import type { ThreadDumpElement } from "$lib/api/threaddump";

	/** Same shape as StackTracePanel's TraceState, with interleaved elements. */
	export type ThreadTraceState =
		| { status: "idle" }
		| { status: "loading"; tid: number; timestamp: number }
		| { status: "error"; message: string }
		| {
				status: "ready";
				tid: number;
				timestamp: number;
				/** null = the thread was dumped without a stack */
				elements: ThreadDumpElement[] | null;
		  };
</script>

<script lang="ts">
	/**
	 * One thread's stack with its lock lines IN PLACE — "- locked <obj>"
	 * under the frame that took it, "- waiting to lock" under the frame
	 * that's stuck — because where a lock sits in the stack is the story.
	 */
	import { copyText } from "$lib/clipboard";
	import { formatTimestamp } from "$lib/format";
	import Icon from "$lib/components/Icon.svelte";

	interface Props {
		trace: ThreadTraceState;
	}

	let { trace }: Props = $props();

	let copied = $state(false);

	const timeFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});

	function asText(elements: ThreadDumpElement[]): string {
		return elements
			.map((e) =>
				e.kind === "frame"
					? `\tat ${e.method}(${e.source})`
					: `\t- ${e.object}`,
			)
			.join("\n");
	}

	async function copy() {
		if (trace.status !== "ready" || trace.elements === null) return;
		if (await copyText(asText(trace.elements))) {
			copied = true;
			setTimeout(() => (copied = false), 1500);
		}
	}
</script>

<div class="panel">
	<div class="head">
		<span class="title">Stack trace</span>
		{#if trace.status === "ready" || trace.status === "loading"}
			<span class="mono where"
				>tid {trace.tid} · {formatTimestamp(timeFormat, trace.timestamp)}</span
			>
		{/if}
		{#if trace.status === "ready" && trace.elements !== null}
			<button
				class="copy"
				onclick={copy}
				title="Copy stack"
				aria-label="Copy stack"
				><Icon name={copied ? "check" : "copy"} size={12} /></button
			>
		{/if}
	</div>

	{#if trace.status === "idle"}
		<p class="hint">Click a thread to see its stack.</p>
	{:else if trace.status === "loading"}
		<p class="hint">Loading…</p>
	{:else if trace.status === "error"}
		<p class="hint error">{trace.message}</p>
	{:else if trace.elements === null}
		<p class="hint">This thread was dumped without a stack trace.</p>
	{:else}
		<!-- unkeyed on purpose: elements have no identity of their own -->
		<ol class="elements">
			{#each trace.elements as e}
				{#if e.kind === "frame"}
					<li class="frame">
						<span class="method">{e.method}</span>
						<span class="source">({e.source})</span>
					</li>
				{:else}
					<li class="lock">
						<span class="dash">—</span> {e.object}
					</li>
				{/if}
			{/each}
		</ol>
	{/if}
</div>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		height: 100%;
		min-height: 0;
	}
	.head {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 10px;
		border-bottom: 1px solid var(--hairline);
		flex-shrink: 0;
	}
	.title {
		font-size: 11px;
		font-weight: 600;
		text-transform: uppercase;
		letter-spacing: 0.07em;
		color: var(--fg-muted);
	}
	.where {
		font-size: 11px;
		color: var(--fg-muted);
	}
	.mono {
		font-family: var(--font-mono);
	}
	.copy {
		display: grid;
		place-items: center;
		margin-left: auto;
		padding: 4px;
		color: var(--fg-muted);
	}
	.copy:hover {
		background: var(--bg-hover);
		color: var(--fg);
	}
	.hint {
		padding: 16px 12px;
		font-size: 12.5px;
		color: var(--fg-muted);
	}
	.hint.error {
		color: var(--red);
	}
	.elements {
		flex: 1;
		margin: 0;
		padding: 8px 12px 8px 36px;
		overflow: auto;
		font-family: var(--font-mono);
		font-size: 11.5px;
		white-space: nowrap;
	}
	.frame .method {
		color: var(--fg);
	}
	.frame .source {
		color: var(--fg-muted);
		margin-left: 6px;
	}
	.lock {
		list-style: none;
		margin-left: -14px;
		color: var(--yellow);
	}
	.dash {
		color: var(--fg-muted);
	}
</style>
