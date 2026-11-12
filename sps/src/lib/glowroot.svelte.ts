/**
 * Glowroot ingest state — everything the /glowroot page and its components
 * share, so a run keeps going (and keeps reporting) while the user is on
 * another page.
 *
 * Three groups:
 *  - the FORM: one persisted() value per field (bindable with
 *    `bind:value={form.x.value}`; persisted() only saves on assignment, so a
 *    single object holding all fields would silently not save nested edits);
 *  - the SESSION: collector, chosen agent, its layout, trace counts;
 *  - the RUN: folded from the glowroot:* events (api/glowroot-events.ts).
 *    Those carry cumulative totals from a single writer, so folding is plain
 *    assignment. The listeners are registered once at module init and never
 *    torn down, like lib/ingest.svelte.ts.
 */
import { listen } from "@tauri-apps/api/event";
import { persisted } from "$lib/persisted.svelte";
import { ensureOpen } from "$lib/database.svelte";
import { parseZonedTime } from "$lib/timezone.svelte";
import {
	glowrootAgentLayout,
	glowrootCancel,
	glowrootConnect,
	glowrootCount,
	glowrootDisconnect,
	glowrootIngest,
	glowrootLogin,
	type Agent,
	type AgentLayout,
	type Collector,
	type Comparator,
	type TextFilter,
	type IngestOptions,
	type TraceCount,
} from "$lib/api/glowroot";
import {
	GlowrootEvent,
	type GlowrootFinished,
	type GlowrootListed,
	type GlowrootListing,
	type GlowrootProblem,
	type GlowrootProgress,
	type GlowrootStarted,
} from "$lib/api/glowroot-events";

// ── form ────────────────────────────────────────────────────────────────

export type RangePreset = "1h" | "2h" | "4h" | "8h" | "24h" | "2d" | "7d" | "custom";

export const PRESETS: { id: Exclude<RangePreset, "custom">; label: string; ms: number }[] = [
	{ id: "1h", label: "1h", ms: 3_600_000 },
	{ id: "2h", label: "2h", ms: 2 * 3_600_000 },
	{ id: "4h", label: "4h", ms: 4 * 3_600_000 },
	{ id: "8h", label: "8h", ms: 8 * 3_600_000 },
	{ id: "24h", label: "24h", ms: 24 * 3_600_000 },
	{ id: "2d", label: "2d", ms: 48 * 3_600_000 },
	{ id: "7d", label: "7d", ms: 7 * 24 * 3_600_000 },
];

export const MAX_WORKERS = 32;

export const form = {
	protocol: persisted<"http" | "https">("glowroot-protocol", "http"),
	/** hostname, optionally followed by a context path ("host/glowroot") */
	host: persisted("glowroot-host", ""),
	/** "" = the protocol's default */
	port: persisted("glowroot-port", ""),
	/** connect on its own once typing has paused */
	autoConnect: persisted("glowroot-auto-connect", true),
	advancedOpen: persisted("glowroot-advanced-open", false),
	username: persisted("glowroot-username", ""),
	preset: persisted<RangePreset>("glowroot-preset", "4h"),
	/** custom bounds, typed in the display zone ("YYYY-MM-DD HH:MM[:SS]" or "HH:MM") */
	customFrom: persisted("glowroot-custom-from", ""),
	customTo: persisted("glowroot-custom-to", ""),
	types: persisted<string[]>("glowroot-types", []),
	transactionName: persisted("glowroot-name", ""),
	/** text filters: empty text = off; the comparator is remembered regardless */
	headline: persisted("glowroot-headline", ""),
	headlineCmp: persisted<Comparator>("glowroot-headline-cmp", "contains"),
	user: persisted("glowroot-user", ""),
	userCmp: persisted<Comparator>("glowroot-user-cmp", "equals"),
	errorMessage: persisted("glowroot-error-message", ""),
	errorMessageCmp: persisted<Comparator>("glowroot-error-message-cmp", "contains"),
	attributeName: persisted("glowroot-attribute-name", ""),
	attributeValue: persisted("glowroot-attribute-value", ""),
	attributeCmp: persisted<Comparator>("glowroot-attribute-cmp", "equals"),
	slow: persisted("glowroot-slow", true),
	error: persisted("glowroot-error", true),
	/** duration band in SECONDS, text so a half-typed value survives; "" = unbounded */
	minSeconds: persisted("glowroot-min-s", ""),
	maxSeconds: persisted("glowroot-max-s", ""),
	maxTraces: persisted("glowroot-max-traces", 10_000),
	workers: persisted("glowroot-workers", 8),
	entries: persisted("glowroot-entries", true),
	queries: persisted("glowroot-queries", true),
	profiles: persisted("glowroot-profiles", true),
	skipExisting: persisted("glowroot-skip-existing", true),
	agentId: persisted("glowroot-agent", ""),
};

