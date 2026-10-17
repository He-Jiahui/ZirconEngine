import assert from 'node:assert/strict';
import test from 'node:test';
import { sha256 } from './zui-layout-real-session-contract.mjs';
import { uiIndexAssetNames, verifyUiBytes, verifyUiSourceHashes } from './zui-layout-real-ui.mjs';

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
