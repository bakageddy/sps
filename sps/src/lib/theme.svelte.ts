/**
 * Theme state + the themes themselves, all in TypeScript.
 *
 * The palettes are data (Record<Colorscheme, Record<Mode, Tokens>>), and
 * applyTheme() writes them as CSS custom properties onto <html>, so
 * components keep consuming var(--bg) etc. — only the SOURCE of the values
 * moved from app.css into typed TS:
 *
 *  - Tokens is a Record over a const tuple of variable names, so TypeScript
 *    proves every palette defines every token (a missing --chart-5 in
 *    gruvbox-light is a compile error, not a broken chart).
 *  - Because this app is an SPA (ssr = false), nothing renders until the JS
 *    bundle runs; the module-init applyTheme() call below therefore lands
 *    before first paint — no flash, and no pre-paint script in app.html
 *    duplicating values it couldn't import.
 */

export const Mode = {
  Dark: "dark",
  Light: "light",
} as const;
export type Mode = (typeof Mode)[keyof typeof Mode];

export const Colorscheme = {
  Nord: "nord",
  Gruvbox: "gruvbox",
  Modus: "modus",
  Kanagawa: "kanagawa",
  OneDark: "onedark",
  Solarized: "solarized",
} as const;
export type Colorscheme = (typeof Colorscheme)[keyof typeof Colorscheme];

/** Display names for the selector UI. */
export const COLORSCHEME_LABELS: Record<Colorscheme, string> = {
  [Colorscheme.Nord]: "Nord",
  [Colorscheme.Gruvbox]: "Gruvbox",
  [Colorscheme.Modus]: "Modus",
  [Colorscheme.Kanagawa]: "Kanagawa",
  [Colorscheme.OneDark]: "One Dark",
  [Colorscheme.Solarized]: "Solarized",
};

const StorageKey = {
  Mode: "theme",
  Colorscheme: "palette",
} as const;

// ---------------------------------------------------------------------------
// The themes
// ---------------------------------------------------------------------------

const TOKEN_NAMES = [
  "--bg-hard",   // deepest surface: sidebar, code blocks, inputs
  "--bg",        // main background
  "--bg-soft",   // raised surfaces: panels, table header
  "--bg-hover",  // hover/active fills
  "--border",
  "--fg",        // primary text
  "--fg-strong", // headings, emphasized numbers
  "--fg-muted",  // secondary text, labels, axis ticks
  "--accent",    // interactive highlights
  "--red",
  "--green",
  "--yellow",
  "--blue",
  "--purple",
  "--aqua",
  // cycling palette for chart series (one color per thread)
  "--chart-1",
  "--chart-2",
  "--chart-3",
  "--chart-4",
  "--chart-5",
  "--chart-6",
  "--chart-7",
] as const;

type TokenName = (typeof TOKEN_NAMES)[number];
type Tokens = Record<TokenName, string>;

