import assert from "node:assert/strict";
import test from "node:test";

import { ClearanceAgentState } from "../src/state.mjs";
import { RefreshSupervisor } from "../src/supervisor.mjs";

const NOW_MS = Date.parse("2026-07-31T04:00:00.000Z");

test("failed solves enter capped exponential backoff", async () => {
  const timers = [];
  const state = new ClearanceAgentState({
    solve: async () => {
      throw new Error("challenge failed");
    },
    clock: () => NOW_MS,
  });
  const supervisor = new RefreshSupervisor({
    state,
    clock: () => NOW_MS,
    setTimer: (callback, delay) => {
      const timer = { callback, delay, cancelled: false };
      timers.push(timer);
      return timer;
    },
    clearTimer: (timer) => {
      timer.cancelled = true;
    },
    baseBackoffMs: 1000,
    maxBackoffMs: 2000,
  });

  supervisor.start();
  await runLastLiveTimer(timers);
  assert.equal(lastLiveTimer(timers).delay, 1000);
  await runLastLiveTimer(timers);
  assert.equal(lastLiveTimer(timers).delay, 2000);
  await runLastLiveTimer(timers);
  assert.equal(lastLiveTimer(timers).delay, 2000);

  supervisor.stop();
});

test("successful solves schedule refresh before clearance expiry", async () => {
  const timers = [];
  const state = new ClearanceAgentState({
    solve: async () => ({
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
    }),
    clock: () => NOW_MS,
  });
  const supervisor = new RefreshSupervisor({
    state,
    clock: () => NOW_MS,
    setTimer: (callback, delay) => {
      const timer = { callback, delay, cancelled: false };
      timers.push(timer);
      return timer;
    },
    clearTimer: (timer) => {
      timer.cancelled = true;
    },
    refreshLeadMs: 300_000,
  });

  supervisor.start();
  await runLastLiveTimer(timers);

  assert.equal(state.status, "ready");
  assert.equal(lastLiveTimer(timers).delay, 3_300_000);
  supervisor.stop();
});

test("unchanged near-expiry identity enters backoff without advancing generation", async () => {
  const timers = [];
  const identity = {
    user_agent: "Mozilla/5.0 Test Browser",
    cookies: [
      {
        name: "cf_clearance",
        value: "unchanged",
        domain: ".dzmm.ai",
        path: "/",
        expires: NOW_MS / 1000 + 60,
      },
    ],
    expires_at: new Date(NOW_MS + 60_000).toISOString(),
    verified_at: new Date(NOW_MS).toISOString(),
  };
  const state = new ClearanceAgentState({
    solve: async () => structuredClone(identity),
    clock: () => NOW_MS,
  });
  const supervisor = new RefreshSupervisor({
    state,
    clock: () => NOW_MS,
    setTimer: (callback, delay) => {
      const timer = { callback, delay, cancelled: false };
      timers.push(timer);
      return timer;
    },
    clearTimer: (timer) => {
      timer.cancelled = true;
    },
    baseBackoffMs: 5_000,
    maxBackoffMs: 60_000,
    refreshLeadMs: 300_000,
  });

  supervisor.start();
  await runLastLiveTimer(timers);
  assert.equal(state.generation, 1);
  assert.equal(lastLiveTimer(timers).delay, 1_000);

  await runLastLiveTimer(timers);
  assert.equal(state.generation, 1);
  assert.equal(state.status, "degraded");
  assert.equal(lastLiveTimer(timers).delay, 5_000);
  supervisor.stop();
});

test("auxiliary cookie rotation with unchanged clearance expiry uses renewal backoff", async () => {
  const timers = [];
  let solveCount = 0;
  const state = new ClearanceAgentState({
    solve: async () => {
      solveCount += 1;
      return {
        user_agent: "Mozilla/5.0 Test Browser",
        cookies: [
          {
            name: "cf_clearance",
            value: "unchanged-clearance",
            domain: ".dzmm.ai",
            path: "/",
            expires: NOW_MS / 1000 + 60,
          },
          {
            name: "__cf_bm",
            value: `auxiliary-${solveCount}`,
            domain: ".dzmm.ai",
            path: "/",
            expires: NOW_MS / 1000 + 30,
          },
        ],
        expires_at: new Date(NOW_MS + 60_000).toISOString(),
        verified_at: new Date(NOW_MS).toISOString(),
      };
    },
    clock: () => NOW_MS,
  });
  const supervisor = new RefreshSupervisor({
    state,
    clock: () => NOW_MS,
    setTimer: (callback, delay) => {
      const timer = { callback, delay, cancelled: false };
      timers.push(timer);
      return timer;
    },
    clearTimer: (timer) => {
      timer.cancelled = true;
    },
    baseBackoffMs: 5_000,
    maxBackoffMs: 60_000,
    refreshLeadMs: 300_000,
  });

  supervisor.start();
  await runLastLiveTimer(timers);
  assert.equal(state.generation, 1);
  assert.equal(lastLiveTimer(timers).delay, 1_000);

  await runLastLiveTimer(timers);
  assert.equal(state.generation, 2);
  assert.equal(lastLiveTimer(timers).delay, 5_000);

  await runLastLiveTimer(timers);
  assert.equal(state.generation, 3);
  assert.equal(lastLiveTimer(timers).delay, 10_000);
  supervisor.stop();
});

async function runLastLiveTimer(timers) {
  const timer = lastLiveTimer(timers);
  timer.cancelled = true;
  await timer.callback();
}

function lastLiveTimer(timers) {
  return timers.findLast(({ cancelled }) => !cancelled);
}
