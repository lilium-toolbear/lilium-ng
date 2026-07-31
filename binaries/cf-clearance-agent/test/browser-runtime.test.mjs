import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { PassThrough } from "node:stream";
import test from "node:test";

import { createBrowserRuntime } from "../src/browser-runtime.mjs";

test("runtime launches headed Chromium externally and attaches only through CDP", async () => {
  const launches = [];
  const connected = [];
  const page = fakePage();
  const browser = {
    contexts: () => [
      {
        pages: () => [page],
        cookies: async () => [],
      },
    ],
    close: async () => {},
  };
  const chromium = {
    executablePath: () => "/opt/playwright/chromium/chrome",
    launch: () => {
      throw new Error("Playwright launch must not be used");
    },
    launchPersistentContext: () => {
      throw new Error("Playwright persistent launch must not be used");
    },
    connectOverCDP: async (endpoint, options) => {
      connected.push({ endpoint, options });
      return browser;
    },
  };
  const runtime = createBrowserRuntime({
    chromium,
    config: {
      display: ":99",
      profileDir: "/data/chrome-profile",
      cdpPort: 9223,
      browserStartupTimeoutMs: 30_000,
    },
    mkdir: async () => {},
    spawnProcess: (command, args, options) => {
      launches.push({ command, args, options });
      return fakeChild();
    },
    waitForXServer: async () => {},
    waitForCdp: async () => {},
  });

  const first = await runtime.getSession();
  const second = await runtime.getSession();

  assert.equal(first, second);
  assert.equal(launches[0].command, "Xvfb");
  assert.deepEqual(launches[0].args.slice(0, 3), [
    ":99",
    "-screen",
    "0",
  ]);
  assert.equal(launches[1].command, "/opt/playwright/chromium/chrome");
  assert.ok(
    launches[1].args.includes("--remote-debugging-address=127.0.0.1"),
  );
  assert.ok(launches[1].args.includes("--remote-debugging-port=9223"));
  assert.ok(
    launches[1].args.includes("--user-data-dir=/data/chrome-profile"),
  );
  assert.equal(launches[1].args.some((arg) => arg.includes("headless")), false);
  assert.equal(
    launches[1].args.some((arg) => arg.includes("enable-automation")),
    false,
  );
  assert.equal(launches[1].args.some((arg) => arg === "--no-sandbox"), false);
  assert.equal(launches[1].options.env.DISPLAY, ":99");
  assert.deepEqual(connected, [
    {
      endpoint: "http://127.0.0.1:9223",
      options: {
        isLocal: true,
        noDefaults: true,
        timeout: 30_000,
      },
    },
  ]);
});

test("browser session finds a visible Cloudflare iframe and clicks its checkbox area", async () => {
  const mouseClicks = [];
  const mouseMoves = [];
  const pauses = [];
  const page = fakePage({
    frames: [
      fakeFrame({
        src: "https://example.test/widget",
        title: "unrelated",
        visible: true,
      }),
      fakeFrame({
        src: "https://challenges.cloudflare.com/cdn-cgi/challenge-platform/h/g/turnstile/if/ov2/av0/rcv0/",
        title: "Widget containing a Cloudflare security challenge",
        visible: true,
        box: { x: 100, y: 200, width: 300, height: 65 },
      }),
    ],
    mouseClicks,
    mouseMoves,
  });
  const context = {
    pages: () => [page],
    cookies: async () => [],
  };
  const runtime = createRuntimeWithContext(context, {
    random: () => 0.5,
    interactionSleep: async (milliseconds) => pauses.push(milliseconds),
  });
  const session = await runtime.getSession();

  const widget = await session.getChallengeWidget();
  await session.clickChallengeWidget(widget);

  assert.deepEqual(widget, { x: 100, y: 200, width: 300, height: 65 });
  assert.deepEqual(mouseMoves, [
    { x: 50, y: 212.5, options: { steps: 12 } },
    { x: 130, y: 232.5, options: { steps: 8 } },
  ]);
  assert.deepEqual(pauses, [180, 120]);
  assert.deepEqual(mouseClicks, [
    { x: 130, y: 232.5, options: { delay: 80 } },
  ]);
});

