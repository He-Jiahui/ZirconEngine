import assert from 'node:assert/strict';
import test from 'node:test';
import { assertExistingPenpotContentPreserved, verifySourceOwnerAppendAdmission,
  assertNativeInstanceGeometryPreserved, sourceColorRevisionSnapshot, assertSourceColorRevisionPreserved,
  persistenceClaim, persistedZuiMetadata, persistedViewportToolbarGroupMetadata, requireRealSessionDirectory,
  verifyReopenedFile, verifyReviewScrollCleanupAdmission, verifySaveResponse, workspaceIdentity, realSessionExecutionPlan } from './zui-layout-real-session-contract.mjs';

const identity = { teamId: '12345678-1111-4111-8111-123456789abc',
  fileId: '12345678-2222-4222-8222-123456789abc', pageId: '12345678-3333-4333-8333-123456789abc' };
function reuseSessionFixture() {
  const expectedWorkspace = { origin: 'http://127.0.0.1:19001', ...identity };
  const options = { expectedWorkspace, existingPlugin: {
    host: 'http://127.0.0.1:19002/', pluginId: identity.teamId,
    manifestSha256: 'a'.repeat(64), bundleSha256: 'b'.repeat(64), registryEntrySha256: 'c'.repeat(64),
  } };
  const context = {
    workspaceUrl: expectedWorkspace.origin + '/#/workspace?' +
      new URLSearchParams({ 'team-id': identity.teamId, 'file-id': identity.fileId, 'page-id': identity.pageId }),
    authenticatedStorage: true, uiBuildReceipt: true, workerBuildReceipt: true,
  };
  return { options, context };
}

test('workspace URL alone never grants bootstrap account, plugin install or host creation', () => {
  const { context } = reuseSessionFixture();
  assert.throws(() => realSessionExecutionPlan({}, context), /explicitly pinned existing plugin/);
});

test('default reuse admits the exact assigned identity without granting bootstrap side effects', () => {
  const { options, context } = reuseSessionFixture(), before = JSON.stringify(options);
  assert.deepEqual(realSessionExecutionPlan(options, context), { mode: 'reuse-existing', bootstrapAllowed: false });
  assert.equal(JSON.stringify(options), before);
});

