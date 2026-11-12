# Glowroot HTTP API — what sps needs to pull traces

Derived from the Glowroot source at tag `v0.14.8` (`ui/` module). The `ui/`
module is shared by **embedded** Glowroot (agent + UI in the monitored JVM)
and **Glowroot Central**, so every endpoint below exists in both; the
differences are how the agent is addressed and how you authenticate.

Stability: `git log v0.14.0..HEAD` over the trace services shows one
behavioural change (trace-count now honours the same filters as points) and
performance work only. Paths, parameters and JSON shapes are the same across
0.14.0 → 0.14.8.

Everything marked **unverified** was not read in full; verify before relying
on it.

---

## 1. Conventions

- **Query parameters are kebab-case**, mapped to camelCase fields
  (`CaseFormat.LOWER_HYPHEN → LOWER_CAMEL`): `transaction-type`, `from`,
  `duration-millis-low`, `check-live-traces`.
- **Enums** are passed kebab-case too and upper-cased with `-`→`_`:
  `headline-comparator=not-contains`, `sort-order=total-time`.
- **Lists** repeat the parameter: `percentile=50&percentile=95`.
- **Times** are epoch **milliseconds**. `from`/`to` are inclusive bounds;
  `to=0` means "now" and includes still-running traces.
- **Durations in responses are nanoseconds** unless the field name says
  `Millis`.
- Responses are JSON (`application/json; charset=utf-8`). Error status codes
  are plain HTTP (`400` missing param, `401`/`403` not authorised, `404` no
  such trace component).

### Addressing the agent — the embedded/central split

| parameter         | embedded                                        | central                                                        |
| ----------------- | ----------------------------------------------- | -------------------------------------------------------------- |
| `agent-id`        | **must be present, may be empty** (`agent-id=`) | required, non-empty, the agent's id; must **not** end in `::`  |
| `agent-rollup-id` | **must be present, may be empty**               | required; an agent id **or** a rollup id (rollups end in `::`) |

Which one an endpoint takes is listed per endpoint below. Per-trace
endpoints take `agent-id` (a trace belongs to one agent); listing and
aggregate endpoints take `agent-rollup-id` (they can span a rollup).
Omitting the parameter is a `400` in both modes; sending it empty is what
the embedded UI does.

Discovering agents on central:

| endpoint                               | params                       | purpose                                       |
| -------------------------------------- | ---------------------------- | --------------------------------------------- |
| `GET /backend/top-level-agent-rollups` | `from`, `to`                 | top-level agents/rollups active in the window |
| `GET /backend/child-agent-rollups`     | `top-level-id`, `from`, `to` | children of one top-level rollup              |
| `GET /backend/agent-rollup`            | `id`                         | display info for one id                       |

