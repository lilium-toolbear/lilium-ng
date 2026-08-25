import assert from "node:assert/strict";
import test from "node:test";

import { loadConfig } from "../src/config.mjs";

test("configuration maps the documented container environment", () => {
  const config = loadConfig({
    CF_CLEARANCE_ORIGIN: "https://www.dzmm.ai/",
    CF_CLEARANCE_LISTEN_HOST: "0.0.0.0",
    CF_CLEARANCE_LISTEN_PORT: "8787",
    CF_CLEARANCE_CDP_PORT: "9223",
    CF_CLEARANCE_PROFILE_DIR: "/data/chrome-profile",
    CF_CLEARANCE_SOLVE_TIMEOUT_MS: "90000",
    DISPLAY: ":99",
  });

  assert.deepEqual(config, {
    origin: "https://www.dzmm.ai",
    listenHost: "0.0.0.0",
    listenPort: 8787,
    cdpPort: 9223,
    profileDir: "/data/chrome-profile",
    solveTimeoutMs: 90_000,
    display: ":99",
    browserStartupTimeoutMs: 30_000,
    bodyLimitBytes: 4096,
  });
});

test("configuration rejects unsafe origin and invalid numeric values", () => {
  assert.throws(
    () => loadConfig({ CF_CLEARANCE_ORIGIN: "http://www.dzmm.ai" }),
    /HTTPS/,
  );
  assert.throws(
    () => loadConfig({ CF_CLEARANCE_LISTEN_PORT: "not-a-port" }),
    /CF_CLEARANCE_LISTEN_PORT/,
  );
  assert.throws(
    () => loadConfig({ CF_CLEARANCE_PROFILE_DIR: "relative/profile" }),
    /CF_CLEARANCE_PROFILE_DIR/,
  );
});