/**
 * The collector URL the connection fields spell, or what is wrong with them.
 * The host may carry a context path ("host/glowroot"); the port goes between
 * the hostname and that path.
 */
export function composeUrl(): { url: string } | { error: string } {
	const raw = form.host.value.trim();
	if (raw === "") return { error: "Enter the collector's host" };
	if (/\s/.test(raw)) return { error: "The host has a space in it" };
	const slash = raw.indexOf("/");
	const hostname = slash < 0 ? raw : raw.slice(0, slash);
	const path = slash < 0 ? "" : raw.slice(slash).replace(/\/+$/, "");
	if (hostname === "") return { error: "Enter the collector's host" };
	const port = form.port.value.trim();
	if (port !== "" && !(/^\d{1,5}$/.test(port) && Number(port) >= 1 && Number(port) <= 65535))
		return { error: "Port must be a number from 1 to 65535" };
	return { url: `${form.protocol.value}://${hostname}${port === "" ? "" : `:${port}`}${path}` };
}

/**
 * Edit of the host field: a pasted URL ("https://host:4000/glowroot") or a
 * "host:port" is split across protocol, host and port instead of living in
 * the host box. Returns what the host box should now hold.
 */
export function normaliseHostInput(text: string): string {
	const t = text.trim();
	const full = /^(https?):\/\/([^/\s:]+)(?::(\d{1,5}))?(\/\S*)?$/i.exec(t);
	if (full) {
		form.protocol.value = full[1].toLowerCase() as "http" | "https";
		form.port.value = full[3] ?? "";
		return full[2] + (full[4] && full[4] !== "/" ? full[4].replace(/\/+$/, "") : "");
	}
	const hp = /^([^/\s:]+):(\d{1,5})(\/\S*)?$/.exec(t);
	if (hp) {
		form.port.value = hp[2];
		return hp[1] + (hp[3] && hp[3] !== "/" ? hp[3].replace(/\/+$/, "") : "");
	}
	return text;
}

/**
 * The [from, to] window the form currently means, resolved to concrete
 * instants. A preset is "now minus N"; resolving at call time (not
 * remembering "to=0") keeps a run reproducible and recorded as-is in history.
 */
export function resolveWindow(): [number, number] | string {
	const now = Date.now();
	if (form.preset.value !== "custom") {
		const p = PRESETS.find((x) => x.id === form.preset.value) ?? PRESETS[2];
		return [now - p.ms, now];
	}
	const from = parseZonedTime(form.customFrom.value, now);
	const to = parseZonedTime(form.customTo.value, now);
	if (from === null) return "“From” is not a time (try 2026-11-07 09:30 or 09:30)";
	if (to === null) return "“To” is not a time (try 2026-11-07 09:30 or 09:30)";
	if (from >= to) return "“From” must be before “To”";
	return [from, to];
}

/** "" → null, otherwise a non-negative finite number of seconds, or undefined if bad */
function parseSeconds(text: string): number | null | undefined {
	const t = text.trim();
	if (t === "") return null;
	const n = Number(t);
	return Number.isFinite(n) && n >= 0 ? n : undefined;
}

/** a text filter from its form fields, or null when the text is empty */
function textFilter(text: string, comparator: Comparator): TextFilter | null {
	const t = text.trim();
	return t === "" ? null : { text: t, comparator };
}

/** how many of the four text filters are in use (for the folded summary) */
export function filterCount(): number {
	return [
		form.headline.value,
		form.user.value,
		form.errorMessage.value,
		form.attributeName.value,
	].filter((t) => t.trim() !== "").length;
}

