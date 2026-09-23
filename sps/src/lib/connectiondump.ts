/**
 * Connection-dump presentation helpers (pure functions, no state).
 */

/**
 * Stack-frame prefixes that are connection/persistence plumbing — the fixed
 * sandwich every trace shares. The first frame NOT matching any of these is
 * the "owner" frame: the application code that actually wanted the
 * connection. Sole home of this heuristic — the backend ships raw stacks
 * (ConnDumpHolder.stackTrace) and never derives owner frames.
 */
const PLUMBING = [
	"java.",
	"jdk.",
	"com.zoho.cp.",
	"com.zoho.mickey.",
	"com.adventnet.ds.",
	"com.adventnet.db.",
	"com.adventnet.persistence",
	"com.adventnet.mfw.",
	"com.adventnet.authorization.",
	"com.manageengine.mdh.QueryInterceptor",
	// thin DataAccess wrappers — naming them would tell you nothing
	"com.adventnet.servicedesk.utils.DataAccessUtil",
	"com.manageengine.sdpod.v3api.utils.DataAccessUtil",
	"com.adventnet.servicedesk.utils.ResourcesUtil",
	"com.adventnet.servicedesk.utils.DBUtilities",
	"com.manageengine.servicedesk.utils.SDQueryExecutor",
	"com.manageengine.servicedesk.v3api.utils.DBUtilitiesOP",
];

/** First non-plumbing frame of a stack, or null when it's plumbing all the
 *  way down. */
export function appFrame(stack: string[]): string | null {
	for (const frame of stack) {
		if (!PLUMBING.some((p) => frame.startsWith(p))) return frame;
	}
	return null;
}

/**
 * HTTP worker thread names carry the request: "<url>-<epoch>_###_<thread>".
 * Returns the url part for those, the name unchanged otherwise.
 */
export function threadLabel(threadName: string): string {
	const i = threadName.indexOf("_###_");
	if (i === -1) return threadName;
	const head = threadName.slice(0, i);
	// strip the trailing "-<epoch millis>" stamp if present
	return head.replace(/-\d{10,}$/, "");
}

/** Alarm tick color per cause (CSS custom property reference). */
export function causeColor(cause: string): string {
	switch (cause) {
		case "No ManagedConnections":
			return "var(--red)";
		case "High CPU":
			return "var(--yellow)";
		case "High Memory Consumption":
			return "var(--chart-4)";
		default: // URL invocation and anything new
			return "var(--chart-6)";
	}
}
