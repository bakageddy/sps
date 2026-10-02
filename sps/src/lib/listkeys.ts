/**
 * Keyboard stepping for selection lists: j/k and the arrows move one row,
 * PageUp/Down ten, Home/End to the ends. Returns the new index, or null
 * when the key isn't ours (so the event keeps bubbling). `index` = -1 when
 * nothing is selected — the first step then lands on the first row.
 */
export function stepIndex(
	e: KeyboardEvent,
	index: number,
	length: number,
): number | null {
	if (length === 0) return null;
	// typing in a filter box inside the list must not move the selection
	const target = e.target as HTMLElement | null;
	if (target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA"))
		return null;
	const last = length - 1;
	const cur = index < 0 ? -1 : index;
	switch (e.key) {
		case "j":
		case "ArrowDown":
			return Math.min(last, cur + 1);
		case "k":
		case "ArrowUp":
			return cur < 0 ? 0 : Math.max(0, cur - 1);
		case "PageDown":
			return Math.min(last, cur + 10);
		case "PageUp":
			return Math.max(0, cur - 10);
		case "Home":
			return 0;
		case "End":
			return last;
		default:
			return null;
	}
}
