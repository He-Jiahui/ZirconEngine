import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { realpathSync, statSync } from 'node:fs';
import { win32 } from 'node:path';

const uuid = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
export const sha256 = (value) => createHash('sha256').update(value).digest('hex');

/** Only physical directories in this chat's granted compiler storage are writable. */
export function requireRealSessionDirectory(path, resolvePhysical = realpathSync) {
  assert.equal(typeof path, 'string', 'A real session artifact directory is required');
  assert.ok(win32.isAbsolute(path), 'The artifact directory must be absolute');
  const candidate = win32.normalize(path).replace(/[\\/]+$/, '');
  const root = /^[DEF]:\\cargo-targets\\zircon-local(?:\\|$)/i;
  assert.ok(root.test(candidate), 'Real session output must be under D/E/F:\\cargo-targets\\zircon-local');
  const physical = resolvePhysical(candidate).replace(/[\\/]+$/, '');
  assert.equal(physical.toLowerCase(), candidate.toLowerCase(), 'Path aliases and junctions are not allowed');
  assert.ok(root.test(physical), 'The physical output directory is outside the write grant');
  if (resolvePhysical === realpathSync) assert.ok(statSync(physical).isDirectory());
  return physical;
}

export function workspaceIdentity(workspaceUrl) {
  const url = new URL(workspaceUrl);
  assert.ok(url.hash.startsWith('#/workspace?'), 'Use an actual Penpot workspace URL');
  const params = new URLSearchParams(url.hash.slice(url.hash.indexOf('?') + 1));
  const identity = Object.fromEntries(['team-id', 'file-id', 'page-id'].map((key) => {
    const value = params.get(key);
    assert.ok(value && uuid.test(value), `The actual workspace URL needs ${key}`);
    return [key.replace(/-id$/, 'Id').replace('-', ''), value];
  }));
  return { origin: url.origin, teamId: identity.teamId, fileId: identity.fileId, pageId: identity.pageId };
}

export function verifySaveResponse(response, identity, previousRevision, saveRequest) {
  assert.ok(response && typeof response === 'object' && !Array.isArray(response),
    'Penpot 2.17 update-file returns a revision and lagged change records');
  assert.equal(saveRequest?.id, identity.fileId, 'The observed save request must identify this actual file');
  assert.ok(uuid.test(saveRequest['session-id'] ?? ''), 'The request needs its actual frontend session ID');
  assert.ok(Number.isSafeInteger(saveRequest.revn) && saveRequest.revn >= previousRevision);
  assert.ok(Array.isArray(saveRequest.changes) && saveRequest.changes.length, 'The actual save request must contain changes');
  assert.ok(Number.isSafeInteger(response.revn) && response.revn >= saveRequest.revn);
  assert.ok(Array.isArray(response.lagged), 'The acknowledgement must include its actual persisted change records');
  const saved = response.lagged.filter((change) => change['file-id'] === identity.fileId &&
    change['session-id'] === saveRequest['session-id'] && isDeepStrictEqual(change.changes, saveRequest.changes));
  assert.ok(saved.length, 'update-file did not acknowledge this file');
  for (const change of saved) {
    assert.ok(uuid.test(change.id ?? ''), 'Saved change needs an actual backend change ID');
    assert.ok(uuid.test(change['session-id'] ?? ''), 'Saved change needs the actual frontend session ID');
    assert.ok(Number.isSafeInteger(change.revn) && change.revn > saveRequest.revn, 'The backend revision must advance');
    assert.ok(Array.isArray(change.changes) && change.changes.length, 'The response must acknowledge changes');
  }
  return Math.max(...saved.map((change) => change.revn));
}

export function verifyReopenedFile(before, after, identity, acknowledgedRevision, metadataBefore, metadataAfter) {
  assert.equal(before.id, identity.fileId);
  assert.equal(after.id, identity.fileId);
  assert.ok(Number.isSafeInteger(after.revn) && after.revn >= acknowledgedRevision);
  assert.ok(Number.isSafeInteger(after.vern) && after.vern >= before.vern);
  assert.ok(metadataBefore.length > 0, 'Import must persist reversible ZUI metadata');
  assert.deepEqual(metadataAfter, metadataBefore, 'Metadata changed across a new authenticated browser context');
  return true;
}

export function persistenceClaim(evidence) {
  return evidence.status === 'passed' && evidence.backend === 'real' && evidence.saveAcknowledged === true &&
    evidence.closedOriginalContext === true && evidence.newAuthenticatedContext === true &&
    evidence.reopenedSameFile === true && evidence.metadataMatched === true &&
    evidence.exportSemanticMatch === true && evidence.sourceFingerprintMatched === true;
}

