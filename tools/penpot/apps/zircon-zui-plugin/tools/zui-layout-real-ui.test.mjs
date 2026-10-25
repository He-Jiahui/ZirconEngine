import assert from 'node:assert/strict';
import test from 'node:test';
import { resolve } from 'node:path';
import { sha256 } from './zui-layout-real-session-contract.mjs';
import { uiIndexAssetNames, verifyCurrentPluginWorker, verifyPluginWorkerBuildReceipt, verifyUiBytes, verifyUiSourceHashes } from './zui-layout-real-ui.mjs';

function workerBuildFixture() {
  const repositoryRoot = 'E:/Git/ZirconEngine';
  const entry = resolve(repositoryRoot, 'tools/penpot/apps/zircon-zui-plugin/src/plugin.ts');
  const dependency = resolve(repositoryRoot, 'tools/penpot/apps/zircon-zui-plugin/src/penpot-native-components.ts');
  const inputs = [{ path: entry, sha256: sha256('compiled entry') }, { path: dependency, sha256: sha256('compiled dependency') }];
  const build = { status: 'compiled-current', typescriptNoEmitPassed: true, sourceUnchanged: true,
    sourceCopiesCreated: false, inputs, output: { sha256: sha256('independent selected worker') } };
  return { build, options: { repositoryRoot, bundleSha256: sha256('independent selected worker'),
    currentInputs: Object.fromEntries(inputs.map(input => [input.path, input.sha256])) } };
}

test('matching served bytes require the actual strict worker source closure as well', () => {
  const { build, options } = workerBuildFixture();
  assert.equal(verifyPluginWorkerBuildReceipt(build, options).inputCount, 2);
  options.currentInputs[build.inputs[1].path] = sha256('later uncompiled dependency');
  assert.throws(() => verifyPluginWorkerBuildReceipt(build, options), /input changed since compilation/);
});

test('worker receipt rejects missing inputs, another source entry and a different selected bundle', () => {
  const { build, options } = workerBuildFixture();
  assert.throws(() => verifyPluginWorkerBuildReceipt(build, { ...options, currentInputs: {} }), /input changed/);
  assert.throws(() => verifyPluginWorkerBuildReceipt({ ...build, inputs: build.inputs.slice(1) }, options), /actual checkout entry/);
  assert.throws(() => verifyPluginWorkerBuildReceipt(build, { ...options, bundleSha256: sha256('another worker') }), /selected bundle/);
});

test('unchecked compilation, source copies and ambiguous input lists cannot seal a worker', () => {
  const { build, options } = workerBuildFixture();
  for (const change of [{ typescriptNoEmitPassed: false }, { sourceUnchanged: false },
    { sourceCopiesCreated: true }, { inputs: [...build.inputs, build.inputs[0]] }])
    assert.throws(() => verifyPluginWorkerBuildReceipt({ ...build, ...change }, options));
});

const outputs = ['main-current.js', 'polyfills-current.js', 'styles-current.css']
  .map((relative) => ({ relative }));

test('current Angular index binds its main, polyfill and stylesheet products', () => {
  assert.deepEqual(uiIndexAssetNames('<link rel="stylesheet" href="styles-current.css">' +
    '<script src="polyfills-current.js" type="module"></script><script src="main-current.js"></script>', outputs),
  ['styles-current.css', 'polyfills-current.js', 'main-current.js']);
});

test('an old UI index is rejected even when the independent worker is current', () => {
  assert.throws(() => uiIndexAssetNames('<script src="main-old.js"></script>', outputs),
    /unregistered build asset: main-old.js/);
});

test('old HTTP UI bytes fail despite a matching worker bundle', () => {
  assert.throws(() => verifyUiBytes(sha256('current UI with Canvas PNG'), Buffer.from('old UI'), 'HTTP main-current.js'),
    /Current plugin UI bytes differ: HTTP main-current.js/);
});

test('a changed Angular component invalidates an otherwise unchanged UI build', () => {
  assert.throws(() => verifyUiSourceHashes({ 'app.component.ts': sha256('old component') },
    { 'app.component.ts': sha256('new component with PNG button') }), /source changed since compilation/);
});

test('a missing source input cannot be treated as unchanged', () => {
  assert.throws(() => verifyUiSourceHashes({ 'app.component.ts': sha256('component') }, {}),
    /source changed since compilation/);
});

test('an index without current local JavaScript cannot establish UI identity', () => {
  assert.throws(() => uiIndexAssetNames('<link href="styles-current.css">', outputs), /load compiled JavaScript/);
  assert.throws(() => uiIndexAssetNames('<script src="https://elsewhere/main-current.js"></script>', outputs),
    /local compiled assets/);
});

const workerHost = 'http://127.0.0.1:19002/';
const workerUrl = workerHost + 'assets/plugin.js';
const workerBytes = Buffer.from('independent selected worker');
const workerResponse = (bytes, { status = 200, url = workerUrl } = {}) =>
  ({ status: () => status, url: () => url, body: async () => bytes });

test('reused HTTP host verifies the exact selected worker and distribution bytes', async () => {
  const calls = [];
  const evidence = await verifyCurrentPluginWorker({ host: workerHost, bundleBytes: workerBytes,
    distributionWorkerBytes: workerBytes, get: async url => {
      calls.push(url); return workerResponse(workerBytes);
    } });
  assert.deepEqual(calls, [workerUrl]);
  assert.equal(evidence.workerSha256, sha256(workerBytes));
  assert.equal(evidence.workerByteLength, workerBytes.byteLength);
  assert.equal(evidence.localDistributionVerified, true);
  assert.equal(evidence.httpWorkerVerified, true);
});

test('matching manifest and UI cannot admit a stale HTTP worker', async () => {
  await assert.rejects(verifyCurrentPluginWorker({ host: workerHost, bundleBytes: workerBytes,
    distributionWorkerBytes: workerBytes,
    get: async () => workerResponse(Buffer.from('older worker under the same manifest')) }),
  /Managed HTTP worker differs/);
});

test('a stale distribution is rejected before requesting the HTTP worker', async () => {
  let calls = 0;
  await assert.rejects(verifyCurrentPluginWorker({ host: workerHost, bundleBytes: workerBytes,
    distributionWorkerBytes: Buffer.from('older local distribution'),
    get: async () => { calls++; return workerResponse(workerBytes); } }),
  /Selected distribution worker differs/);
  assert.equal(calls, 0);
});

test('HTTP failures and redirected worker responses cannot establish managed identity', async () => {
  for (const status of [302, 404, 500]) {
    await assert.rejects(verifyCurrentPluginWorker({ host: workerHost, bundleBytes: workerBytes,
      distributionWorkerBytes: workerBytes, get: async () => workerResponse(workerBytes, { status }) }),
    /must serve its selected worker/);
  }
  await assert.rejects(verifyCurrentPluginWorker({ host: workerHost, bundleBytes: workerBytes,
    distributionWorkerBytes: workerBytes,
    get: async () => workerResponse(workerBytes, { url: 'http://127.0.0.1:19003/assets/plugin.js' }) }),
  /worker response redirected/);
});

test('worker identity cannot use another host or a decorated plugin root', async () => {
  let calls = 0;
  for (const host of ['https://example.com/', 'http://user@127.0.0.1:19002/',
    workerHost + '?build=other', workerHost + '#other', workerHost + 'other/']) {
    await assert.rejects(verifyCurrentPluginWorker({ host, bundleBytes: workerBytes,
      distributionWorkerBytes: workerBytes, get: async () => { calls++; return workerResponse(workerBytes); } }),
    /exact local plugin host root/);
  }
  assert.equal(calls, 0);
});
