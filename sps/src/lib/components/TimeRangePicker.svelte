<script lang="ts">
  /**
   * Explicit [from, to] window picker — the precision counterpart to
   * sweep-zoom. Shows the active window (or the full domain when not
   * zoomed); editing either end writes the page's shared `view` state,
   * so it composes with presets, magnifiers and sweeping.
   *
   * Plain text inputs, not <input type="datetime-local">, on purpose:
   * WebKitGTK's datetime-local is half-implemented (date picker works,
   * time segments don't commit). Text also matches the real workflow —
   * pasting a timestamp straight out of a log line. Accepted formats:
   * "YYYY-MM-DD HH:MM:SS" or a bare "HH:MM[:SS]" (keeps that bound's
   * current date). Commit on Enter/blur; invalid input reverts.
   */
  interface Props {
    /** full data domain, ms epoch */
    domain: [number, number];
    view: [number, number] | null;
    onviewchange: (view: [number, number] | null) => void;
  }

  let { domain, view, onviewchange }: Props = $props();

  const window_ = $derived(view ?? domain);

  /** past this, Date throws — corrupt rows must not break the inputs */
  const DATE_RANGE_MAX = 8.64e15;

  const pad = (n: number) => String(n).padStart(2, "0");

  /** ms epoch → "YYYY-MM-DD HH:MM:SS" in local time */
  function fmt(ms: number): string {
    if (!Number.isFinite(ms) || Math.abs(ms) > DATE_RANGE_MAX) return "";
    const d = new Date(ms);
    return (
      `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ` +
      `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`
    );
  }

  /**
   * Parse user input against a base timestamp: full date-times stand
   * alone; a bare time-of-day inherits the base's date.
   */
  function parse(value: string, base: number): number | null {
    const v = value.trim();
    const timeOnly = v.match(/^(\d{1,2}):(\d{2})(?::(\d{2}))?$/);
    if (timeOnly !== null) {
      const d = new Date(base);
      d.setHours(Number(timeOnly[1]), Number(timeOnly[2]), Number(timeOnly[3] ?? 0), 0);
      return d.getTime();
    }
    // "YYYY-MM-DD HH:MM:SS" → ISO-ish; no zone suffix = parsed as local
    const t = new Date(v.replace(" ", "T")).getTime();
    return Number.isFinite(t) ? t : null;
  }

  function commit(event: Event & { currentTarget: HTMLInputElement }, which: 0 | 1) {
    const base = window_[which];
    const t = parse(event.currentTarget.value, base);
    const valid =
      t !== null && (which === 0 ? t < window_[1] : t > window_[0]);
    if (valid) {
      onviewchange(which === 0 ? [t, window_[1]] : [window_[0], t]);
    } else {
      event.currentTarget.value = fmt(base); // revert visibly, don't apply
    }
  }
</script>

<span class="range">
  <input
    type="text"
    value={fmt(window_[0])}
    onchange={(event) => commit(event, 0)}
    placeholder="YYYY-MM-DD HH:MM:SS"
    title="Window start — full date-time, or HH:MM[:SS] to keep the date"
    aria-label="Window start"
  />
  <span class="sep">→</span>
  <input
    type="text"
    value={fmt(window_[1])}
    onchange={(event) => commit(event, 1)}
    placeholder="YYYY-MM-DD HH:MM:SS"
    title="Window end — full date-time, or HH:MM[:SS] to keep the date"
    aria-label="Window end"
  />
</span>

<style>
  .range {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  input {
    width: 138px;
    padding: 2px 6px;
    background: var(--bg-hard);
    border: none;
    border-radius: var(--radius);
    color: var(--fg);
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .sep {
    color: var(--fg-muted);
    font-size: 11px;
  }
</style>
