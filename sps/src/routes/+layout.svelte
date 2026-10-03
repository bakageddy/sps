<script lang="ts">
	/**
	 * The root layout wraps EVERY route: whatever page is active is passed in
	 * as the `children` snippet and rendered where {@render children()} sits.
	 * Global CSS is imported once, here — Vite injects it app-wide.
	 *
	 * macOS-style shell: a persistent sidebar (navigation on top, app-level
	 * options pinned to the bottom) and a content area. No toolbar — actions
	 * live where their context is (ingest on the landing page).
	 */
	import "../app.css";
	import type { Snippet } from "svelte";
	import { page } from "$app/state"; // reactive info about the current route
	import { sync } from "$lib/database.svelte";
	import DatabaseControl from "$lib/components/DatabaseControl.svelte";
	import ThemeControl from "$lib/components/ThemeControl.svelte";
	import { persisted } from "$lib/persisted.svelte";
	import { getCurrentWebview } from "@tauri-apps/api/webview";
	import Icon, { type IconName } from "$lib/components/Icon.svelte";
	import TimeStrip from "$lib/components/TimeStrip.svelte";
	import CommandPalette, {
		type Command,
	} from "$lib/components/CommandPalette.svelte";
	import { goto } from "$app/navigation";
	import { timeWindow, bundleDomain } from "$lib/timewindow.svelte";
	import { tzMode, parseZonedTime } from "$lib/timezone.svelte";
	import { formatTimestamp } from "$lib/format";
	import NotesPanel from "$lib/components/NotesPanel.svelte";
	import QuickNote from "$lib/components/QuickNote.svelte";
	import { toggleQuickNote, openQuickNote, loadNotes } from "$lib/notes.svelte";
	import { trackHistory, back, forward, canBack, canForward } from "$lib/navhistory.svelte";
	import { db } from "$lib/database.svelte";

	let { children }: { children: Snippet } = $props();

	// Both survive an app restart — sidebar geometry is a preference,
	// not session state.
	const sidebarWidth = persisted("sidebar-width", 220);
	const collapsed = persisted("sidebar-collapsed", false);

	const MIN_WIDTH = 160;
	const MAX_WIDTH = 400;

	// Same drag technique as SplitPane: capture the pointer so fast drags
	// don't escape the 6px handle.
	function onpointerdown(event: PointerEvent) {
		(event.target as HTMLElement).setPointerCapture(event.pointerId);
	}

	function onpointermove(event: PointerEvent) {
		if (event.buttons === 0) return;
		// The sidebar starts at the window's left edge, so clientX IS the width.
		sidebarWidth.value = Math.min(
			MAX_WIDTH,
			Math.max(MIN_WIDTH, event.clientX),
		);
	}

	function onkeydown(event: KeyboardEvent) {
		if (event.key === "ArrowLeft")
			sidebarWidth.value = Math.max(MIN_WIDTH, sidebarWidth.value - 16);
		if (event.key === "ArrowRight")
			sidebarWidth.value = Math.min(MAX_WIDTH, sidebarWidth.value + 16);
	}

	// Native webview zoom (like Ctrl+± in a browser), persisted across runs.
	// If setZoom rejects with a permissions error, add
	// "core:webview:allow-set-webview-zoom" to src-tauri/capabilities/*.json.
	const zoom = persisted("zoom", 1);

	function applyZoom() {
		// getCurrentWebview() THROWS (sync) outside Tauri — the .catch only
		// covers the setZoom promise. Without the try, this line kills the
		// whole app boot in any plain browser.
		try {
			getCurrentWebview()
				.setZoom(zoom.value)
				.catch((e) => console.warn("setZoom failed:", e));
		} catch (e) {
			console.warn("webview zoom unavailable:", e);
		}
	}

	applyZoom(); // restore the saved level on startup

	function setZoomLevel(next: number) {
		// toFixed dance: 0.1 steps accumulate float error (0.30000000000000004)
		zoom.value = Number(Math.min(3, Math.max(0.5, next)).toFixed(2));
		applyZoom();
	}

	// App-wide shortcuts: Ctrl/Cmd+B sidebar, Ctrl/Cmd+K palette,
	// Ctrl/Cmd+\ quick note, Ctrl/Cmd +/-/0 zoom.
	let paletteOpen = $state(false);
	function onwindowkeydown(event: KeyboardEvent) {
		// Alt+←/→: history, like a browser (also Ctrl+[ / ] below)
		if (event.altKey && !event.ctrlKey && !event.metaKey) {
			if (event.key === "ArrowLeft") {
				event.preventDefault();
				back();
			} else if (event.key === "ArrowRight") {
				event.preventDefault();
				forward();
			}
			return;
		}
		if (!(event.ctrlKey || event.metaKey)) return;
		switch (event.key) {
			case "[":
				event.preventDefault();
				back();
				break;
			case "]":
				event.preventDefault();
				forward();
				break;
			case "\\":
				event.preventDefault();
				toggleQuickNote();
				break;
			case "k":
				event.preventDefault();
				paletteOpen = !paletteOpen;
				break;
			case "b":
				event.preventDefault();
				collapsed.value = !collapsed.value;
				break;
			case "+":
			case "=": // the +/= key without shift reports "="
				event.preventDefault();
				setZoomLevel(zoom.value + 0.1);
				break;
			case "-":
				event.preventDefault();
				setZoomLevel(zoom.value - 0.1);
				break;
			case "0":
				event.preventDefault();
				setZoomLevel(1);
				break;
		}
	}

	// The layout mounts exactly once per app load — the right place for
	// app-level init like re-syncing with whatever database the backend
	// already has open (matters after a dev-mode webview reload).
	sync();

	// back/forward position tracking (afterNavigate has to be registered
	// during component init — this is the only component that always exists)
	trackHistory();

	// notes live in the database: (re)load on every open, drop on close
	$effect(() => {
		if (db.state.status === "open") void db.epoch;
		loadNotes();
	});

	interface NavItem {
		href: string;
		label: string;
		icon: IconName;
		/** sub-pages rendered indented under their parent */
		children?: NavItem[];
	}

	const nav: NavItem[] = [
		{ href: "/", label: "Ingest", icon: "ingest" },
		// Connection Dumps leads the analyzers: it is the incident hub the
		// others are resolved around (same order as the landing page).
		{
			href: "/connectiondump",
			label: "Connection Dumps",
			icon: "link",
			children: [
				{
					href: "/connectiondump/incident",
					label: "Incident",
					icon: "graph",
				},
			],
		},
		{
			href: "/threaddump",
			label: "Thread Dumps",
			icon: "stuck",
			children: [
				// a thread census paired with the queries the database was
				// running at that moment (running-query ticks, optionally
				// stuck-query snapshots), linked by time
				{
					href: "/threaddump/queries",
					label: "Queries",
					icon: "database",
				},
			],
		},
		// next to Thread Dumps: its periodic partner
		{ href: "/runningqueries", label: "Running Queries", icon: "database" },
		{ href: "/cpumonitoring", label: "CPU Monitoring", icon: "cpu" },
		{
			href: "/cpumemstats",
			label: "CPU/Mem Statistics",
			icon: "stats",
			children: [
				{
					href: "/cpumemstats/overview",
					label: "Overview",
					icon: "graph",
				},
				{
					href: "/cpumemstats/correlation",
					label: "JVM vs Machine",
					icon: "graph",
				},
				{
					href: "/cpumemstats/linked",
					label: "Linked Dumps",
					icon: "link",
				},
			],
		},
		{
			href: "/stuckthreads",
			label: "Stuck Threads",
			icon: "stuck",
			children: [
				{
					href: "/stuckthreads/concurrency",
					label: "Concurrency",
					icon: "graph",
				},
				{
					href: "/stuckthreads/queries",
					label: "Queries",
					icon: "database",
				},
			],
		},
		{ href: "/stuckqueries", label: "Stuck Queries", icon: "database" },
		// not an analyzer: the escape hatch when none of them asks your question
		{ href: "/sql", label: "SQL Console", icon: "terminal" },
	];

	// "jump to time": typed into the palette, it selects the nearest
	// snapshot on the current analyzer (or the incident page, the hub, when
	// the current page has no timeline) via the pages' ?t= deep link
	const JUMPABLE = [
		"/connectiondump/incident",
		"/threaddump",
		"/runningqueries",
		"/cpumonitoring",
		"/cpumemstats",
		"/stuckthreads/queries",
	];
	const jumpFormat = new Intl.DateTimeFormat(undefined, {
		dateStyle: "medium",
		timeStyle: "medium",
		hourCycle: "h23",
	});
	function jumpCommands(query: string): Command[] {
		const d = bundleDomain.value;
		const ref = d === null ? Date.now() : Math.round((d[0] + d[1]) / 2);
		const ms = parseZonedTime(query, ref);
		if (ms === null) return [];
		const here = page.url.pathname;
		const route = JUMPABLE.includes(here) ? here : "/connectiondump/incident";
		return [
			{
				label: `Jump to ${formatTimestamp(jumpFormat, ms)}`,
				hint: route,
				run: () => goto(`${route}?t=${ms}`),
			},
		];
	}

	// the time strip decorates analyzers only — not the ingest hub or the console
	const showStrip = $derived(
		page.url.pathname !== "/" && !page.url.pathname.startsWith("/sql"),
	);

	// Ctrl+K palette: every page (children as "Parent › Child") plus the
	// handful of app-level toggles worth a keystroke
	const commands = $derived.by<Command[]>(() => {
		const out: Command[] = [];
		for (const item of nav) {
			out.push({ label: item.label, hint: item.href, run: () => goto(item.href) });
			for (const c of item.children ?? [])
				out.push({
					label: `${item.label} › ${c.label}`,
					hint: c.href,
					run: () => goto(c.href),
				});
		}
		out.push(
			{ label: "New note", hint: "Ctrl+\\", run: () => openQuickNote(null) },
			{
				label: "Clear time window",
				hint: timeWindow.value === null ? "none set" : "active",
				run: () => (timeWindow.value = null),
			},
			{
				label: collapsed.value ? "Show sidebar" : "Hide sidebar",
				hint: "Ctrl+B",
				run: () => (collapsed.value = !collapsed.value),
			},
			{ label: "Timezone: Bundle", hint: tzMode.value === "auto" ? "current" : "", run: () => (tzMode.value = "auto") },
			{ label: "Timezone: Local", hint: tzMode.value === "local" ? "current" : "", run: () => (tzMode.value = "local") },
			{ label: "Timezone: UTC", hint: tzMode.value === "utc" ? "current" : "", run: () => (tzMode.value = "utc") },
			{ label: "Reset zoom", hint: "100%", run: () => setZoomLevel(1) },
		);
		return out;
	});
