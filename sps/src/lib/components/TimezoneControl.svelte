<script lang="ts">
	/**
	 * Display timezone, set once per bundle on the Ingest page.
	 *
	 * Auto follows HealthMeter.html (asked from the backend after every
	 * parse, beside the dropped path); the other modes pin a zone. Custom
	 * takes a typed IANA name OR a pasted HealthMeter "Server Time" cell —
	 * the zone is extracted and validated against Intl before it applies.
	 * A live preview shows what "now" looks like in the chosen zone.
	 */
	import { healthmeterInfo } from "$lib/api/healthmeter";
	import { ingest } from "$lib/ingest.svelte";
	import {
		tzMode,
		tzCustom,
		tzDetected,
		tzDetectedAt,
		extractZone,
		isValidZone,
		resolveZone,
		zoneOffsetLabel,
		type TzMode,
	} from "$lib/timezone.svelte";
	import { formatTimestamp } from "$lib/format";

	let detectError = $state<string | null>(null);
	let pasted = $state("");
	let pasteNote = $state<string | null>(null);

	// after each finished run, read the bundle's declared zone
	$effect(() => {
		if (ingest.generation === 0 || ingest.lastPath === null) return;
		const path = ingest.lastPath;
		detectError = null;
		healthmeterInfo(path).then(
			(info) => {
				if (info === null) return;
				tzDetected.value = info.timezone;
				tzDetectedAt.value = info.serverTime;
			},
			(e) => (detectError = String(e)),
		);
	});

	function applyPaste() {
		const zone = extractZone(pasted);
		if (zone === null) {
			pasteNote = "no timezone found in that text";
			return;
		}
		tzCustom.value = zone;
		tzMode.value = "custom";
		pasteNote = `using ${zone}`;
		pasted = "";
	}

	const customValid = $derived(
		tzCustom.value === "" || isValidZone(tzCustom.value),
	);

	const previewFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
	// re-evaluated when any of the persisted knobs change (resolveZone
	// reads them) — the preview is the proof the setting took
	const preview = $derived.by(() => {
		const zone = resolveZone();
		const now = Date.now();
		return {
			zone: zone ?? "browser local",
			offset: zoneOffsetLabel(zone, now),
			now: formatTimestamp(previewFormat, now),
		};
	});

	const modes: { value: TzMode; label: string }[] = [
		{ value: "auto", label: "Bundle" },
		{ value: "local", label: "Local" },
		{ value: "utc", label: "UTC" },
		{ value: "custom", label: "Custom" },
	];
</script>

<section class="tz">
	<header>
		<span class="preview mono" title="What 'now' looks like in the chosen zone">
			{preview.now} · {preview.zone}
			{#if preview.offset}({preview.offset}){/if}
		</span>
	</header>

	<div class="modes" role="radiogroup" aria-label="Display timezone">
		{#each modes as m (m.value)}
			<label class:active={tzMode.value === m.value}>
				<input type="radio" name="tz-mode" value={m.value} bind:group={tzMode.value} />
				{m.label}
				{#if m.value === "auto"}
					<span class="hint mono">
						{#if tzDetected.value}
							{tzDetected.value}
						{:else}
							not detected → local
						{/if}
					</span>
				{/if}
			</label>
		{/each}
	</div>

	{#if tzMode.value === "auto" && tzDetected.value && tzDetectedAt.value}
		<p class="note">
			HealthMeter says the server was at <span class="mono">{tzDetectedAt.value}</span>
			in <span class="mono">{tzDetected.value}</span>.
		</p>
	{/if}
	{#if detectError}
		<p class="note error">HealthMeter lookup failed: {detectError}</p>
	{/if}

	{#if tzMode.value === "custom"}
		<div class="custom">
			<label>
				zone
				<input
					type="text"
					placeholder="Asia/Kolkata, UTC, +05:30"
					bind:value={tzCustom.value}
					class:invalid={!customValid}
					spellcheck="false"
				/>
			</label>
			{#if !customValid}
				<span class="note error">Intl doesn't know that zone</span>
			{/if}
		</div>
	{/if}

	<div class="paste">
		<input
			type="text"
			placeholder="or paste the HealthMeter “Server Time” cell here (Sep 10, 2026 05:20 PM  Asia/Kolkata)"
			bind:value={pasted}
			onkeydown={(e) => e.key === "Enter" && applyPaste()}
			spellcheck="false"
		/>
		<button onclick={applyPaste} disabled={pasted.trim() === ""}>use</button>
		{#if pasteNote}
			<span class="note mono">{pasteNote}</span>
		{/if}
	</div>
</section>

<style>
	.tz {
		padding: 12px 14px;
		background: var(--bg-soft);
		border-radius: var(--radius);
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	header {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		flex-wrap: wrap;
	}
	.preview {
		font-size: 11.5px;
		color: var(--green);
	}

	.modes {
		display: flex;
		gap: 2px;
		padding: 2px;
		background: var(--bg-hard);
		border-radius: 999px;
		align-self: flex-start;
		flex-wrap: wrap;
	}
	.modes label {
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 2px 12px;
		border-radius: 999px;
		font-size: 11.5px;
		color: var(--fg-muted);
		cursor: pointer;
	}
	.modes label:hover {
		color: var(--fg);
	}
	.modes label.active {
		background: var(--accent);
		color: var(--bg-hard);
		font-weight: 600;
	}
	.modes input {
		/* the pill IS the control; keep the radio for a11y, hide the dot */
		position: absolute;
		opacity: 0;
		width: 0;
		height: 0;
	}
	.hint {
		font-size: 10.5px;
		font-weight: 400;
		opacity: 0.85;
	}

	.custom,
	.paste {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.custom label {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	input[type="text"] {
		padding: 3px 8px;
		background: var(--bg-hard);
		border: 1px solid transparent;
		border-radius: var(--radius);
		color: var(--fg);
		font-family: var(--font-mono);
		font-size: 11.5px;
	}
	.custom input {
		width: 200px;
	}
	.paste input {
		flex: 1;
		min-width: 240px;
	}
	input.invalid {
		border-color: var(--alert);
	}
	.paste button {
		padding: 2px 12px;
		border-radius: 999px;
		background: var(--bg-hard);
		color: var(--accent);
		font-size: 11.5px;
		font-weight: 600;
	}
	.paste button:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	.paste button:disabled {
		color: var(--fg-muted);
		opacity: 0.6;
	}

	.note {
		margin: 0;
		font-size: 11.5px;
		color: var(--fg-muted);
	}
	.note.error {
		color: var(--alert);
	}
	.mono {
		font-family: var(--font-mono);
	}
</style>
