/**
 * Navigation history — back/forward over the webview's own history stack,
 * plus the bookkeeping the webview won't do for us.
 *
 * SvelteKit already pushes an entry per goto()/link, so history.back()
 * works; what's missing is (a) knowing whether back/forward CAN go
 * anywhere (the buttons' enabled state — `history` exposes a length but
 * no position) and (b) making in-page SELECTIONS entries too, so back
 * steps through the incidents you looked at, not just the pages.
 *
 * Position is tracked from afterNavigate: a push moves the cursor to a new
 * end, a popstate moves it by `delta`, a replace leaves it. Replace vs push
 * is told apart by whether history.length grew — the one signal the
 * browser gives.
 */
import { afterNavigate, goto } from "$app/navigation";
import { page } from "$app/state";

export const nav = $state({ index: 0, length: 1 });

let lastLength = 0;
let lastChange = 0;

/** call once from the root layout (afterNavigate must run in a component) */
export function trackHistory(): void {
	lastLength = history.length;
	afterNavigate((n) => {
		const now = Date.now();
		if (n.type === "popstate") {
			nav.index = Math.max(0, Math.min(nav.length - 1, nav.index + (n.delta ?? 0)));
		} else if (n.type !== "enter") {
			if (history.length > lastLength) {
				// pushed: everything "forward" of here is gone
				nav.index += 1;
				nav.length = nav.index + 1;
			}
			// else replaced: cursor stays
		}
		lastLength = history.length;
		lastChange = now;
	});
}

export const canBack = () => nav.index > 0;
export const canForward = () => nav.index < nav.length - 1;
export function back(): void {
	if (canBack()) history.back();
}
export function forward(): void {
	if (canForward()) history.forward();
}

/**
 * Put a selection into the URL (?t=<ms>) as a history entry. Rapid
 * successive selections — j/k stepping, or the deep-link handler snapping
 * a jump to the nearest row — REPLACE the previous entry instead of
 * stacking one per keystroke; a click after a pause pushes.
 */
const COALESCE_MS = 600;
export function pushSelection(timestamp: number): void {
	const url = new URL(page.url);
	const current = url.searchParams.get("t");
	const value = String(timestamp);
	if (current === value) return;
	const replace = current !== null && Date.now() - lastChange < COALESCE_MS;
	url.searchParams.set("t", value);
	void goto(url, { replaceState: replace, keepFocus: true, noScroll: true });
}
