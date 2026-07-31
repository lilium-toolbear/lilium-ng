# CF Clearance Agent Integration

## Status

Approved design for implementation.

Date: 2026-07-31

## Problem

DZMM now returns a Cloudflare Challenge Page for requests that previously reached
the application:

```text
HTTP 403 Forbidden
cf-mitigated: challenge
content-type: text/html
<title>Just a moment...</title>
```

The current Rust client classifies every non-business `403` as a DZMM
authentication failure. It then refreshes account cookies, falls back to account
login, and retries the same challenged edge request. Account authentication
cannot complete a Cloudflare browser challenge, so this path never reaches
`user.getMe`.

The current client also sends hard-coded browser identities:

- HTTP uses Chrome/Brave 148 client hints for macOS.
- Socket.IO uses an Edge/Chrome 143 user agent.
- Account cookies and response cookies live in each `DzmmApi`.

The successful local contract probe established the integration boundary:

1. An externally launched, headed Chrome instance connected through Playwright
   CDP reported `navigator.webdriver == false`.
2. Chrome completed the challenge and obtained `cf_clearance`.
3. A non-browser HTTP client reached `user.getMe` with HTTP 200 when it reused
   the browser cookies, fixed egress IP, and the exact browser user agent.
4. The existing hard-coded Rust browser identity received the challenge.

The service therefore needs a browser only to produce a Cloudflare identity.
Ordinary DZMM HTTP and Socket.IO traffic stays in the existing Rust clients.

## Goals

- Complete Cloudflare challenges without human interaction.
- Publish one atomic Cloudflare identity for the fixed DZMM origin and egress:
  generation, exact user agent, Cloudflare cookies, and expiry.
- Let every microservice reuse that identity without proxying business traffic
  through Chrome.
- Keep ordinary traffic on the existing default HTTP and Socket.IO user agents
  while Cloudflare does not challenge it.
- Detect `cf-mitigated: challenge` before DZMM authentication retry logic.
- Refresh clearance once through a process-wide singleflight operation and
  retry the challenged request once with the new generation.
- Use the same generation for HTTP and Socket.IO handshakes.
- Keep DZMM account cookies owned by each Rust `DzmmApi`.
- Persist refreshed DZMM account cookies through an independent database
  transaction, matching the Python implementation.
- Avoid a database migration.

## Non-goals

- A generic HTTP gateway.
- Passing DZMM account cookies to the browser agent.
- Cloudflare Turnstile Siteverify integration. The upstream owns the site key
  and secret; the observed response is a Challenge Page, not an application
  widget submission owned by Lilium.
- Manual checkbox completion, noVNC, and an operator approval state.
- Browser automation inside the Rust process.
- Multiple browser automation implementations.

## Component Boundary

```text
┌──────────────────────────────────────────────────────────────┐
│ Fixed egress host                                            │
│                                                              │
│  lilium and other microservices                              │
│      │                                                       │
│      │ POST /v1/refresh after cf-mitigated challenge         │
│      ▼                                                       │
│  cf-clearance-agent                                          │
│  Node.js + playwright-core                                   │
│      │ connectOverCDP                                        │
│      ▼                                                       │
│  externally launched headed Chromium under Xvfb              │
│  persistent anonymous profile                                │
│                                                              │
│  Rust HTTP/Socket.IO ─────────────────────────────► DZMM      │
│  Chromium challenge/probe ─────────────────────────► DZMM     │
└──────────────────────────────────────────────────────────────┘
```

The agent lives under `binaries/cf-clearance-agent/` and runs only as a Docker
service. It is not added to the Rust workspace and is not a `lilium` subcommand.

## Browser Runtime

The container uses a Playwright image and `playwright-core` pinned to the same
exact version. Chromium runs as the image's unprivileged `pwuser` with its
sandbox enabled through the matching Playwright seccomp profile.

The Node process owns the following child processes:

1. Xvfb on display `:99`.
2. Chromium started directly with a fixed loopback CDP port.
3. A Playwright CDP connection to Chromium's default persistent context.

Playwright must not call `chromium.launch()` or
`chromium.launchPersistentContext()`. The direct Chromium launch plus
`connectOverCDP()` preserves the browser shape proven by the POC.

