const DEFAULT_BASE_BACKOFF_MS = 1000;
const DEFAULT_MAX_BACKOFF_MS = 60_000;
const DEFAULT_REFRESH_LEAD_MS = 300_000;
const MIN_SCHEDULE_DELAY_MS = 1000;

export class RefreshSupervisor {
  constructor({
    state,
    clock = Date.now,
    setTimer = setTimeout,
    clearTimer = clearTimeout,
    logger = null,
    baseBackoffMs = DEFAULT_BASE_BACKOFF_MS,
    maxBackoffMs = DEFAULT_MAX_BACKOFF_MS,
    refreshLeadMs = DEFAULT_REFRESH_LEAD_MS,
  }) {
    this.state = state;
    this.clock = clock;
    this.setTimer = setTimer;
    this.clearTimer = clearTimer;
    this.logger = logger;
    this.baseBackoffMs = baseBackoffMs;
    this.maxBackoffMs = maxBackoffMs;
    this.refreshLeadMs = refreshLeadMs;
    this.failureCount = 0;
    this.lastExpiryMs = null;
    this.timer = null;
    this.unsubscribe = null;
    this.running = false;
  }

  start() {
    if (this.running) {
      return;
    }
    this.running = true;
    this.unsubscribe = this.state.subscribe((transition) =>
      this.#onTransition(transition),
    );
  }

  stop() {
    this.running = false;
    this.unsubscribe?.();
    this.unsubscribe = null;
    this.#clearScheduled();
  }

  #onTransition({ state }) {
    if (!this.running) {
      return;
    }
    if (state === "refreshing") {
      this.#clearScheduled();
      return;
    }
    if (state === "degraded") {
      this.failureCount += 1;
      const delay = Math.min(
        this.baseBackoffMs * 2 ** (this.failureCount - 1),
        this.maxBackoffMs,
      );
      this.logger?.warn?.("refresh_retry_scheduled", {
        error_code: "CLEARANCE_UNAVAILABLE",
        retry_delay_ms: delay,
      });
      this.#schedule(delay, "backoff");
      return;
    }
    if (state === "ready") {
      const snapshot = this.state.getSnapshot();
      if (!snapshot) {
        return;
      }
      const expiryMs = Date.parse(snapshot.expires_at);
      if (
        this.lastExpiryMs !== null &&
        expiryMs <= this.lastExpiryMs
      ) {
        this.failureCount += 1;
        const delay = Math.min(
          this.baseBackoffMs * 2 ** (this.failureCount - 1),
          this.maxBackoffMs,
        );
        this.logger?.warn?.("refresh_retry_scheduled", {
          error_code: "CLEARANCE_NOT_RENEWED",
          retry_delay_ms: delay,
        });
        this.#schedule(delay, "renewal-backoff");
        return;
      }
      this.failureCount = 0;
      this.lastExpiryMs = expiryMs;
      const refreshAt = expiryMs - this.refreshLeadMs;
      const delay = Math.max(
        MIN_SCHEDULE_DELAY_MS,
        refreshAt - this.clock(),
      );
      this.#schedule(delay, "scheduled");
    }
  }

  #schedule(delay, reason) {
    if (!this.running) {
      return;
    }
    this.#clearScheduled();
    this.timer = this.setTimer(async () => {
      this.timer = null;
      try {
        await this.state.refresh({
          observed_generation: this.state.generation,
          reason,
        });
      } catch {
        // The degraded state transition owns backoff scheduling and logging.
      }
    }, delay);
  }

  #clearScheduled() {
    if (this.timer !== null) {
      this.clearTimer(this.timer);
      this.timer = null;
    }
  }
}
