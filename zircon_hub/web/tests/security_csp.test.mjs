import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const configUrl = new URL("../../tauri.conf.json", import.meta.url);

test("Hub production config carries a restrictive non-null CSP", async () => {
  const config = JSON.parse(await readFile(configUrl, "utf8"));
  const csp = config?.app?.security?.csp;

  assert.equal(typeof csp, "string");
  assert.notEqual(csp.trim(), "");
  assert.match(csp, /default-src\s+'self'/);
  assert.match(csp, /connect-src[^;]*\bipc:/);
  assert.match(csp, /connect-src[^;]*http:\/\/ipc\.localhost/);
  assert.match(csp, /img-src[^;]*\basset:/);
  assert.match(csp, /style-src[^;]*'unsafe-inline'/);
  assert.match(csp, /object-src\s+'none'/);
  assert.match(csp, /base-uri\s+'self'/);
  assert.match(csp, /frame-ancestors\s+'none'/);
  assert.doesNotMatch(csp, /script-src[^;]*\*/);
  assert.doesNotMatch(csp, /script-src[^;]*unsafe-eval/);
});

test("Hub CSP does not grant wildcard or remote script execution", async () => {
  const config = JSON.parse(await readFile(configUrl, "utf8"));
  const csp = config.app.security.csp;
  const devCsp = config.app.security.devCsp;

  assert.doesNotMatch(csp, /(?:^|[;\s])\*(?:[;\s]|$)/);
  assert.doesNotMatch(csp, /script-src[^;]*(?:https?:|data:|blob:)/);
  assert.doesNotMatch(csp, /(?:default-src|connect-src|img-src|font-src)[^;]*\bhttps?:\/\/[^\s;*]+\/(?:\*|$)/);
  assert.equal(typeof devCsp, "string");
  assert.match(devCsp, /connect-src[^;]*ws:\/\/localhost:24678/);
  assert.match(devCsp, /script-src\s+'self'\s+'unsafe-inline'/);
  assert.doesNotMatch(devCsp, /script-src[^;]*unsafe-eval/);
  assert.doesNotMatch(devCsp, /script-src[^;]*(?:https?:|data:|blob:)/);
});
