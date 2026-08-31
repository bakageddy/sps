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
    sidebarWidth.value = Math.min(MAX_WIDTH, Math.max(MIN_WIDTH, event.clientX));
  }

  function onkeydown(event: KeyboardEvent) {
    if (event.key === "ArrowLeft") sidebarWidth.value = Math.max(MIN_WIDTH, sidebarWidth.value - 16);
    if (event.key === "ArrowRight") sidebarWidth.value = Math.min(MAX_WIDTH, sidebarWidth.value + 16);
  }

  // Native webview zoom (like Ctrl+± in a browser), persisted across runs.
  // If setZoom rejects with a permissions error, add
  // "core:webview:allow-set-webview-zoom" to src-tauri/capabilities/*.json.
  const zoom = persisted("zoom", 1);

  function applyZoom() {
    getCurrentWebview()
      .setZoom(zoom.value)
      .catch((e) => console.warn("setZoom failed:", e));
  }

  applyZoom(); // restore the saved level on startup

  function setZoomLevel(next: number) {
    // toFixed dance: 0.1 steps accumulate float error (0.30000000000000004)
    zoom.value = Number(Math.min(3, Math.max(0.5, next)).toFixed(2));
    applyZoom();
  }

  // App-wide shortcuts: Ctrl/Cmd+B sidebar, Ctrl/Cmd +/-/0 zoom.
  function onwindowkeydown(event: KeyboardEvent) {
    if (!(event.ctrlKey || event.metaKey)) return;
    switch (event.key) {
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

  interface NavItem {
    href: string;
    label: string;
    icon: IconName;
    /** sub-pages rendered indented under their parent */
    children?: NavItem[];
  }

  const nav: NavItem[] = [
    { href: "/", label: "Ingest", icon: "ingest" },
    { href: "/cpumonitoring", label: "CPU Monitoring", icon: "cpu" },
    {
      href: "/cpumemstats",
      label: "CPU/Mem Statistics",
      icon: "stats",
      children: [
        { href: "/cpumemstats/overview", label: "Overview", icon: "graph" },
        { href: "/cpumemstats/correlation", label: "JVM vs Machine", icon: "graph" },
        { href: "/cpumemstats/linked", label: "Linked Dumps", icon: "link" },
      ],
    },
  ];
</script>

<!-- svelte:window attaches listeners to window with automatic cleanup —
     no addEventListener/onMount bookkeeping. -->
<svelte:window onkeydown={onwindowkeydown} />

<div class="shell">
  {#if !collapsed.value}
    <aside class="sidebar" style:width="{sidebarWidth.value}px">
      <div class="brand-row">
        <div class="brand">sps</div>
        <button
          class="collapse"
          onclick={() => (collapsed.value = true)}
          title="Hide sidebar (Ctrl+B)"
          aria-label="Hide sidebar"
        ><Icon name="chevronLeft" /></button>
      </div>

    <nav>
      {#each nav as item (item.href)}
        <!-- aria-current is the accessible way to mark the active link;
             we style off the attribute instead of a custom class. -->
        <a
          href={item.href}
          aria-current={page.url.pathname === item.href ? "page" : undefined}
        >
          <span class="icon"><Icon name={item.icon} /></span>
          {item.label}
        </a>
        {#each item.children ?? [] as child (child.href)}
          <a
            class="child"
            href={child.href}
            aria-current={page.url.pathname === child.href ? "page" : undefined}
          >
            <span class="icon"><Icon name={child.icon} /></span>
            {child.label}
          </a>
        {/each}
      {/each}
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
        aria-label="Show sidebar"
      ><Icon name="menu" /></button>
    </div>
  {/if}

  <main>
    {@render children()}
  </main>
</div>

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

  /* Wordmark: caps, Space Grotesk at its heaviest, wide tracking — small
     text needs letter-spacing to read as a mark rather than a typo. */
  .brand {
    font-weight: 700;
    font-size: 15px;
    text-transform: uppercase;
    letter-spacing: 0.18em;
    color: var(--accent);
    padding: 4px 8px;
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

  /* Nav in caps Grotesk to match the wordmark; smaller size + tracking
     because uppercase reads visually larger than lowercase at equal px. */
  nav a {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 10px;
    border-radius: var(--radius);
    color: var(--fg-muted);
    text-decoration: none;
    font-size: 11.5px;
    font-weight: 500;
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
  }

  /* No orientation media query here on purpose: the sidebar is ALWAYS a
     vertical left rail (collapse it with Ctrl+B if space is tight).
     Portrait adaptation is the content's job — the analyzer pages restack
     their own panes via MediaQuery. */
</style>
