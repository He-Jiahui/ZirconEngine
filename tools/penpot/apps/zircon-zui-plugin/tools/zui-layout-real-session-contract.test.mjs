import assert from 'node:assert/strict';
import test from 'node:test';
import { persistenceClaim, persistedZuiMetadata, persistedViewportToolbarGroupMetadata, requireRealSessionDirectory,
  verifyReopenedFile, verifySaveResponse, workspaceIdentity } from './zui-layout-real-session-contract.mjs';

const identity = { teamId: '12345678-1111-4111-8111-123456789abc',
  fileId: '12345678-2222-4222-8222-123456789abc', pageId: '12345678-3333-4333-8333-123456789abc' };
const changes = [{ type: 'mod-obj' }];
const saveRequest = { id: identity.fileId, 'session-id': identity.pageId, revn: 7, changes };
const savedChanges = [{ 'file-id': identity.fileId, id: identity.teamId,
  'session-id': identity.pageId, revn: 8, changes: [{ type: 'mod-obj' }] }];
const save = { revn: 7, lagged: savedChanges };

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