/** "headline contains “x”" per filter in use, for history rows */
export function describeFilters(o: IngestOptions): string[] {
	const out: string[] = [];
	const one = (label: string, f: TextFilter | null | undefined) => {
		if (f) out.push(`${label} ${f.comparator.replace("-", " ")} “${f.text}”`);
	};
	one("headline", o.headline);
	one("user", o.user);
	one("error", o.errorMessage);
	if (o.attribute) one(`attr ${o.attribute.name}`, o.attribute.value);
	return out;
}

/** Validate the form into the backend's options, or say what is wrong. */
export function buildOptions(): { options: IngestOptions } | { error: string } {
	const w = resolveWindow();
	if (typeof w === "string") return { error: w };
	const low = parseSeconds(form.minSeconds.value);
	const high = parseSeconds(form.maxSeconds.value);
	if (low === undefined) return { error: "Minimum duration is not a number of seconds" };
	if (high === undefined) return { error: "Maximum duration is not a number of seconds" };
	if (high !== null && high <= (low ?? 0))
		return { error: "Maximum duration must be above the minimum" };
	if (form.types.value.length === 0) return { error: "Pick at least one transaction type" };
	if (!form.slow.value && !form.error.value)
		return { error: "Pick slow traces, error traces, or both" };
	const maxTraces = Math.floor(Number(form.maxTraces.value));
	if (!Number.isFinite(maxTraces) || maxTraces < 1)
		return { error: "Max traces must be at least 1" };
	const workers = Math.min(MAX_WORKERS, Math.max(1, Math.floor(Number(form.workers.value) || 1)));
	const name = form.transactionName.value.trim();
	const attrName = form.attributeName.value.trim();
	const attrValue = form.attributeValue.value.trim();
	if (attrName !== "" && attrValue === "")
		return { error: "The attribute filter needs a value to compare" };
	// a part the user may not read would only produce 403s: turn it off
	const perm = session.layout?.permissions;
	return {
		options: {
			agentRollupId: session.collector?.central ? form.agentId.value : "",
			transactionTypes: [...form.types.value],
			transactionName: name === "" ? null : name,
			headline: textFilter(form.headline.value, form.headlineCmp.value),
			user: textFilter(form.user.value, form.userCmp.value),
			errorMessage: textFilter(form.errorMessage.value, form.errorMessageCmp.value),
			attribute:
				attrName === ""
					? null
					: { name: attrName, value: { text: attrValue, comparator: form.attributeCmp.value } },
			slow: form.slow.value,
			error: form.error.value,
			from: w[0],
			to: w[1],
			durationMillisLow: Math.round((low ?? 0) * 1000),
			durationMillisHigh: high === null ? null : Math.round(high * 1000),
			maxTraces,
			workers,
			entries: form.entries.value && (perm?.trace ?? true),
			queries: form.queries.value && (perm?.queries ?? true),
			profiles: form.profiles.value && (perm?.threadProfile ?? true),
			skipExisting: form.skipExisting.value,
		},
	};
}

// ── session ─────────────────────────────────────────────────────────────

export type Busy = "connect" | "login" | "layout" | "count" | null;

export const session = $state<{
	collector: Collector | null;
	/** the chosen agent (Central); null on embedded or before a pick */
	agent: Agent | null;
	layout: AgentLayout | null;
	counts: TraceCount[] | null;
	busy: Busy;
	/** the URL the last connect attempt used, so auto-connect does not repeat itself */
	attempted: string | null;
	/** last failure of connect/login/layout, shown under the step it belongs to */
	error: string | null;
	countError: string | null;
}>({
	collector: null,
	agent: null,
	layout: null,
	counts: null,
	busy: null,
	attempted: null,
	error: null,
	countError: null,
});

/** Whether the page can move past the Connect step: reachable and, if the
 *  collector demands it, logged in. */
export function ready(): boolean {
	const c = session.collector;
	return c !== null && (!c.needsLogin || c.loggedIn);
}

function resetBelowConnect(): void {
	session.agent = null;
	session.layout = null;
	session.counts = null;
	session.countError = null;
}