const THEMES: Record<Colorscheme, Record<Mode, Tokens>> = {
  // Nord (https://www.nordtheme.com) — Polar Night / Snow Storm / Frost /
  // Aurora. Light mode darkens the Aurora hues: they're tuned for dark
  // backgrounds and wash out as text or 1.5px chart lines on Snow Storm.
  [Colorscheme.Nord]: {
    [Mode.Dark]: {
      "--bg-hard": "#242933",
      "--bg": "#2e3440",
      "--bg-soft": "#3b4252",
      "--bg-hover": "#434c5e",
      "--border": "#4c566a",
      "--fg": "#d8dee9",
      "--fg-strong": "#eceff4",
      "--fg-muted": "#7b88a1",
      "--accent": "#88c0d0",
      "--red": "#bf616a",
      "--green": "#a3be8c",
      "--yellow": "#ebcb8b",
      "--blue": "#81a1c1",
      "--purple": "#b48ead",
      "--aqua": "#8fbcbb",
      "--chart-1": "#88c0d0",
      "--chart-2": "#d08770",
      "--chart-3": "#a3be8c",
      "--chart-4": "#b48ead",
      "--chart-5": "#ebcb8b",
      "--chart-6": "#81a1c1",
      "--chart-7": "#bf616a",
    },
    [Mode.Light]: {
      "--bg-hard": "#d8dee9",
      "--bg": "#eceff4",
      "--bg-soft": "#e5e9f0",
      "--bg-hover": "#d8dee9",
      "--border": "#c8d0e0",
      "--fg": "#3b4252",
      "--fg-strong": "#2e3440",
      "--fg-muted": "#616e88",
      "--accent": "#5e81ac",
      "--red": "#a94a55",
      "--green": "#6f8a51",
      "--yellow": "#a5802c",
      "--blue": "#5e81ac",
      "--purple": "#9d6f90",
      "--aqua": "#59848a",
      "--chart-1": "#5e81ac",
      "--chart-2": "#b25c3e",
      "--chart-3": "#6f8a51",
      "--chart-4": "#9d6f90",
      "--chart-5": "#a5802c",
      "--chart-6": "#59848a",
      "--chart-7": "#a94a55",
    },
  },

  // Gruvbox (https://github.com/morhetz/gruvbox)
  [Colorscheme.Gruvbox]: {
    [Mode.Dark]: {
      "--bg-hard": "#1d2021",
      "--bg": "#282828",
      "--bg-soft": "#3c3836",
      "--bg-hover": "#504945",
      "--border": "#504945",
      "--fg": "#ebdbb2",
      "--fg-strong": "#fbf1c7",
      "--fg-muted": "#a89984",
      "--accent": "#fe8019",
      "--red": "#fb4934",
      "--green": "#b8bb26",
      "--yellow": "#fabd2f",
      "--blue": "#83a598",
      "--purple": "#d3869b",
      "--aqua": "#8ec07c",
      "--chart-1": "#fe8019",
      "--chart-2": "#83a598",
      "--chart-3": "#b8bb26",
      "--chart-4": "#d3869b",
      "--chart-5": "#fabd2f",
      "--chart-6": "#8ec07c",
      "--chart-7": "#fb4934",
    },
    [Mode.Light]: {
      "--bg-hard": "#f9f5d7",
      "--bg": "#fbf1c7",
      "--bg-soft": "#ebdbb2",
      "--bg-hover": "#d5c4a1",
      "--border": "#d5c4a1",
      "--fg": "#3c3836",
      "--fg-strong": "#282828",
      "--fg-muted": "#7c6f64",
      "--accent": "#af3a03",
      "--red": "#9d0006",
      "--green": "#79740e",
      "--yellow": "#b57614",
      "--blue": "#076678",
      "--purple": "#8f3f71",
      "--aqua": "#427b58",
      "--chart-1": "#af3a03",
      "--chart-2": "#076678",
      "--chart-3": "#79740e",
      "--chart-4": "#8f3f71",
      "--chart-5": "#b57614",
      "--chart-6": "#427b58",
      "--chart-7": "#9d0006",
    },
  },

  // Modus (https://protesilaos.com/emacs/modus-themes) — high contrast.
  // Dark = modus-vivendi (true black; relies on borders, not surface
  // shades, to separate regions), light = modus-operandi.
  [Colorscheme.Modus]: {
    [Mode.Dark]: {
      "--bg-hard": "#000000",
      "--bg": "#000000",
      "--bg-soft": "#1e1e1e",
      "--bg-hover": "#2b2b2b",
      "--border": "#646464",
      "--fg": "#ffffff",
      "--fg-strong": "#ffffff",
      "--fg-muted": "#989898",
      "--accent": "#2fafff",
      "--red": "#ff5f59",
      "--green": "#44bc44",
      "--yellow": "#d0bc00",
      "--blue": "#2fafff",
      "--purple": "#feacd0",
      "--aqua": "#00d3d0",
      "--chart-1": "#2fafff",
      "--chart-2": "#fec43f",
      "--chart-3": "#44bc44",
      "--chart-4": "#feacd0",
      "--chart-5": "#d0bc00",
      "--chart-6": "#00d3d0",
      "--chart-7": "#ff5f59",
    },
    [Mode.Light]: {
      "--bg-hard": "#f0f0f0",
      "--bg": "#ffffff",
      "--bg-soft": "#f2f2f2",
      "--bg-hover": "#e0e0e0",
      "--border": "#9f9f9f",
      "--fg": "#000000",
      "--fg-strong": "#000000",
      "--fg-muted": "#595959",
      "--accent": "#0031a9",
      "--red": "#a60000",
      "--green": "#006800",
      "--yellow": "#6f5500",
      "--blue": "#0031a9",
      "--purple": "#721045",
      "--aqua": "#005e8b",
      "--chart-1": "#0031a9",
      "--chart-2": "#972500",
      "--chart-3": "#006800",
      "--chart-4": "#721045",
      "--chart-5": "#6f5500",
      "--chart-6": "#005e8b",
      "--chart-7": "#a60000",
    },
  },
  // Kanagawa (https://github.com/rebornix/kanagawa) — dark = Wave
  // (sumi-ink surfaces), light = Lotus. A few surface/border shades are
  // interpolated where the palette defines no slot for them.
  [Colorscheme.Kanagawa]: {
    [Mode.Dark]: {
      "--bg-hard": "#16161d",
      "--bg": "#1f1f28",
      "--bg-soft": "#2a2a37",
      "--bg-hover": "#363646",
      "--border": "#54546d",
      "--fg": "#dcd7ba",
      "--fg-strong": "#f2ecbc",
      "--fg-muted": "#727169",
      "--accent": "#7e9cd8",
      "--red": "#e46876",
      "--green": "#98bb6c",
      "--yellow": "#e6c384",
      "--blue": "#7e9cd8",
      "--purple": "#957fb8",
      "--aqua": "#7aa89f",
      "--chart-1": "#7e9cd8",
      "--chart-2": "#ffa066",
      "--chart-3": "#98bb6c",
      "--chart-4": "#957fb8",
      "--chart-5": "#e6c384",
      "--chart-6": "#7aa89f",
      "--chart-7": "#e46876",
    },
    [Mode.Light]: {
      "--bg-hard": "#e4d794",
      "--bg": "#f2ecbc",
      "--bg-soft": "#e5ddb0",
      "--bg-hover": "#dcd5ac",
      "--border": "#bcb695",
      "--fg": "#545464",
      "--fg-strong": "#1f1f28",
      "--fg-muted": "#8a8980",
      "--accent": "#4d699b",
      "--red": "#c84053",
      "--green": "#6f894e",
      "--yellow": "#77713f",
      "--blue": "#4d699b",
      "--purple": "#624c83",
      "--aqua": "#597b75",
      "--chart-1": "#4d699b",
      "--chart-2": "#cc6d00",
      "--chart-3": "#6f894e",
      "--chart-4": "#624c83",
      "--chart-5": "#77713f",
      "--chart-6": "#597b75",
      "--chart-7": "#c84053",
    },
  },

  // One Dark / One Light (Atom's editor themes)
  [Colorscheme.OneDark]: {
    [Mode.Dark]: {
      "--bg-hard": "#21252b",
      "--bg": "#282c34",
      "--bg-soft": "#2c313c",
      "--bg-hover": "#3e4451",
      "--border": "#4b5263",
      "--fg": "#abb2bf",
      "--fg-strong": "#e6e6e6",
      "--fg-muted": "#7f848e",
      "--accent": "#61afef",
      "--red": "#e06c75",
      "--green": "#98c379",
      "--yellow": "#e5c07b",
      "--blue": "#61afef",
      "--purple": "#c678dd",
      "--aqua": "#56b6c2",
      "--chart-1": "#61afef",
      "--chart-2": "#d19a66",
      "--chart-3": "#98c379",
      "--chart-4": "#c678dd",
      "--chart-5": "#e5c07b",
      "--chart-6": "#56b6c2",
      "--chart-7": "#e06c75",
    },
    [Mode.Light]: {
      "--bg-hard": "#eaeaeb",
      "--bg": "#fafafa",
      "--bg-soft": "#f0f0f1",
      "--bg-hover": "#e5e5e6",
      "--border": "#d4d4d5",
      "--fg": "#383a42",
      "--fg-strong": "#232324",
      "--fg-muted": "#696c77",
      "--accent": "#4078f2",
      "--red": "#e45649",
      "--green": "#50a14f",
      "--yellow": "#c18401",
      "--blue": "#4078f2",
      "--purple": "#a626a4",
      "--aqua": "#0184bc",
      "--chart-1": "#4078f2",
      "--chart-2": "#986801",
      "--chart-3": "#50a14f",
      "--chart-4": "#a626a4",
      "--chart-5": "#c18401",
      "--chart-6": "#0184bc",
      "--chart-7": "#e45649",
    },
  },

  // Solarized (https://ethanschoonover.com/solarized) — the base palette is
  // symmetric: dark uses base03/02 surfaces with base0/1 text, light flips
  // to base3/2 surfaces with base00/02 text. The eight accents are designed
  // to work on both backgrounds, so they're shared.
  [Colorscheme.Solarized]: {
    [Mode.Dark]: {
      "--bg-hard": "#00212b",
      "--bg": "#002b36",
      "--bg-soft": "#073642",
      "--bg-hover": "#0e4451",
      "--border": "#586e75",
      "--fg": "#839496",
      "--fg-strong": "#93a1a1",
      "--fg-muted": "#586e75",
      "--accent": "#268bd2",
      "--red": "#dc322f",
      "--green": "#859900",
      "--yellow": "#b58900",
      "--blue": "#268bd2",
      "--purple": "#6c71c4",
      "--aqua": "#2aa198",
      "--chart-1": "#268bd2",
      "--chart-2": "#cb4b16",
      "--chart-3": "#859900",
      "--chart-4": "#6c71c4",
      "--chart-5": "#b58900",
      "--chart-6": "#2aa198",
      "--chart-7": "#dc322f",
    },
    [Mode.Light]: {
      "--bg-hard": "#eee8d5",
      "--bg": "#fdf6e3",
      "--bg-soft": "#f5eed6",
      "--bg-hover": "#eee8d5",
      "--border": "#d3cbb7",
      "--fg": "#657b83",
      "--fg-strong": "#073642",
      "--fg-muted": "#93a1a1",
      "--accent": "#268bd2",
      "--red": "#dc322f",
      "--green": "#859900",
      "--yellow": "#b58900",
      "--blue": "#268bd2",
      "--purple": "#6c71c4",
      "--aqua": "#2aa198",
      "--chart-1": "#268bd2",
      "--chart-2": "#cb4b16",
      "--chart-3": "#859900",
      "--chart-4": "#6c71c4",
      "--chart-5": "#b58900",
      "--chart-6": "#2aa198",
      "--chart-7": "#dc322f",
    },
  },
};

