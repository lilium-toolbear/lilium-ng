import assert from "node:assert/strict";
import test from "node:test";

import { createLogger } from "../src/logger.mjs";

test("structured logs never serialize cookie values or profile paths", () => {
  const lines = [];
  const logger = createLogger({
    sink: (line) => lines.push(line),
    clock: () => Date.parse("2026-07-31T04:00:00.000Z"),
  });

  logger.info("probe_complete", {
    generation: 3,
    cookie_header: "cf_clearance=top-secret",
    cookies: [{ name: "cf_clearance", value: "top-secret" }],
    profile_dir: "/data/chrome-profile",
    cf_ray: "test-ray",
  });

  assert.equal(lines.length, 1);
  assert.doesNotMatch(lines[0], /top-secret|chrome-profile/);
  assert.match(lines[0], /probe_complete/);
  assert.match(lines[0], /test-ray/);
});
