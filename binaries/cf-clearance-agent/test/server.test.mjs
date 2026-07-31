import assert from "node:assert/strict";
import test from "node:test";

import { createAgentServer } from "../src/server.mjs";

const SNAPSHOT = {
  generation: 7,
  user_agent: "Mozilla/5.0 Test Browser",
  cookies: [
    {
      name: "cf_clearance",
      value: "secret",
      domain: ".dzmm.ai",
      path: "/",
      expires: 1_780_000_000,
    },
  ],
  expires_at: "2026-08-01T04:00:00.000Z",
  verified_at: "2026-07-31T04:00:00.000Z",
};

test("health is live while readiness and snapshot report unavailable state", async (t) => {
  const harness = await startServer(
    fakeState({ status: "starting", snapshot: null }),
  );
  t.after(harness.close);

  const health = await getJson(`${harness.url}/healthz`);
  assert.equal(health.response.status, 200);
  assert.deepEqual(health.body, { status: "ok" });

  const readiness = await getJson(`${harness.url}/readyz`);
  assert.equal(readiness.response.status, 503);
  assert.deepEqual(readiness.body, {
    state: "starting",
    generation: 0,
  });

  const snapshot = await getJson(`${harness.url}/v1/snapshot`);
  assert.equal(snapshot.response.status, 503);
  assert.deepEqual(snapshot.body, unavailableError());
});

test("ready and snapshot endpoints publish the current verified generation", async (t) => {
  const harness = await startServer(
    fakeState({ status: "ready", snapshot: SNAPSHOT }),
  );
  t.after(harness.close);

  const readiness = await getJson(`${harness.url}/readyz`);
  assert.equal(readiness.response.status, 200);
  assert.deepEqual(readiness.body, { state: "ready", generation: 7 });

  const snapshot = await getJson(`${harness.url}/v1/snapshot`);
  assert.equal(snapshot.response.status, 200);
  assert.deepEqual(snapshot.body, SNAPSHOT);
});

test("refresh validates its request and returns the state refresh result", async (t) => {
  const calls = [];
  const state = fakeState({ status: "ready", snapshot: SNAPSHOT });
  state.refresh = async (request) => {
    calls.push(request);
    return { ...SNAPSHOT, generation: 8 };
  };
  const harness = await startServer(state);
  t.after(harness.close);

  const response = await fetch(`${harness.url}/v1/refresh`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      observed_generation: 7,
      reason: "cf-mitigated",
    }),
  });

  assert.equal(response.status, 200);
  assert.deepEqual(await response.json(), { ...SNAPSHOT, generation: 8 });
  assert.deepEqual(calls, [
    { observed_generation: 7, reason: "cf-mitigated" },
  ]);
});

test("refresh maps solve failure to the typed retryable error", async (t) => {
  const state = fakeState({ status: "degraded", snapshot: null });
  state.refresh = async () => {
    throw new Error("browser failed");
  };
  const harness = await startServer(state);
  t.after(harness.close);

  const response = await fetch(`${harness.url}/v1/refresh`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      observed_generation: 0,
      reason: "startup",
    }),
  });

  assert.equal(response.status, 503);
  assert.deepEqual(await response.json(), unavailableError());
});

test("unknown routes and unsupported methods are rejected", async (t) => {
  const harness = await startServer(fakeState({ snapshot: SNAPSHOT }));
  t.after(harness.close);

  const missing = await getJson(`${harness.url}/missing`);
  assert.equal(missing.response.status, 404);
  assert.equal(missing.body.error.code, "NOT_FOUND");

  const wrongMethod = await fetch(`${harness.url}/v1/refresh`);
  assert.equal(wrongMethod.status, 405);
  assert.equal(wrongMethod.headers.get("allow"), "POST");
  assert.equal((await wrongMethod.json()).error.code, "METHOD_NOT_ALLOWED");
});

test("refresh rejects malformed and oversized request bodies", async (t) => {
  const harness = await startServer(fakeState({ snapshot: SNAPSHOT }), {
    bodyLimitBytes: 64,
  });
  t.after(harness.close);

  const malformed = await fetch(`${harness.url}/v1/refresh`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ reason: "missing generation" }),
  });
  assert.equal(malformed.status, 400);
  assert.equal((await malformed.json()).error.code, "INVALID_REQUEST");

  const oversized = await fetch(`${harness.url}/v1/refresh`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      observed_generation: 0,
      reason: "x".repeat(128),
    }),
  });
  assert.equal(oversized.status, 413);
  assert.equal((await oversized.json()).error.code, "REQUEST_TOO_LARGE");
});

function fakeState({ status = "ready", snapshot }) {
  return {
    status,
    generation: snapshot?.generation ?? 0,
    getSnapshot: () => structuredClone(snapshot),
    refresh: async () => structuredClone(snapshot),
  };
}

function unavailableError() {
  return {
    error: {
      code: "CLEARANCE_UNAVAILABLE",
      message: "No verified Cloudflare identity is available",
      retryable: true,
    },
  };
}

async function startServer(state, options = {}) {
  const server = createAgentServer({ state, ...options });
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(0, "127.0.0.1", resolve);
  });
  const { port } = server.address();
  return {
    url: `http://127.0.0.1:${port}`,
    close: () => new Promise((resolve) => server.close(resolve)),
  };
}

async function getJson(url) {
  const response = await fetch(url);
  return { response, body: await response.json() };
}
