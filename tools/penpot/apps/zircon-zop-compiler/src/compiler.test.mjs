import assert from 'node:assert/strict';
import fs from 'node:fs';
import { spawnSync } from 'node:child_process';
import test from 'node:test';
import { compileBundle, sha256 } from './compiler.mjs';
import { parseRoleContract, validateAnnotation } from './contract.mjs';
import { generatedPath, planApply } from './apply-plan.mjs';
import { approvedArtifactRoot } from './cli.mjs';
import { fixtureBundle, screenFixture, node } from './fixture.mjs';

const source = fs.readFileSync(new URL('../../../../../zircon_editor/assets/ui/operator/contract/roles.contract.toml', import.meta.url), 'utf8');
const contract = parseRoleContract(source);
const decode = bytes => new TextDecoder().decode(bytes);

test('canonical role contract, deterministic output, golden tokens and role-based command routes', async () => {
  assert.equal(Object.keys(contract.roles).length, 17);
  const bundle = await fixtureBundle(), first = compileBundle(bundle, source), second = compileBundle(bundle, source);
  assert.deepEqual(first.files, second.files);
  assert.equal(decode(first.files['theme/zop_tokens.zui']), fs.readFileSync(new URL('fixtures/theme.golden.zui', import.meta.url), 'utf8'));
  const doc = first.documents['shell/project_browser.zui'];
  assert.equal(doc.nodes.n_open.events[0].action.action, 'file.project.open');
  assert.equal(doc.nodes.n_projects.component, 'Slot');
  assert.equal(doc.nodes.n_root.children[0].slot.layout.linear_size.rule, 'Stretch');
  assert.equal(doc.nodes.n_root.children[1].slot.layout.linear_size.rule, 'Auto');
  const roles = JSON.parse(decode(first.files['shell/project_browser.roles.json']));
  assert.equal(roles.bindings.find(binding => binding.role === 'project.browser.open').control_id, 'zop_open');
  for (const [path, hash] of Object.entries(first.lock.outputs)) assert.equal(sha256(first.files[path]), hash);
});

test('manifest integrity, extra entries, versions and traversal are rejected', async () => {
  const bundle = await fixtureBundle();
  const corrupt = structuredClone(bundle); corrupt.entries['tokens.dtcg.json'][0] ^= 1;
  assert.throws(() => compileBundle(corrupt, source), /corrupt entry/);
  const extra = structuredClone(bundle); extra.entries['unmanifested.txt'] = new Uint8Array();
  assert.throws(() => compileBundle(extra, source), /unmanifested/);
  const version = structuredClone(bundle); version.manifest.compiler_version = '99';
  assert.throws(() => compileBundle(version, source), /version/);
  const traversal = structuredClone(bundle); traversal.manifest.entries['../escape'] = 'a'.repeat(64);
  assert.throws(() => compileBundle(traversal, source), /unsafe/);
});

for (const [name, mutate, message] of [
  ['missing role', screen => screen.root.children.pop(), /missing required roles/],
  ['duplicate role', screen => { const copy = structuredClone(screen.root.children[1]); copy.id = 'copy'; screen.root.children.push(copy); }, /duplicate role/],
  ['unknown role', screen => { screen.root.binding.role = 'project.browser.unknown'; }, /unknown role/],
  ['unknown command', screen => { screen.root.children[1].binding.command = 'fake.queued'; }, /command/],
  ['unannotated interactive', screen => { screen.root.children[1].binding = {}; }, /interactive component/],
  ['shadow', screen => { screen.root.shadows = [{}]; }, /shadows/],
  ['blur', screen => { screen.root.blur = {}; }, /blur/],
  ['multiple fills', screen => { screen.root.fills = [{}, {}]; }, /fills/],
  ['nonuniform grid', screen => { screen.root.layout.kind = 'grid'; screen.root.layout.tracks = [1, 2]; }, /nonuniform/],
  ['unsupported alignment', screen => { screen.root.layout.justify = 'space-between'; }, /alignment/],
  ['absolute flex child', screen => { screen.root.children[0].absolute = true; }, /free positioning/],
  ['foreign component', screen => { screen.root.children[1].component_ref = 'other/Button'; }, /nonlibrary/],
  ['external image', screen => { screen.root.image = 'https://example.test/image.png'; }, /image/],
  ['unsupported state', screen => { screen.root.children[1].binding.visible_when = 'play.mode == Edit'; }, /visible_when/],
]) {
  test(`compiler rejects ${name} with a board path`, async () => {
    const screen = screenFixture(); mutate(screen);
    const bundle = await fixtureBundle(screen);
    assert.throws(() => compileBundle(bundle, source), error => message.test(error.message) && error.message.includes('shell/project_browser.zui'));
  });
}

for (const component of ['Label', 'Panel']) {
  test(`command roles reject noninteractive ${component} nodes`, async () => {
    const screen = screenFixture();
    screen.root.children[1].component = component;
    const bundle = await fixtureBundle(screen);
    assert.throws(() => compileBundle(bundle, source), error =>
      /command role requires an interactive/.test(error.message)
        && error.message.includes('shell/project_browser.zui'));
  });
}