Response shape of these three is **unverified** (list of
`{id, display, …}` objects per the UI's usage).

---

## 2. Authentication and layout

| endpoint                       | method | body / params                                 | response                                                                                                                               |
| ------------------------------ | ------ | --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `/backend/login`               | POST   | JSON `{"username": "...", "password": "..."}` | `200` + `Set-Cookie: GLOWROOT_SESSION_ID=<id>` on success; `200` with body `{"incorrectLogin": true}` on failure (not a 401)           |
| `/backend/sign-out`            | POST   | –                                             | anonymous layout JSON; clears the cookie                                                                                               |
| `/backend/layout`              | GET    | –                                             | global layout (below)                                                                                                                  |
| `/backend/agent-rollup-layout` | GET    | `agent-rollup-id`                             | per-agent layout (central; embedded has it inline)                                                                                     |
| `/backend/check-layout`        | GET    | –                                             | headers `Glowroot-Layout-Version` (+ `Glowroot-Agent-Rollup-Layout-Version` with `agent-rollup-id`) — cheap "has config changed" probe |

The cookie name is configurable (`sessionCookieName` in web config);
`GLOWROOT_SESSION_ID` is the default for both embedded and central. Send it
on every subsequent request. Sessions idle-expire; `check-layout` does not
touch the session, everything else does. The username `anonymous` cannot
log in; anonymous access works only when the anonymous role carries the
permissions (see `permissions` in the layout).

Permissions the trace endpoints check (per agent rollup):
`agent:transaction:traces`, `agent:error:traces`, `agent:trace`,
`agent:transaction:overview`, `agent:transaction:queries`,
`agent:transaction:threadProfile`.

### `GET /backend/layout`

```jsonc
{
  "central": false,                 // embedded vs central
  "offlineViewer": false,
  "glowrootVersion": "0.14.8, built …",
  "loginEnabled": true,
  "loggedIn": false,
  "ldap": false,
  "redirectToLogin": false,         // true = anonymous has no permissions, must log in
  "adminView": false, "adminEdit": false,
  "showNavbarTransaction": true, "showNavbarError": true, "showNavbarJvm": true,
  "showNavbarSyntheticMonitor": false, "showNavbarIncident": false,
  "showNavbarReport": false, "showNavbarConfig": false,
  "rollupConfigs": [ { "intervalMillis": 60000, "viewThresholdMillis": … }, … ],
  "rollupExpirationMillis": [ … ], "queryAndServiceCallRollupExpirationMillis": [ … ],
  "profileRollupExpirationMillis": [ … ],
  "gaugeCollectionIntervalMillis": 5000,
  "defaultTimeZoneId": "Asia/Kolkata", "timeZoneIds": [ … ],
  "embeddedAgentRollup": { /* AgentRollupLayout, embedded only; null on central */ }
}
```

`AgentRollupLayout` (inline for embedded, via `/backend/agent-rollup-layout`
on central):

```jsonc
{
  "id": "", "topLevelId": "", "topLevelDisplay": "", "childDisplay": "", "lastDisplayPart": "",
  "glowrootVersion": "…",
  "permissions": { "transaction": { "overview": true, "traces": true, "queries": true,
                                    "serviceCalls": true, "threadProfile": true },
                   "error": { "overview": true, "traces": true },
                   "jvm": { "gauges": true, … }, "syntheticMonitor": false, "incident": false,
                   "config": { … } },
  "transactionTypes": [ "Background", "Schedule", "Web" ],   // ← the list to iterate
  "traceAttributeNames": { "Web": [ "Request http method", … ] },  // key = transaction type
  "defaultTransactionType": "Web",
  "defaultPercentiles": [ 50.0, 95.0, 99.0 ],
  "defaultGaugeNames": [ … ]
}
```

`transactionTypes` is the union of what is stored, what is live, and the
configured default. This is where "Web" and "Schedule" come from; don't
hard-code them.

---

## 3. Listing traces in a window

Two parallel families: **slow** traces (`/backend/transaction/…`) and
**error** traces (`/backend/error/…`). Same parameters, same response
shape. A trace that both errored and exceeded the slow threshold is
reachable from both.

| endpoint                               | returns                     |
| -------------------------------------- | --------------------------- |
| `GET /backend/transaction/points`      | slow trace points           |
| `GET /backend/error/points`            | error trace points          |
| `GET /backend/transaction/trace-count` | plain integer, same filters |
| `GET /backend/error/trace-count`       | plain integer, same filters |

Parameters (all take `agent-rollup-id`):

| param                                                             | type         | notes                                                               |
| ----------------------------------------------------------------- | ------------ | ------------------------------------------------------------------- |
| `transaction-type`                                                | string       | **required**                                                        |
| `transaction-name`                                                | string       | optional; exact match                                               |
| `from`, `to`                                                      | ms           | required; `to=0` = now (adds live traces)                           |
| `duration-millis-low`                                             | double       | default 0                                                           |
| `duration-millis-high`                                            | double       | optional                                                            |
| `headline`, `headline-comparator`                                 | string, enum | comparator ∈ `begins`, `equals`, `ends`, `contains`, `not-contains` |
| `error-message`, `error-message-comparator`                       | string, enum | error family only in practice                                       |
| `user`, `user-comparator`                                         | string, enum |                                                                     |
| `attribute-name`, `attribute-value`, `attribute-value-comparator` |              | trace attributes (see `traceAttributeNames`)                        |
| `limit`                                                           | int          | **max points returned; `0` = unlimited** (only trace-count uses 0)  |

Response:

```jsonc
{
  "normalPoints":  [ [captureTime, durationMillis, agentId, traceId], … ],
  "errorPoints":   [ … ],
  "partialPoints": [ … ],      // traces still in progress / partially stored, plus live traces
  "limitExceeded": true,       // present only when more matched than `limit`
  "expired": true              // present only when nothing matched (retention passed or empty)
}
```

Each point is a **positional array**:

| index | value                                                                                                                                                            |
| ----- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0     | `captureTime` ms epoch (when the trace _ended_/was captured)                                                                                                     |
| 1     | duration in **milliseconds** (double)                                                                                                                            |
| 2     | `agentId` (empty string on embedded)                                                                                                                             |
| 3     | `traceId`                                                                                                                                                        |
| 4     | optional `true` — "check live traces": pass `check-live-traces=true` on the detail calls for this trace (central only has to consult the agent when this is set) |

Pagination: there is **no offset**. The UI handles `limitExceeded` by
asking the user to narrow the window or raise the limit. For a bulk pull
over two days, bisect: request with a limit; if `limitExceeded`, split
`[from, to]` in half and recurse. Points carry their own capture time, so
re-sorting and de-duplicating by `(agentId, traceId)` across sub-windows is
safe (a trace appears in one window by its capture time). Combine with
`duration-millis-low` if you only want traces above a threshold.

---

## 4. One trace

All take `agent-id` and `trace-id`, plus optional `check-live-traces=true`
(forward the point's 5th element). On central a just-completed trace may
still be in transit; the server retries internally for a few seconds.

| endpoint                                 | returns                                                           | when missing        |
| ---------------------------------------- | ----------------------------------------------------------------- | ------------------- |
| `GET /backend/trace/header`              | header object                                                     | `{"expired": true}` |
| `GET /backend/trace/entries`             | `{ "entries": [Entry…], "sharedQueryTexts": [SharedQueryText…] }` | `404`               |
| `GET /backend/trace/queries`             | `{ "queries": [Query…], "sharedQueryTexts": [SharedQueryText…] }` | `404`               |
| `GET /backend/trace/main-thread-profile` | profile tree                                                      | `404`               |
| `GET /backend/trace/aux-thread-profile`  | profile tree                                                      | `404`               |
| `GET /export/trace`                      | self-contained HTML of the whole trace                            | –                   |

Check the header's `entriesExistence` / `queriesExistence` /
`profileExistence` (`"yes"`, `"no"`, `"expired"`) before fetching those
parts; `"no"` means the trace never had them, `"expired"` means retention
removed them while the header survived.

### Header

```jsonc
{
  "agent": "display name",              // central only (omitted when agentId is empty)
  "active": true,                       // only when still running
  "partial": true,                      // only when captured mid-flight
  "async": true,                        // only for async transactions
  "startTime": 1762420000000,           // ms epoch
  "captureTime": 1762420003210,         // ms epoch
  "durationNanos": 3210000000,
  "transactionType": "Web",
  "transactionName": "/api/v3/requests",
  "headline": "GET /api/v3/requests?…",
  "user": "",
  "attributes": { "Request http method": ["GET"], "Request query string": ["…"] },  // omitted when none
  "detail": { … },                      // DetailMap, omitted when none
  "locationStackTraceElements": [ "org.apache…(File.java:123)", … ],              // omitted when none
  "error": Error,                       // omitted when none
  "mainThreadRootTimer": Timer,
  "mainThreadStats": ThreadStats,
  "auxThreadRootTimer": Timer,          // omitted when none
  "auxThreadStats": ThreadStats,        // omitted when none
  "asyncTimers": [ Timer, … ],          // always present (may be empty)
  "entryCount": 412,
  "entryLimitExceeded": true,           // only when true
  "queryCount": 37,
  "queryLimitExceeded": true,           // only when true
  "mainThreadProfileSampleCount": 120,
  "mainThreadProfileSampleLimitExceeded": true,   // only when true
  "auxThreadProfileSampleCount": 0,
  "auxThreadProfileSampleLimitExceeded": true,    // only when true
  "entriesExistence": "yes",            // "yes" | "no" | "expired"
  "queriesExistence": "yes",
  "profileExistence": "no"
}
```

### Timer (recursive)

```jsonc
{ "name": "http request", "extended": true, "totalNanos": 3210000000, "count": 1,
  "active": true, "childTimers": [ Timer, … ] }
```

`extended`, `active`, `childTimers` omitted when false/empty.

### ThreadStats

```jsonc
{ "totalCpuNanos": …, "totalBlockedNanos": …, "totalWaitedNanos": …, "totalAllocatedBytes": … }
```

Older agents send these as nullable wrappers; the JSON is identical.

### Entry (recursive via `childEntries`)

Entries are serialised as a **tree**: the server converts the flat
depth-indexed list into nested `childEntries` arrays.

```jsonc
{
  "startOffsetNanos": 125000,           // from trace start
  "durationNanos": 48000000,
  "active": true,                       // only when still running
  "message": "jdbc execution: …",       // EITHER message …
  "queryMessage": {                     // … OR queryMessage (SQL and other queries)
    "sharedQueryTextIndex": 3,          // index into sharedQueryTexts of the same response
    "prefix": "jdbc query: ",
    "suffix": " => 12 rows"
  },
  "detail": { … },                      // DetailMap, omitted when none
  "locationStackTraceElements": [ "…", … ],   // omitted when none
  "error": Error,                       // omitted when none
  "childEntries": [ Entry, … ]          // omitted when leaf
}
```

### SharedQueryText

Either the full text, or a truncated form plus a hash to fetch the rest:

```jsonc
{ "fullText": "select … from …" }
{ "truncatedText": "select … ", "truncatedEndText": "… where id = ?", "fullTextSha1": "9f2c…" }
```

Resolve a hash with
`GET /backend/transaction/full-query-text?agent-rollup-id=…&full-text-sha1=…`
→ `{"fullText": "…"}` or `{"expired": true}`. Cache by sha1; texts repeat
heavily across traces.

### Query (per-trace aggregate, from `/backend/trace/queries`)

```jsonc
{
  "type": "SQL",
  "sharedQueryTextIndex": 3,
  "totalDurationNanos": 91000000,
  "executionCount": 14,
  "totalRows": 168,
  "active": false,
}
```

`totalRows` omitted when the plugin didn't report rows.

### Error / Throwable

```jsonc
{ "message": "…",
  "exception": { "className": "java.sql.SQLException", "message": "…",
                 "stackTraceElements": [ "…", … ],
                 "framesInCommonWithEnclosing": 12,   // only on nested causes
                 "cause": Throwable } }                // omitted when none
```

### DetailMap

Arbitrary plugin detail. Object whose keys are detail names and whose
values are a scalar (string/number/boolean/null), an array of scalars, or a
nested DetailMap.

### Profile tree (`main-thread-profile`, `aux-thread-profile`)

`MutableProfile.toJson()`. Fields observed in the writer:
`unfilteredSampleCount`, `rootNodes[]`, and per node `stackTraceElement`,
`sampleCount`, `leafThreadState`, `ellipsedSampleCount`, `childNodes[]`.
(`name`/`value`/`children`/`height`/`totalSampleCount` in the same class
belong to the flame-graph output.) Exact nesting **unverified**; dump one
response before writing the parser.

---

## 5. Aggregates over a window (optional, for the overview)

All take `agent-rollup-id`, `transaction-type`, `from`, `to`, and most
accept optional `transaction-name`.

| endpoint                                                  | extra params                                                    | response                                                                                                                 |
| --------------------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `GET /backend/transaction/summaries`                      | `sort-order`, `limit`                                           | `{ "overall": Summary, "transactions": [ Summary+transactionName … ], "moreAvailable": bool }`                           |
| `GET /backend/transaction/average`                        |                                                                 | time series + overall timers (**shape unverified**)                                                                      |
| `GET /backend/transaction/percentiles`                    | `percentile` (repeat)                                           | series per percentile (**unverified**)                                                                                   |
| `GET /backend/transaction/throughput`                     |                                                                 | series (**unverified**)                                                                                                  |
| `GET /backend/transaction/queries`                        |                                                                 | `[ { "queryType", "truncatedQueryText", "fullQueryTextSha1", "totalDurationNanos", "executionCount", "totalRows"? } … ]` |
| `GET /backend/transaction/service-calls`                  |                                                                 | (**unverified**)                                                                                                         |
| `GET /backend/transaction/profile`                        | `auxiliary`, `include`, `exclude`, `truncate-branch-percentage` | profile tree                                                                                                             |
| `GET /backend/transaction/flame-graph`                    | same                                                            | flame graph                                                                                                              |
| `GET /backend/error/summaries`, `/backend/error/messages` |                                                                 | (**unverified**)                                                                                                         |

`sort-order` ∈ `total-time`, `average-time`, `throughput`, `total-cpu-time`,
`average-cpu-time`, `total-allocated-memory`, `average-allocated-memory`.

Summary record: `{ "transactionName", "totalDurationNanos", "totalCpuNanos",
"totalAllocatedBytes", "transactionCount" }` (overall has no name). This is
the cheap way to enumerate transaction names before pulling traces.

---

## 6. Pull sequence for sps

1. `GET /backend/layout` → `central`, `loginEnabled`, `redirectToLogin`.
   If needed, `POST /backend/login`, keep the cookie.
2. Agent list: embedded → `agent-id=` / `agent-rollup-id=` (empty).
   Central → `/backend/top-level-agent-rollups?from&to`, user picks.
3. Transaction types: `embeddedAgentRollup.transactionTypes` or
   `/backend/agent-rollup-layout`. Filter to the ones the user asked for
   (Web, Schedule).
4. For each type: `/backend/transaction/points` and `/backend/error/points`
   with `from`, `to`, `limit`; bisect on `limitExceeded`; stop at the
   user's trace cap. De-duplicate by `(agentId, traceId)` across the two
   families and across sub-windows.
5. For each point, with bounded concurrency (central is Cassandra-backed;
   4–8 in flight is polite): `header` → if `entriesExistence == "yes"`
   `entries`, if `queriesExistence == "yes"` `queries`, profiles only if
   asked. Forward `check-live-traces=true` when the point had the flag.
6. Resolve `fullTextSha1` once per hash via `full-query-text`.
7. Persist: trace header (one row), entries flattened with a parent index
   and depth, per-trace queries, shared query texts keyed by sha1,
   attributes as key/value rows. Keep `startTime`/`captureTime` as instants
   (they already are UTC millis — no zone handling needed, unlike the logs).
