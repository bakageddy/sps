import { resolveZone } from "$lib/timezone.svelte";

/**
 * Format a ms-epoch timestamp in the app's DISPLAY TIMEZONE, surviving
 * corrupt values.
 *
 * Components declare their formatters once (`new Intl.DateTimeFormat(…)`)
 * and never think about zones; this splices the resolved zone in by
 * rebuilding an equivalent formatter from `resolvedOptions()`, cached per
 * (formatter, zone) so the rebuild happens once per zone change, not per
 * row. Reading the zone here also makes every call site reactive to the
 * Ingest-page setting — no plumbing through props.
 *
 * Intl throws a RangeError past ±8.64e15 (the JS Date range) — and a u64
 * subtraction wrapping in a release-build parser produces exactly such
 * values. One bad row must label itself, not crash the page.
 */
const zoned = new WeakMap<
	Intl.DateTimeFormat,
	{ zone: string | undefined; fmt: Intl.DateTimeFormat }
>();

function inZone(fmt: Intl.DateTimeFormat, zone: string | undefined) {
	if (zone === undefined) return fmt;
	const hit = zoned.get(fmt);
	if (hit !== undefined && hit.zone === zone) return hit.fmt;
	// resolvedOptions() is typed loosely (weekday: string) though its values
	// are exactly the DateTimeFormatOptions that produced it — cast back
	const { locale, ...opts } = fmt.resolvedOptions();
	// hour12 overrides hourCycle in the constructor, and a resolved
	// `hour12: false` can come back as h24 ("24:05") — keep only the cycle
	if (opts.hourCycle !== undefined) delete (opts as { hour12?: boolean }).hour12;
	let rebuilt: Intl.DateTimeFormat;
	try {
		rebuilt = new Intl.DateTimeFormat(locale, {
			...(opts as Intl.DateTimeFormatOptions),
			timeZone: zone,
		});
	} catch {
		rebuilt = fmt; // unknown zone on this runtime: fall back to local
	}
	zoned.set(fmt, { zone, fmt: rebuilt });
	return rebuilt;
}

export function formatTimestamp(fmt: Intl.DateTimeFormat, ms: number): string {
	return Number.isFinite(ms) && Math.abs(ms) <= 8.64e15
		? inZone(fmt, resolveZone()).format(ms)
		: "corrupt ts";
}

/** Human-scale duration: 843 ms → "843 ms", 10222 → "10.2 s", 154000 → "2m 34s". */
export function formatDuration(ms: number): string {
	if (ms < 1000) return `${ms} ms`;
	if (ms < 60_000) return `${(ms / 1000).toFixed(1)} s`;
	const m = Math.floor(ms / 60_000);
	return `${m}m ${Math.round((ms % 60_000) / 1000)}s`;
}