// ---------------------------------------------------------------------------
// State + application
// ---------------------------------------------------------------------------

function isMode(value: unknown): value is Mode {
  return Object.values(Mode).includes(value as Mode);
}

function isColorscheme(value: unknown): value is Colorscheme {
  return Object.values(Colorscheme).includes(value as Colorscheme);
}

function initialMode(): Mode {
  const value = localStorage.getItem(StorageKey.Mode);
  return isMode(value) ? value : Mode.Dark;
}

function initialColorscheme(): Colorscheme {
  const value = localStorage.getItem(StorageKey.Colorscheme);
  return isColorscheme(value) ? value : Colorscheme.Nord;
}

export const theme = $state<{ mode: Mode; colorscheme: Colorscheme }>({
  mode: initialMode(),
  colorscheme: initialColorscheme(),
});

/** Write the active palette's tokens onto <html> as CSS custom properties. */
function applyTheme(): void {
  const root = document.documentElement;
  const tokens = THEMES[theme.colorscheme][theme.mode];
  for (const name of TOKEN_NAMES) {
    root.style.setProperty(name, tokens[name]);
  }
  // Native widgets (scrollbars, select popups) follow the mode too.
  root.style.colorScheme = theme.mode;
}

// Module init runs on first import, before Svelte mounts anything —
// the restored theme is in place for the very first paint.
applyTheme();

export function toggleMode(): void {
  theme.mode = theme.mode === Mode.Dark ? Mode.Light : Mode.Dark;
  applyTheme();
  localStorage.setItem(StorageKey.Mode, theme.mode);
}

export function setColorscheme(colorscheme: Colorscheme): void {
  theme.colorscheme = colorscheme;
  applyTheme();
  localStorage.setItem(StorageKey.Colorscheme, colorscheme);
}