/** only the newest connect may write its result: typing keeps changing the URL */
let connectSeq = 0;

/** Connect to what the connection fields say. Returns once settled. */
export async function connect(): Promise<void> {
	const target = composeUrl();
	if ("error" in target) {
		session.error = target.error;
		return;
	}
	const mine = ++connectSeq;
	session.busy = "connect";
	session.error = null;
	session.attempted = target.url;
	resetBelowConnect();
	try {
		const c = await glowrootConnect(target.url);
		if (mine !== connectSeq) return;
		session.collector = c;
		// embedded has one implicit agent; Central waits for a pick
		if (!c.central && (!c.needsLogin || c.loggedIn)) await loadLayout("");
	} catch (e) {
		if (mine !== connectSeq) return;
		session.collector = null;
		session.error = String(e);
	} finally {
		if (mine === connectSeq) session.busy = null;
	}
}

export async function login(password: string): Promise<void> {
	session.busy = "login";
	session.error = null;
	try {
		const c = await glowrootLogin(form.username.value, password);
		session.collector = c;
		if (!c.central && (!c.needsLogin || c.loggedIn)) await loadLayout("");
	} catch (e) {
		session.error = String(e);
	} finally {
		session.busy = null;
	}
}

export async function disconnect(): Promise<void> {
	connectSeq += 1; // an attempt still in flight must not resurrect the session
	session.busy = null;
	session.attempted = null;
	await glowrootDisconnect().catch(() => {});
	session.collector = null;
	session.error = null;
	resetBelowConnect();
}

/** Pick (Central) an agent or rollup and load its layout. */
export async function selectAgent(agent: Agent): Promise<void> {
	session.agent = agent;
	form.agentId.value = agent.id;
	await loadLayout(agent.id);
}

async function loadLayout(agentRollupId: string): Promise<void> {
	session.busy = "layout";
	session.layout = null;
	session.counts = null;
	session.countError = null;
	try {
		const layout = await glowrootAgentLayout(agentRollupId);
		session.layout = layout;
		// keep the remembered types that still exist; otherwise default to
		// Web + Schedule (the point of this feature), else everything
		const kept = form.types.value.filter((t) => layout.transactionTypes.includes(t));
		if (kept.length > 0) form.types.value = kept;
		else {
			const wanted = layout.transactionTypes.filter((t) => t === "Web" || t === "Schedule");
			form.types.value = wanted.length > 0 ? wanted : [...layout.transactionTypes];
		}
	} catch (e) {
		session.error = String(e);
	} finally {
		session.busy = null;
	}
}

export async function count(): Promise<void> {
	const built = buildOptions();
	session.counts = null;
	if ("error" in built) {
		session.countError = built.error;
		return;
	}
	session.busy = "count";
	session.countError = null;
	try {
		session.counts = await glowrootCount(built.options);
	} catch (e) {
		session.countError = String(e);
	} finally {
		session.busy = null;
	}
}

// ── run ─────────────────────────────────────────────────────────────────

export type RunPhase = "idle" | "starting" | "listing" | "fetching" | "finished";

/** one progress observation, for the rate window */
interface Sample {
	/** performance.now() */
	t: number;
	fetched: number;
}

/** how far back the rate looks: long enough to smooth bursts, short enough
 *  to follow a slowdown */
const RATE_WINDOW_MS = 15_000;
/** the problem list is capped here too (the backend caps its own at 200) */
const MAX_PROBLEMS = 200;

export const run = $state<{
	id: number | null;
	phase: RunPhase;
	/** listing: points seen so far */
	found: number;
	total: number;
	skippedExisting: number;
	capped: boolean;
	fetched: number;
	stored: number;
	failed: number;
	inFlight: number;
	problems: GlowrootProblem[];
	/** Date.now() at glowroot:started */
	startedAt: number | null;
	samples: Sample[];
	result: GlowrootFinished | null;
	/** glowroot_ingest itself rejected (validation) — nothing ran */
	startError: string | null;
}>({
	id: null,
	phase: "idle",
	found: 0,
	total: 0,
	skippedExisting: 0,
	capped: false,
	fetched: 0,
	stored: 0,
	failed: 0,
	inFlight: 0,
	problems: [],
	startedAt: null,
	samples: [],
	result: null,
	startError: null,
});

