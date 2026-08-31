<script lang="ts">
  /**
   * Colorscheme selector + dark/light toggle for the sidebar footer.
   *
   * The select uses a FUNCTION BINDING: bind:value={get, set}. Two reasons,
   * one per direction:
   *  - why bind at all (not value= + onchange): a plain value attribute is
   *    assigned when the <select> is created, BEFORE its {#each} options
   *    exist — the DOM ignores a value with no matching option, so the
   *    picker showed the first entry instead of the saved scheme.
   *    bind:value is select-aware and applies the value after the options.
   *  - why functions (not bind:value={theme.colorscheme}): a bare bind
   *    writes state directly, skipping setColorscheme's side effects (the
   *    <html> attribute + localStorage). The setter half routes every write
   *    through the function that owns those effects.
   */
  import {
    theme,
    toggleMode,
    setColorscheme,
    Mode,
    Colorscheme,
    COLORSCHEME_LABELS,
  } from "$lib/theme.svelte";

  // The option list falls out of the enum — add a scheme to the const
  // object (and its label) and it appears here, no component change.
  const schemes = Object.values(Colorscheme);
</script>

<div class="row">
  <select
    bind:value={() => theme.colorscheme, setColorscheme}
    aria-label="Colorscheme"
  >
    {#each schemes as scheme (scheme)}
      <option value={scheme}>{COLORSCHEME_LABELS[scheme]}</option>
    {/each}
  </select>

  <button
    onclick={toggleMode}
    title="Switch to {theme.mode === Mode.Dark ? 'light' : 'dark'} mode"
  >
    {theme.mode === Mode.Dark ? "☀" : "☾"}
  </button>
</div>

<style>
  .row {
    display: flex;
    gap: 8px;
  }

  select {
    flex: 1;
    min-width: 0;
    padding: 4px 8px;
    font-size: 12px;
    background: var(--bg);
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
  }

  button {
    padding: 4px 10px;
    font-size: 13px;
    background: none;
    border: none;
    border-radius: var(--radius);
    cursor: pointer;
    color: var(--fg-muted);
  }

  select:hover,
  button:hover {
    background: var(--bg-hover);
  }
  button:hover {
    color: var(--fg);
  }
</style>
