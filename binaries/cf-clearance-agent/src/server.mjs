import http from "node:http";

const CLEARANCE_UNAVAILABLE = {
  error: {
    code: "CLEARANCE_UNAVAILABLE",
    message: "No verified Cloudflare identity is available",
    retryable: true,
  },
};

const ROUTE_METHODS = new Map([
  ["/healthz", "GET"],
  ["/readyz", "GET"],
  ["/v1/snapshot", "GET"],
  ["/v1/refresh", "POST"],
]);

export function createAgentServer({
  state,
  bodyLimitBytes = 4096,
  logger = null,
}) {
  if (!state) {
    throw new TypeError("state is required");
  }
  return http.createServer((request, response) => {
    handleRequest({ request, response, state, bodyLimitBytes, logger }).catch(
      (error) => {
        const pathname = new URL(
          request.url,
          "http://agent.invalid",
        ).pathname;
        logger?.error?.("http_request_failed", {
          error_code: error?.code ?? "INTERNAL_ERROR",
          method: request.method,
          path: pathname,
        });
        if (!response.headersSent) {
          sendError(
            response,
            500,
            "INTERNAL_ERROR",
            "Internal server error",
            true,
          );
        } else {
          response.destroy();
        }
      },
    );
  });
}

async function handleRequest({
  request,
  response,
  state,
  bodyLimitBytes,
}) {
  const pathname = new URL(request.url, "http://agent.invalid").pathname;
  const allowedMethod = ROUTE_METHODS.get(pathname);
  if (!allowedMethod) {
    sendError(response, 404, "NOT_FOUND", "Route not found", false);
    return;
  }
  if (request.method !== allowedMethod) {
    response.setHeader("allow", allowedMethod);
    sendError(
      response,
      405,
      "METHOD_NOT_ALLOWED",
      "Method not allowed",
      false,
    );
    return;
  }

  if (pathname === "/healthz") {
    sendJson(response, 200, { status: "ok" });
    return;
  }

  const snapshot = state.getSnapshot();
  if (pathname === "/readyz") {
    const ready = state.status === "ready" && snapshot !== null;
    sendJson(response, ready ? 200 : 503, {
      state: state.status,
      generation: state.generation,
    });
    return;
  }
  if (pathname === "/v1/snapshot") {
    if (!snapshot) {
      sendJson(response, 503, CLEARANCE_UNAVAILABLE);
      return;
    }
    sendJson(response, 200, snapshot);
    return;
  }

  let input;
  try {
    input = await readRefreshRequest(request, bodyLimitBytes);
  } catch (error) {
    if (error instanceof RequestBodyError) {
      sendError(response, error.status, error.code, error.message, false);
      return;
    }
    throw error;
  }

  try {
    sendJson(response, 200, await state.refresh(input));
  } catch {
    sendJson(response, 503, CLEARANCE_UNAVAILABLE);
  }
}

async function readRefreshRequest(request, bodyLimitBytes) {
  const contentLength = Number(request.headers["content-length"]);
  if (Number.isFinite(contentLength) && contentLength > bodyLimitBytes) {
    request.resume();
    throw new RequestBodyError(
      413,
      "REQUEST_TOO_LARGE",
      "Request body is too large",
    );
  }

  const chunks = [];
  let size = 0;
  for await (const chunk of request) {
    size += chunk.length;
    if (size > bodyLimitBytes) {
      throw new RequestBodyError(
        413,
        "REQUEST_TOO_LARGE",
        "Request body is too large",
      );
    }
    chunks.push(chunk);
  }

  let body;
  try {
    body = JSON.parse(Buffer.concat(chunks).toString("utf8"));
  } catch {
    throw new RequestBodyError(
      400,
      "INVALID_REQUEST",
      "Request body must be valid JSON",
    );
  }
  validateRefreshRequest(body);
  return body;
}

function validateRefreshRequest(body) {
  const validKeys =
    body &&
    typeof body === "object" &&
    !Array.isArray(body) &&
    Object.keys(body).every((key) =>
      ["observed_generation", "reason"].includes(key),
    );
  if (
    !validKeys ||
    !Number.isSafeInteger(body.observed_generation) ||
    body.observed_generation < 0 ||
    typeof body.reason !== "string" ||
    body.reason.length === 0 ||
    body.reason.length > 128
  ) {
    throw new RequestBodyError(
      400,
      "INVALID_REQUEST",
      "Request body does not match the refresh contract",
    );
  }
}

function sendError(response, status, code, message, retryable) {
  sendJson(response, status, {
    error: { code, message, retryable },
  });
}

function sendJson(response, status, body) {
  const payload = JSON.stringify(body);
  response.writeHead(status, {
    "cache-control": "no-store",
    "content-length": Buffer.byteLength(payload),
    "content-type": "application/json; charset=utf-8",
  });
  response.end(payload);
}

class RequestBodyError extends Error {
  constructor(status, code, message) {
    super(message);
    this.status = status;
    this.code = code;
  }
}
