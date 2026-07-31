import {
  cloudflareCookieHeader,
  createVerifiedIdentity,
  filterCloudflareCookies,
} from "./identity.mjs";

const PROBE_PATH = "/api/trpc/user.getMe";
const PROBE_INPUT = JSON.stringify({ "0": { json: null } });
const POLL_INTERVAL_MS = 250;
const CLICK_INTERVAL_MS = 8000;
const MAX_WIDGET_CLICKS = 3;
const CROSS_PROBE_TIMEOUT_MS = 15_000;
const CROSS_PROBE_INTERVAL_MS = 8000;

export class ChallengeSolveError extends Error {
  constructor(code, message, { cause } = {}) {
    super(message);
    this.name = "ChallengeSolveError";
    this.code = code;
    this.cause = cause;
  }
}

export function createChallengeSolver({
  runtime,
  origin,
  timeoutMs,
  fetchImpl = fetch,
  clock = Date.now,
  sleep = defaultSleep,
  setTimer = setTimeout,
  clearTimer = clearTimeout,
  logger = null,
}) {
  const probeUrl = buildProbeUrl(origin);

  return async function solve() {
    let deadlineTimer;
    const hardDeadline = new Promise((_, reject) => {
      deadlineTimer = setTimer(() => {
        logger?.error?.("solve_failed", {
          error_code: "SOLVE_TIMEOUT",
          solve_duration_ms: timeoutMs,
          deadline_kind: "hard",
        });
        reject(
          new ChallengeSolveError(
            "SOLVE_TIMEOUT",
            "Cloudflare challenge solve exceeded its hard deadline",
          ),
        );
      }, timeoutMs);
    });

    try {
      return await Promise.race([runSolveAttempt(), hardDeadline]);
    } catch (error) {
      if (error?.code === "SOLVE_TIMEOUT") {
        try {
          await runtime.resetSession?.();
        } catch (resetError) {
          logger?.warn?.("browser_session_reset_failed", {
            error_code: resetError?.code ?? "BROWSER_RESET_FAILED",
          });
        }
      }
      throw error;
    } finally {
      clearTimer(deadlineTimer);
    }
  };

  async function runSolveAttempt() {
    const startedAt = clock();
    const deadline = startedAt + timeoutMs;
    const session = await runtime.getSession();
    let clickCount = 0;
    let nextClickAt = startedAt;
    let lastCrossProbeAt = Number.NEGATIVE_INFINITY;
    let lastCrossCookieSignature = null;
    let lastBrowserProbeLogAt = Number.NEGATIVE_INFINITY;
    let lastBrowserProbeLogKey = null;
    let lastRecoverableErrorLogAt = Number.NEGATIVE_INFINITY;
    let lastRecoverableErrorKey = null;
    let lastBrowserProbe = null;
    let lastCrossProbe = null;

    const logRecoverableError = (operation, error) => {
      const errorCode = error?.code ?? "PROBE_FAILED";
      const key = `${operation}:${errorCode}`;
      const now = clock();
      if (
        key !== lastRecoverableErrorKey ||
        now - lastRecoverableErrorLogAt >= CROSS_PROBE_INTERVAL_MS
      ) {
        logger?.warn?.("solve_probe_failed", {
          error_code: errorCode,
          operation,
        });
        lastRecoverableErrorLogAt = now;
        lastRecoverableErrorKey = key;
      }
    };

    try {
      await session.navigate(probeUrl, Math.max(1, deadline - clock()));
    } catch (error) {
      throw new ChallengeSolveError(
        "BROWSER_NAVIGATION_FAILED",
        "Chromium could not navigate to the Cloudflare probe",
        { cause: error },
      );
    }

    while (clock() < deadline) {
      try {
        const [browserProbe, cookies] = await Promise.all([
          session.browserProbe(probeUrl, Math.max(1, deadline - clock())),
          session.getCookies(origin),
        ]);
        lastBrowserProbe = browserProbe;
        const browserProbeLogKey = JSON.stringify([
          browserProbe.status,
          browserProbe.challenged,
          browserProbe.is_json,
        ]);
        if (
          browserProbeLogKey !== lastBrowserProbeLogKey ||
          clock() - lastBrowserProbeLogAt >= CROSS_PROBE_INTERVAL_MS
        ) {
          logger?.info?.("browser_probe", {
            status: browserProbe.status,
            challenged: browserProbe.challenged,
            json: browserProbe.is_json,
            cf_ray: browserProbe.cf_ray,
          });
          lastBrowserProbeLogAt = clock();
          lastBrowserProbeLogKey = browserProbeLogKey;
        }

        if (probePassed(browserProbe)) {
          const cloudflareCookies = filterCloudflareCookies(cookies);
          const userAgent = await session.getUserAgent();
          const cookieSignature = cloudflareCookies
            .map(({ name, value, domain, path }) =>
              JSON.stringify([name, value, domain, path]),
            )
            .join("|");
          if (
            cookieSignature !== lastCrossCookieSignature ||
            clock() - lastCrossProbeAt >= CROSS_PROBE_INTERVAL_MS
          ) {
            const crossProbe = await probeWithFetch({
              fetchImpl,
              probeUrl,
              userAgent,
              cookies: cloudflareCookies,
              timeoutMs: Math.min(
                CROSS_PROBE_TIMEOUT_MS,
                Math.max(1, deadline - clock()),
              ),
            });
            lastCrossProbe = crossProbe;
            lastCrossProbeAt = clock();
            lastCrossCookieSignature = cookieSignature;
            logger?.info?.("cross_client_probe", {
              status: crossProbe.status,
              challenged: crossProbe.challenged,
              json: crossProbe.is_json,
              cf_ray: crossProbe.cf_ray,
            });

            if (probePassed(crossProbe)) {
              const identity = createVerifiedIdentity({
                userAgent,
                cookies: cloudflareCookies,
                nowMs: clock(),
              });
              logger?.info?.("solve_complete", {
                solve_duration_ms: clock() - startedAt,
                click_count: clickCount,
                expires_at: identity.expires_at,
                browser_probe_status: browserProbe.status,
                cross_probe_status: crossProbe.status,
                cf_ray: crossProbe.cf_ray ?? browserProbe.cf_ray,
              });
              return identity;
            }
          }
        }
      } catch (error) {
        logRecoverableError("probe", error);
      }

      const now = clock();
      if (clickCount < MAX_WIDGET_CLICKS && now >= nextClickAt) {
        try {
          const widget = await session.getChallengeWidget();
          if (widget) {
            await session.clickChallengeWidget(widget);
            clickCount += 1;
            nextClickAt = now + CLICK_INTERVAL_MS;
            logger?.info?.("challenge_widget_clicked", {
              click_count: clickCount,
            });
          }
        } catch (error) {
          logRecoverableError("challenge_widget", error);
        }
      }

      const remainingMs = deadline - clock();
      if (remainingMs > 0) {
        await sleep(Math.min(POLL_INTERVAL_MS, remainingMs));
      }
    }

    logger?.error?.("solve_failed", {
      error_code: "SOLVE_TIMEOUT",
      solve_duration_ms: clock() - startedAt,
      click_count: clickCount,
      browser_probe_status: lastBrowserProbe?.status,
      cross_probe_status: lastCrossProbe?.status,
      cf_ray: lastCrossProbe?.cf_ray ?? lastBrowserProbe?.cf_ray,
    });
    throw new ChallengeSolveError(
      "SOLVE_TIMEOUT",
      "Cloudflare challenge did not pass before the solve deadline",
    );
  }
}

