import assert from "node:assert/strict";
import test from "node:test";

import { ChallengeSolveError, createChallengeSolver } from "../src/solver.mjs";

const NOW_MS = Date.parse("2026-07-31T04:00:00.000Z");
const ORIGIN = "https://www.dzmm.ai";

test("solver publishes only after browser and exact-identity cross-client probes pass", async () => {
  const requests = [];
  const session = successfulSession();
  const solve = createChallengeSolver({
    runtime: { getSession: async () => session },
    origin: ORIGIN,
    timeoutMs: 90_000,
    clock: () => NOW_MS,
    sleep: async () => {},
    fetchImpl: async (url, init) => {
      requests.push({ url, init });
      return jsonResponse(200);
    },
  });

  const identity = await solve();

  assert.equal(
    session.navigations[0],
    `${ORIGIN}/api/trpc/user.getMe?batch=1&input=%7B%220%22%3A%7B%22json%22%3Anull%7D%7D`,
  );
  assert.equal(identity.user_agent, "Mozilla/5.0 Exact Browser");
  assert.deepEqual(
    identity.cookies.map(({ name }) => name),
    ["cf_clearance", "__cf_bm"],
  );
  assert.equal(requests.length, 1);
  assert.equal(requests[0].init.headers["user-agent"], identity.user_agent);
  assert.equal(
    requests[0].init.headers.accept,
    "application/json, text/plain, */*",
  );
  assert.equal(requests[0].init.headers["accept-language"], "en-US,en;q=0.9");
  assert.equal(
    requests[0].init.headers.cookie,
    "cf_clearance=clearance; __cf_bm=bot-management",
  );
  assert.doesNotMatch(requests[0].init.headers.cookie, /account-session/);
});

test("cross-client challenge never publishes a browser-only identity", async () => {
  let now = NOW_MS;
  const solve = createChallengeSolver({
    runtime: { getSession: async () => successfulSession() },
    origin: ORIGIN,
    timeoutMs: 2_000,
    clock: () => now,
    sleep: async (milliseconds) => {
      now += milliseconds;
    },
    fetchImpl: async () =>
      new Response("<html>challenge</html>", {
        status: 403,
        headers: {
          "cf-mitigated": "challenge",
          "content-type": "text/html",
        },
      }),
  });

  await assert.rejects(solve(), (error) => {
    assert.ok(error instanceof ChallengeSolveError);
    assert.equal(error.code, "SOLVE_TIMEOUT");
    return true;
  });
});

test("visible challenge widgets receive at most three paced coordinate clicks", async () => {
  let now = NOW_MS;
  const clicked = [];
  const session = {
    navigations: [],
    navigate: async (url) => session.navigations.push(url),
    browserProbe: async () => ({
      status: 403,
      content_type: "text/html",
      challenged: true,
      is_json: false,
      cf_ray: "test-ray",
    }),
    getCookies: async () => [],
    getUserAgent: async () => "Mozilla/5.0 Exact Browser",
    getChallengeWidget: async () => ({
      x: 10,
      y: 20,
      width: 300,
      height: 65,
    }),
    clickChallengeWidget: async (widget) => clicked.push({ ...widget, at: now }),
  };
  const solve = createChallengeSolver({
    runtime: { getSession: async () => session },
    origin: ORIGIN,
    timeoutMs: 25_000,
    clock: () => now,
    sleep: async (milliseconds) => {
      now += milliseconds;
    },
    fetchImpl: async () => jsonResponse(500),
  });

  await assert.rejects(solve(), ChallengeSolveError);
  assert.equal(clicked.length, 3);
  assert.ok(clicked[1].at - clicked[0].at >= 8_000);
  assert.ok(clicked[2].at - clicked[1].at >= 8_000);
});

test("unchanged 250ms browser probes do not flood structured logs", async () => {
  let now = NOW_MS;
  const browserProbeLogs = [];
  const session = {
    ...successfulSession(),
    browserProbe: async () => ({
      status: 403,
      content_type: "text/html",
      challenged: true,
      is_json: false,
      cf_ray: "same-ray",
    }),
    getCookies: async () => [],
  };
  const solve = createChallengeSolver({
    runtime: { getSession: async () => session },
    origin: ORIGIN,
    timeoutMs: 2_000,
    clock: () => now,
    sleep: async (milliseconds) => {
      now += milliseconds;
    },
    fetchImpl: async () => jsonResponse(500),
    logger: {
      info: (event) => {
        if (event === "browser_probe") {
          browserProbeLogs.push(event);
        }
      },
      warn: () => {},
      error: () => {},
    },
  });

  await assert.rejects(solve(), ChallengeSolveError);
  assert.equal(browserProbeLogs.length, 1);
});

