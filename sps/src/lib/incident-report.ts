/**
 * The resolved incident as Markdown — what goes into the ticket. Pure
 * function over what the incident page already holds; nothing is fetched.
 * Lists are capped (top-N by the thing that matters) so the report stays
 * pasteable; the app has the full tables.
 */
import type { ConnDumpSignal } from "$lib/api/connectiondump";
import type { UsageRow } from "$lib/components/ProcessTable.svelte";
import type { IncidentThread } from "$lib/connectiondump";
import type {
	MssqlQuery,
	PgsqlQuery,
	MssqlBlockingRow,
	SpWho2Row,
} from "$lib/api/runningquery";
import { findDeadlocks } from "$lib/threaddump";
import { formatDuration, formatTimestamp } from "$lib/format";
import { resolveZone } from "$lib/timezone.svelte";

export interface IncidentReportInput {
	signal: ConnDumpSignal;
	toleranceMs: number;
	anchors: {
		threaddump: number | null;
		cpumonitoring: number | null;
		cpumemstats: number | null;
		runningqueries: number | null;
		traces: number | null;
	};
	census: IncidentThread[];
	processes: UsageRow[];
	mssql: MssqlQuery[];
	pgsql: PgsqlQuery[];
	blocking: MssqlBlockingRow[];
	spwho2: SpWho2Row[];
	/** stack lines per tid, fetched by the page for the threads worth
	 *  printing (hot, deadlocked, blocked); absent = no trace in the report */
	stacks?: Map<number, string[]>;
}

/** how many frames a stack gets in the report before it is elided */
export const REPORT_STACK_FRAMES = 15;

const fmt = new Intl.DateTimeFormat("en-GB", {
	dateStyle: "medium",
	timeStyle: "medium",
	hourCycle: "h23",
});
const ts = (ms: number | null) => (ms === null ? "—" : formatTimestamp(fmt, ms));
const oneLine = (s: string, max = 200) => {
	const t = s.replace(/\s+/g, " ").trim();
	return t.length > max ? t.slice(0, max - 1) + "…" : t;
};
const md = (s: string) => s.replace(/\|/g, "\\|");