function buildProbeUrl(origin) {
  const probeUrl = new URL(PROBE_PATH, origin);
  probeUrl.searchParams.set("batch", "1");
  probeUrl.searchParams.set("input", PROBE_INPUT);
  return probeUrl.toString();
}

async function probeWithFetch({
  fetchImpl,
  probeUrl,
  userAgent,
  cookies,
  timeoutMs,
}) {
  const response = await fetchImpl(probeUrl, {
    method: "GET",
    redirect: "manual",
    signal: AbortSignal.timeout(timeoutMs),
    headers: {
      accept: "application/json, text/plain, */*",
      "accept-language": "en-US,en;q=0.9",
      cookie: cloudflareCookieHeader(cookies),
      "user-agent": userAgent,
    },
  });
  const contentType = response.headers.get("content-type") ?? "";
  const body = await response.text();
  return {
    status: response.status,
    content_type: contentType,
    challenged: response.headers.get("cf-mitigated") === "challenge",
    is_json: contentType.toLowerCase().includes("json") && isJson(body),
    cf_ray: response.headers.get("cf-ray"),
  };
}

function probePassed(probe) {
  return (
    probe.status === 200 &&
    probe.challenged === false &&
    probe.is_json === true
  );
}

function isJson(body) {
  try {
    JSON.parse(body);
    return true;
  } catch {
    return false;
  }
}

function defaultSleep(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
