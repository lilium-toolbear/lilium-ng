import assert from "node:assert/strict";
import test from "node:test";

import {
  ClearanceAgentState,
  ClearanceUnavailableError,
} from "../src/state.mjs";

const NOW_MS = Date.parse("2026-07-31T04:00:00.000Z");

function solvedIdentity(overrides = {}) {
  return {
    user_agent: "Mozilla/5.0 Test Browser",
    cookies: [
      {
        name: "cf_clearance",
        value: "secret",
        domain: ".dzmm.ai",
        path: "/",
        expires: NOW_MS / 1000 + 3600,
      },
    ],
    expires_at: new Date(NOW_MS + 3_600_000).toISOString(),
    verified_at: new Date(NOW_MS).toISOString(),
    ...overrides,
  };
}

test("snapshot is unavailable before the first verified solve", () => {
  const state = new ClearanceAgentState({
    solve: async () => solvedIdentity(),
    clock: () => NOW_MS,
  });

  assert.equal(state.status, "starting");
  assert.equal(state.getSnapshot(), null);
});

test("expired snapshots are not served", async () => {
  let now = NOW_MS;
  const state = new ClearanceAgentState({
    solve: async () => solvedIdentity(),
    clock: () => now,
  });

  await state.refresh({ observed_generation: 0, reason: "startup" });
  now += 3_600_001;

  assert.equal(state.getSnapshot(), null);
  assert.equal(state.isReady(), false);
});

test("concurrent refresh callers share one solve", async () => {
  let solveCalls = 0;
  let finishSolve;
  const solveGate = new Promise((resolve) => {
    finishSolve = resolve;
  });
  const state = new ClearanceAgentState({
    solve: async () => {
      solveCalls += 1;
      await solveGate;
      return solvedIdentity();
    },
    clock: () => NOW_MS,
  });

  const first = state.refresh({
    observed_generation: 0,
    reason: "cf-mitigated",
  });
  const second = state.refresh({
    observed_generation: 0,
    reason: "cf-mitigated",
  });
  finishSolve();

  const [firstSnapshot, secondSnapshot] = await Promise.all([first, second]);
  assert.equal(solveCalls, 1);
  assert.deepEqual(firstSnapshot, secondSnapshot);
  assert.equal(firstSnapshot.generation, 1);
});

test("an already advanced generation bypasses solve", async () => {
  let solveCalls = 0;
  const state = new ClearanceAgentState({
    solve: async () => {
      solveCalls += 1;
      return solvedIdentity();
    },
    clock: () => NOW_MS,
  });

  const initial = await state.refresh({
    observed_generation: 0,
    reason: "startup",
  });
  const current = await state.refresh({
    observed_generation: 0,
    reason: "cf-mitigated",
  });

  assert.equal(solveCalls, 1);
  assert.deepEqual(current, initial);
});

test("a failed solve keeps the previous snapshot but never serves it after expiry", async () => {
  let now = NOW_MS;
  let shouldFail = false;
  const state = new ClearanceAgentState({
    solve: async () => {
      if (shouldFail) {
        throw new Error("challenge failed");
      }
      return solvedIdentity();
    },
    clock: () => now,
  });

  const first = await state.refresh({
    observed_generation: 0,
    reason: "startup",
  });
  now += 3_600_001;
  shouldFail = true;

  await assert.rejects(
    state.refresh({
      observed_generation: first.generation,
      reason: "cf-mitigated",
    }),
    ClearanceUnavailableError,
  );
  assert.equal(state.status, "degraded");
  assert.equal(state.generation, 1);
  assert.equal(state.getSnapshot(), null);
});

test("an unchanged solved identity does not advance the generation", async () => {
  const state = new ClearanceAgentState({
    solve: async () => solvedIdentity(),
    clock: () => NOW_MS,
  });
  const first = await state.refresh({
    observed_generation: 0,
    reason: "startup",
  });

  await assert.rejects(
    state.refresh({
      observed_generation: first.generation,
      reason: "scheduled",
    }),
    ClearanceUnavailableError,
  );

  assert.equal(state.generation, first.generation);
  assert.deepEqual(state.getSnapshot(), first);
  assert.equal(state.status, "degraded");
});

test("an auxiliary Cloudflare cookie change publishes one new atomic identity", async () => {
  let solveCount = 0;
  const state = new ClearanceAgentState({
    solve: async () => {
      solveCount += 1;
      return solvedIdentity({
        cookies: [
          solvedIdentity().cookies[0],
          {
            name: "__cf_bm",
            value: `auxiliary-${solveCount}`,
            domain: ".dzmm.ai",
            path: "/",
            expires: NOW_MS / 1000 + 1800,
          },
        ],
      });
    },
    clock: () => NOW_MS,
  });
  const first = await state.refresh({
    observed_generation: 0,
    reason: "startup",
  });

  const second = await state.refresh({
    observed_generation: first.generation,
    reason: "scheduled",
  });

  assert.equal(second.generation, first.generation + 1);
  assert.equal(second.cookies[1].value, "auxiliary-2");
});
