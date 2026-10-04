import { createHash, randomBytes } from "node:crypto";
import { createServer } from "node:http";

export async function request(url, options = {}) {
  const response = await fetch(url, { redirect: "manual", signal: AbortSignal.timeout(15000), ...options });
  const bytes = Buffer.from(await response.arrayBuffer());
  let data = null;
  try { data = JSON.parse(bytes.toString("utf8")); } catch {}
  return { status: response.status, data, bytes, headers: response.headers };
}

const form = (url, fields) => request(url, {
  method: "POST", headers: { "content-type": "application/x-www-form-urlencoded" },
  body: new URLSearchParams(fields),
});

function requireStatus(response, status, action) {
  if (response.status !== status) throw new Error(`${action}: HTTP ${response.status}`);
  return response;
}

export class OidcProbeClient {
  constructor(issuer, clientId, callbackPort, browser) {
    this.issuer = issuer;
    this.clientId = clientId;
    this.callbackPort = callbackPort;
    this.browser = browser;
    this.users = [];
    this.sessions = [];
  }

  async initialize(credentials) {
    const metadata = requireStatus(await request(`${this.issuer}/.well-known/openid-configuration`), 200, "OIDC discovery");
    if (metadata.data.issuer !== this.issuer) throw new Error("OIDC issuer mismatch");
    this.metadata = metadata.data;
    const origin = new URL(this.issuer).origin;
    for (const name of ["authorization_endpoint", "token_endpoint", "end_session_endpoint"]) {
      if (new URL(this.metadata[name]).origin !== origin) throw new Error("OIDC endpoint origin mismatch");
    }
    const admin = requireStatus(await form(`${origin}/realms/master/protocol/openid-connect/token`, {
      grant_type: "password", client_id: "admin-cli", username: credentials.adminUsername, password: credentials.adminPassword,
    }), 200, "isolated realm administration");
    this.adminUrl = `${origin}/admin${new URL(this.issuer).pathname}`;
    this.adminHeaders = { authorization: `Bearer ${admin.data.access_token}`, "content-type": "application/json" };
  }

  async createUser(label) {
    const user = { username: `astra-probe-${label}-${randomBytes(8).toString("hex")}`, password: randomBytes(32).toString("base64url") };
    const created = requireStatus(await request(`${this.adminUrl}/users`, {
      method: "POST", headers: this.adminHeaders,
      body: JSON.stringify({ username: user.username, firstName: "Astra", lastName: "Probe", email: `${user.username}@example.invalid`, emailVerified: true, enabled: true, credentials: [{ type: "password", value: user.password, temporary: false }] }),
    }), 201, "synthetic realm user creation");
    user.subject = created.headers.get("location").split("/").at(-1);
    this.users.push(user);
    return user;
  }

  async login(user, screenshotPath) {
    const redirect = `http://127.0.0.1:${this.callbackPort}/callback`;
    const verifier = randomBytes(48).toString("base64url");
    const state = randomBytes(32).toString("base64url");
    const nonce = randomBytes(32).toString("base64url");
    const authorize = new URL(this.metadata.authorization_endpoint);
    authorize.search = new URLSearchParams({ client_id: this.clientId, redirect_uri: redirect, response_type: "code", scope: "openid profile", state, nonce, code_challenge: createHash("sha256").update(verifier).digest("base64url"), code_challenge_method: "S256" });
    let resolveCallback;
    const callback = new Promise(resolve => { resolveCallback = resolve; });
    const server = createServer((incoming, response) => {
      const url = new URL(incoming.url, redirect);
      const accepted = url.pathname === "/callback" && url.searchParams.get("state") === state && url.searchParams.has("code");
      response.writeHead(accepted ? 200 : 400, { "content-type": "text/plain", "cache-control": "no-store" });
      response.end(accepted ? "Signed in" : "Callback rejected");
      if (accepted) resolveCallback(url.searchParams.get("code"));
    });
    const context = await this.browser.newContext({ viewport: { width: 1280, height: 900 } });
    try {
      await new Promise((resolve, reject) => { server.once("error", reject); server.listen(this.callbackPort, "127.0.0.1", resolve); });
      const page = await context.newPage();
      await page.goto(authorize.toString());
      await page.locator("#username").waitFor();
      if (screenshotPath) await page.screenshot({ path: screenshotPath, fullPage: true });
      await page.locator("#username").fill(user.username);
      await page.locator("#password").fill(user.password);
      await page.locator("#kc-login").click();
      let timeout;
      const code = await Promise.race([callback, new Promise((_, reject) => { timeout = setTimeout(() => reject(new Error("PKCE callback timed out")), 15000); })]).finally(() => clearTimeout(timeout));
      const result = requireStatus(await form(this.metadata.token_endpoint, { grant_type: "authorization_code", client_id: this.clientId, redirect_uri: redirect, code, code_verifier: verifier }), 200, "PKCE code exchange");
      const session = { user, accessToken: result.data.access_token, refreshToken: result.data.refresh_token };
      if (!session.accessToken || !session.refreshToken) throw new Error("OIDC token response incomplete");
      this.sessions.push(session);
      return session;
    } finally {
      await context.close();
      server.closeAllConnections();
      await new Promise(resolve => server.close(resolve));
    }
  }

  async refresh(session) {
    const result = requireStatus(await form(this.metadata.token_endpoint, { grant_type: "refresh_token", client_id: this.clientId, refresh_token: session.refreshToken }), 200, "OIDC refresh");
    session.accessToken = result.data.access_token;
    session.refreshToken = result.data.refresh_token;
  }

  async logout(session) {
    const result = await form(this.metadata.end_session_endpoint, { client_id: this.clientId, refresh_token: session.refreshToken });
    if (![200, 204].includes(result.status)) throw new Error(`OIDC logout: HTTP ${result.status}`);
    this.sessions = this.sessions.filter(item => item !== session);
  }

  async close() {
    const failures = [];
    for (const session of [...this.sessions]) await this.logout(session).catch(() => failures.push("logout"));
    for (const user of this.users) {
      const removed = await request(`${this.adminUrl}/users/${user.subject}`, { method: "DELETE", headers: this.adminHeaders }).catch(() => null);
      if (removed?.status !== 204) failures.push("synthetic-user-delete");
    }
    return failures;
  }
}