</script>

<!-- svelte:window attaches listeners to window with automatic cleanup —
     no addEventListener/onMount bookkeeping. -->
<svelte:window onkeydown={onwindowkeydown} />

<div class="shell">
	{#if !collapsed.value}
		<aside class="sidebar" style:width="{sidebarWidth.value}px">
			<div class="brand-row">
				<div class="brand">
					<img class="logo" src="/logo.svg" alt="" width="22" height="22" />
					sps
				</div>
				<span class="hist" role="group" aria-label="History">
					<button
						class="collapse"
						onclick={back}
						disabled={!canBack()}
						title="Back (Alt+← / Ctrl+[)"
						aria-label="Back">‹</button
					>
					<button
						class="collapse"
						onclick={forward}
						disabled={!canForward()}
						title="Forward (Alt+→ / Ctrl+])"
						aria-label="Forward">›</button
					>
				</span>
				<button
					class="collapse"
					onclick={() => (collapsed.value = true)}
					title="Hide sidebar (Ctrl+B)"
					aria-label="Hide sidebar"
					><Icon name="chevronLeft" /></button
				>
			</div>

			<nav>
				{#each nav as item (item.href)}
					<!-- aria-current is the accessible way to mark the active link;
             we style off the attribute instead of a custom class. -->
					<a
						href={item.href}
						aria-current={page.url.pathname === item.href
							? "page"
							: undefined}
					>
						<span class="icon"><Icon name={item.icon} /></span>
						{item.label}
					</a>
					{#each item.children ?? [] as child (child.href)}
						<a
							class="child"
							href={child.href}
							aria-current={page.url.pathname === child.href
								? "page"
								: undefined}
						>
							<span class="icon"><Icon name={child.icon} /></span>
							{child.label}
						</a>
					{/each}
				{/each}

				<!-- notes live in the selector, right under the last page -->
				<NotesPanel />
			</nav>

			<div class="footer">
				<DatabaseControl />
				<ThemeControl />
			</div>
		</aside>

		<!-- Same ARIA window-splitter pattern as SplitPane's divider. -->
		<!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
		<div
			class="resize-handle"
			role="separator"
			aria-orientation="vertical"
			aria-valuenow={sidebarWidth.value}
			tabindex="0"
			{onpointerdown}
			{onpointermove}
			{onkeydown}
		></div>
	{:else}
		<!-- Collapsed: a slim rail in normal flow instead of a floating button —
         it can't overlap page content, and ☰ keeps a constant position. -->
		<div class="rail">
			<button
				class="reveal"
				onclick={() => (collapsed.value = false)}
				title="Show sidebar (Ctrl+B)"
				aria-label="Show sidebar"><Icon name="menu" /></button
			>
		</div>
	{/if}

	<main>
		{#if showStrip}
			<TimeStrip />
		{/if}
		<div class="page-slot">
			{@render children()}
		</div>
	</main>
</div>

<QuickNote />
<CommandPalette
	open={paletteOpen}
	items={commands}
	dynamic={jumpCommands}
	onclose={() => (paletteOpen = false)}
/>

<style>
	.shell {
		display: flex;
		height: 100vh;
	}

	.sidebar {
		display: flex;
		flex-direction: column;
		/* width comes from the inline style (user-resizable) */
		flex-shrink: 0;
		padding: 12px;
		gap: 16px;
		background: var(--bg-hard);
		overflow: hidden; /* content clips during resize instead of wrapping */
	}

	.brand-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}

	/* Wordmark: caps, Geist Mono at its heaviest, wide tracking — small
     text needs letter-spacing to read as a mark rather than a typo. */
	.brand {
		display: flex;
		align-items: center;
		gap: 8px;
		font-weight: 700;
		font-size: 15px;
		text-transform: uppercase;
		letter-spacing: 0.18em;
		color: var(--accent);
		padding: 4px 8px;
	}
	.brand .logo {
		display: block;
		border-radius: 5px; /* the tile's rx at this size */
		flex-shrink: 0;
	}

	.collapse,
	.reveal {
		display: grid;
		place-items: center; /* centers the svg exactly, unlike text baselines */
		padding: 4px;
		background: none;
		border: none;
		border-radius: var(--radius);
		cursor: pointer;
		color: var(--fg-muted);
	}
	.hist {
		display: flex;
		margin-left: auto;
		margin-right: 2px;
	}
	.hist button {
		width: 22px;
		font-size: 15px;
		line-height: 1;
	}
	.hist button:disabled {
		opacity: 0.3;
		cursor: default;
	}
	.collapse:hover,
	.reveal:hover {
		color: var(--fg);
		background: var(--bg-hover);
	}

	.rail {
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		/* padding-top matches the expanded sidebar's (12px) + brand padding so
       ☰ sits at the same height as the brand row it replaces */
		padding: 16px 4px;
		background: var(--bg-hard);
		border-right: 1px solid var(--hairline);
	}

	/* Wide enough to grab (6px hit area), but visually a 1px hairline —
     the line is a border on the transparent handle, and the whole strip
     only lights up while interacting. */
	.resize-handle {
		flex: 0 0 6px;
		cursor: col-resize;
		background: transparent;
		border-left: 1px solid var(--hairline);
		touch-action: none;
	}
	.resize-handle:hover,
	.resize-handle:focus-visible {
		background: var(--accent);
		outline: none;
	}

	nav {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1; /* pushes .footer to the bottom */
	}

	/* Nav in caps Geist Mono to match the wordmark; smaller size + tracking
     because uppercase reads visually larger than lowercase at equal px. */
	nav a {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 7px 10px;
		border-radius: var(--radius);
		color: var(--fg-muted);
		text-decoration: none;
		/* Mono face: hierarchy comes from WEIGHT, not size — parents bold,
		   children regular, both 12px (a size gap reads as a bug in mono). */
		font-size: 12px;
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.07em;
	}
	nav a:hover {
		color: var(--fg);
		background: var(--bg-hover);
	}
	nav a[aria-current="page"] {
		color: var(--fg-strong);
		background: color-mix(in srgb, var(--accent) 14%, transparent);
	}

	nav a.child {
		margin-left: 18px;
		font-size: 12px;
		font-weight: 300; /* light: the variable file covers 100–900 */
		padding: 5px 10px;
	}

	.icon {
		display: grid;
		place-items: center;
		color: var(--accent);
	}

	.footer {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	main {
		flex: 1;
		min-width: 0;
		min-height: 0; /* without this, children can't shrink below content size */
		/* the shell is the viewport: pages scroll INSIDE their own panes,
		   never the document — whatever a page does wrong, it clips here */
		overflow: hidden;
		display: flex;
		flex-direction: column;
	}
	/* the page sits under the (optional) time strip and takes the rest */
	.page-slot {
		flex: 1;
		min-height: 0;
		overflow: hidden;
	}

	/* No orientation media query here on purpose: the sidebar is ALWAYS a
     vertical left rail (collapse it with Ctrl+B if space is tight).
     Portrait adaptation is the content's job — the analyzer pages restack
     their own panes via MediaQuery. */
</style>
