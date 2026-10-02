<script lang="ts">
  // Landing page = ingest hub: drop a bundle, see what was recognized,
  // then jump into an analyzer. Each future log kind (threaddump,
  // stuckquery, ...) gets a card here and a route of its own.
  import DropZone from "$lib/components/DropZone.svelte";
  import TimezoneControl from "$lib/components/TimezoneControl.svelte";
  import SignalHeatmap from "$lib/components/SignalHeatmap.svelte";
  import { persisted } from "$lib/persisted.svelte";

  // the two tools below the analyzers fold away; the fold survives restarts
  const heatmapOpen = persisted("home-heatmap-open", true);
  const timezoneOpen = persisted("home-timezone-open", false);

  // Connection Dumps leads: it is the incident hub every other analyzer
  // hangs off (signals = triggers; the rest are resolved around them).
  const analyzers = [
    {
      href: "/connectiondump",
      title: "Connection Dumps",
      description:
        "Pool occupancy, threshold alarms, and who holds each connection — across dumps.",
    },
    {
      href: "/cpumonitoring",
      title: "CPU Monitoring",
      description: "Per-thread CPU usage over time, with captured stack traces.",
    },
    {
      href: "/cpumemstats",
      title: "CPU/Mem Statistics",
      description: "Per-process CPU and memory usage across triggered dumps.",
    },
    {
      href: "/stuckthreads",
      title: "Stuck Threads",
      description: "Waterfall of requests Tomcat flagged as stuck, with their stack traces.",
    },
    {
      href: "/stuckqueries",
      title: "Stuck Queries",
      description:
        "Database queries running at each stuck-thread moment — blocking chains and long runners.",
    },
    {
      href: "/threaddump",
      title: "Thread Dumps",
      description:
        "Every thread's state and stack at each dump, with lock ownership, deadlocks and hot locks.",
    },
    {
      href: "/runningqueries",
      title: "Running Queries",
      description:
        "Every running statement at each periodic tick — the always-on view of the database, joined to incidents.",
    },
    {
      href: "/sql",
      title: "SQL Console",
      description:
        "Ask the parsed data anything the analyzers don't — read-only SELECTs, paged results, CSV export.",
    },
  ];
</script>

<!-- the page scrolls on its own: the shell's <main> clips, so the scroll
     container has to be here -->
<div class="scroll">
  <div class="home">
    <DropZone />

    <h2>Analyzers</h2>
    <div class="cards">
      {#each analyzers as a (a.href)}
        <a class="card" href={a.href}>
          <h3>{a.title}</h3>
          <p>{a.description}</p>
        </a>
      {/each}
    </div>

    <details class="fold" bind:open={heatmapOpen.value}>
      <summary>
        <span class="chev" aria-hidden="true">{heatmapOpen.value ? "▾" : "▸"}</span>
        Signals by day
      </summary>
      <SignalHeatmap />
    </details>

    <details class="fold" bind:open={timezoneOpen.value}>
      <summary>
        <span class="chev" aria-hidden="true">{timezoneOpen.value ? "▾" : "▸"}</span>
        Display timezone
      </summary>
      <TimezoneControl />
    </details>
  </div>
</div>

<style>
  .scroll {
    height: 100%;
    overflow-y: auto;
  }
  .home {
    max-width: 760px;
    margin: 0 auto;
    padding: 40px 24px 48px;
  }

  /* collapsible sections, styled like the section headings above them */
  .fold {
    margin-top: 32px;
  }
  .fold summary {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0 0 12px;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-muted);
    cursor: pointer;
    list-style: none; /* our own chevron, not the UA triangle */
    user-select: none;
  }
  .fold summary::-webkit-details-marker {
    display: none;
  }
  .fold summary:hover {
    color: var(--fg);
  }
  .chev {
    width: 10px;
    font-size: 11px;
    color: var(--accent);
  }

  h2 {
    margin: 32px 0 12px;
    font-size: 13px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--fg-muted);
  }

  .cards {
    display: grid;
    /* auto-fill + minmax: as many 260px+ columns as fit — the standard
       responsive-grid one-liner, no media queries needed. */
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 16px;
  }

  .card {
    display: block;
    padding: 16px;
    background: var(--bg-soft);
    border: none;
    border-radius: var(--radius);
    text-decoration: none;
    color: inherit;
  }
  .card:hover {
    background: var(--bg-hover);
  }

  /* Caps Geist Mono, matching the sidebar nav treatment (smaller size +
     tracking, since uppercase reads larger than lowercase at equal px). */
  .card h3 {
    margin: 0 0 6px;
    font-size: 13px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--accent);
  }
  .card p {
    margin: 0;
    font-size: 13px;
    color: var(--fg-muted);
  }
</style>
