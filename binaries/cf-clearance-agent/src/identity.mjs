const CLOUDFLARE_COOKIE_NAME =
  /^(?:cf_clearance|__cf[^=]*|_cf[^=]*)$/;

export function filterCloudflareCookies(cookies) {
  return cookies
    .filter(({ name }) => CLOUDFLARE_COOKIE_NAME.test(name))
    .map(({ name, value, domain, path, expires }) => ({
      name,
      value,
      domain,
      path,
      expires,
    }));
}

export function cloudflareCookieHeader(cookies) {
  return filterCloudflareCookies(cookies)
    .map(({ name, value }) => `${name}=${value}`)
    .join("; ");
}

export function createVerifiedIdentity({ userAgent, cookies, nowMs }) {
  if (typeof userAgent !== "string" || userAgent.length === 0) {
    throw new TypeError("browser returned no user agent");
  }

  const cloudflareCookies = filterCloudflareCookies(cookies);
  const clearanceExpiries = cloudflareCookies
    .filter(({ name }) => name === "cf_clearance")
    .map(({ expires }) => expires * 1000)
    .filter(Number.isFinite);
  if (clearanceExpiries.length === 0) {
    throw new TypeError("browser returned no cf_clearance cookie");
  }

  const expiresAtMs = Math.min(...clearanceExpiries);
  if (expiresAtMs <= nowMs) {
    throw new TypeError("browser returned an expired cf_clearance cookie");
  }

  return {
    user_agent: userAgent,
    cookies: cloudflareCookies,
    expires_at: new Date(expiresAtMs).toISOString(),
    verified_at: new Date(nowMs).toISOString(),
  };
}
