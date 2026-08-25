import { chromium } from "playwright-core";

import { createBrowserRuntime } from "./browser-runtime.mjs";
import { loadConfig } from "./config.mjs";
import { createLogger } from "./logger.mjs";
import { createAgentServer } from "./server.mjs";
import { createChallengeSolver } from "./solver.mjs";
import { ClearanceAgentState } from "./state.mjs";
import { RefreshSupervisor } from "./supervisor.mjs";

const logger = createLogger();
let runtime = null;
let server = null;
let supervisor = null;
let shuttingDown = false;

try {
  const config = loadConfig();
  runtime = createBrowserRuntime({ chromium, config, logger });
  const solve = createChallengeSolver({
    runtime,
    origin: config.origin,
    timeoutMs: config.solveTimeoutMs,
    logger,
  });
  const state = new ClearanceAgentState({ solve, logger });
  supervisor = new RefreshSupervisor({ state, logger });
  server = createAgentServer({
    state,
    bodyLimitBytes: config.bodyLimitBytes,
    logger,
  });

  await listen(server, config.listenPort, config.listenHost);
  logger.info("http_server_listening", {
    host: config.listenHost,
    port: config.listenPort,
    origin: config.origin,
  });
  supervisor.start();

  process.once("SIGINT", () => void shutdown("SIGINT"));
  process.once("SIGTERM", () => void shutdown("SIGTERM"));
} catch (error) {
  logger.error("agent_start_failed", {
    error_code: error?.code ?? "AGENT_START_FAILED",
    error_type: error?.name ?? "Error",
  });
  await runtime?.close();
  process.exitCode = 1;
}

async function shutdown(signal) {
  if (shuttingDown) {
    return;
  }
  shuttingDown = true;
  logger.info("agent_stopping", { signal });
  supervisor?.stop();
  if (server) {
    server.closeAllConnections?.();
    await new Promise((resolve) => server.close(resolve));
  }
  await runtime?.close();
  logger.info("agent_stopped", { signal });
}

function listen(httpServer, port, host) {
  return new Promise((resolve, reject) => {
    const onError = (error) => {
      httpServer.off("listening", onListening);
      reject(error);
    };
    const onListening = () => {
      httpServer.off("error", onError);
      resolve();
    };
    httpServer.once("error", onError);
    httpServer.once("listening", onListening);
    httpServer.listen(port, host);
  });
}
