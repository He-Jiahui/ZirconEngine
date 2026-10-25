import assert from 'node:assert/strict';
import fs from 'node:fs';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { createBundle, zipBundle } from './bundle.mjs';
import { sha256, compileBundle } from './compiler.mjs';
import { parseRoleContract, stableJson } from './contract.mjs';
import { normalizeBoard, captureTokens } from '../../zircon-zui-plugin/src/zop/normalizer.mjs';

const source = fs.readFileSync(new URL('../../../../../zircon_editor/assets/ui/operator/contract/roles.contract.toml', import.meta.url), 'utf8');
const contract = parseRoleContract(source), encode = value => new TextEncoder().encode(stableJson(value));

function shape(id, binding, extra = {}) {
  return { id, name: id, type: 'rectangle', width: 80, height: 28, parentX: 0, parentY: 0,
    rotation: 0, opacity: 1, visible: true, borderRadius: 0, fills: [], strokes: [], shadows: [], children: [],
    getSharedPluginData: namespace => { assert.equal(namespace, 'dev.zircon.zop'); return JSON.stringify(binding); },
    isComponentRoot: () => false, ...extra };
}
function boardFixture() {
  const board = shape('board', { role: 'project.browser.root' }, { type: 'board', width: 1280, height: 720,
    flex: { dir: 'column', columnGap: 0, rowGap: 4, alignItems: 'stretch', justifyContent: 'start', wrap: 'nowrap',
      topPadding: 4, rightPadding: 4, bottomPadding: 4, leftPadding: 4, horizontalSizing: 'fix', verticalSizing: 'fix' } });
  board.children.push(shape('projects', { role: 'project.browser.projects', slot: 'recent.projects' },
    { layoutChild: { horizontalSizing: 'fill', verticalSizing: 'fill', absolute: false } }));
  board.children.push(shape('open', { role: 'project.browser.open', command: 'file.project.open', text: 'Open' }, {
    isComponentRoot: () => true, component: () => ({ path: 'zop', name: 'Button/md', libraryId: 'file-fixture' }),
  }));
  return board;
}
function catalogFixture() {
  return { sets: [{ active: true, tokens: [{ name: 'editor.surface.base', type: 'color', resolvedValue: '#151515' }] }] };
}

test('shared exporter starts without Web globals and preserves Unicode manifest and ZIP bytes', async () => {
  const original = Object.getOwnPropertyDescriptor(globalThis, 'TextEncoder');
  try {
    Object.defineProperty(globalThis, 'TextEncoder', { value: undefined, configurable: true });
    const sandboxBundle = await import('./bundle.mjs?without-web-text-encoder');
    const file = { id: '中文界面-é-😀-\ud800', revision: 7 };
    const entries = { 'screens/board.json': Uint8Array.from([0, 127, 128, 255]) };
    const sandbox = await sandboxBundle.createBundle(entries, [file], async bytes => sha256(bytes));
    assert.deepEqual(sandbox.entries['manifest.json'],
      new Uint8Array(Buffer.from(stableJson(sandbox.manifest) + '\n', 'utf8')));
    const reference = await createBundle(entries, [file], async bytes => sha256(bytes));
    assert.deepEqual(sandboxBundle.zipBundle(sandbox.entries), zipBundle(reference.entries));
  } finally {
    if (original) Object.defineProperty(globalThis, 'TextEncoder', original);
    else delete globalThis.TextEncoder;
  }
});

test('a fresh Penpot board originates a normalized bundle without imported ZUI metadata', async () => {
  const board = boardFixture();
  const screen = normalizeBoard(board, contract, 'project', 'shell/project_browser.zui', ['file-fixture']);
  const tokens = captureTokens(catalogFixture());
  const bundle = await createBundle({ 'screens/board.json': encode(screen), 'tokens.dtcg.json': encode(tokens) },
    [{ id: 'file-fixture', revision: 7 }], async bytes => sha256(bytes));
  assert.equal(bundle.manifest.files[0].revision, 7);
  const compiled = compileBundle(bundle, source);
  assert.equal(compiled.documents['shell/project_browser.zui'].nodes.n_open.events[0].action.action, 'file.project.open');
});

test('nonlibrary components, unsupported text and unresolved tokens are rejected', () => {
  const board = boardFixture();
  assert.throws(() => normalizeBoard(board, contract, 'project', 'shell/project_browser.zui', ['different-file']), /versioned zop library/);
  board.children[1].component = () => ({ path: 'foreign', name: 'Button', libraryId: 'file-fixture' });
  assert.throws(() => normalizeBoard(board, contract, 'project', 'shell/project_browser.zui', ['file-fixture']), /versioned zop/);
  const text = shape('text', {}, { type: 'text', characters: 'x', fontFamily: 'mixed' });
  board.children = [text];
  assert.throws(() => normalizeBoard(board, contract, 'project', 'shell/project_browser.zui', ['file-fixture']), /styled text/);
  const catalog = catalogFixture(); catalog.sets[0].tokens[0].resolvedValue = undefined;
  assert.throws(() => captureTokens(catalog), /unresolved/);
});

test('export ZIP is deterministic and a standard ZIP reader verifies every entry CRC and hash', async () => {
  const entries = { 'screens/board.json': encode({ test_fixture: true }),
    'tokens.dtcg.json': encode(captureTokens(catalogFixture())),
    'previews/board.png': Uint8Array.from(Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+a3ioAAAAASUVORK5CYII=', 'base64')) };
  const bundle = await createBundle(entries, [{ id: 'file-fixture', revision: 7 }], async bytes => sha256(bytes));
  const zip = zipBundle(bundle.entries);
  assert.deepEqual(zip, zipBundle(bundle.entries));
  const python = process.env.ZOP_TEST_PYTHON ?? (process.platform === 'win32' ? 'python' : 'python3');
  const standard = spawnSync(python, ['-B', '-c', `import hashlib,io,json,sys,zipfile
with zipfile.ZipFile(io.BytesIO(sys.stdin.buffer.read())) as archive:
 assert archive.testzip() is None
 manifest=json.loads(archive.read('manifest.json'))
 for name,digest in manifest['entries'].items():
  assert hashlib.sha256(archive.read(name)).hexdigest()==digest
 assert len(archive.namelist())==len(manifest['entries'])+1
`], { input: zip, encoding: 'utf8' });
  assert.equal(standard.status, 0, standard.stderr || String(standard.error));
});
