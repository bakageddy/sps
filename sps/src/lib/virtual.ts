/**
 * Fixed-row-height list virtualization: given the scroll position and the
 * viewport, which rows to render and how much empty space to pad above and
 * below so the scrollbar still spans the whole list.
 *
 * The one invariant callers must uphold: every row is EXACTLY `rowHeight`
 * px tall (enforce it in CSS — height + box-sizing + overflow hidden). A
 * 35k-row list then costs the same as a 35-row one; the rest of the rows
 * exist only as padding.
 */
export interface VirtualWindow {
	/** first rendered index (inclusive) */
	start: number;
	/** last rendered index (exclusive) */
	end: number;
	padTop: number;
	padBottom: number;
}

export function virtualWindow(
	scrollTop: number,
	viewport: number,
	rowHeight: number,
	total: number,
	overscan = 10,
): VirtualWindow {
	if (total === 0) return { start: 0, end: 0, padTop: 0, padBottom: 0 };
	const visible = Math.ceil(Math.max(viewport, rowHeight) / rowHeight);
	// clamp: the list may have shrunk under a deep scroll position (filter
	// toggled), leaving scrollTop pointing past the end
	const first = Math.min(
		Math.max(0, total - visible),
		Math.floor(Math.max(0, scrollTop) / rowHeight),
	);
	const start = Math.max(0, first - overscan);
	const end = Math.min(total, first + visible + overscan);
	return {
		start,
		end,
		padTop: start * rowHeight,
		padBottom: (total - end) * rowHeight,
	};
}
