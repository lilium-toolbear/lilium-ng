# Cloudflare clearance agent

This sidecar owns the anonymous Chromium profile used to solve the upstream
Cloudflare challenge. It publishes only the verified browser user agent and
Cloudflare cookie namespace; DZMM account cookies remain in the Rust clients.

The container runs Chromium as `pwuser` with its sandbox enabled. The checked-in
`seccomp_profile.json` is the Playwright `v1.61.1` profile and is wired through
the root Compose file. The Playwright package, container image, and seccomp
profile must stay on the same release.

## Run

```sh
docker compose -f compose.cf-clearance-agent.yaml up -d --build
curl --fail http://127.0.0.1:8787/healthz
```

The named `cf-clearance-profile` volume preserves the anonymous browser profile
across restarts. The agent and all clients must use the same fixed egress IP.
The supervisor is lazy: startup does not contact DZMM or solve a challenge.
`/readyz` therefore returns `503` until a consumer observes
`cf-mitigated: challenge` and calls `POST /v1/refresh`.

For a host process, the Rust client default is already:

```sh
CF_CLEARANCE_AGENT_URL=http://127.0.0.1:8787
```

For a Rust service in the same Compose network, set:

```sh
CF_CLEARANCE_AGENT_URL=http://cf-clearance-agent:8787
```

The API is intentionally small:

- `GET /healthz` reports process liveness.
- `GET /readyz` reports whether a current verified identity exists.
- `GET /v1/snapshot` returns the current generation, exact user agent, and
  Cloudflare cookies.
- `POST /v1/refresh` singleflights an unattended challenge solve.

There is no manual verification fallback. A failed solve returns a retryable
unavailable error and the supervisor retries with capped exponential backoff.
Consumers use their normal Python-parity UA and account cookies while no
challenge is observed. They activate the returned browser UA and Cloudflare
cookies only after a successful refresh.