test("a hung browser operation is cut off by the hard solve deadline and recycles the session", async () => {
  let resetCalls = 0;
  const solve = createChallengeSolver({
    runtime: {
      getSession: async () => ({
        navigate: async () => {},
        browserProbe: async () => new Promise(() => {}),
        getCookies: async () => [],
      }),
      resetSession: async () => {
        resetCalls += 1;
      },
    },
    origin: ORIGIN,
    timeoutMs: 20,
    fetchImpl: async () => jsonResponse(500),
  });

  const outcome = await Promise.race([
    solve().then(
      () => ({ kind: "resolved" }),
      (error) => ({ kind: "rejected", error }),
    ),
    new Promise((resolve) =>
      setTimeout(() => resolve({ kind: "hung" }), 200),
    ),
  ]);

  assert.notEqual(outcome.kind, "hung");
  assert.equal(outcome.kind, "rejected");
  assert.equal(outcome.error.code, "SOLVE_TIMEOUT");
  assert.equal(resetCalls, 1);
});

test("transient challenge iframe replacement does not abort the solve", async () => {
  let now = NOW_MS;
  let widgetCalls = 0;
  const session = {
    navigate: async () => {},
    browserProbe: async () => ({
      status: 403,
      content_type: "text/html",
      challenged: true,
      is_json: false,
      cf_ray: "test-ray",
    }),
    getCookies: async () => [],
    getChallengeWidget: async () => {
      widgetCalls += 1;
      if (widgetCalls === 1) {
        throw new Error("iframe was replaced");
      }
      return null;
    },
    clickChallengeWidget: async () => {},
  };
  const solve = createChallengeSolver({
    runtime: { getSession: async () => session },
    origin: ORIGIN,
    timeoutMs: 1_000,
    clock: () => now,
    sleep: async (milliseconds) => {
      now += milliseconds;
    },
    fetchImpl: async () => jsonResponse(500),
  });

  await assert.rejects(solve(), (error) => {
    assert.equal(error.code, "SOLVE_TIMEOUT");
    return true;
  });
  assert.ok(widgetCalls > 1);
});

test("repeated probe exceptions are rate limited in structured logs", async () => {
  let now = NOW_MS;
  const probeFailures = [];
  const solve = createChallengeSolver({
    runtime: {
      getSession: async () => ({
        navigate: async () => {},
        browserProbe: async () => {
          throw new Error("execution context replaced");
        },
        getCookies: async () => [],
        getChallengeWidget: async () => null,
      }),
    },
    origin: ORIGIN,
    timeoutMs: 2_000,
    clock: () => now,
    sleep: async (milliseconds) => {
      now += milliseconds;
    },
    fetchImpl: async () => jsonResponse(500),
    logger: {
      info: () => {},
      warn: (event) => {
        if (event === "solve_probe_failed") {
          probeFailures.push(event);
        }
      },
      error: () => {},
    },
  });

  await assert.rejects(solve(), ChallengeSolveError);
  assert.equal(probeFailures.length, 1);
});

function successfulSession() {
  const session = {
    navigations: [],
    navigate: async (url) => session.navigations.push(url),
    browserProbe: async () => ({
      status: 200,
      content_type: "application/json",
      challenged: false,
      is_json: true,
      cf_ray: "test-ray",
    }),
    getCookies: async () => [
      cookie("cf_clearance", "clearance"),
      cookie("__cf_bm", "bot-management"),
      cookie("session", "account-session"),
    ],
    getUserAgent: async () => "Mozilla/5.0 Exact Browser",
    getChallengeWidget: async () => null,
    clickChallengeWidget: async () => {
      throw new Error("unexpected click");
    },
  };
  return session;
}

function cookie(name, value) {
  return {
    name,
    value,
    domain: ".dzmm.ai",
    path: "/",
    expires: NOW_MS / 1000 + 3600,
  };
}

function jsonResponse(status) {
  return new Response(JSON.stringify([{ result: { data: null } }]), {
    status,
    headers: { "content-type": "application/json" },
  });
}