/** bumps whenever a run starts or finishes; RunHistory refetches on it */
export const history = $state({ version: 0 });

export function running(): boolean {
	return run.phase === "starting" || run.phase === "listing" || run.phase === "fetching";
}

function resetRun(): void {
	run.id = null;
	run.phase = "starting";
	run.found = 0;
	run.total = 0;
	run.skippedExisting = 0;
	run.capped = false;
	run.fetched = 0;
	run.stored = 0;
	run.failed = 0;
	run.inFlight = 0;
	run.problems = [];
	run.startedAt = null;
	run.samples = [];
	run.result = null;
	run.startError = null;
}

/** an event belongs to the run we are tracking (or adopts it: the id is
 *  unknown until glowroot_ingest resolves, and started can beat that) */
function ours(runId: number): boolean {
	if (run.phase === "idle" || run.phase === "finished") return false;
	if (run.id === null) {
		run.id = runId;
		return true;
	}
	return run.id === runId;
}

listen<GlowrootStarted>(GlowrootEvent.Started, (e) => {
	if (!ours(e.payload.runId)) return;
	run.startedAt = Date.now();
	run.phase = "listing";
	history.version += 1;
});

listen<GlowrootListing>(GlowrootEvent.Listing, (e) => {
	if (!ours(e.payload.runId)) return;
	run.phase = "listing";
	run.found = e.payload.found;
});

listen<GlowrootListed>(GlowrootEvent.Listed, (e) => {
	if (!ours(e.payload.runId)) return;
	run.total = e.payload.total;
	run.skippedExisting = e.payload.skippedExisting;
	run.capped = e.payload.capped;
	run.phase = "fetching";
});

listen<GlowrootProgress>(GlowrootEvent.Progress, (e) => {
	const p = e.payload;
	if (!ours(p.runId)) return;
	run.phase = "fetching";
	run.fetched = p.fetched;
	run.stored = p.stored;
	run.failed = p.failed;
	run.inFlight = p.inFlight;
	const now = performance.now();
	run.samples.push({ t: now, fetched: p.fetched });
	while (run.samples.length > 2 && now - run.samples[0].t > RATE_WINDOW_MS) run.samples.shift();
});

listen<GlowrootProblem>(GlowrootEvent.Problem, (e) => {
	if (!ours(e.payload.runId)) return;
	if (run.problems.length < MAX_PROBLEMS) run.problems.push(e.payload);
});

listen<GlowrootFinished>(GlowrootEvent.Finished, (e) => {
	const f = e.payload;
	if (!ours(f.runId)) return;
	run.total = f.total;
	run.stored = f.stored;
	run.failed = f.failed;
	run.fetched = f.stored + f.failed;
	run.inFlight = 0;
	run.result = f;
	run.phase = "finished";
	history.version += 1;
});

/** traces per second over the recent window, or null until it can be told */
export function rate(): number | null {
	const s = run.samples;
	if (s.length < 2) return null;
	const dt = (s[s.length - 1].t - s[0].t) / 1000;
	if (dt < 1) return null;
	const per = (s[s.length - 1].fetched - s[0].fetched) / dt;
	return per > 0 ? per : null;
}

/** seconds left at the current rate, or null when unknown */
export function etaSeconds(): number | null {
	const r = rate();
	if (r === null || run.phase !== "fetching") return null;
	return Math.max(0, (run.total - run.fetched) / r);
}

/** Validate, open a database if needed, and start. Never throws: a refusal
 *  lands in run.startError. */
export async function start(): Promise<void> {
	const built = buildOptions();
	resetRun();
	if ("error" in built) {
		run.phase = "idle";
		run.startError = built.error;
		return;
	}
	try {
		await ensureOpen();
		const id = await glowrootIngest(built.options);
		if (run.id === null) run.id = id;
	} catch (e) {
		run.phase = "idle";
		run.startError = String(e);
	}
}

export async function cancel(): Promise<void> {
	if (run.id === null) return;
	try {
		await glowrootCancel(run.id);
	} catch (e) {
		run.startError = String(e);
	}
}
