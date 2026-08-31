<script lang="ts">
  /**
   * Two panes with a draggable divider.
   *
   * Concepts on display:
   *  - Snippets: `a` and `b` are chunks of markup the PARENT writes and this
   *    component renders with {@render}. This is Svelte 5's replacement for
   *    slots, and the standard way to build layout components.
   *  - $props(): components declare their inputs by destructuring one rune
   *    call; defaults are plain destructuring defaults.
   *  - Pointer capture: setPointerCapture keeps sending us pointermove even
   *    when the cursor leaves the divider mid-drag — without it, fast drags
   *    "drop" the divider. This is the standard way to implement dragging.
   */
  import type { Snippet } from "svelte";

  interface Props {
    /** "row" = panes side by side, "column" = stacked. */
    direction?: "row" | "column";
    /** Initial share of pane A, 0..1 */
    initial?: number;
    a: Snippet;
    b: Snippet;
  }

  let { direction = "row", initial = 0.4, a, b }: Props = $props();

  // Svelte warns when a prop seeds $state ("captures only the initial
  // value") because it's often a bug. Here it's the intent: `initial` is
  // a starting point and the user's dragging owns the value afterwards.
  // svelte-ignore state_referenced_locally
  let fraction = $state(initial);
  let container: HTMLDivElement; // filled by bind:this once mounted

  function onpointerdown(event: PointerEvent) {
    (event.target as HTMLElement).setPointerCapture(event.pointerId);
  }

  function onpointermove(event: PointerEvent) {
    // buttons === 0 means no button held: ignore plain hovering
    if (event.buttons === 0) return;
    const rect = container.getBoundingClientRect();
    const ratio =
      direction === "row"
        ? (event.clientX - rect.left) / rect.width
        : (event.clientY - rect.top) / rect.height;
    fraction = Math.min(0.85, Math.max(0.15, ratio));
  }

  // Keyboard support: dividers are focusable separators per the ARIA spec,
  // adjustable with arrow keys — screen-reader and keyboard users can resize too.
  function onkeydown(event: KeyboardEvent) {
    const step = 0.05;
    const decrease = direction === "row" ? "ArrowLeft" : "ArrowUp";
    const increase = direction === "row" ? "ArrowRight" : "ArrowDown";
    if (event.key === decrease) fraction = Math.max(0.15, fraction - step);
    if (event.key === increase) fraction = Math.min(0.85, fraction + step);
  }
</script>

<div class="split {direction}" bind:this={container}>
  <div class="pane" style:flex-basis="{fraction * 100}%">
    {@render a()}
  </div>

  <!-- The a11y linter thinks a separator can't be focusable/interactive, but
       the ARIA "window splitter" pattern is exactly that: a focusable
       separator with aria-valuenow. Suppressing is correct here. -->
  <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
  <div
    class="divider"
    role="separator"
    aria-orientation={direction === "row" ? "vertical" : "horizontal"}
    aria-valuenow={Math.round(fraction * 100)}
    tabindex="0"
    {onpointerdown}
    {onpointermove}
    {onkeydown}
  ></div>

  <div class="pane grow">
    {@render b()}
  </div>
</div>

<style>
  /* Svelte scopes these rules to this component: .pane here can never
     collide with a .pane elsewhere in the app. */
  .split {
    display: flex;
    width: 100%;
    height: 100%;
    min-height: 0; /* lets panes shrink inside a flex/grid parent */
  }
  .split.row {
    flex-direction: row;
  }
  .split.column {
    flex-direction: column;
  }

  .pane {
    min-width: 0;
    min-height: 0;
    overflow: auto;
  }
  .pane.grow {
    flex: 1;
  }

  .divider {
    flex: 0 0 6px;
    background: var(--hairline); /* subtle line; accent reveals on hover */
    background-clip: content-box;
    padding: 0 2px; /* 6px grab area, ~2px visible line */
    touch-action: none; /* stop scrolling from hijacking the drag on touch */
  }
  .column > .divider {
    padding: 2px 0;
  }
  .divider:hover,
  .divider:focus-visible {
    background: var(--accent);
    outline: none;
  }
  .row > .divider {
    cursor: col-resize;
  }
  .column > .divider {
    cursor: row-resize;
  }
</style>
