/**
 * Display timezone — app-wide, chosen on the Ingest page.
 *
 * Log timestamps are the server's wall clock; the browser's locale is
 * usually somewhere else. This module resolves ONE IANA zone (or undefined
 * = browser local) that every timestamp formatter applies via
 * lib/format.formatTimestamp — components keep declaring their formatters
 * exactly as before, the zone is spliced in at format time.
 *
 * Modes: "auto" follows the bundle (HealthMeter's Server Time zone when
 * detected, else local); "local" / "utc" are fixed; "custom" is whatever
 * the user typed or pasted (validated against Intl before use).
 */
import { persisted } from "$lib/persisted.svelte";

export type TzMode = "auto" | "local" | "utc" | "custom";

export const tzMode = persisted<TzMode>("display-tz-mode", "auto");
export const tzCustom = persisted<string>("display-tz-custom", "");
/** zone the last bundle's HealthMeter declared, or null */
export const tzDetected = persisted<string | null>("display-tz-detected", null);
/** the Server Time text that came with it, for the Ingest page to show */
export const tzDetectedAt = persisted<string | null>("display-tz-detected-at", null);

/** Intl knows this zone (IANA names, "UTC", and "+05:30"-style offsets) */
export function isValidZone(zone: string): boolean {
	try {
		new Intl.DateTimeFormat(undefined, { timeZone: zone });
		return true;
	} catch {
		return false;
	}
}

/**
 * Pull a zone out of pasted text — the whole HealthMeter "Server Time"
 * cell, a bare "Asia/Kolkata", "UTC", or a "+05:30" offset — or null.
 */
export function extractZone(text: string): string | null {
	const candidates = [
		...text.matchAll(/[A-Z][A-Za-z]+(?:\/[A-Za-z_+\-]+)+/g),
		...text.matchAll(/\b(?:UTC|GMT|Etc\/[A-Za-z0-9+\-]+)\b/g),
		...text.matchAll(/[+\-]\d{2}:?\d{2}\b/g),
		// bare abbreviations (IST, EST, PST, ...) — Intl accepts these as
		// legacy aliases even though they're not in the canonical
		// supportedValuesOf() list (verified: IST resolves to Asia/Calcutta).
		// A/PM and other short all-caps noise in the same string (e.g. the
		// "PM" in "04:39 PM  IST") are harmless here: isValidZone() below
		// rejects them (confirmed: Intl rejects "AM"/"PM" outright), so only
		// a genuine zone abbreviation ever survives this candidate list.
		...text.matchAll(/\b[A-Z]{2,5}\b/g),
	].map((m) => m[0]);
	for (const c of candidates) {
		const z = /^[+\-]\d{4}$/.test(c) ? `${c.slice(0, 3)}:${c.slice(3)}` : c;
		if (isValidZone(z)) return z;
	}
	return null;
}

/** the zone every formatter should use right now; undefined = browser local */
export function resolveZone(): string | undefined {
	switch (tzMode.value) {
		case "local":
			return undefined;
		case "utc":
			return "UTC";
		case "custom":
			return tzCustom.value !== "" && isValidZone(tzCustom.value)
				? tzCustom.value
				: undefined;
		case "auto":
		default:
			return tzDetected.value !== null && isValidZone(tzDetected.value)
				? tzDetected.value
				: undefined;
	}
}

/** "GMT+05:30" for a zone at a given instant (offsets vary with DST) */
export function zoneOffsetLabel(zone: string | undefined, at = Date.now()): string {
	try {
		const parts = new Intl.DateTimeFormat("en-US", {
			timeZone: zone,
			timeZoneName: "longOffset",
		}).formatToParts(at);
		return parts.find((p) => p.type === "timeZoneName")?.value ?? "";
	} catch {
		return "";
	}
}

/** minutes east of UTC for `zone` (browser local when undefined) at `at` */
function zoneOffsetMinutes(zone: string | undefined, at: number): number {
	if (zone === undefined) return -new Date(at).getTimezoneOffset();
	const label = zoneOffsetLabel(zone, at); // "GMT+05:30" / "GMT-4" / "GMT"
	const m = /GMT([+-])(\d{1,2})(?::(\d{2}))?/.exec(label);
	if (m === null) return 0;
	const sign = m[1] === "-" ? -1 : 1;
	return sign * (Number(m[2]) * 60 + Number(m[3] ?? 0));
}

/** y/m/d of an instant as seen in `zone` */
export function zonedDate(ms: number, zone: string | undefined): [number, number, number] {
	const parts = new Intl.DateTimeFormat("en-US", {
		timeZone: zone,
		year: "numeric",
		month: "numeric",
		day: "numeric",
	}).formatToParts(ms);
	const get = (t: string) => Number(parts.find((p) => p.type === t)?.value);
	return [get("year"), get("month"), get("day")];
}

/**
 * Parse what a person types for a moment — "14:32", "14:32:05",
 * "2026-09-19 14:32:05", "2026-09-19T14:32", or anything Date() accepts
 * ("Sep 19, 2026 21:07:25") — as WALL-CLOCK TIME IN THE DISPLAY ZONE,
 * returning ms epoch. A bare time takes its date from `refMs` (the middle
 * of the bundle, normally). null when it isn't a time.
 */
export function parseZonedTime(text: string, refMs: number): number | null {
	const zone = resolveZone();
	const v = text.trim();
	if (v === "") return null;
	let y: number, mo: number, d: number, h: number, mi: number, sec: number;
	let m = /^(\d{1,2}):(\d{2})(?::(\d{2}))?$/.exec(v);
	if (m !== null) {
		[y, mo, d] = zonedDate(refMs, zone);
		[h, mi, sec] = [Number(m[1]), Number(m[2]), Number(m[3] ?? 0)];
	} else if (
		(m = /^(\d{4})-(\d{2})-(\d{2})[ T](\d{1,2}):(\d{2})(?::(\d{2}))?$/.exec(v)) !== null
	) {
		[y, mo, d, h, mi, sec] = [1, 2, 3, 4, 5, 6].map((i) => Number(m![i] ?? 0)) as [
			number, number, number, number, number, number,
		];
	} else {
		// Date() parses in LOCAL time; re-read its wall-clock fields and
		// reinterpret them in the display zone below
		const local = new Date(v);
		if (!Number.isFinite(local.getTime())) return null;
		[y, mo, d, h, mi, sec] = [
			local.getFullYear(), local.getMonth() + 1, local.getDate(),
			local.getHours(), local.getMinutes(), local.getSeconds(),
		];
	}
	if (h > 23 || mi > 59 || sec > 59) return null;
	const naive = Date.UTC(y, mo - 1, d, h, mi, sec);
	return naive - zoneOffsetMinutes(zone, naive) * 60_000;
}

/** start of the calendar day containing `ms`, in the display zone (ms epoch) */
export function dayStart(ms: number): number {
	const zone = resolveZone();
	const [y, mo, d] = zonedDate(ms, zone);
	const naive = Date.UTC(y, mo - 1, d);
	return naive - zoneOffsetMinutes(zone, naive) * 60_000;
}