Chromium uses `/data/chrome-profile` as its user data directory. A named Docker
volume persists that directory across container restarts. The profile is
anonymous and never performs DZMM account login.

The CDP port listens only on the container loopback interface. The agent API
listens on `0.0.0.0:8787` inside the internal Docker network. Host deployments
publish it only on `127.0.0.1:8787`.

Session recycling waits for Chromium to exit after `SIGTERM`, escalates to
`SIGKILL` after a bounded grace period, and refuses to launch a replacement
while the old process can still hold the CDP port or profile lock.

## Challenge Solver

The solver navigates to the fixed `user.getMe` probe URL without account
cookies.

Success requires all of the following:

1. The browser has a non-expired `cf_clearance` cookie for the DZMM origin.
2. The browser receives HTTP 200 JSON from the probe URL without
   `cf-mitigated`.
3. Node `fetch` receives HTTP 200 JSON from the same probe URL using the exact
   browser user agent and the filtered Cloudflare cookie set.

The published cookie set contains cookies whose names match the Cloudflare
namespace:

- `cf_clearance`
- names beginning with `__cf`
- names beginning with `_cf`

This filter prevents application and account cookies from entering the shared
identity.

The unattended solver follows one bounded flow:

1. Wait for Cloudflare's JavaScript challenge to complete.
2. Detect a visible Cloudflare iframe by its challenge host or title.
3. Perform at most three paced coordinate clicks inside the visible widget.
4. Enforce a 90-second wall-clock deadline over the entire solve, including
   Playwright calls, and recycle the browser session on timeout.

There is no human fallback. A failed solve leaves the service unready, returns a
typed `503` response, emits a structured error, and enters bounded exponential
backoff.

## Agent State

The agent owns one in-memory state machine:

```text
starting -> refreshing -> ready
                    └──> degraded -> refreshing
```

The ready state contains:

```json
{
  "generation": 12,
  "user_agent": "Mozilla/5.0 ...",
  "cookies": [
    {
      "name": "cf_clearance",
      "value": "...",
      "domain": ".dzmm.ai",
      "path": "/",
      "expires": 1780000000
    }
  ],
  "expires_at": "2026-08-01T04:00:00.000Z",
  "verified_at": "2026-07-31T04:00:00.000Z"
}
```

Cookie values are never logged.

Generation starts at one and increments only after all success checks pass.
Failed refreshes never replace the last verified snapshot. A snapshot is not
served after its expiry.

The supervisor treats a refresh as a clearance renewal only when
`cf_clearance` expiry advances. A changed auxiliary Cloudflare cookie is still
published as a new atomic generation, but an unchanged clearance expiry enters
capped exponential renewal backoff instead of scheduling another one-second
refresh.

All refresh callers share one Promise. This is the only singleflight owner;
Rust processes do not coordinate refreshes among themselves.

The supervisor is lazy. Starting the sidecar does not launch a solve or contact
DZMM. The first `POST /v1/refresh`, triggered by an observed challenge, starts
the browser flow. After the first verified identity exists, the supervisor owns
proactive renewal and retry backoff.

## Agent HTTP Contract

### `GET /healthz`

Returns `200` while the Node process and HTTP server are alive. This endpoint
does not assert that clearance is ready.

### `GET /readyz`

Returns `200` with state and generation while a non-expired verified snapshot
exists. Returns `503` in `starting`, `refreshing`, and `degraded`.

### `GET /v1/snapshot`

Returns `200` with the current non-expired snapshot.

Returns:

```json
{
  "error": {
    "code": "CLEARANCE_UNAVAILABLE",
    "message": "No verified Cloudflare identity is available",
    "retryable": true
  }
}
```

with status `503` when no usable snapshot exists.

### `POST /v1/refresh`

Request:

```json
{
  "observed_generation": 12,
  "reason": "cf-mitigated"
}
```

When the current generation is already greater than
`observed_generation`, the endpoint immediately returns the current snapshot.
Otherwise it joins or starts the singleflight refresh and waits for completion.

It returns `200` with the new snapshot after success. It returns the typed
`503` error after a failed solve.

