const SENSITIVE_FIELD = /(?:authorization|cookie|profile|secret|token|value)/i;

export function createLogger({
  sink = (line) => process.stdout.write(`${line}\n`),
  clock = Date.now,
} = {}) {
  const emit = (level, event, fields = {}) => {
    const entry = {
      ...sanitize(fields),
      timestamp: new Date(clock()).toISOString(),
      level,
      event,
    };
    sink(JSON.stringify(entry));
  };

  return {
    debug: (event, fields) => emit("debug", event, fields),
    info: (event, fields) => emit("info", event, fields),
    warn: (event, fields) => emit("warn", event, fields),
    error: (event, fields) => emit("error", event, fields),
  };
}

function sanitize(value, key = "") {
  if (SENSITIVE_FIELD.test(key)) {
    return "[REDACTED]";
  }
  if (Array.isArray(value)) {
    return value.map((item) => sanitize(item));
  }
  if (value && typeof value === "object") {
    return Object.fromEntries(
      Object.entries(value).map(([childKey, childValue]) => [
        childKey,
        sanitize(childValue, childKey),
      ]),
    );
  }
  return value;
}
