/** Clipboard write that reports success instead of throwing — callers show
 * a ✓ or an error bar, never a crash. */
export async function copyText(text: string): Promise<boolean> {
	try {
		await navigator.clipboard.writeText(text);
		return true;
	} catch {
		return false;
	}
}
