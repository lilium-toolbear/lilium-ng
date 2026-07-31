import { spawn } from "node:child_process";
import { access, mkdir as makeDirectory } from "node:fs/promises";

const XVFB_SCREEN = "1920x1080x24";
const STARTUP_POLL_MS = 100;
const CHILD_EXIT_GRACE_MS = 2_000;
const STDERR_TAIL_BYTES = 16_384;
const LOG_DIAGNOSTIC_BYTES = 4_096;

export function createBrowserRuntime({
  chromium,
  config,
  logger = null,
  mkdir = makeDirectory,
  spawnProcess = spawn,
  waitForXServer = defaultWaitForXServer,
  waitForCdp = defaultWaitForCdp,
  random = Math.random,
  interactionSleep = defaultSleep,
}) {
  let xvfbProcess = null;
  let chromiumProcess = null;
  let browser = null;
  let sessionPromise = null;
  let chromiumStopPromise = null;
  let stoppingChromium = false;
  let closing = false;

  async function startSession() {
    await chromiumStopPromise;
    await mkdir(config.profileDir, { recursive: true });
    if (!xvfbProcess || xvfbProcess.exitCode !== null) {
      xvfbProcess = spawnProcess(
        "Xvfb",
        [config.display, "-screen", "0", XVFB_SCREEN, "-nolisten", "tcp", "-ac"],
        { stdio: ["ignore", "ignore", "pipe"] },
      );
      const xvfbStderr = captureStderrTail(xvfbProcess);
      try {
        await waitForXServer({
          display: config.display,
          child: xvfbProcess,
          timeoutMs: config.browserStartupTimeoutMs,
        });
      } catch (error) {
        logger?.error?.("xvfb_start_failed", {
          error_code: "XVFB_START_FAILED",
          diagnostic: sanitizeChildDiagnostic(
            xvfbStderr(),
            config.profileDir,
          ),
        });
        await terminateChild(xvfbProcess);
        xvfbProcess = null;
        throw error;
      }
      logger?.info?.("xvfb_ready", { display: config.display });
    }

    const cdpEndpoint = `http://127.0.0.1:${config.cdpPort}`;
    chromiumProcess = spawnProcess(
      chromium.executablePath(),
      [
        "--remote-debugging-address=127.0.0.1",
        `--remote-debugging-port=${config.cdpPort}`,
        `--user-data-dir=${config.profileDir}`,
        "--no-first-run",
        "--no-default-browser-check",
        "--password-store=basic",
        "--window-size=1280,960",
        "about:blank",
      ],
      {
        env: { ...process.env, DISPLAY: config.display },
        stdio: ["ignore", "ignore", "pipe"],
      },
    );
    const chromiumStderr = captureStderrTail(chromiumProcess);
    try {
      await waitForCdp({
        endpoint: cdpEndpoint,
        child: chromiumProcess,
        timeoutMs: config.browserStartupTimeoutMs,
      });

      browser = await chromium.connectOverCDP(cdpEndpoint, {
        isLocal: true,
        noDefaults: true,
        timeout: config.browserStartupTimeoutMs,
      });
      const context = browser.contexts()[0];
      if (!context) {
        throw new Error(
          "externally launched Chromium has no default context",
        );
      }
      const page = context.pages()[0] ?? (await context.newPage());
      browser.on?.("disconnected", () => {
        if (!closing && !stoppingChromium) {
          browser = null;
          sessionPromise = null;
          void stopChromium().catch((error) => {
            logger?.error?.("chromium_stop_failed", {
              error_code: error?.code ?? "CHROMIUM_STOP_FAILED",
            });
          });
          logger?.warn?.("chromium_disconnected", {
            error_code: "CHROMIUM_DISCONNECTED",
          });
        }
      });
      logger?.info?.("chromium_connected", {
        cdp_host: "127.0.0.1",
        cdp_port: config.cdpPort,
      });
      return new PlaywrightBrowserSession({
        context,
        page,
        random,
        interactionSleep,
      });
    } catch (error) {
      logger?.error?.("chromium_start_failed", {
        error_code: "CHROMIUM_START_FAILED",
        diagnostic: sanitizeChildDiagnostic(
          chromiumStderr(),
          config.profileDir,
        ),
      });
      throw error;
    }
  }

  async function stopChromium() {
    if (chromiumStopPromise) {
      return chromiumStopPromise;
    }
    const connectedBrowser = browser;
    const child = chromiumProcess;
    browser = null;
    stoppingChromium = true;
    chromiumStopPromise = (async () => {
      if (connectedBrowser) {
        await connectedBrowser.close().catch(() => {});
      }
      await terminateChild(child);
      if (chromiumProcess === child) {
        chromiumProcess = null;
      }
    })().finally(() => {
      stoppingChromium = false;
      chromiumStopPromise = null;
    });
    return chromiumStopPromise;
  }

  return {
    getSession() {
      if (!sessionPromise) {
        sessionPromise = startSession().catch(async (error) => {
          sessionPromise = null;
          await stopChromium();
          throw error;
        });
      }
      return sessionPromise;
    },

    async resetSession() {
      sessionPromise = null;
      await stopChromium();
    },

    async close() {
      closing = true;
      sessionPromise = null;
      await stopChromium();
      await terminateChild(xvfbProcess);
      xvfbProcess = null;
    },
  };
}