export function buildIncidentReport(i: IncidentReportInput): string {
	const zone = resolveZone() ?? "browser local time";
	const out: string[] = [];
	const push = (...lines: string[]) => out.push(...lines);
	const stacks = i.stacks ?? new Map<number, string[]>();
	/** the stacks of `threads` as collapsed <details> blocks, in order */
	const pushStacks = (threads: IncidentThread[]) => {
		for (const t of threads) {
			const lines = stacks.get(t.tid);
			if (!lines || lines.length === 0) continue;
			const shown = lines.slice(0, REPORT_STACK_FRAMES);
			push(
				`<details><summary><code>${md(t.name)}</code> (${t.tid}) · ${t.state}${
					t.cpu !== null ? ` · ${t.cpu.toFixed(1)} % cpu` : ""
				}</summary>`,
				"",
				"```",
				...shown,
				...(lines.length > shown.length ? [`… ${lines.length - shown.length} more frames`] : []),
				"```",
				"",
				"</details>",
				"",
			);
		}
	};

	push(
		`# Incident — ${i.signal.cause} @ ${ts(i.signal.timestamp)}`,
		"",
		`- signal: \`${i.signal.timestamp}\` (${zone})${i.signal.suppressed ? " · dump suppressed" : ""}`,
		`- window: ±${(i.toleranceMs / 1000).toFixed(1)} s`,
		`- thread dump: ${ts(i.anchors.threaddump)}`,
		`- cpu monitoring: ${ts(i.anchors.cpumonitoring)}`,
		`- cpu/mem statistics: ${ts(i.anchors.cpumemstats)}`,
		`- running queries: ${ts(i.anchors.runningqueries)}`,
		`- connection holders: ${ts(i.anchors.traces)}`,
		"",
	);

	// --- threads ---------------------------------------------------------
	if (i.census.length > 0) {
		const byState = new Map<string, number>();
		for (const t of i.census) byState.set(t.state, (byState.get(t.state) ?? 0) + 1);
		push(
			`## Threads (${i.census.length})`,
			"",
			[...byState.entries()]
				.toSorted((a, b) => b[1] - a[1])
				.map(([s, n]) => `${s} ${n}`)
				.join(" · "),
			"",
		);

		const cycles = findDeadlocks(i.census);
		if (cycles.length > 0) {
			push(`### Deadlocks (${cycles.length})`, "");
			for (const c of cycles)
				push(`- ${c.map((t) => `${t.name} (${t.tid})`).join(" → ")} → …`);
			push("");
			pushStacks(cycles.flat() as IncidentThread[]);
		}

		const blocked = i.census
			.filter((t) => t.state === "BLOCKED")
			.toSorted((a, b) => (b.heldFor ?? 0) - (a.heldFor ?? 0));
		if (blocked.length > 0) {
			push(
				`### Blocked (${blocked.length}${blocked.length > 25 ? ", top 25" : ""})`,
				"",
				"| tid | thread | waiting on | owner |",
				"|---:|---|---|---|",
			);
			for (const t of blocked.slice(0, 25))
				push(
					`| ${t.tid} | ${md(t.name)} | ${md(t.waitingOn ?? "—")} | ${
						t.lockOwnerTid === null ? "—" : `${md(t.lockOwnerName ?? "")} (${t.lockOwnerTid})`
					} |`,
				);
			push("");
			pushStacks(blocked.slice(0, 25));
		}

		const hot = i.census
			.filter((t) => t.cpu !== null && t.cpu >= 5)
			.toSorted((a, b) => (b.cpu ?? 0) - (a.cpu ?? 0))
			.slice(0, 15);
		if (hot.length > 0) {
			push("### Hot threads (cpu ≥ 5 %)", "", "| tid | thread | cpu % | state |", "|---:|---|---:|---|");
			for (const t of hot)
				push(`| ${t.tid} | ${md(t.name)} | ${t.cpu!.toFixed(1)} | ${t.state} |`);
			push("");
			pushStacks(hot);
		}

		const holders = i.census
			.filter((t) => t.heldFor !== null)
			.toSorted((a, b) => (b.heldFor ?? 0) - (a.heldFor ?? 0))
			.slice(0, 15);
		if (holders.length > 0) {
			push("### Connection holders", "", "| tid | thread | held | state |", "|---:|---|---:|---|");
			for (const t of holders)
				push(`| ${t.tid} | ${md(t.name)} | ${formatDuration(t.heldFor!)} | ${t.state} |`);
			push("");
		}
	}

	// --- processes -------------------------------------------------------
	if (i.processes.length > 0) {
		const top = i.processes.toSorted((a, b) => b.value - a.value).slice(0, 10);
		push("## Processes (top 10 by CPU %)", "", "| pid | name | cpu % |", "|---:|---|---:|");
		for (const p of top) push(`| ${p.pid} | ${md(p.name)} | ${p.value.toFixed(1)} |`);
		push("");
	}

	// --- database --------------------------------------------------------
	if (i.mssql.length > 0) {
		const top = i.mssql.toSorted((a, b) => b.elapsed - a.elapsed).slice(0, 10);
		push(
			`## Running queries — MSSQL (${i.mssql.length}, top 10 by elapsed)`,
			"",
			"| session | status | elapsed | blocked by | statement |",
			"|---:|---|---:|---:|---|",
		);
		for (const q of top)
			push(
				`| ${q.sessionId} | ${q.status} | ${formatDuration(q.elapsed)} | ${
					q.blockedBy === 0 ? "—" : q.blockedBy
				} | ${md(oneLine(q.statement))} |`,
			);
		push("");
	}
	if (i.blocking.length > 0) {
		const heads = new Set(i.blocking.map((b) => b.headBlocker));
		push(`### Blocking chains (${heads.size})`, "");
		for (const h of heads) {
			const victims = i.blocking.filter((b) => b.headBlocker === h);
			const head = victims[0];
			push(
				`- head blocker session ${h} blocks ${victims.length}: ${md(
					oneLine(head.blockerQueryOrMostRecentQuery, 160),
				)}`,
			);
		}
		push("");
	}
	if (i.pgsql.length > 0) {
		const top = i.pgsql
			.toSorted((a, b) => (b.queryTime ?? 0) - (a.queryTime ?? 0))
			.slice(0, 10);
		push(
			`## Running queries — PGSQL (${i.pgsql.length}, top 10 by query time)`,
			"",
			"| pid | state | query time | waiting | query |",
			"|---:|---|---:|---|---|",
		);
		for (const q of top)
			push(
				`| ${q.pid} | ${q.state} | ${q.queryTime === null ? "—" : formatDuration(q.queryTime)} | ${
					q.waiting ? "yes" : ""
				} | ${md(oneLine(q.query))} |`,
			);
		push("");
	}
	const blockedSessions = i.spwho2.filter((s) => s.blockedBy !== null && s.blockedBy !== 0);
	if (blockedSessions.length > 0) {
		push(
			`### sp_who2 — blocked sessions (${blockedSessions.length})`,
			"",
			"| spid | status | blocked by | login | command |",
			"|---:|---|---:|---|---|",
		);
		for (const s of blockedSessions.slice(0, 15))
			push(`| ${s.spid} | ${s.status} | ${s.blockedBy} | ${md(s.login)} | ${md(s.command)} |`);
		push("");
	}

	return out.join("\n").trimEnd() + "\n";
}

/** the threads whose stacks the report prints — same cuts as the sections above */
export function reportStackThreads(census: IncidentThread[]): IncidentThread[] {
	const seen = new Set<number>();
	const out: IncidentThread[] = [];
	const add = (t: IncidentThread) => {
		if (!seen.has(t.tid)) {
			seen.add(t.tid);
			out.push(t);
		}
	};
	for (const c of findDeadlocks(census)) for (const t of c) add(t as IncidentThread);
	census
		.filter((t) => t.state === "BLOCKED")
		.toSorted((a, b) => (b.heldFor ?? 0) - (a.heldFor ?? 0))
		.slice(0, 25)
		.forEach(add);
	census
		.filter((t) => t.cpu !== null && t.cpu >= 5)
		.toSorted((a, b) => (b.cpu ?? 0) - (a.cpu ?? 0))
		.slice(0, 15)
		.forEach(add);
	return out;
}
