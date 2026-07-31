export class ClearanceUnavailableError extends Error {
  constructor(
    message = "No verified Cloudflare identity is available",
    { cause } = {},
  ) {
    super(message);
    this.name = "ClearanceUnavailableError";
    this.code = "CLEARANCE_UNAVAILABLE";
    this.retryable = true;
    this.cause = cause;
  }
}

export class ClearanceAgentState {
  constructor({ solve, clock = Date.now, logger = null }) {
    if (typeof solve !== "function") {
      throw new TypeError("solve must be a function");
    }
    this.solve = solve;
    this.clock = clock;
    this.logger = logger;
    this.status = "starting";
    this.generation = 0;
    this.snapshot = null;
    this.refreshPromise = null;
    this.transitionListeners = new Set();
  }

  getSnapshot() {
    if (
      !this.snapshot ||
      !isFutureIsoTimestamp(this.snapshot.expires_at, this.clock())
    ) {
      return null;
    }
    return structuredClone(this.snapshot);
  }

  isReady() {
    return this.getSnapshot() !== null;
  }

  subscribe(listener) {
    this.transitionListeners.add(listener);
    return () => this.transitionListeners.delete(listener);
  }

  async refresh({ observed_generation: observedGeneration, reason }) {
    const current = this.getSnapshot();
    if (current && current.generation > observedGeneration) {
      return current;
    }
    if (this.refreshPromise) {
      return this.refreshPromise;
    }

    this.#transition("refreshing", { reason });
    const refreshPromise = this.#runRefresh(reason);
    this.refreshPromise = refreshPromise;
    try {
      return await refreshPromise;
    } finally {
      if (this.refreshPromise === refreshPromise) {
        this.refreshPromise = null;
      }
    }
  }

  async #runRefresh(reason) {
    const startedAt = this.clock();
    try {
      const solved = validateSolvedIdentity(await this.solve());
      if (
        this.snapshot &&
        verifiedIdentityFingerprint(this.snapshot) ===
          verifiedIdentityFingerprint(solved)
      ) {
        const error = new Error(
          "solver returned the currently published Cloudflare identity",
        );
        error.code = "CLEARANCE_NOT_RENEWED";
        throw error;
      }
      const snapshot = {
        generation: this.generation + 1,
        ...structuredClone(solved),
      };
      if (!isFutureIsoTimestamp(snapshot.expires_at, this.clock())) {
        throw new Error("solver returned an expired Cloudflare identity");
      }

      this.snapshot = snapshot;
      this.generation = snapshot.generation;
      this.#transition("ready", {
        generation: snapshot.generation,
        expires_at: snapshot.expires_at,
        solve_duration_ms: this.clock() - startedAt,
        reason,
      });
      return structuredClone(snapshot);
    } catch (error) {
      this.#transition("degraded", {
        error_code: error?.code ?? "CLEARANCE_SOLVE_FAILED",
        solve_duration_ms: this.clock() - startedAt,
        reason,
      });
      throw new ClearanceUnavailableError(undefined, { cause: error });
    }
  }

  #transition(nextStatus, fields) {
    const previousStatus = this.status;
    this.status = nextStatus;
    this.logger?.info?.("state_transition", {
      previous_state: previousStatus,
      state: nextStatus,
      generation: this.generation,
      ...fields,
    });
    for (const listener of this.transitionListeners) {
      try {
        listener({
          previous_state: previousStatus,
          state: nextStatus,
          generation: this.generation,
        });
      } catch {
        this.logger?.warn?.("transition_listener_failed", {
          error_code: "TRANSITION_LISTENER_FAILED",
        });
      }
    }
  }
}

function isFutureIsoTimestamp(value, nowMs) {
  const timestamp = Date.parse(value);
  return Number.isFinite(timestamp) && timestamp > nowMs;
}

function validateSolvedIdentity(identity) {
  if (!identity || typeof identity !== "object") {
    throw new TypeError("solver returned no identity");
  }
  if (
    typeof identity.user_agent !== "string" ||
    identity.user_agent.length === 0
  ) {
    throw new TypeError("solver returned no user agent");
  }
  if (
    !Array.isArray(identity.cookies) ||
    !identity.cookies.some((cookie) => cookie.name === "cf_clearance")
  ) {
    throw new TypeError("solver returned no cf_clearance cookie");
  }
  if (!Number.isFinite(Date.parse(identity.expires_at))) {
    throw new TypeError("solver returned an invalid expiry");
  }
  if (!Number.isFinite(Date.parse(identity.verified_at))) {
    throw new TypeError("solver returned an invalid verification time");
  }
  return identity;
}

function verifiedIdentityFingerprint(identity) {
  const cookies = identity.cookies
    .map(({ name, value, domain, path, expires }) => ({
      name,
      value,
      domain,
      path,
      expires,
    }))
    .sort((left, right) =>
      `${left.name}\0${left.domain}\0${left.path}`.localeCompare(
        `${right.name}\0${right.domain}\0${right.path}`,
      ),
    );
  return JSON.stringify({
    user_agent: identity.user_agent,
    cookies,
    expires_at: identity.expires_at,
  });
}