class PlaywrightBrowserSession {
  constructor({ context, page, random, interactionSleep }) {
    this.context = context;
    this.page = page;
    this.random = random;
    this.interactionSleep = interactionSleep;
  }

  async navigate(url, timeoutMs) {
    await this.page.goto(url, {
      waitUntil: "domcontentloaded",
      timeout: Math.min(timeoutMs, 30_000),
    });
  }

  async browserProbe(url, timeoutMs) {
    return this.page.evaluate(async ({ probeUrl, timeoutMs }) => {
      const controller = new AbortController();
      const timeout = setTimeout(
        () => controller.abort(),
        Math.max(1, timeoutMs),
      );
      try {
        const response = await window.fetch(probeUrl, {
          method: "GET",
          credentials: "include",
          redirect: "follow",
          signal: controller.signal,
          headers: { accept: "application/json" },
        });
        const contentType = response.headers.get("content-type") ?? "";
        const body = await response.text();
        let isJson = false;
        try {
          JSON.parse(body);
          isJson = contentType.toLowerCase().includes("json");
        } catch {
          isJson = false;
        }
        return {
          status: response.status,
          content_type: contentType,
          challenged: response.headers.get("cf-mitigated") === "challenge",
          is_json: isJson,
          cf_ray: response.headers.get("cf-ray"),
        };
      } catch {
        return {
          status: 0,
          content_type: "",
          challenged: false,
          is_json: false,
          cf_ray: null,
        };
      } finally {
        clearTimeout(timeout);
      }
    }, { probeUrl: url, timeoutMs });
  }

  getCookies(origin) {
    return this.context.cookies(origin);
  }

  getUserAgent() {
    return this.page.evaluate(() => navigator.userAgent);
  }

  async getChallengeWidget() {
    const frames = this.page.locator("iframe");
    const frameCount = await frames.count();
    for (let index = 0; index < frameCount; index += 1) {
      const frame = frames.nth(index);
      try {
        const [src, title, visible] = await Promise.all([
          frame.getAttribute("src"),
          frame.getAttribute("title"),
          frame.isVisible(),
        ]);
        if (!visible || !isCloudflareFrame(src, title)) {
          continue;
        }
        const box = await frame.boundingBox();
        if (box && box.width > 0 && box.height > 0) {
          return box;
        }
      } catch {
        // The challenge DOM can replace the iframe between locator operations.
      }
    }
    return null;
  }