The API has a small request-body limit and rejects unknown routes and methods.
Network isolation is the access-control boundary. No CDP endpoint is reachable
through this API.

## Rust API Client Integration

`lilium-api-client` gains a `clearance` module containing:

- `ClearanceCookie`
- `ClearanceSnapshot`
- `ClearanceAgentClient`
- a `ClearanceProvider` dependency boundary for deterministic tests

`ApiClientConfig` gains `clearance_agent_url`. The production default is
`http://127.0.0.1:8787`; deployments with both services in one Docker network
set it to `http://cf-clearance-agent:8787`.

Each `DzmmApi` owns a two-state edge identity:

```text
Default -> Clearance { generation, user_agent, cookies, expires_at }
```

The initial `Default` state never calls the agent. HTTP uses the Python-parity
Chrome 148 user agent and client hints; Socket.IO uses the Python-parity
Edge/Chrome 143 user agent. Both paths carry account cookies only.

Only a response classified as `cf-mitigated: challenge` calls
`POST /v1/refresh`. The first call uses observed generation zero. A successful
response atomically activates the returned generation, exact browser user
agent, and Cloudflare cookies, then rebuilds and retries the challenged request
once. Later HTTP requests and the Socket.IO handshake use the cached active
identity without querying the agent.

Cloudflare cookies overwrite same-named stale values from the account cookie
store for the outgoing request only. They are not inserted into the account
cookie jar and are not persisted to the account row.

Default client hints are sent only in `Default` state. They are omitted in
`Clearance` state because the successful non-browser POC did not send synthetic
client hints, and Rust must not manufacture values that contradict the browser
identity.

An expired cached identity is discarded before the next request. That request
uses the default identity without Cloudflare cookies. If Cloudflare still
requires mitigation, the resulting challenge reacquires clearance through the
same single retry path. Once a challenge has been observed, agent failure is a
semantic clearance error; Rust does not retry that challenged request with the
default identity.

All DZMM endpoints, including token refresh, password login, QR login, avatar
updates, and multipart uploads, use the same clearance-aware request path.
Retryable request bodies are represented as owned data so a request can be
rebuilt exactly once after a clearance refresh.

Third-party media downloads remain on the existing plain request path and never
receive DZMM cookies or clearance headers.

## Error and Retry Ordering

The response classifier runs in this order:

1. `403` plus `cf-mitigated: challenge`
2. successful JSON response
3. `429` rate limiting
4. tRPC business forbidden
5. DZMM `401` and non-challenge `403` authentication failure
6. generic HTTP error

For a Cloudflare challenge:

1. Capture the active generation, or zero when the request used the default
   identity.
2. Call `POST /v1/refresh` with that generation.
3. Cache the returned atomic identity and rebuild the request once.
4. Return a semantic, retryable clearance error after a second challenge.

Cloudflare challenge retries do not consume the existing one-time DZMM
authentication retry. Account refresh begins only after the clearance path has
completed.

## Socket.IO Integration

After `DzmmApi::authenticate()` succeeds, the worker asks the same `DzmmApi` for
connection credentials:

```text
generation + selected user agent + selected outgoing cookie header
```

`WsClient` receives these typed credentials instead of a standalone cookie
string. Generation zero means the default Socket.IO user agent and account
cookies. A positive generation means the exact browser user agent and merged
account plus Cloudflare cookies.

A disconnected socket returns to the worker reconnect loop. The next
connection authenticates over HTTP first and then uses the edge identity
selected by that `DzmmApi`. No background mutation of a live Socket.IO client's
headers is attempted.

## Account Cookie Persistence

Python `AccountService.create_auth_client` always installs a cookie refresh
callback that opens an independent database session and updates the account
row. Rust's shared `account::create_auth_client` currently sets the callback to
`None`; only the WebSocket worker has a local callback.

Rust gains `AuthClientFactory`, owned by the application layer and constructed
from a clone of `lilium_database::Database`. The factory:

1. Builds `DzmmApiAuth` from the account model.
2. Installs one callback.
3. Opens an independent `lilium_database::transaction!` operation.
4. Calls `account::update_cookies`.
5. Logs account ID and success or failure without logging cookie contents.

