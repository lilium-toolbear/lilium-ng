import assert from "node:assert/strict";
import test from "node:test";

import {
  cloudflareCookieHeader,
  createVerifiedIdentity,
  filterCloudflareCookies,
} from "../src/identity.mjs";

const NOW_MS = Date.parse("2026-07-31T04:00:00.000Z");

test("cookie filtering publishes only Cloudflare namespaces", () => {
  const cookies = filterCloudflareCookies([
    cookie("cf_clearance", "clearance"),
    cookie("__cf_bm", "bot-management"),
    cookie("_cfuvid", "visitor"),
    cookie("session", "account-session"),
    cookie("authjs.session-token", "account-auth"),
  ]);

  assert.deepEqual(
    cookies.map(({ name }) => name),
    ["cf_clearance", "__cf_bm", "_cfuvid"],
  );
  assert.equal(
    cookies.some(({ value }) => value === "account-session"),
    false,
  );
});

test("verified identity expires with cf_clearance rather than an auxiliary cookie", () => {
  const identity = createVerifiedIdentity({
    userAgent: "Mozilla/5.0 Test Browser",
    cookies: [
      cookie("cf_clearance", "clearance", NOW_MS / 1000 + 3600),
      cookie("__cf_bm", "bot-management", NOW_MS / 1000 + 60),
    ],
    nowMs: NOW_MS,
  });

  assert.equal(identity.expires_at, new Date(NOW_MS + 3_600_000).toISOString());
  assert.equal(identity.verified_at, new Date(NOW_MS).toISOString());
});

test("identity creation rejects missing or expired clearance", () => {
  assert.throws(
    () =>
      createVerifiedIdentity({
        userAgent: "Mozilla/5.0 Test Browser",
        cookies: [cookie("__cf_bm", "bot-management")],
        nowMs: NOW_MS,
      }),
    /cf_clearance/,
  );
  assert.throws(
    () =>
      createVerifiedIdentity({
        userAgent: "Mozilla/5.0 Test Browser",
        cookies: [cookie("cf_clearance", "expired", NOW_MS / 1000)],
        nowMs: NOW_MS,
      }),
    /expired/,
  );
});

test("cross-client cookie header contains only filtered Cloudflare cookies", () => {
  const header = cloudflareCookieHeader([
    cookie("cf_clearance", "clearance"),
    cookie("__cf_bm", "bot-management"),
    cookie("session", "account-session"),
  ]);

  assert.equal(header, "cf_clearance=clearance; __cf_bm=bot-management");
});

function cookie(name, value, expires = NOW_MS / 1000 + 3600) {
  return {
    name,
    value,
    domain: ".dzmm.ai",
    path: "/",
    expires,
    httpOnly: true,
    secure: true,
    sameSite: "None",
  };
}