test('reuse rejects a changed team, file, page, origin or missing managed prerequisite', () => {
  const { options, context } = reuseSessionFixture();
  for (const key of ['teamId', 'fileId', 'pageId', 'origin']) {
    const wrong = { ...options, expectedWorkspace: { ...options.expectedWorkspace,
      [key]: key === 'origin' ? 'http://127.0.0.1:19003' : 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa' } };
    assert.throws(() => realSessionExecutionPlan(wrong, context), /differs from expected/);
  }
  for (const key of ['authenticatedStorage', 'uiBuildReceipt', 'workerBuildReceipt'])
    assert.throws(() => realSessionExecutionPlan(options, { ...context, [key]: false }), /managed UI\/worker/);
  assert.throws(() => realSessionExecutionPlan(options, { ...context, workspaceUrl: undefined }));
});

test('reuse rejects unpinned bundles and nonlocal or credential-bearing hosts', () => {
  const { options, context } = reuseSessionFixture();
  for (const host of ['https://example.invalid/', 'http://127.0.0.1:19002/other',
    'http://account:password@127.0.0.1:19002/', 'http://127.0.0.1:19002/?other']) {
    assert.throws(() => realSessionExecutionPlan({ ...options, existingPlugin: { ...options.existingPlugin, host } }, context));
  }
  for (const key of ['manifestSha256', 'bundleSha256', 'registryEntrySha256'])
    assert.throws(() => realSessionExecutionPlan({ ...options, existingPlugin: { ...options.existingPlugin, [key]: '' } }, context));
});

test('only explicit managed bootstrap grants creation and cannot adopt an existing plugin or append', () => {
  const context = { authenticatedStorage: false, uiBuildReceipt: false, workerBuildReceipt: false };
  assert.deepEqual(realSessionExecutionPlan({ mode: 'managed-bootstrap' }, context),
    { mode: 'managed-bootstrap', bootstrapAllowed: true });
  const { options } = reuseSessionFixture();
  assert.throws(() => realSessionExecutionPlan({ ...options, mode: 'managed-bootstrap' }, context), /cannot adopt/);
  assert.throws(() => realSessionExecutionPlan({ mode: 'managed-bootstrap', append: {} }, context), /cannot adopt/);
  assert.throws(() => realSessionExecutionPlan({ mode: 'other' }, context), /Unknown real session/);
  assert.throws(() => realSessionExecutionPlan({ mode: 'managed-bootstrap', createHost: true }, context), /Unknown session option/);
});

const changes = [{ type: 'mod-obj' }];
const saveRequest = { id: identity.fileId, 'session-id': identity.pageId, revn: 7, changes };
const savedChanges = [{ 'file-id': identity.fileId, id: identity.teamId,
  'session-id': identity.pageId, revn: 8, changes: [{ type: 'mod-obj' }] }];
const save = { revn: 7, lagged: savedChanges };

const pageRoot = '00000000-0000-0000-0000-000000000000';
function nativeWaveFile() {
  return { id: identity.fileId, data: { 'pages-index': { [identity.pageId]: { objects: {
    copy: { id: 'copy', type: 'frame', shapes: ['wave', 'time'], x: 1800, y: 6800,
      width: 900, height: 620, 'component-id': 'soundwave', 'component-file': 'source-file',
      'shape-ref': 'source-main', touched: [], transform: { a: 1, b: 0, c: 0, d: 1, e: 0, f: 0 } },
    wave: { id: 'wave', type: 'path', 'parent-id': 'copy', x: 1814, y: 6991,
      width: 694, height: 240, 'shape-ref': 'source-wave',
      content: [{ command: 'move-to', params: { x: 1814, y: 6991 } }],
      selrect: { x: 1814, y: 6991, width: 694, height: 240 } },
    time: { id: 'time', type: 'text', 'parent-id': 'copy', 'shape-ref': 'source-time',
      touched: ['fill-group'], fills: [{ 'fill-color': '#e8e8e8' }],
      content: { type: 'root', children: [{ text: '0.125 s', 'font-size': '14',
        'typography-ref-id': null, 'typography-ref-file': null }] },
      'position-data': [{ x: 1900, y: 7300, width: 40, height: 20, text: '0.125 s',
        fills: [{ 'fill-color-ref-id': null, 'fill-color-ref-file': null }] }] },
  } } } } };
}

test('native copy readback checks path payload, transforms, bounds, line caches and every original source ref', () => {
  const before = nativeWaveFile();
  const readback = assertNativeInstanceGeometryPreserved(before, structuredClone(before));
  assert.deepEqual(readback.copies, [{ pageId: identity.pageId, rootId: 'copy' }]);
  assert.equal(readback.shapesChecked, 3);
  assert.equal(readback.protectedStatePreserved, true);
  const mutations = [
    shapes => { shapes.copy.x++; }, shapes => { shapes.copy.transform.a = 0.8; },
    shapes => { shapes.wave.width *= 0.8; }, shapes => { shapes.wave.selrect.height--; },
    shapes => { shapes.wave.content[0].params.y++; },
    shapes => { shapes.time['position-data'][0].width--; },
    shapes => { shapes.time.content.children[0].text = '0.25 s'; },
    shapes => { shapes.time['shape-ref'] = 'another-label'; },
    shapes => { shapes.copy.shapes.reverse(); }, shapes => { delete shapes.wave; },
    shapes => { shapes.time.fills[0]['fill-color'] = '#123456'; },
  ];
  for (const mutate of mutations) {
    const after = structuredClone(before); mutate(after.data['pages-index'][identity.pageId].objects);
    assert.throws(() => assertNativeInstanceGeometryPreserved(before, after), /Native copy/);
  }
});

test('native readback allows style reference caches to update while retaining all text geometry and local overrides', () => {
  const before = nativeWaveFile(), after = structuredClone(before);
  const text = after.data['pages-index'][identity.pageId].objects.time;
  text.content.children[0]['typography-ref-id'] = 'caption-style';
  text.content.children[0]['typography-ref-file'] = 'shared-file';
  text['position-data'][0].fills[0]['fill-color-ref-id'] = 'text-style';
  text['position-data'][0].fills[0]['fill-color-ref-file'] = 'shared-file';
  assert.doesNotThrow(() => assertNativeInstanceGeometryPreserved(before, after));
  text.content.children[0]['font-size'] = '16';
  assert.throws(() => assertNativeInstanceGeometryPreserved(before, after), /Native copy/);
});

test('native geometry checks retain Transit override sets and handle nested copies before their outer root', () => {
  const before = nativeWaveFile();
  const objects = before.data['pages-index'][identity.pageId].objects;
  objects.wave['component-id'] = 'nested-wave';
  objects.time.touched = new Set(['fill-group']);
  before.data['pages-index'][identity.pageId].objects = {
    wave: objects.wave, time: objects.time, copy: objects.copy,
  };
  assert.equal(assertNativeInstanceGeometryPreserved(before, structuredClone(before)).shapesChecked, 3);
});

test('native readback cannot claim a save/reopen pass after a copy or its cached time label moved', () => {
  const before = nativeWaveFile(), after = structuredClone(before);
  Object.assign(before, { revn: 7, vern: 0 }); Object.assign(after, { revn: 8, vern: 0 });
  const metadata = [{ boardId: 'copy', metadata: { document: 'same source' } }];
  after.data['pages-index'][identity.pageId].objects.time['position-data'][0].x++;
  verifyReopenedFile(before, after, identity, 8, metadata, metadata);
  assert.throws(() => assertNativeInstanceGeometryPreserved(before, after), /Native copy/);
});

test('native readback rejects a full typography sync replacing the explicit CJK font family', () => {
  const before = nativeWaveFile();
  before.data['pages-index'][identity.pageId].objects.time.content.children[0]['font-family'] = 'Noto Sans SC';
  const after = structuredClone(before);
  after.data['pages-index'][identity.pageId].objects.time.content.children[0]['font-family'] = 'Roboto';
  assert.throws(() => assertNativeInstanceGeometryPreserved(before, after), /Native copy/);
});

test('native readback failure records every changed copy node and the actual field delta without tolerance', () => {
  const before = nativeWaveFile(), after = structuredClone(before);
  const shapes = after.data['pages-index'][identity.pageId].objects;
  shapes.wave.content[0].params.y += 0.000061;
  shapes.time['position-data'][0].height -= 0.380005;
  shapes.copy.transform.e = 1;
  let failure;
  try { assertNativeInstanceGeometryPreserved(before, after); } catch (error) { failure = error; }
  assert.match(failure?.message ?? '', /Native copy/);
  const delta = failure.nativeCopyDelta;
  assert.equal(delta.protectedStatePreserved, false);
  assert.equal(delta.shapesChecked, 3);
  assert.deepEqual(delta.changedNodes.map(node => node.id).sort(), ['copy', 'time', 'wave']);
  assert.equal(delta.changedNodes.find(node => node.id === 'time').fields['position-data'].after.value[0].height,
    shapes.time['position-data'][0].height);
  for (const node of delta.nodes) {
    assert.match(node.beforeSha256, /^[a-f0-9]{64}$/);
    assert.notEqual(node.afterSha256, node.beforeSha256);
  }
});

test('native failure retains a removed descendant and reports source-owner metadata by hash without duplicating source', () => {
  const before = nativeWaveFile(), after = structuredClone(before);
  before.data['pages-index'][identity.pageId].objects.copy['plugin-data'] = { document: 'original source body' };
  after.data['pages-index'][identity.pageId].objects.copy['plugin-data'] = { document: 'changed source body' };
  delete after.data['pages-index'][identity.pageId].objects.wave;
  let failure;
  try { assertNativeInstanceGeometryPreserved(before, after); } catch (error) { failure = error; }
  const delta = failure.nativeCopyDelta;
  assert.equal(delta.nodes.find(node => node.id === 'wave').afterSha256, null);
  const metadata = delta.changedNodes.find(node => node.id === 'copy').fields['plugin-data'];
  assert.equal(Object.hasOwn(metadata.before, 'value'), false);
  assert.notEqual(metadata.before.sha256, metadata.after.sha256);
  assert.equal(JSON.stringify(delta).includes('source body'), false);
});

function sourceColorFiles() {
  const file = nativeWaveFile(); Object.assign(file, { revn: 54, vern: 0 });
  const libraryId = 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', colorId = identity.teamId;
  const sourcePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui', sourceSha256 = 'a'.repeat(64);
  const mapping = { schema: 'zircon.penpot-token-source.v1', version: 1, penpotStyleId: colorId,
    sourcePath, sourceSha256, token: 'editor.surface.1', sourceTable: 'palette', sourceKey: 'surface',
    sourceIndex: 0, originalValue: '#ABCDEF80' };
  const color = { id: colorId, color: '#abcdef', opacity: 128 / 255,
    'plugin-data': { 'shared/dev.zircon.zui': { 'token-source': JSON.stringify(mapping) } } };
  const library = { id: libraryId, revn: 2, vern: 0, data: { colors: { [colorId]: color } } };
  file.data['pages-index'][identity.pageId].objects.copy.fills = [{ 'fill-color': '#abcdef',
    'fill-opacity': 128 / 255, 'fill-color-ref-id': colorId, 'fill-color-ref-file': libraryId }];
  const document = { tokens: { 'editor.surface.1': '#ABCDEF80' }, penpot_host_theme_source: sourcePath,
    penpot_host_palette: { surface: ['#ABCDEF80'] }, penpot_source_fingerprints: [{ sourcePath, sha256: sourceSha256 }],
    penpot_token_source_owners: { 'editor.surface.1': { sourcePath, sha256: sourceSha256 } } };
  const snapshot = () => sourceColorRevisionSnapshot(file, [library], identity.pageId, 'copy', document);
  return { file, library, color, document, snapshot };
}

test('source-color wrapper records real library revision and exact RGBA/source without losing alpha', () => {
  const { snapshot } = sourceColorFiles(), before = snapshot();
  assert.deepEqual(before.libraries, [{ id: 'aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa', revn: 2, vern: 0 }]);
  assert.equal(before.references[0].opacity, 128 / 255);
  assert.equal(before.references[0].token, 'editor.surface.1');
  assert.doesNotThrow(() => assertSourceColorRevisionPreserved(before, snapshot()));
});

test('source-color wrapper rejects an advanced library revision even when its cached color is unchanged', () => {
  const { library, snapshot } = sourceColorFiles(), before = snapshot();
  library.revn++;
  assert.throws(() => assertSourceColorRevisionPreserved(before, snapshot()), /revision, RGBA or canonical source changed/);
});

test('source-color wrapper rejects wrong library, unknown UUID, stale source and one-byte alpha drift', () => {
  const failures = [
    fixture => { fixture.file.data['pages-index'][identity.pageId].objects.copy.fills[0]['fill-color-ref-file'] = identity.pageId; },
    fixture => { fixture.file.data['pages-index'][identity.pageId].objects.copy.fills[0]['fill-color-ref-id'] = identity.pageId; },
    fixture => { fixture.document.penpot_source_fingerprints[0].sha256 = 'b'.repeat(64); },
    fixture => { fixture.color.opacity = 129 / 255; },
    fixture => { fixture.file.data['pages-index'][identity.pageId].objects.copy.fills[0]['fill-opacity'] = 129 / 255; },
  ];
  for (const mutate of failures) {
    const fixture = sourceColorFiles(); mutate(fixture);
    assert.throws(fixture.snapshot, /library|UUID|closure|RGBA/);
  }
});

function appendFile() {
  return { id: identity.fileId, revn: 7, vern: 0, data: {
    pages: [identity.pageId], colors: { color: { color: '#333333' } },
    typographies: { body: { 'font-size': '12' } }, components: { master: { name: 'Original master' } },
    'pages-index': { [identity.pageId]: { id: identity.pageId, name: 'Existing design', objects: {
      [pageRoot]: { id: pageRoot, shapes: [identity.teamId], width: 0 },
      [identity.teamId]: { id: identity.teamId, name: 'Business draft', x: 12,
        'component-file': identity.fileId, 'component-id': 'master', 'shape-ref': 'original-main-child' },
    } } },
  } };
}

test('source owner append admits only the exact assigned version and original closure', () => {
  const file = appendFile(), append = { fileId: identity.fileId, pageId: identity.pageId, revn: 7, vern: 0 };
  const context = { sourceClosure: true, existingPlugin: true, authenticatedStorage: true,
    previousReceipt: false, editRequested: false, fullWorkbench: false };
  assert.equal(verifySourceOwnerAppendAdmission(file, identity, append, context), true);
  for (const update of [{ fileId: identity.teamId }, { pageId: identity.teamId }, { revn: 6 }, { vern: 1 }, { revn: -1 }])
    assert.throws(() => verifySourceOwnerAppendAdmission(file, identity, { ...append, ...update }, context));
  for (const key of Object.keys(context))
    assert.throws(() => verifySourceOwnerAppendAdmission(file, identity, append, { ...context, [key]: !context[key] }));
  file.data['pages-index'][identity.pageId].objects[identity.teamId]['plugin-data'] = {
    'shared/dev.zircon.zui': { role: 'asset', document: 'original owner' },
  };
  assert.throws(() => verifySourceOwnerAppendAdmission(file, identity, append, context));
});

test('source owner append preserves existing content and allows new roots and masters', () => {
  const before = appendFile(), after = structuredClone(before);
  after.revn++;
  const page = after.data['pages-index'][identity.pageId];
  page.objects[pageRoot].shapes.push(identity.fileId);
  page.objects[identity.fileId] = { id: identity.fileId, name: 'New official owner' };
  after.data.components.newMaster = { name: 'New source master' };
  assert.doesNotThrow(() => assertExistingPenpotContentPreserved(before, after));
  assert.deepEqual(before, appendFile());
});

test('source owner append rejects changes to foreign geometry, references, styles, components and pages', () => {
  const before = appendFile();
  const mutations = [
    file => { file.id = identity.teamId; },
    file => { file.data.colors.color.color = '#444444'; },
    file => { file.data.typographies.body['font-size'] = '13'; },
    file => { file.data.components.master.name = 'Changed'; },
    file => { file.data.pages = []; },
    file => { file.data['pages-index'][identity.teamId] = { objects: {} }; },
    file => { file.data['pages-index'][identity.pageId].name = 'Changed'; },
    file => { file.data['pages-index'][identity.pageId].objects[identity.teamId].x++; },
    file => { file.data['pages-index'][identity.pageId].objects[identity.teamId]['shape-ref'] = 'detached'; },
    file => { delete file.data['pages-index'][identity.pageId].objects[identity.teamId]; },
    file => { file.data['pages-index'][identity.pageId].objects[pageRoot].width++; },
    file => { file.data['pages-index'][identity.pageId].objects[pageRoot].shapes = []; },
  ];
  for (const mutate of mutations) {
    const after = structuredClone(before); mutate(after);
    assert.throws(() => assertExistingPenpotContentPreserved(before, after));
  }
});

test('identity comes from a complete actual workspace URL', () => {
  const result = workspaceIdentity(`http://127.0.0.1:9001/#/workspace?team-id=${identity.teamId}&file-id=${identity.fileId}&page-id=${identity.pageId}`);
  assert.deepEqual(result, { origin: 'http://127.0.0.1:9001', ...identity });
  assert.throws(() => workspaceIdentity('http://127.0.0.1:9001/#/workspace?file-id=placeholder'));
});
test('write grant rejects lookalike roots and physical aliases', () => {
  const physical = (path) => path;
  assert.equal(requireRealSessionDirectory('D:/cargo-targets/zircon-local/penpot/evidence', physical),
    'D:\\cargo-targets\\zircon-local\\penpot\\evidence');
  for (const path of ['C:/cargo-targets/zircon-local/evidence', 'D:/cargo-targets/evidence',
    'D:/nested/cargo-targets/zircon-local', 'E:/Git/ZirconEngine/target', 'D:/targets/zircon-local'])
    assert.throws(() => requireRealSessionDirectory(path, physical));
  assert.throws(() => requireRealSessionDirectory('D:/cargo-targets/zircon-local/evidence',
    () => 'D:\\cargo-targets\\other\\evidence'));
});
test('save binds the main-method acknowledgement to its actual request and advanced change revision', () => {
  assert.equal(verifySaveResponse(save, identity, 7, saveRequest), 8);
  assert.throws(() => verifySaveResponse({ revn: 8, lagged: [] }, identity, 7, saveRequest));
  assert.throws(() => verifySaveResponse(savedChanges, identity, 7, saveRequest));
  assert.throws(() => verifySaveResponse(save, identity, 8, saveRequest));
  assert.throws(() => verifySaveResponse(save, identity, 7, { ...saveRequest, id: identity.teamId }));
  assert.throws(() => verifySaveResponse(save, identity, 7, { ...saveRequest, 'session-id': identity.teamId }));
  assert.throws(() => verifySaveResponse(save, identity, 7, { ...saveRequest, changes: [] }));
  assert.throws(() => verifySaveResponse(save, identity, 7, { ...saveRequest, changes: [{ type: 'add-obj' }] }));
  assert.throws(() => verifySaveResponse({ ...save, lagged: [{ ...savedChanges[0], revn: 7 }] }, identity, 7, saveRequest));
  assert.throws(() => verifySaveResponse({ ...save, lagged: [{ ...savedChanges[0], 'file-id': identity.teamId }] }, identity, 7, saveRequest));
});
test('reopen rejects missing, altered metadata and stale revisions', () => {
  const before = { id: identity.fileId, revn: 7, vern: 0 };
  const after = { ...before, revn: 8 };
  const metadata = [{ boardId: identity.pageId, metadata: { document: 'original' } }];
  assert.ok(verifyReopenedFile(before, after, identity, 8, metadata, metadata));
  assert.throws(() => verifyReopenedFile(before, after, identity, 8, metadata, []));
  assert.throws(() => verifyReopenedFile(before, before, identity, 8, metadata, metadata));
  assert.throws(() => verifyReopenedFile(before, after, identity, 8, [], []));
});

test('scroll cleanup admits only the known additive empty markers without mutating actual metadata', () => {
  const archived = [{ boardId: identity.pageId, boardName: 'Owning board', metadata: { document: 'original' } }];
  const current = structuredClone(archived);
  const keys = ['review-scroll-captured-x', 'review-scroll-captured-y', 'review-scroll-captured-positions'];
  for (const key of keys) current[0].metadata[key] = '';
  const before = structuredClone(current);
  assert.deepEqual(verifyReviewScrollCleanupAdmission(archived, current),
    keys.map(key => ({ boardId: identity.pageId, key })));
  assert.deepEqual(current, before);
  assert.deepEqual(verifyReviewScrollCleanupAdmission(archived, archived), []);
  assert.throws(() => verifyReopenedFile({ id: identity.fileId, vern: 0 },
    { id: identity.fileId, revn: 8, vern: 0 }, identity, 8, archived, current));
});

test('scroll cleanup rejects authored changes, nonempty markers and different board identities', () => {
  const archived = [{ boardId: identity.pageId, boardName: 'Owning board', metadata: { document: 'original' } }];
  for (const change of [
    item => { item.metadata.document = 'changed'; },
    item => { item.metadata['unknown-empty-key'] = ''; },
    item => { item.metadata['review-scroll-captured-x'] = '12'; },
    item => { item.metadata['review-scroll-captured-y'] = null; },
    item => { item.boardId = identity.teamId; },
    item => { item.boardName = 'Changed'; },
  ]) {
    const current = structuredClone(archived); change(current[0]);
    assert.throws(() => verifyReviewScrollCleanupAdmission(archived, current));
  }
  assert.throws(() => verifyReviewScrollCleanupAdmission(archived, []));
  const previouslyCaptured = structuredClone(archived);
  previouslyCaptured[0].metadata['review-scroll-captured-x'] = '';
  assert.throws(() => verifyReviewScrollCleanupAdmission(previouslyCaptured, previouslyCaptured));
});
test('matching actual change sets preserves set semantics across CLJS and backend Transit ordering', () => {
  const request = { ...saveRequest, changes: [{ touched: new Set(['geometry', 'fill']) }] };
  const response = { ...save, lagged: [{ ...savedChanges[0], changes: [{ touched: new Set(['fill', 'geometry']) }] }] };
  assert.equal(verifySaveResponse(response, identity, 7, request), 8);
  assert.throws(() => verifySaveResponse({ ...response, lagged: [{ ...response.lagged[0],
    changes: [{ touched: new Set(['fill']) }] }] }, identity, 7, request));
});
test('backend metadata is selected from the actual page and shared namespace', () => {
  const metadata = { role: 'asset', document: 'complete original document' };
  const file = { data: { 'pages-index': { [identity.pageId]: { objects: {
    [identity.teamId]: { id: identity.teamId, name: 'Imported', 'plugin-data': { 'shared/dev.zircon.zui': metadata } },
    [identity.fileId]: { id: identity.fileId, 'plugin-data': { 'plugin/unrelated': metadata } },
  } } } } };
  assert.deepEqual(persistedZuiMetadata(file, identity.pageId), [{ boardId: identity.teamId, boardName: 'Imported', metadata }]);
  assert.throws(() => persistedZuiMetadata(file, identity.teamId));
});
test('a persistence claim requires every independent real evidence gate', () => {
  const gates = { status: 'passed', backend: 'real', saveAcknowledged: true, closedOriginalContext: true,
    newAuthenticatedContext: true, reopenedSameFile: true, metadataMatched: true,
    exportSemanticMatch: true, sourceFingerprintMatched: true };
  assert.equal(persistenceClaim(gates), true);
  for (const gate of Object.keys(gates)) assert.equal(persistenceClaim({ ...gates, [gate]: false }), false, gate);
  assert.equal(persistenceClaim({ ...gates, backend: 'repository-mock' }), false);
  for (const status of ['pending', 'failed', undefined]) {
    assert.equal(persistenceClaim({ ...gates, status, persistenceClaim: true }), false, status);
  }
});

function toolbarBackendFixture() {
  const contract = { row: 'row', popup: 'popup', groups: ['g1', 'g2', 'g3'] };
  const source = { left: 2, right: 6, top: 3, bottom: 5 }, zero = { left: 0, right: 0, top: 0, bottom: 0 };
  const object = (id, nodeId, parent, role = 'node') => ({ id, 'parent-id': parent,
    'plugin-data': { 'shared/dev.zircon.zui': { role, 'node-id': nodeId, sourcePath: 'res://viewport.zui',
      sourceNodeId: nodeId, instancePath: '[]', baseline: '{"slotPadding":{"left":2,"right":6,"top":3,"bottom":5}}' } } });
  const objects = { asset: object('asset', 'root', null, 'asset'), row: object('row', 'row', 'asset'),
    popup: object('popup', 'popup', 'asset'), g1: object('g1', 'g1', 'row'), g2: object('g2', 'g2', 'popup'), g3: object('g3', 'g3', 'popup') };
  for (const id of ['g2', 'g3']) objects[id]['plugin-data']['shared/dev.zircon.zui']['viewport-toolbar-slot-padding-allocation'] =
    JSON.stringify({ version: 1, group: id, row: 'row', popup: 'popup', baseline: source, authored: source, runtime: zero });
  return { contract, file: { data: { 'pages-index': { [identity.pageId]: { objects } } } }, objects };
}
test('actual saved toolbar groups preserve their full shared namespace through a fresh backend read', () => {
  const fixture = toolbarBackendFixture();
  const saved = persistedViewportToolbarGroupMetadata(fixture.file, identity.pageId, 'asset', fixture.contract);
  const reopened = persistedViewportToolbarGroupMetadata(structuredClone(fixture.file), identity.pageId, 'asset', fixture.contract);
  assert.deepEqual(reopened, saved);
  assert.deepEqual(saved.map(group => [group.nodeId, group.parentNodeId]), [['g1', 'row'], ['g2', 'popup'], ['g3', 'popup']]);
  assert.equal(JSON.parse(saved[1].metadata['viewport-toolbar-slot-padding-allocation']).authored.right, 6);
  assert.equal(saved[1].metadata.sourceNodeId, 'g2');
  const changed = structuredClone(fixture.file), data = changed.data['pages-index'][identity.pageId].objects.g2['plugin-data']['shared/dev.zircon.zui'];
  const allocation = JSON.parse(data['viewport-toolbar-slot-padding-allocation']); allocation.authored.right = 9;
  data['viewport-toolbar-slot-padding-allocation'] = JSON.stringify(allocation);
  assert.throws(() => assert.deepEqual(persistedViewportToolbarGroupMetadata(changed, identity.pageId, 'asset', fixture.contract), saved));
});
test('actual toolbar readback rejects lost allocation, malformed values, escaped groups and unordered parents', () => {
  for (const mutate of [
    fixture => delete fixture.objects.g2['plugin-data']['shared/dev.zircon.zui']['viewport-toolbar-slot-padding-allocation'],
    fixture => fixture.objects.g2['plugin-data']['shared/dev.zircon.zui']['viewport-toolbar-slot-padding-allocation'] = '{}',
    fixture => fixture.objects.g2['parent-id'] = 'asset',
    fixture => fixture.objects.g3['parent-id'] = 'row',
  ]) {
    const fixture = toolbarBackendFixture(); mutate(fixture);
    assert.throws(() => persistedViewportToolbarGroupMetadata(fixture.file, identity.pageId, 'asset', fixture.contract));
  }
});
test('toolbar metadata admission is scoped to the owning asset and rejects duplicate groups or parent cycles', () => {
  const fixture = toolbarBackendFixture(), outside = structuredClone(fixture.objects.g2);
  outside.id = 'foreign-g2'; outside['parent-id'] = 'foreign-asset'; fixture.objects['foreign-g2'] = outside;
  assert.equal(persistedViewportToolbarGroupMetadata(fixture.file, identity.pageId, 'asset', fixture.contract).length, 3);
  outside['parent-id'] = 'popup';
  assert.throws(() => persistedViewportToolbarGroupMetadata(fixture.file, identity.pageId, 'asset', fixture.contract));
  delete fixture.objects['foreign-g2']; fixture.objects.g2['parent-id'] = 'g2';
  assert.throws(() => persistedViewportToolbarGroupMetadata(fixture.file, identity.pageId, 'asset', fixture.contract));
});
