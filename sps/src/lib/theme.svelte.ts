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
	RosePine: "rosepine",
	RosePineMoon: "rosepine-moon",
	KansoZen: "kanso-zen",
	KansoMist: "kanso-mist",
	Catppuccin: "catppuccin",
	Base16: "base16",
	TokyoNight: "tokyonight",
	Dracula: "dracula",
	Everforest: "everforest",
	Ayu: "ayu",
	GitHub: "github",
	Monokai: "monokai",
	Zenburn: "zenburn",
	Poimandres: "poimandres",
	Oxocarbon: "oxocarbon",
} as const;
export type Colorscheme = (typeof Colorscheme)[keyof typeof Colorscheme];

/** Display names for the selector UI. */
export const COLORSCHEME_LABELS: Record<Colorscheme, string> = {
	[Colorscheme.Nord]: "Nord",
	[Colorscheme.Gruvbox]: "Gruvbox",
	[Colorscheme.Modus]: "Modus",
	[Colorscheme.Kanagawa]: "Kanagawa Dragon",
	[Colorscheme.OneDark]: "One Dark",
	[Colorscheme.Solarized]: "Solarized",
	[Colorscheme.RosePine]: "Rosé Pine",
	[Colorscheme.RosePineMoon]: "Rosé Pine Moon",
	[Colorscheme.KansoZen]: "Kanso Zen",
	[Colorscheme.KansoMist]: "Kanso Mist",
	[Colorscheme.Catppuccin]: "Catppuccin Mocha",
	[Colorscheme.Base16]: "Base16 Classic",
	[Colorscheme.TokyoNight]: "Tokyo Night",
	[Colorscheme.Dracula]: "Dracula",
	[Colorscheme.Everforest]: "Everforest",
	[Colorscheme.Ayu]: "Ayu Dark",
	[Colorscheme.GitHub]: "GitHub Dark",
	[Colorscheme.Monokai]: "Monokai",
	[Colorscheme.Zenburn]: "Zenburn",
	[Colorscheme.Poimandres]: "Poimandres",
	[Colorscheme.Oxocarbon]: "Oxocarbon",
};

const StorageKey = {
	Mode: "theme",
	Colorscheme: "palette",
} as const;

// ---------------------------------------------------------------------------
// The themes
// ---------------------------------------------------------------------------

