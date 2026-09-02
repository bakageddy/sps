<script lang="ts" module>
  /**
   * Nerd Font icons (https://www.nerdfonts.com), rendered as text glyphs.
   *
   * The font is BUNDLED (static/fonts, @font-face in app.css) — never rely
   * on a Nerd Font being installed on the user's machine; on machines
   * without it every icon becomes a tofu box. Explicitly setting
   * font-family below also means we never gamble on the OS fallback chain
   * (the unicode-glyph icons this replaced were landing in the color-emoji
   * font and ignoring the theme).
   *
   * Glyphs live in the Private Use Area; the escapes below are their
   * codepoints (see the cheat sheet at nerdfonts.com/cheat-sheet). The
   * const-object + derived-type pattern keeps icon names compile-checked.
   */
  export const Icons = {
    ingest: "\u{f019}", // nf-fa-download
    cpu: "\u{f2db}", // nf-fa-microchip
    stats: "\u{f080}", // nf-fa-bar_chart
    graph: "\u{f201}", // nf-fa-line_chart
    link: "\u{f0c1}", // nf-fa-link
    stuck: "\u{f254}", // nf-fa-hourglass
    expand: "\u{f065}", // nf-fa-expand
    compress: "\u{f066}", // nf-fa-compress
    chevronLeft: "\u{f053}", // nf-fa-chevron_left
    zoomIn: "\u{f00e}", // nf-fa-search_plus
    zoomOut: "\u{f010}", // nf-fa-search_minus
    menu: "\u{f0c9}", // nf-fa-bars
  } as const;
  export type IconName = keyof typeof Icons;
</script>

<script lang="ts">
  let { name, size = 16 }: { name: IconName; size?: number } = $props();
</script>

<span class="nf" style:font-size="{size}px" aria-hidden="true">{Icons[name]}</span>

<style>
  .nf {
    font-family: "Symbols Nerd Font", var(--font-mono);
    line-height: 1; /* icon height = font-size, no baseline slack */
    display: inline-grid;
    place-items: center;
  }
</style>
