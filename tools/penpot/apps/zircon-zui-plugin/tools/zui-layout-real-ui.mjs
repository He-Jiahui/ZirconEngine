import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { dirname, relative, resolve, sep } from 'node:path';
import { requireRealSessionDirectory, sha256 } from './zui-layout-real-session-contract.mjs';

export function verifyUiBytes(expected, bytes, label) {
  assert.equal(sha256(bytes), expected, `Current plugin UI bytes differ: ${label}`);
}

export function verifyUiSourceHashes(expected, current) {
  assert.ok(Object.keys(expected).length, 'The UI build must identify its source inputs');
  for (const [path, hash] of Object.entries(expected)) {
    assert.equal(current[path], hash, `Plugin UI source changed since compilation: ${path}`);
  }
}

export function uiIndexAssetNames(html, outputs) {
  const registered = new Set(outputs.map((output) => output.relative.replaceAll('\\', '/')));
  const assets = [...html.matchAll(/<(?:script|link)\b[^>]*\b(?:src|href)=["']([^"']+)["'][^>]*>/gi)]
    .map((match) => match[1]).filter((name) => /\.(?:js|css)(?:[?#]|$)/i.test(name));
  assert.ok(assets.some((name) => name.endsWith('.js')), 'The UI index must load compiled JavaScript');
  for (const name of assets) {
    assert.ok(!name.startsWith('/') && !name.includes(':') && !name.split('/').includes('..'),
      `The UI index must use local compiled assets: ${name}`);
    assert.ok(registered.has(name), `The UI index references an unregistered build asset: ${name}`);
  }
  return [...new Set(assets)];
}

/** Bind Angular source and output identity to bytes from the actual plugin HTTP server. */
export async function verifyCurrentPluginUi({ buildReceiptPath, repositoryRoot, dist, host, indexBytes, get }) {
  requireRealSessionDirectory(dirname(buildReceiptPath));
  requireRealSessionDirectory(dist);
  const receiptBytes = await readFile(buildReceiptPath);
  const build = JSON.parse(receiptBytes.toString('utf8'));
  assert.equal(build.schema, 'actual-checkout-angular-ui-production/v1');
  assert.equal(build.status, 'compiled-current');
  assert.equal(build.aot, true);
  assert.equal(build.sourceUnchanged, true);
  assert.equal(build.sourceCopiesCreated, false);
  assert.equal(resolve(build.sourceRoot), resolve(repositoryRoot));
  assert.equal(resolve(build.workspaceRoot), resolve(repositoryRoot, 'tools/penpot'));
  const component = resolve(build.workspaceRoot, 'apps/zircon-zui-plugin/src/app/app.component.ts');
  const main = resolve(build.workspaceRoot, 'apps/zircon-zui-plugin/src/main.ts');
  assert.ok(build.sourceInputs?.[component], 'The UI receipt must identify the actual Angular component');
  assert.ok(build.sourceInputs?.[main], 'The UI receipt must identify the actual Angular entry');
  const inputs = { ...build.sourceInputs };
  for (const input of build.compiledInputs) {
    if (inputs[input.path]) assert.equal(inputs[input.path], input.sha256);
    inputs[input.path] = input.sha256;
  }
  const current = Object.fromEntries(await Promise.all(Object.keys(inputs).map(async (path) =>
    [path, sha256(await readFile(path))])));
  verifyUiSourceHashes(inputs, current);
  const references = uiIndexAssetNames(indexBytes.toString('utf8'), build.outputs);
  const index = build.outputs.find((output) => output.relative === 'index.html');
  assert.ok(index, 'The UI build must identify its index.html');
  verifyUiBytes(index.sha256, indexBytes, 'HTTP index.html');
  const httpAssets = [];
  for (const output of build.outputs) {
    requireRealSessionDirectory(dirname(output.path));
    verifyUiBytes(output.sha256, await readFile(output.path), output.path);
    const name = output.relative.replaceAll('\\', '/');
    // Angular copies a source worker asset. The independent worker compiler owns the served worker.
    if (name === 'assets/plugin.js') continue;
    const served = resolve(dist, name);
    assert.ok(!relative(dist, served).startsWith(`..${sep}`) && relative(dist, served) !== '..');
    verifyUiBytes(output.sha256, await readFile(served), served);
    if (!/\.(?:js|css|svg|woff2?)$/i.test(name)) continue;
    const response = await get(new URL(name, host).href);
    assert.equal(response.status(), 200, `The current UI asset must be served: ${name}`);
    verifyUiBytes(output.sha256, await response.body(), `HTTP ${name}`);
    httpAssets.push({ relative: name, sha256: output.sha256 });
  }
  return { schema: 'actual-plugin-ui-http-identity/v1', buildReceiptPath,
    buildReceiptSha256: sha256(receiptBytes), sourceInputsSha256: sha256(JSON.stringify(inputs)),
    sourceInputCount: Object.keys(inputs).length, sourceUnchanged: true,
    indexSha256: index.sha256, indexReferences: references, httpAssets,
    currentUiHttpVerified: true, workerIdentityVerifiedSeparately: true };
}

/** Validate the real strict worker receipt against current inputs; byte equality alone is insufficient. */
export function verifyPluginWorkerBuildReceipt(build, { repositoryRoot, bundleSha256, currentInputs }) {
  assert.equal(build.status, 'compiled-current');
  assert.equal(build.typescriptNoEmitPassed, true, 'Worker needs its strict TypeScript receipt');
  assert.equal(build.sourceUnchanged, true);
  assert.equal(build.sourceCopiesCreated, false);
  assert.ok(Array.isArray(build.inputs) && build.inputs.length, 'Worker build needs its complete recorded input closure');
  const entry = resolve(repositoryRoot, 'tools/penpot/apps/zircon-zui-plugin/src/plugin.ts');
  assert.ok(build.inputs.some(input => resolve(input.path) === entry), 'Worker receipt must name the actual checkout entry');
  const seen = new Set();
  for (const input of build.inputs) {
    const path = resolve(input.path);
    assert.ok(!seen.has(path), 'Worker receipt has duplicate input paths');
    seen.add(path);
    assert.match(input.sha256, /^[0-9a-f]{64}$/);
    assert.equal(currentInputs[input.path], input.sha256, 'Worker input changed since compilation: ' + input.path);
  }
  assert.equal(build.output?.sha256, bundleSha256, 'Worker build receipt differs from the selected bundle');
  return { inputCount: build.inputs.length, sourceUnchanged: true, workerSha256: bundleSha256 };
}

export async function verifyCurrentPluginWorkerBuild({
  buildReceiptPath, buildReceiptSha256, repositoryRoot, bundleBytes,
}) {
  requireRealSessionDirectory(dirname(buildReceiptPath));
  assert.match(buildReceiptSha256, /^[0-9a-f]{64}$/);
  const bytes = await readFile(buildReceiptPath);
  assert.equal(sha256(bytes), buildReceiptSha256, 'Pinned worker compile receipt changed');
  const build = JSON.parse(bytes.toString('utf8'));
  assert.ok(Array.isArray(build.inputs), 'Worker compile receipt has no input list');
  const currentInputs = Object.fromEntries(await Promise.all(build.inputs.map(async input =>
    [input.path, sha256(await readFile(input.path))])));
  const verified = verifyPluginWorkerBuildReceipt(build, {
    repositoryRoot, bundleSha256: sha256(bundleBytes), currentInputs,
  });
  requireRealSessionDirectory(dirname(build.output.path));
  assert.equal(sha256(await readFile(build.output.path)), verified.workerSha256, 'Compiled worker output changed');
  return { schema: 'actual-plugin-worker-build-closure/v1', buildReceiptPath,
    buildReceiptSha256, ...verified };
}

/** Bind the selected worker and local distribution to the actual reused HTTP host. */
export async function verifyCurrentPluginWorker({ host, bundleBytes, distributionWorkerBytes, get }) {
  const origin = new URL(host);
  assert.ok(['http:', 'https:'].includes(origin.protocol) &&
    ['127.0.0.1', 'localhost', '[::1]'].includes(origin.hostname) &&
    origin.pathname === '/' && !origin.search && !origin.hash && !origin.username && !origin.password,
  'Worker verification requires the exact local plugin host root');
  const expected = sha256(bundleBytes);
  assert.equal(sha256(distributionWorkerBytes), expected,
    'Selected distribution worker differs from the independent plugin bundle');
  const workerUrl = new URL('assets/plugin.js', origin).href;
  const response = await get(workerUrl);
  assert.equal(response.status(), 200, 'The managed plugin host must serve its selected worker');
  assert.equal(response.url(), workerUrl, 'The managed plugin worker response redirected');
  const served = await response.body();
  assert.equal(sha256(served), expected, 'Managed HTTP worker differs from the selected independent plugin bundle');
  return { schema: 'actual-plugin-worker-http-identity/v1', workerUrl,
    workerSha256: expected, workerByteLength: bundleBytes.byteLength,
    localDistributionVerified: true, httpWorkerVerified: true };
}