const TOKEN_NAMES = [
	"--bg-hard", // deepest surface: sidebar, code blocks, inputs
	"--bg", // main background
	"--bg-soft", // raised surfaces: panels, table header
	"--bg-hover", // hover/active fills
	"--border",
	"--fg", // primary text
	"--fg-strong", // headings, emphasized numbers
	"--fg-muted", // secondary text, labels, axis ticks
	"--accent", // interactive highlights
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

/** A colorscheme always has a dark palette; light is OPTIONAL — several
 * upstream themes ship none, and we don't invent one (a dark-only scheme
 * simply stays dark when the mode toggle says light). */
type Palette = { [Mode.Dark]: Tokens; [Mode.Light]?: Tokens };

// Palette values are transcribed from each project's published colors;
// where a theme defines no slot for a token (surface steps, muted text) the
// value is interpolated and the comment says so.
const THEMES: Record<Colorscheme, Palette> = {
	// Nord (https://www.nordtheme.com) — Polar Night / Snow Storm / Frost / Aurora. Light mode darkens the Aurora hues: they're tuned for dark backgrounds and wash out as text or 1.5px chart lines on Snow Storm.
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

	// Modus (https://protesilaos.com/emacs/modus-themes) — high contrast. Dark = modus-vivendi (true black; relies on borders, not surface shades, to separate regions), light = modus-operandi.
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

	// Kanagawa (https://github.com/rebornix/kanagawa) — dark = Dragon (the muted, low-saturation variant; black1/3/4/5/6 surfaces), light = Lotus.
	[Colorscheme.Kanagawa]: {
		[Mode.Dark]: {
			"--bg-hard": "#12120f",
			"--bg": "#181616",
			"--bg-soft": "#282727",
			"--bg-hover": "#393836",
			"--border": "#625e5a",
			"--fg": "#c5c9c5",
			"--fg-strong": "#c8c093",
			"--fg-muted": "#a6a69c",
			"--accent": "#8ba4b0",
			"--red": "#c4746e",
			"--green": "#8a9a7b",
			"--yellow": "#c4b28a",
			"--blue": "#8ba4b0",
			"--purple": "#a292a3",
			"--aqua": "#8ea4a2",
			"--chart-1": "#8ba4b0",
			"--chart-2": "#b6927b",
			"--chart-3": "#8a9a7b",
			"--chart-4": "#a292a3",
			"--chart-5": "#c4b28a",
			"--chart-6": "#8ea4a2",
			"--chart-7": "#c4746e",
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

	// Solarized (https://ethanschoonover.com/solarized) — symmetric base palette: dark uses base03/02 surfaces with base0/1 text, light flips to base3/2 with base00/02. The eight accents work on both, so they're shared.
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

	// Rosé Pine (https://rosepinetheme.com) — main; light = Dawn. No green in the palette: pine stands in for it.
	[Colorscheme.RosePine]: {
		[Mode.Dark]: {
			"--bg-hard": "#191724",
			"--bg": "#1f1d2e",
			"--bg-soft": "#26233a",
			"--bg-hover": "#403d52",
			"--border": "#524f67",
			"--fg": "#e0def4",
			"--fg-strong": "#e0def4",
			"--fg-muted": "#908caa",
			"--accent": "#9ccfd8",
			"--red": "#eb6f92",
			"--green": "#31748f",
			"--yellow": "#f6c177",
			"--blue": "#31748f",
			"--purple": "#c4a7e7",
			"--aqua": "#9ccfd8",
			"--chart-1": "#c4a7e7",
			"--chart-2": "#f6c177",
			"--chart-3": "#9ccfd8",
			"--chart-4": "#ebbcba",
			"--chart-5": "#31748f",
			"--chart-6": "#eb6f92",
			"--chart-7": "#908caa",
		},
		[Mode.Light]: {
			"--bg-hard": "#f2e9e1",
			"--bg": "#faf4ed",
			"--bg-soft": "#fffaf3",
			"--bg-hover": "#dfdad9",
			"--border": "#cecacd",
			"--fg": "#575279",
			"--fg-strong": "#26233a",
			"--fg-muted": "#797593",
			"--accent": "#56949f",
			"--red": "#b4637a",
			"--green": "#286983",
			"--yellow": "#ea9d34",
			"--blue": "#56949f",
			"--purple": "#907aa9",
			"--aqua": "#56949f",
			"--chart-1": "#907aa9",
			"--chart-2": "#ea9d34",
			"--chart-3": "#56949f",
			"--chart-4": "#d7827e",
			"--chart-5": "#286983",
			"--chart-6": "#b4637a",
			"--chart-7": "#797593",
		},
	},

	// Rosé Pine Moon — the lifted-black variant; light = Dawn.
	[Colorscheme.RosePineMoon]: {
		[Mode.Dark]: {
			"--bg-hard": "#232136",
			"--bg": "#2a273f",
			"--bg-soft": "#393552",
			"--bg-hover": "#44415a",
			"--border": "#56526e",
			"--fg": "#e0def4",
			"--fg-strong": "#e0def4",
			"--fg-muted": "#908caa",
			"--accent": "#9ccfd8",
			"--red": "#eb6f92",
			"--green": "#3e8fb0",
			"--yellow": "#f6c177",
			"--blue": "#3e8fb0",
			"--purple": "#c4a7e7",
			"--aqua": "#9ccfd8",
			"--chart-1": "#c4a7e7",
			"--chart-2": "#f6c177",
			"--chart-3": "#9ccfd8",
			"--chart-4": "#ea9a97",
			"--chart-5": "#3e8fb0",
			"--chart-6": "#eb6f92",
			"--chart-7": "#908caa",
		},
		[Mode.Light]: {
			"--bg-hard": "#f2e9e1",
			"--bg": "#faf4ed",
			"--bg-soft": "#fffaf3",
			"--bg-hover": "#dfdad9",
			"--border": "#cecacd",
			"--fg": "#575279",
			"--fg-strong": "#26233a",
			"--fg-muted": "#797593",
			"--accent": "#56949f",
			"--red": "#b4637a",
			"--green": "#286983",
			"--yellow": "#ea9d34",
			"--blue": "#56949f",
			"--purple": "#907aa9",
			"--aqua": "#56949f",
			"--chart-1": "#907aa9",
			"--chart-2": "#ea9d34",
			"--chart-3": "#56949f",
			"--chart-4": "#d7827e",
			"--chart-5": "#286983",
			"--chart-6": "#b4637a",
			"--chart-7": "#797593",
		},
	},

	// Kanso (https://github.com/webhooked/kanso.nvim) — Zen, the near-black variant. Dark only (no Pearl by request). Surface steps interpolated.
	[Colorscheme.KansoZen]: {
		[Mode.Dark]: {
			"--bg-hard": "#05080b",
			"--bg": "#090e13",
			"--bg-soft": "#14171d",
			"--bg-hover": "#1c1f26",
			"--border": "#2a2e36",
			"--fg": "#c5c9c7",
			"--fg-strong": "#e2e5e3",
			"--fg-muted": "#717c7c",
			"--accent": "#8ba4b0",
			"--red": "#c4746e",
			"--green": "#8a9a7b",
			"--yellow": "#c4b28a",
			"--blue": "#8ba4b0",
			"--purple": "#a292a3",
			"--aqua": "#8ea4a2",
			"--chart-1": "#8ba4b0",
			"--chart-2": "#b6927b",
			"--chart-3": "#8a9a7b",
			"--chart-4": "#a292a3",
			"--chart-5": "#c4b28a",
			"--chart-6": "#8ea4a2",
			"--chart-7": "#c4746e",
		},
	},

	// Kanso — Mist, the soft grey variant. Dark only.
	[Colorscheme.KansoMist]: {
		[Mode.Dark]: {
			"--bg-hard": "#1a1d23",
			"--bg": "#22262d",
			"--bg-soft": "#2a2e36",
			"--bg-hover": "#333840",
			"--border": "#404550",
			"--fg": "#c5c9c7",
			"--fg-strong": "#e2e5e3",
			"--fg-muted": "#8a9198",
			"--accent": "#8ba4b0",
			"--red": "#c4746e",
			"--green": "#8a9a7b",
			"--yellow": "#c4b28a",
			"--blue": "#8ba4b0",
			"--purple": "#a292a3",
			"--aqua": "#8ea4a2",
			"--chart-1": "#8ba4b0",
			"--chart-2": "#b6927b",
			"--chart-3": "#8a9a7b",
			"--chart-4": "#a292a3",
			"--chart-5": "#c4b28a",
			"--chart-6": "#8ea4a2",
			"--chart-7": "#c4746e",
		},
	},

	// Catppuccin (https://catppuccin.com) — Mocha only, by request.
	[Colorscheme.Catppuccin]: {
		[Mode.Dark]: {
			"--bg-hard": "#11111b",
			"--bg": "#1e1e2e",
			"--bg-soft": "#313244",
			"--bg-hover": "#45475a",
			"--border": "#585b70",
			"--fg": "#cdd6f4",
			"--fg-strong": "#f5e0dc",
			"--fg-muted": "#a6adc8",
			"--accent": "#89b4fa",
			"--red": "#f38ba8",
			"--green": "#a6e3a1",
			"--yellow": "#f9e2af",
			"--blue": "#89b4fa",
			"--purple": "#cba6f7",
			"--aqua": "#94e2d5",
			"--chart-1": "#89b4fa",
			"--chart-2": "#fab387",
			"--chart-3": "#a6e3a1",
			"--chart-4": "#cba6f7",
			"--chart-5": "#f9e2af",
			"--chart-6": "#94e2d5",
			"--chart-7": "#f38ba8",
		},
	},

	// Base16 Classic (Chris Kempson) — dark / light. Accents shared; base03/04 are too extreme for muted text, so it's interpolated.
	[Colorscheme.Base16]: {
		[Mode.Dark]: {
			"--bg-hard": "#151515",
			"--bg": "#202020",
			"--bg-soft": "#303030",
			"--bg-hover": "#3a3a3a",
			"--border": "#505050",
			"--fg": "#d0d0d0",
			"--fg-strong": "#f5f5f5",
			"--fg-muted": "#8a8a8a",
			"--accent": "#6a9fb5",
			"--red": "#ac4142",
			"--green": "#90a959",
			"--yellow": "#f4bf75",
			"--blue": "#6a9fb5",
			"--purple": "#aa759f",
			"--aqua": "#75b5aa",
			"--chart-1": "#6a9fb5",
			"--chart-2": "#d28445",
			"--chart-3": "#90a959",
			"--chart-4": "#aa759f",
			"--chart-5": "#f4bf75",
			"--chart-6": "#75b5aa",
			"--chart-7": "#ac4142",
		},
		[Mode.Light]: {
			"--bg-hard": "#e0e0e0",
			"--bg": "#f5f5f5",
			"--bg-soft": "#ebebeb",
			"--bg-hover": "#d0d0d0",
			"--border": "#b0b0b0",
			"--fg": "#303030",
			"--fg-strong": "#151515",
			"--fg-muted": "#6a6a6a",
			"--accent": "#6a9fb5",
			"--red": "#ac4142",
			"--green": "#90a959",
			"--yellow": "#d28445",
			"--blue": "#6a9fb5",
			"--purple": "#aa759f",
			"--aqua": "#75b5aa",
			"--chart-1": "#6a9fb5",
			"--chart-2": "#d28445",
			"--chart-3": "#90a959",
			"--chart-4": "#aa759f",
			"--chart-5": "#f4bf75",
			"--chart-6": "#75b5aa",
			"--chart-7": "#ac4142",
		},
	},

	// Tokyo Night (https://github.com/folke/tokyonight.nvim) — Night; light = Day.
	[Colorscheme.TokyoNight]: {
		[Mode.Dark]: {
			"--bg-hard": "#16161e",
			"--bg": "#1a1b26",
			"--bg-soft": "#24283b",
			"--bg-hover": "#292e42",
			"--border": "#3b4261",
			"--fg": "#c0caf5",
			"--fg-strong": "#e0e6ff",
			"--fg-muted": "#737aa2",
			"--accent": "#7aa2f7",
			"--red": "#f7768e",
			"--green": "#9ece6a",
			"--yellow": "#e0af68",
			"--blue": "#7aa2f7",
			"--purple": "#bb9af7",
			"--aqua": "#7dcfff",
			"--chart-1": "#7aa2f7",
			"--chart-2": "#ff9e64",
			"--chart-3": "#9ece6a",
			"--chart-4": "#bb9af7",
			"--chart-5": "#e0af68",
			"--chart-6": "#7dcfff",
			"--chart-7": "#f7768e",
		},
		[Mode.Light]: {
			"--bg-hard": "#d5d6db",
			"--bg": "#e1e2e7",
			"--bg-soft": "#e9e9ec",
			"--bg-hover": "#c4c8da",
			"--border": "#a8aecb",
			"--fg": "#3760bf",
			"--fg-strong": "#1a1b26",
			"--fg-muted": "#848cb5",
			"--accent": "#2e7de9",
			"--red": "#f52a65",
			"--green": "#587539",
			"--yellow": "#8c6c3e",
			"--blue": "#2e7de9",
			"--purple": "#9854f1",
			"--aqua": "#007197",
			"--chart-1": "#2e7de9",
			"--chart-2": "#b15c00",
			"--chart-3": "#587539",
			"--chart-4": "#9854f1",
			"--chart-5": "#8c6c3e",
			"--chart-6": "#007197",
			"--chart-7": "#f52a65",
		},
	},

	// Dracula (https://draculatheme.com); light = Alucard, its official light counterpart.
	[Colorscheme.Dracula]: {
		[Mode.Dark]: {
			"--bg-hard": "#21222c",
			"--bg": "#282a36",
			"--bg-soft": "#343746",
			"--bg-hover": "#44475a",
			"--border": "#6272a4",
			"--fg": "#f8f8f2",
			"--fg-strong": "#ffffff",
			"--fg-muted": "#6272a4",
			"--accent": "#bd93f9",
			"--red": "#ff5555",
			"--green": "#50fa7b",
			"--yellow": "#f1fa8c",
			"--blue": "#8be9fd",
			"--purple": "#bd93f9",
			"--aqua": "#8be9fd",
			"--chart-1": "#bd93f9",
			"--chart-2": "#ffb86c",
			"--chart-3": "#50fa7b",
			"--chart-4": "#ff79c6",
			"--chart-5": "#f1fa8c",
			"--chart-6": "#8be9fd",
			"--chart-7": "#ff5555",
		},
		[Mode.Light]: {
			"--bg-hard": "#f5f0dc",
			"--bg": "#fffbeb",
			"--bg-soft": "#f7f2e0",
			"--bg-hover": "#ebe5cf",
			"--border": "#cfc9b0",
			"--fg": "#1f1f1f",
			"--fg-strong": "#000000",
			"--fg-muted": "#6c664b",
			"--accent": "#644ac9",
			"--red": "#cb3a2a",
			"--green": "#14710a",
			"--yellow": "#846e15",
			"--blue": "#036a96",
			"--purple": "#644ac9",
			"--aqua": "#036a96",
			"--chart-1": "#644ac9",
			"--chart-2": "#a34d14",
			"--chart-3": "#14710a",
			"--chart-4": "#a3144d",
			"--chart-5": "#846e15",
			"--chart-6": "#036a96",
			"--chart-7": "#cb3a2a",
		},
	},

	// Everforest (https://github.com/sainnhe/everforest) — medium contrast, dark / light.
	[Colorscheme.Everforest]: {
		[Mode.Dark]: {
			"--bg-hard": "#232a2e",
			"--bg": "#2d353b",
			"--bg-soft": "#343f44",
			"--bg-hover": "#3d484d",
			"--border": "#475258",
			"--fg": "#d3c6aa",
			"--fg-strong": "#d3c6aa",
			"--fg-muted": "#859289",
			"--accent": "#a7c080",
			"--red": "#e67e80",
			"--green": "#a7c080",
			"--yellow": "#dbbc7f",
			"--blue": "#7fbbb3",
			"--purple": "#d699b6",
			"--aqua": "#83c092",
			"--chart-1": "#a7c080",
			"--chart-2": "#e69875",
			"--chart-3": "#7fbbb3",
			"--chart-4": "#d699b6",
			"--chart-5": "#dbbc7f",
			"--chart-6": "#83c092",
			"--chart-7": "#e67e80",
		},
		[Mode.Light]: {
			"--bg-hard": "#efebd4",
			"--bg": "#fdf6e3",
			"--bg-soft": "#f4f0d9",
			"--bg-hover": "#e6e2cc",
			"--border": "#bdc3af",
			"--fg": "#5c6a72",
			"--fg-strong": "#333c43",
			"--fg-muted": "#939f91",
			"--accent": "#8da101",
			"--red": "#f85552",
			"--green": "#8da101",
			"--yellow": "#dfa000",
			"--blue": "#3a94c5",
			"--purple": "#df69ba",
			"--aqua": "#35a77c",
			"--chart-1": "#8da101",
			"--chart-2": "#f57d26",
			"--chart-3": "#3a94c5",
			"--chart-4": "#df69ba",
			"--chart-5": "#dfa000",
			"--chart-6": "#35a77c",
			"--chart-7": "#f85552",
		},
	},

	// Ayu (https://github.com/ayu-theme/ayu-colors) — Dark only, by request.
	[Colorscheme.Ayu]: {
		[Mode.Dark]: {
			"--bg-hard": "#0b0e14",
			"--bg": "#0d1017",
			"--bg-soft": "#0f131a",
			"--bg-hover": "#131721",
			"--border": "#262e3a",
			"--fg": "#bfbdb6",
			"--fg-strong": "#e6e1cf",
			"--fg-muted": "#565b66",
			"--accent": "#e6b450",
			"--red": "#d95757",
			"--green": "#aad94c",
			"--yellow": "#ffb454",
			"--blue": "#59c2ff",
			"--purple": "#d2a6ff",
			"--aqua": "#95e6cb",
			"--chart-1": "#e6b450",
			"--chart-2": "#ff8f40",
			"--chart-3": "#aad94c",
			"--chart-4": "#d2a6ff",
			"--chart-5": "#59c2ff",
			"--chart-6": "#95e6cb",
			"--chart-7": "#d95757",
		},
	},

	// GitHub Dark (Primer, default dark) — dark only, by request.
	[Colorscheme.GitHub]: {
		[Mode.Dark]: {
			"--bg-hard": "#010409",
			"--bg": "#0d1117",
			"--bg-soft": "#161b22",
			"--bg-hover": "#21262d",
			"--border": "#30363d",
			"--fg": "#c9d1d9",
			"--fg-strong": "#f0f6fc",
			"--fg-muted": "#8b949e",
			"--accent": "#58a6ff",
			"--red": "#f85149",
			"--green": "#3fb950",
			"--yellow": "#d29922",
			"--blue": "#58a6ff",
			"--purple": "#a371f7",
			"--aqua": "#39c5cf",
			"--chart-1": "#58a6ff",
			"--chart-2": "#db6d28",
			"--chart-3": "#3fb950",
			"--chart-4": "#a371f7",
			"--chart-5": "#d29922",
			"--chart-6": "#39c5cf",
			"--chart-7": "#f85149",
		},
	},

	// Monokai (Wimer Hazenberg's original) — dark only.
	[Colorscheme.Monokai]: {
		[Mode.Dark]: {
			"--bg-hard": "#1e1f1c",
			"--bg": "#272822",
			"--bg-soft": "#3e3d32",
			"--bg-hover": "#49483e",
			"--border": "#75715e",
			"--fg": "#f8f8f2",
			"--fg-strong": "#ffffff",
			"--fg-muted": "#a59f85",
			"--accent": "#a6e22e",
			"--red": "#f92672",
			"--green": "#a6e22e",
			"--yellow": "#e6db74",
			"--blue": "#66d9ef",
			"--purple": "#ae81ff",
			"--aqua": "#66d9ef",
			"--chart-1": "#a6e22e",
			"--chart-2": "#fd971f",
			"--chart-3": "#66d9ef",
			"--chart-4": "#ae81ff",
			"--chart-5": "#e6db74",
			"--chart-6": "#f92672",
			"--chart-7": "#75715e",
		},
	},

	// Zenburn (Jani Nurminen) — dark only, low contrast by design.
	[Colorscheme.Zenburn]: {
		[Mode.Dark]: {
			"--bg-hard": "#2b2b2b",
			"--bg": "#3f3f3f",
			"--bg-soft": "#4f4f4f",
			"--bg-hover": "#5f5f5f",
			"--border": "#6f6f6f",
			"--fg": "#dcdccc",
			"--fg-strong": "#ffffef",
			"--fg-muted": "#989890",
			"--accent": "#8cd0d3",
			"--red": "#cc9393",
			"--green": "#7f9f7f",
			"--yellow": "#f0dfaf",
			"--blue": "#94bff3",
			"--purple": "#dc8cc3",
			"--aqua": "#93e0e3",
			"--chart-1": "#8cd0d3",
			"--chart-2": "#dfaf8f",
			"--chart-3": "#7f9f7f",
			"--chart-4": "#dc8cc3",
			"--chart-5": "#f0dfaf",
			"--chart-6": "#94bff3",
			"--chart-7": "#cc9393",
		},
	},

	// Poimandres (https://github.com/drcmda/poimandres-theme) — dark only.
	[Colorscheme.Poimandres]: {
		[Mode.Dark]: {
			"--bg-hard": "#171922",
			"--bg": "#1b1e28",
			"--bg-soft": "#252b37",
			"--bg-hover": "#303340",
			"--border": "#3b3f51",
			"--fg": "#a6accd",
			"--fg-strong": "#e4f0fb",
			"--fg-muted": "#767c9d",
			"--accent": "#5de4c7",
			"--red": "#d0679d",
			"--green": "#5de4c7",
			"--yellow": "#fffac2",
			"--blue": "#89ddff",
			"--purple": "#fcc5e9",
			"--aqua": "#5fb3a1",
			"--chart-1": "#5de4c7",
			"--chart-2": "#fcc5e9",
			"--chart-3": "#89ddff",
			"--chart-4": "#add7ff",
			"--chart-5": "#fffac2",
			"--chart-6": "#5fb3a1",
			"--chart-7": "#d0679d",
		},
	},

	// Oxocarbon (https://github.com/nyoom-engineering/oxocarbon.nvim, IBM Carbon colors) — dark / light. Carbon has no yellow slot; yellow-30 stands in.
	[Colorscheme.Oxocarbon]: {
		[Mode.Dark]: {
			"--bg-hard": "#101010",
			"--bg": "#161616",
			"--bg-soft": "#262626",
			"--bg-hover": "#393939",
			"--border": "#525252",
			"--fg": "#dde1e6",
			"--fg-strong": "#f2f4f8",
			"--fg-muted": "#a2a9b0",
			"--accent": "#3ddbd9",
			"--red": "#ee5396",
			"--green": "#42be65",
			"--yellow": "#f1c21b",
			"--blue": "#78a9ff",
			"--purple": "#be95ff",
			"--aqua": "#08bdba",
			"--chart-1": "#3ddbd9",
			"--chart-2": "#78a9ff",
			"--chart-3": "#42be65",
			"--chart-4": "#be95ff",
			"--chart-5": "#ee5396",
			"--chart-6": "#33b1ff",
			"--chart-7": "#ff7eb6",
		},
		[Mode.Light]: {
			"--bg-hard": "#dde1e6",
			"--bg": "#f2f4f8",
			"--bg-soft": "#e8ebf0",
			"--bg-hover": "#dde1e6",
			"--border": "#c1c7cd",
			"--fg": "#393939",
			"--fg-strong": "#161616",
			"--fg-muted": "#6f6f6f",
			"--accent": "#0f62fe",
			"--red": "#da1e28",
			"--green": "#198038",
			"--yellow": "#b28600",
			"--blue": "#0f62fe",
			"--purple": "#8a3ffc",
			"--aqua": "#007d79",
			"--chart-1": "#0f62fe",
			"--chart-2": "#ff6f00",
			"--chart-3": "#198038",
			"--chart-4": "#8a3ffc",
			"--chart-5": "#b28600",
			"--chart-6": "#007d79",
			"--chart-7": "#da1e28",
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

/** true when the scheme ships a light palette (the mode toggle is inert otherwise) */
export function hasLight(colorscheme: Colorscheme): boolean {
	return THEMES[colorscheme][Mode.Light] !== undefined;
}

/** the mode actually on screen: light only if the scheme has one */
export function effectiveMode(): Mode {
	return theme.mode === Mode.Light && hasLight(theme.colorscheme)
		? Mode.Light
		: Mode.Dark;
}

/** Write the active palette's tokens onto <html> as CSS custom properties. */
function applyTheme(): void {
	const root = document.documentElement;
	const mode = effectiveMode();
	const tokens = THEMES[theme.colorscheme][mode]!;
	for (const name of TOKEN_NAMES) {
		root.style.setProperty(name, tokens[name]);
	}
	// Native widgets (scrollbars, select popups) follow the mode too.
	root.style.colorScheme = mode;
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
