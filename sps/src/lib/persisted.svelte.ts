/**
 * localStorage-backed reactive state.
 *
 * A factory returning an object whose `value` is a getter/setter pair over
 * a closed-over $state variable. Two Svelte 5 ideas worth internalizing:
 *
 *  - Runes work in closures: each persisted() call gets its OWN reactive
 *    `value`, captured by the returned accessors. This is how you build
 *    reusable reactive primitives without classes or stores.
 *  - Accessors keep reactivity across module boundaries. Returning the raw
 *    value would snapshot it; returning `{ get value() {...} }` means every
 *    read goes through the getter, so effects/templates that read it are
 *    tracked as usual.
 *
 * Writes go to localStorage synchronously — these are small UI prefs, not
 * data. Malformed stored JSON falls back to the initial value.
 */
export function persisted<T>(key: string, initial: T) {
  let value = $state<T>(read());

  function read(): T {
    try {
      const raw = localStorage.getItem(key);
      return raw === null ? initial : (JSON.parse(raw) as T);
    } catch {
      return initial;
    }
  }

  return {
    get value(): T {
      return value;
    },
    set value(next: T) {
      value = next;
      localStorage.setItem(key, JSON.stringify(next));
    },
  };
}