test('preview nodes are excluded by default; a required role cannot hide behind preview', async () => {
  const screen = screenFixture();
  const preview = node('sample', 'Label', { preview: true }, { kind: 'leaf', width: { mode: 'hug' }, height: { mode: 'hug' } });
  screen.root.children.push(preview);
  const bundle = await fixtureBundle(screen);
  assert.equal(compileBundle(bundle, source).documents['shell/project_browser.zui'].nodes.n_sample, undefined);
  assert.ok(compileBundle(bundle, source, {}, { includePreview: true }).documents['shell/project_browser.zui'].nodes.n_sample);
  screen.root.children[1].binding.preview = true;
  const hiddenRequired = await fixtureBundle(screen);
  assert.throws(() => compileBundle(hiddenRequired, source), /missing required roles/);
});

test('i18n keys are validated and emitted as native text references', async () => {
  const screen = screenFixture(); screen.root.children[1].binding.text = 'i18n:command.file.project.open.label';
  const bundle = await fixtureBundle(screen);
  assert.throws(() => compileBundle(bundle, source), /missing i18n key/);
  const result = compileBundle(bundle, source, { 'command.file.project.open.label': true });
  assert.deepEqual(result.documents['shell/project_browser.zui'].nodes.n_open.props.text, { text_key: 'command.file.project.open.label', table: 'editor' });
});

test('generated documents are valid TOML; design-token references and hidden visibility survive', async () => {
  const screen = screenFixture();
  screen.root.tokens = { fill: 'editor.surface.base' };
  screen.root.children[0].visible = false;
  const generated = compileBundle(await fixtureBundle(screen), source);
  assert.equal(generated.documents['shell/project_browser.zui'].nodes.n_root.props.background_color, '$editor.surface.base');
  assert.equal(generated.documents['shell/project_browser.zui'].nodes.n_projects.props.visible, false);
  const python = process.env.ZOP_TEST_PYTHON ?? (process.platform === 'win32' ? 'python' : 'python3');
  const parsed = spawnSync(python, ['-B', '-c', `import json,sys,tomllib
documents=json.loads(sys.stdin.read())
for source in documents:
 document=tomllib.loads(source)
 assert document['asset']['version']==2
 if 'nodes' in document:
  assert document['root']['node'] in document['nodes']
`], { input: JSON.stringify(Object.entries(generated.files).filter(([path]) => path.endsWith('.zui')).map(([, bytes]) => decode(bytes))), encoding: 'utf8' });
  assert.equal(parsed.status, 0, parsed.stderr || String(parsed.error));
});

test('apply preserves handwritten files, refuses modified/unowned output and retires only old owned output', async () => {
  const generated = compileBundle(await fixtureBundle(), source).files;
  const handwritten = new TextEncoder().encode(source);
  const initial = planApply({ 'contract/roles.contract.toml': handwritten }, generated);
  assert.equal(initial.writes['contract/roles.contract.toml'], undefined);
  const existing = { ...generated, 'contract/roles.contract.toml': handwritten };
  assert.deepEqual(planApply(existing, generated).deletes, []);
  const modified = { ...existing, 'shell/project_browser.zui': new Uint8Array() };
  assert.throws(() => planApply(modified, generated), /modified/);
  assert.throws(() => planApply({ 'shell/project_browser.zui': generated['shell/project_browser.zui'] }, generated), /unowned/);
  const next = { ...generated };
  delete next['shell/project_browser.zui']; delete next['shell/project_browser.roles.json'];
  const lock = JSON.parse(decode(next['zop-bundle.lock.json']));
  delete lock.outputs['shell/project_browser.zui']; delete lock.outputs['shell/project_browser.roles.json'];
  next['zop-bundle.lock.json'] = new TextEncoder().encode(JSON.stringify(lock));
  assert.deepEqual(planApply(existing, next).deletes.sort(), ['shell/project_browser.roles.json', 'shell/project_browser.zui']);
  for (const path of ['contract/roles.contract.toml', '../outside.zui', 'shell/../../config.zui']) assert.throws(() => generatedPath(path));
});

test('artifact writes reject unsupported storage roots without creating files', () => {
  for (const root of ['/tmp/zop', 'C:\\cargo-targets\\zop', 'E:\\targets\\zop', 'D:\\ZirconBuilds\\zop', 'F:\\nested\\cargo-targets\\zop', 'E:/cargo-targets/zop', 'E:\\cargo-targets\\..\\other']) {
    assert.throws(() => approvedArtifactRoot(root), /physical drive-root/);
  }
});

test('contract rejects duplicate keys and mismatched kinds', () => {
  assert.throws(() => parseRoleContract(source + '\nslot = "wrong"\nslot = "again"\n'), /duplicate/);
  assert.throws(() => parseRoleContract(source.replace('kind = "command"', 'kind = "slot"')), /inconsistent/);
  assert.throws(() => validateAnnotation({ role: 'project.browser.open', command: 'not.registered' }, contract, 'project', 'board/button'), /command/);
});