/** Read the actual realized page and its shared plugin namespace. */
export function persistedZuiMetadata(file, pageId) {
  const page = (file.data?.['pages-index'] ?? file.data?.pagesIndex)?.[pageId];
  assert.ok(page && page.objects, 'get-file did not return the actual, realized page objects');
  return Object.values(page.objects).flatMap((shape) => {
    const metadata = (shape['plugin-data'] ?? shape.pluginData)?.['shared/dev.zircon.zui'];
    return metadata?.role === 'asset' && metadata.document
      ? [{ boardId: shape.id, boardName: shape.name, metadata }]
      : [];
  }).sort((a, b) => a.boardId.localeCompare(b.boardId));
}

/** Read the original editable groups from the actual saved page objects. */
export function persistedViewportToolbarGroupMetadata(file, pageId, assetBoardId, contract) {
  assert.ok(contract && typeof contract.row === 'string' && typeof contract.popup === 'string' &&
    Array.isArray(contract.groups) && contract.groups.length > 0 && contract.groups.length <= 32 &&
    contract.groups.every(id => typeof id === 'string' && id.length > 0) &&
    new Set(contract.groups).size === contract.groups.length, 'Actual Source toolbar group contract is missing');
  const page = (file.data?.['pages-index'] ?? file.data?.pagesIndex)?.[pageId];
  assert.ok(page && page.objects, 'get-file did not return the actual page objects');
  const shapes = Object.values(page.objects), byId = new Map();
  for (const shape of shapes) {
    assert.ok(shape && typeof shape.id === 'string' && !byId.has(shape.id), 'Actual page object ID is missing or duplicated');
    byId.set(shape.id, shape);
  }
  const metadataFor = shape => (shape?.['plugin-data'] ?? shape?.pluginData)?.['shared/dev.zircon.zui'];
  const parentFor = shape => shape?.['parent-id'] ?? shape?.parentId;
  assert.equal(metadataFor(byId.get(assetBoardId))?.role, 'asset', 'Actual imported asset Board is missing');
  const isInside = shape => {
    const visited = new Set();
    for (let current = shape; current; current = byId.get(parentFor(current))) {
      assert.ok(!visited.has(current.id), 'Actual Board parent cycle');
      visited.add(current.id);
      if (current.id === assetBoardId) return true;
    }
    return false;
  };
  let popupSeen = false;
  return contract.groups.map(nodeId => {
    const owned = shapes.filter(shape => {
      const metadata = metadataFor(shape);
      return metadata?.role === 'node' && metadata['node-id'] === nodeId && isInside(shape);
    });
    assert.equal(owned.length, 1, `Actual saved toolbar group ${nodeId} is missing or duplicated`);
    const shape = owned[0], parentBoardId = parentFor(shape), parent = byId.get(parentBoardId);
    const parentMetadata = metadataFor(parent), parentNodeId = parentMetadata?.['node-id'];
    assert.ok(parentMetadata?.role === 'node' && [contract.row, contract.popup].includes(parentNodeId),
      `Actual saved toolbar group ${nodeId} is outside its Source hosts`);
    if (popupSeen) assert.equal(parentNodeId, contract.popup, 'Actual saved toolbar groups do not form an ordered suffix');
    popupSeen ||= parentNodeId === contract.popup;
    const metadata = metadataFor(shape), encoded = metadata['viewport-toolbar-slot-padding-allocation'];
    if (parentNodeId === contract.popup) {
      assert.ok(typeof encoded === 'string' && encoded.length > 0,
        `Actual saved toolbar group ${nodeId} lacks its Source padding allocation`);
      const allocation = JSON.parse(encoded);
      assert.deepEqual(Object.keys(allocation).sort(), ['authored', 'baseline', 'group', 'popup', 'row', 'runtime', 'version']);
      assert.equal(allocation.version, 1); assert.equal(allocation.group, nodeId);
      assert.equal(allocation.row, contract.row); assert.equal(allocation.popup, contract.popup);
      for (const key of ['baseline', 'authored', 'runtime']) {
        assert.ok(allocation[key] && typeof allocation[key] === 'object');
        assert.deepEqual(Object.keys(allocation[key]).sort(), ['bottom', 'left', 'right', 'top']);
        assert.ok(Object.values(allocation[key]).every(value => typeof value === 'number' && Number.isFinite(value) && value >= 0),
          'Actual Source padding allocation is not a finite logical padding');
      }
      assert.ok(Object.values(allocation.runtime).every(value => value === 0), 'Actual ScrollableBox runtime padding differs');
    } else assert.ok(encoded === undefined || encoded === '', 'Inline Source group retains a popup padding allocation');
    return { nodeId, boardId: shape.id, parentBoardId, parentNodeId,
      metadata: JSON.parse(JSON.stringify(metadata)) };
  });
}
