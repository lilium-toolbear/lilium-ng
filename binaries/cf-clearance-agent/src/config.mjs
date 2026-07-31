import path from "node:path";

const DEFAULTS = {
  origin: "https://www.dzmm.ai",
  listenHost: "0.0.0.0",
  listenPort: 8787,
  cdpPort: 9223,
  profileDir: "/data/chrome-profile",
  solveTimeoutMs: 90_000,
  display: ":99",
  browserStartupTimeoutMs: 30_000,
  bodyLimitBytes: 4096,
};

export function loadConfig(environment = process.env) {
  const origin = parseOrigin(
    environment.CF_CLEARANCE_ORIGIN ?? DEFAULTS.origin,
  );
  const profileDir =
    environment.CF_CLEARANCE_PROFILE_DIR ?? DEFAULTS.profileDir;
  if (!path.isAbsolute(profileDir)) {
    throw new TypeError("CF_CLEARANCE_PROFILE_DIR must be an absolute path");
  }
  const display = environment.DISPLAY ?? DEFAULTS.display;
  if (!/^:\d+(?:\.\d+)?$/.test(display)) {
    throw new TypeError("DISPLAY must be an X display such as :99");
  }

  return {
    origin,
    listenHost:
      environment.CF_CLEARANCE_LISTEN_HOST ?? DEFAULTS.listenHost,
    listenPort: parseInteger(
      "CF_CLEARANCE_LISTEN_PORT",
      environment.CF_CLEARANCE_LISTEN_PORT,
      DEFAULTS.listenPort,
      1,
      65_535,
    ),
    cdpPort: parseInteger(
      "CF_CLEARANCE_CDP_PORT",
      environment.CF_CLEARANCE_CDP_PORT,
      DEFAULTS.cdpPort,
      1,
      65_535,
    ),
    profileDir,
    solveTimeoutMs: parseInteger(
      "CF_CLEARANCE_SOLVE_TIMEOUT_MS",
      environment.CF_CLEARANCE_SOLVE_TIMEOUT_MS,
      DEFAULTS.solveTimeoutMs,
      1000,
      300_000,
    ),
    display,
    browserStartupTimeoutMs: DEFAULTS.browserStartupTimeoutMs,
    bodyLimitBytes: DEFAULTS.bodyLimitBytes,
  };
}

function parseOrigin(value) {
  let url;
  try {
    url = new URL(value);
  } catch {
    throw new TypeError("CF_CLEARANCE_ORIGIN must be a valid HTTPS origin");
  }
  if (
    url.protocol !== "https:" ||
    url.username ||
    url.password ||
    (url.pathname !== "/" && url.pathname !== "") ||
    url.search ||
    url.hash
  ) {
    throw new TypeError("CF_CLEARANCE_ORIGIN must be a valid HTTPS origin");
  }
  return url.origin;
}

function parseInteger(name, rawValue, defaultValue, minimum, maximum) {
  if (rawValue === undefined) {
    return defaultValue;
  }
  if (!/^\d+$/.test(rawValue)) {
    throw new TypeError(`${name} must be an integer`);
  }
  const value = Number(rawValue);
  if (!Number.isSafeInteger(value) || value < minimum || value > maximum) {
    throw new RangeError(`${name} must be between ${minimum} and ${maximum}`);
  }
  return value;
}
