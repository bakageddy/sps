<script lang="ts">
	/**
	 * The collector: protocol, host, port. Connects on its own once the user
	 * has stopped typing for a couple of seconds (switchable), or at once on
	 * Enter / the Connect button. A pasted URL is split across the three
	 * fields. If the collector wants a login, the form appears right under.
	 *
	 * The password lives only in this component's state — it is handed to the
	 * backend once and never persisted (the connection fields and the
	 * username are).
	 */
	import { untrack } from "svelte";
	import {
		composeUrl,
		connect,
		disconnect,
		form,
		login,
		normaliseHostInput,
		session,
	} from "$lib/glowroot.svelte";

	/** how long typing must pause before an automatic connect */
	const AUTO_DELAY_MS = 2500;

	let password = $state("");
	/** a connect is scheduled and not yet fired — shown so the pause is not a mystery */
	let waiting = $state(false);

	const c = $derived(session.collector);
	const busy = $derived(session.busy === "connect" || session.busy === "login");
	const target = $derived(composeUrl());
	const canLogin = $derived(c !== null && c.loginEnabled && !c.loggedIn);

	let first = true;
	$effect(() => {
		const t = composeUrl(); // tracks protocol, host and port
		const auto = form.autoConnect.value;
		waiting = false;
		if (!auto || "error" in t) return;
		const done = untrack(() => session.attempted === t.url);
		if (done) return; // already tried exactly this
		// on arrival with remembered fields, don't make them wait
		const delay = first ? 300 : AUTO_DELAY_MS;
		first = false;
		waiting = true;
		const timer = setTimeout(() => {
			waiting = false;
			void connect();
		}, delay);
		return () => clearTimeout(timer);
	});

	function onHostInput(e: Event) {
		form.host.value = normaliseHostInput((e.currentTarget as HTMLInputElement).value);
	}
	function onConnect(e: SubmitEvent) {
		e.preventDefault();
		waiting = false;
		void connect();
	}
	function onLogin(e: SubmitEvent) {
		e.preventDefault();
		void login(password).then(() => {
			if (session.error === null) password = "";
		});
	}
</script>

<form class="row" onsubmit={onConnect}>
	<select bind:value={form.protocol.value} aria-label="Protocol">
		<option value="http">http://</option>
		<option value="https">https://</option>
	</select>
	<input
		class="host mono"
		type="text"
		placeholder="host  (or paste a full URL)"
		value={form.host.value}
		oninput={onHostInput}
		spellcheck="false"
		autocapitalize="off"
		autocomplete="off"
		aria-label="Host"
	/>
	<span class="colon">:</span>
	<input
		class="port mono"
		type="text"
		inputmode="numeric"
		placeholder="port"
		bind:value={form.port.value}
		spellcheck="false"
		autocomplete="off"
		aria-label="Port"
	/>
	<button class="primary" type="submit" disabled={"error" in target || session.busy === "connect"}>
		{session.busy === "connect" ? "Connecting…" : c ? "Reconnect" : "Connect"}
	</button>
	{#if c}
		<button type="button" class="quiet" onclick={() => void disconnect()} disabled={busy}>
			Disconnect
		</button>
	{/if}
</form>

<div class="status">
	<label class="auto" title="Connect by itself a moment after you stop typing">
		<input type="checkbox" bind:checked={form.autoConnect.value} /> connect automatically
	</label>
	{#if waiting && session.busy !== "connect"}
		<span class="muted">connecting when you stop typing…</span>
	{/if}
</div>

{#if c}
	<div class="card">
		<span class="badge" class:central={c.central}>{c.central ? "Central" : "Embedded"}</span>
		<span class="mono ver">{c.version}</span>
		<span class="spacer"></span>
		{#if c.loggedIn}
			<span class="ok">logged in</span>
		{:else if c.needsLogin}
			<span class="warn">login required</span>
		{:else}
			<span class="muted">anonymous access</span>
		{/if}
	</div>
{/if}

{#if canLogin}
	<form class="row login" onsubmit={onLogin}>
		<input
			type="text"
			placeholder="username"
			bind:value={form.username.value}
			disabled={busy}
			autocomplete="username"
			spellcheck="false"
			autocapitalize="off"
			aria-label="Username"
		/>
		<input
			type="password"
			placeholder="password"
			bind:value={password}
			disabled={busy}
			autocomplete="current-password"
			aria-label="Password"
		/>
		<button
			class="primary"
			type="submit"
			disabled={busy || form.username.value.trim() === "" || password === ""}
		>
			{session.busy === "login" ? "Logging in…" : "Log in"}
		</button>
		{#if !c?.needsLogin}<span class="muted">optional — more agents may be visible</span>{/if}
	</form>
{/if}

{#if session.error}
	<p class="error" role="alert">{session.error}</p>
{/if}

<style>
	.mono {
		font-family: var(--font-mono);
	}
	.row {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}
	.login {
		margin-top: 10px;
	}
	input[type="text"],
	input[type="password"],
	select {
		padding: 6px 10px;
		background: var(--bg-hard);
		border: none;
		border-radius: var(--radius);
		font-size: 13px;
	}
	select {
		cursor: pointer;
	}
	.host {
		flex: 1;
		min-width: 200px;
	}
	.port {
		width: 84px;
	}
	.colon {
		color: var(--fg-muted);
	}
	button {
		padding: 6px 14px;
		font-size: 12px;
		background: var(--bg-hard);
	}
	button:hover:not(:disabled) {
		background: var(--bg-hover);
	}
	button:disabled {
		opacity: 0.5;
		cursor: default;
	}
	button.primary {
		background: var(--accent);
		color: var(--bg);
		font-weight: 700;
	}
	button.primary:hover:not(:disabled) {
		background: var(--accent);
		opacity: 0.85;
	}
	button.quiet {
		background: none;
		color: var(--fg-muted);
	}

	.status {
		display: flex;
		align-items: center;
		gap: 14px;
		margin-top: 8px;
		font-size: 12px;
		min-height: 18px;
	}
	.auto {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--fg-muted);
		cursor: pointer;
	}
	.auto input {
		accent-color: var(--accent);
	}

	.card {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 10px;
		padding: 8px 12px;
		background: var(--bg-soft);
		border-radius: var(--radius);
		font-size: 12px;
	}
	.badge {
		padding: 1px 8px;
		border-radius: 99px;
		background: var(--bg-hard);
		color: var(--fg-strong);
		font-weight: 700;
		text-transform: uppercase;
		letter-spacing: 0.06em;
		font-size: 11px;
	}
	.badge.central {
		color: var(--purple);
	}
	.ver {
		color: var(--fg-muted);
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.spacer {
		flex: 1;
	}
	.ok {
		color: var(--green);
	}
	.warn {
		color: var(--alert);
	}
	.muted {
		color: var(--fg-muted);
		font-size: 12px;
	}
	.error {
		margin: 10px 0 0;
		font-size: 12px;
		color: var(--red);
		white-space: pre-wrap;
	}
</style>