test("reset waits for Chromium to exit before launching a replacement", async () => {
  const launches = [];
  const xvfb = fakeChild();
  const firstChromium = deferredExitChild();
  const replacementChromium = fakeChild();
  const children = [xvfb, firstChromium, replacementChromium];
  const runtime = createBrowserRuntime({
    chromium: {
      executablePath: () => "/chromium",
      connectOverCDP: async () => ({
        contexts: () => [
          {
            pages: () => [fakePage()],
            cookies: async () => [],
          },
        ],
        close: async () => {},
      }),
    },
    config: {
      display: ":99",
      profileDir: "/data/chrome-profile",
      cdpPort: 9223,
      browserStartupTimeoutMs: 30_000,
    },
    mkdir: async () => {},
    spawnProcess: (command) => {
      launches.push(command);
      return children.shift();
    },
    waitForXServer: async () => {},
    waitForCdp: async () => {},
  });

  await runtime.getSession();
  const reset = runtime.resetSession();
  const replacement = runtime.getSession();
  await new Promise((resolve) => setImmediate(resolve));

  assert.deepEqual(launches, ["Xvfb", "/chromium"]);
  assert.deepEqual(firstChromium.signals, ["SIGTERM"]);

  firstChromium.exitNow(0);
  await reset;
  await replacement;

  assert.deepEqual(launches, ["Xvfb", "/chromium", "/chromium"]);
});

test("Chromium startup failure logs a bounded redacted stderr diagnostic", async () => {
  const errors = [];
  const chromiumChild = diagnosticChild();
  const runtime = createBrowserRuntime({
    chromium: {
      executablePath: () => "/chromium",
      connectOverCDP: async () => {
        throw new Error("unexpected connect");
      },
    },
    config: {
      display: ":99",
      profileDir: "/data/chrome-profile",
      cdpPort: 9223,
      browserStartupTimeoutMs: 30_000,
    },
    mkdir: async () => {},
    spawnProcess: (command) =>
      command === "Xvfb" ? fakeChild() : chromiumChild,
    waitForXServer: async () => {},
    waitForCdp: async () => {
      chromiumChild.stderr.write(
        "No usable sandbox at /data/chrome-profile cf_clearance=secret https://www.dzmm.ai/path?token=secret",
      );
      throw new Error("Chromium CDP did not become ready");
    },
    logger: {
      info: () => {},
      warn: () => {},
      error: (event, fields) => errors.push({ event, fields }),
    },
  });

  await assert.rejects(runtime.getSession(), /did not become ready/);

  assert.equal(errors[0].event, "chromium_start_failed");
  assert.match(errors[0].fields.diagnostic, /No usable sandbox/);
  assert.doesNotMatch(
    errors[0].fields.diagnostic,
    /chrome-profile|secret|dzmm\.ai/,
  );
  assert.ok(errors[0].fields.diagnostic.length <= 4096);
});

function createRuntimeWithContext(context, options = {}) {
  return createBrowserRuntime({
    chromium: {
      executablePath: () => "/chromium",
      connectOverCDP: async () => ({
        contexts: () => [context],
        close: async () => {},
      }),
    },
    config: {
      display: ":99",
      profileDir: "/data/chrome-profile",
      cdpPort: 9223,
      browserStartupTimeoutMs: 30_000,
    },
    mkdir: async () => {},
    spawnProcess: () => fakeChild(),
    waitForXServer: async () => {},
    waitForCdp: async () => {},
    ...options,
  });
}

function fakePage({ frames = [], mouseClicks = [], mouseMoves = [] } = {}) {
  return {
    goto: async () => {},
    evaluate: async () => "Mozilla/5.0 Exact Browser",
    locator: () => ({
      count: async () => frames.length,
      nth: (index) => frames[index],
    }),
    mouse: {
      move: async (x, y, options) => mouseMoves.push({ x, y, options }),
      click: async (x, y, options) => mouseClicks.push({ x, y, options }),
    },
  };
}

function fakeFrame({
  src,
  title,
  visible,
  box = { x: 0, y: 0, width: 100, height: 100 },
}) {
  return {
    getAttribute: async (name) => (name === "src" ? src : title),
    isVisible: async () => visible,
    boundingBox: async () => box,
  };
}

function fakeChild() {
  const child = new EventEmitter();
  child.exitCode = null;
  child.kill = () => {
    child.exitCode = 0;
    return true;
  };
  return child;
}

function deferredExitChild() {
  const child = new EventEmitter();
  child.exitCode = null;
  child.signalCode = null;
  child.signals = [];
  child.kill = (signal) => {
    child.signals.push(signal);
    return true;
  };
  child.exitNow = (code) => {
    child.exitCode = code;
    child.emit("exit", code, null);
  };
  return child;
}

function diagnosticChild() {
  const child = fakeChild();
  child.stderr = new PassThrough();
  return child;
}