  async clickChallengeWidget({ x, y, width, height }) {
    const clickX = x + Math.min(28 + this.random() * 4, width / 2);
    const clickY = y + height / 2 + (this.random() * 4 - 2);
    await this.page.mouse.move(clickX - 80, clickY - 20, { steps: 12 });
    await this.interactionSleep(180);
    await this.page.mouse.move(clickX, clickY, { steps: 8 });
    await this.interactionSleep(120);
    await this.page.mouse.click(clickX, clickY, { delay: 80 });
  }
}

function isCloudflareFrame(src = "", title = "") {
  const identity = `${src ?? ""} ${title ?? ""}`.toLowerCase();
  return (
    identity.includes("challenges.cloudflare.com") ||
    identity.includes("/cdn-cgi/challenge-platform/") ||
    identity.includes("cloudflare security challenge") ||
    identity.includes("verify you are human")
  );
}

async function defaultWaitForXServer({ display, child, timeoutMs }) {
  const displayNumber = display.replace(/^:/, "").split(".")[0];
  const socketPath = `/tmp/.X11-unix/X${displayNumber}`;
  await waitUntilReady({
    child,
    timeoutMs,
    description: "Xvfb",
    check: async () => {
      await access(socketPath);
      return true;
    },
  });
}

async function defaultWaitForCdp({ endpoint, child, timeoutMs }) {
  await waitUntilReady({
    child,
    timeoutMs,
    description: "Chromium CDP",
    check: async () => {
      const response = await fetch(`${endpoint}/json/version`, {
        signal: AbortSignal.timeout(1000),
      });
      return response.ok;
    },
  });
}

async function waitUntilReady({
  child,
  timeoutMs,
  description,
  check,
}) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`${description} exited before becoming ready`);
    }
    try {
      if (await check()) {
        return;
      }
    } catch {
      // The readiness endpoint or socket is expected to be absent at first.
    }
    await new Promise((resolve) => setTimeout(resolve, STARTUP_POLL_MS));
  }
  throw new Error(`${description} did not become ready before timeout`);
}

async function terminateChild(child) {
  if (!child || child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  const exited = new Promise((resolve) => child.once("exit", resolve));
  child.kill("SIGTERM");
  if (child.exitCode !== null || child.signalCode !== null) {
    return;
  }
  if (await settlesBefore(exited, CHILD_EXIT_GRACE_MS)) {
    return;
  }
  child.kill("SIGKILL");
  if (
    child.exitCode === null &&
    child.signalCode === null &&
    !(await settlesBefore(exited, CHILD_EXIT_GRACE_MS))
  ) {
    const error = new Error("child process did not exit after SIGKILL");
    error.code = "CHILD_EXIT_TIMEOUT";
    throw error;
  }
}

async function settlesBefore(promise, timeoutMs) {
  let timer;
  const timedOut = new Promise((resolve) => {
    timer = setTimeout(() => resolve(false), timeoutMs);
  });
  const settled = promise.then(() => true);
  try {
    return await Promise.race([settled, timedOut]);
  } finally {
    clearTimeout(timer);
  }
}

function captureStderrTail(child) {
  let tail = "";
  child?.stderr?.setEncoding?.("utf8");
  child?.stderr?.on?.("data", (chunk) => {
    tail = `${tail}${chunk}`.slice(-STDERR_TAIL_BYTES);
  });
  return () => tail;
}

function sanitizeChildDiagnostic(diagnostic, profileDir) {
  if (!diagnostic) {
    return "no stderr output";
  }
  let sanitized = diagnostic;
  if (profileDir) {
    sanitized = sanitized.replaceAll(profileDir, "<profile>");
  }
  sanitized = sanitized
    .replace(/https?:\/\/[^\s"'<>]+/giu, "<url>")
    .replace(
      /\b((?:cf_clearance|__cf[\w-]*|_cf[\w-]*)=)[^;\s]+/giu,
      "$1<redacted>",
    )
    .replace(/\b((?:token|secret|password)=)[^;\s]+/giu, "$1<redacted>")
    .replace(/(^|\s)\/(?:[^\s/:]+\/)*[^\s:]*/gmu, "$1<path>");
  return sanitized.trim().slice(-LOG_DIAGNOSTIC_BYTES);
}

function defaultSleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