Every call site receives this factory explicitly. The WebSocket worker deletes
its duplicate callback and uses the factory. Service methods that select an
account receive `&AuthClientFactory` alongside their existing
`ConnectionTrait` input.

This is a required ownership correction, not a fallback path. The callback
remains best-effort because the existing callback type cannot surface a
database error back through a completed authentication request.

## Configuration

### Agent

| Variable | Value |
| --- | --- |
| `CF_CLEARANCE_ORIGIN` | `https://www.dzmm.ai` |
| `CF_CLEARANCE_LISTEN_HOST` | `0.0.0.0` |
| `CF_CLEARANCE_LISTEN_PORT` | `8787` |
| `CF_CLEARANCE_CDP_PORT` | `9223` |
| `CF_CLEARANCE_PROFILE_DIR` | `/data/chrome-profile` |
| `CF_CLEARANCE_SOLVE_TIMEOUT_MS` | `90000` |
| `DISPLAY` | `:99` |

### Rust consumers

| Variable | Value |
| --- | --- |
| `CF_CLEARANCE_AGENT_URL` | `http://127.0.0.1:8787` on the host |

The fixed egress requirement applies to the agent and every consuming
microservice. DNS and routing must place them behind the same public source IP.

## Observability

Agent structured logs record:

- state transition
- generation
- solve duration
- click count
- browser and cross-client probe status
- Cloudflare Ray ID
- expiry
- error code

Rust tracing records:

- agent operation
- snapshot generation
- challenge endpoint
- refresh outcome
- retry outcome

Spans skip cookie values, browser profiles, request bodies, credentials, and
full challenge HTML.

## Test Strategy

### Node unit and contract tests

Using `node:test`:

- snapshot is unavailable before the first verified solve
- expired snapshots are unavailable
- refresh callers share one solve
- an already advanced generation bypasses solve
- failed solve preserves but does not serve an expired snapshot
- cookie filtering publishes only Cloudflare names
- cookie values never enter serialized logs
- route, method, and body limits are enforced

The HTTP server and solver use explicit dependency boundaries. Tests inject a
fake solver through the normal constructor; production code has no test-only
branch.

### Rust tests

- snapshot response parsing and expiry validation
- default HTTP requests do not call the clearance provider
- default HTTP and Socket.IO credentials retain their Python-parity user agents
- account and Cloudflare cookie merge precedence
- first `cf-mitigated` response refreshes and retries with the new generation
- the activated generation is reused without another provider call
- second `cf-mitigated` response returns a semantic error
- business forbidden never triggers clearance refresh
- authentication refresh starts only after clearance handling
- Socket.IO credentials contain one generation's user agent and cookie header
- `AuthClientFactory` installs a callback that persists through an independent
  database operation

Tests use an explicit fake `ClearanceProvider` and existing database fixtures.
No environment mutation and no production test backdoor are introduced.

### Container verification

The local container verification uses a fresh named profile volume and the real
fixed-egress DZMM probe. Completion requires:

1. No manual input.
2. `navigator.webdriver == false`.
3. An unchallenged Rust request completes with generation zero while the agent
   remains unready.
4. A challenged request activates agent readiness and publishes a non-expired
   snapshot.
5. The retried Rust request receives HTTP 200 JSON with that snapshot.

The test runs once per relevant image change. Repeating it without a code,
profile, image, or challenge-state change is not useful evidence.

## Rollout

1. Build and start `cf-clearance-agent` with its persistent volume.
2. Verify `/healthz`; `/readyz` remains unavailable until the first challenge.
3. Point one Rust consumer at the agent.
4. Verify `user.getMe`, one authenticated API request, and a Socket.IO
   connection.
5. Configure the remaining microservices to use the same internal endpoint.
6. Alert on agent unreadiness, clearance refresh failure, and repeated
   `cf-mitigated` responses.

Rollback stops Rust consumers before stopping the agent, restores the previous
application image, and leaves the browser profile volume intact for diagnosis.
The default Python-parity identities are normal operating state, not a
compatibility fallback. The browser identity becomes mandatory only after a
challenge is observed.
