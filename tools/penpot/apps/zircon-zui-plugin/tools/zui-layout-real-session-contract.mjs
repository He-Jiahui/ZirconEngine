import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { isDeepStrictEqual } from 'node:util';
import { serialize } from 'node:v8';
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

/** Decide side effects before authentication, project creation, host startup or browser launch. */
export function realSessionExecutionPlan(options, context) {
  assert.ok(options && typeof options === 'object' && !Array.isArray(options), 'Explicit session options are required');
  assert.ok(Object.keys(options).every(key =>
    ['mode', 'expectedWorkspace', 'existingPlugin', 'append'].includes(key)), 'Unknown session option');
  const mode = options.mode ?? 'reuse-existing';
  assert.ok(mode === 'reuse-existing' || mode === 'managed-bootstrap', 'Unknown real session execution mode');
  if (mode === 'managed-bootstrap') {
    assert.ok(!options.existingPlugin && !options.append, 'Bootstrap cannot adopt an existing plugin or append to a domain file');
  } else {
    assert.ok(options.existingPlugin && typeof options.existingPlugin === 'object',
      'Reuse requires an explicitly pinned existing plugin; bootstrap is never implicit');
    assert.ok(context.authenticatedStorage && context.uiBuildReceipt && context.workerBuildReceipt,
      'Reuse requires authenticated storage and current managed UI/worker build evidence');
    assert.ok(context.workspaceUrl && options.expectedWorkspace,
      'Reuse requires the assigned workspace and independent expected team/file/page identity');
    for (const key of ['manifestSha256', 'bundleSha256', 'registryEntrySha256'])
      assert.match(options.existingPlugin[key] ?? '', /^[0-9a-f]{64}$/, 'Existing plugin needs an exact ' + key);
    assert.ok(uuid.test(options.existingPlugin.pluginId ?? ''), 'Existing plugin needs its actual registered UUID');
    const host = new URL(options.existingPlugin.host);
    assert.ok(['http:', 'https:'].includes(host.protocol) &&
      ['127.0.0.1', 'localhost', '[::1]'].includes(host.hostname) && host.pathname === '/' &&
      !host.search && !host.hash && !host.username && !host.password, 'Reuse requires the exact local plugin host root');
  }
  if (context.workspaceUrl) {
    const identity = workspaceIdentity(context.workspaceUrl);
    assert.ok(options.expectedWorkspace, 'An existing workspace requires independent expected identity');
    assert.deepEqual(identity, options.expectedWorkspace, 'Assigned workspace differs from expected team/file/page identity');
  } else {
    assert.ok(mode === 'managed-bootstrap' && !options.expectedWorkspace, 'Only explicit bootstrap can create a workspace');
  }
  return { mode, bootstrapAllowed: mode === 'managed-bootstrap' };
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

const shapeKey = key => key.replaceAll('-', '').toLowerCase();
function shapeField(shape, name) {
  const entries = Object.entries(shape).filter(([key]) => shapeKey(key) === name);
  assert.ok(entries.length <= 1, `Ambiguous Penpot shape field: ${name}`);
  return entries[0]?.[1];
}

// A color/typography reference may change its cache entry without changing
// glyph positions. Keep every numeric/text metric and every unrecognised field.
function textLayoutGeometry(value) {
  if (Array.isArray(value)) return value.map(textLayoutGeometry);
  if (!value || typeof value !== 'object') return value;
  const prototype = Object.getPrototypeOf(value);
  if (prototype !== Object.prototype && prototype !== null) return value;
  return Object.fromEntries(Object.entries(value)
    .filter(([key]) => !['fills', 'strokes', 'typographyrefid', 'typographyreffile'].includes(shapeKey(key)))
    .map(([key, item]) => [key, textLayoutGeometry(item)]));
}

function nativeShapeGeometry(shape) {
  const fields = new Set(['id', 'type', 'parentid', 'frameid', 'shapes', 'x', 'y', 'width', 'height',
    'selrect', 'bounds', 'boundrect', 'points', 'transform', 'transforminverse', 'rotation', 'flipx', 'flipy', 'hidden',
    'proportion', 'proportionlock', 'fixedscroll', 'locked', 'showcontent', 'clipcontent', 'maskedgroup',
    'componentid', 'componentfile', 'componentroot', 'maininstance', 'shaperef', 'touched',
    'plugindata', 'constraintsh', 'constraintsv', 'constraintsHorizontal'.toLowerCase(),
    'constraintsVertical'.toLowerCase(), 'growtype', 'textalign', 'textverticalalign']);
  const protectedState = Object.fromEntries(Object.entries(shape).filter(([key]) => {
    const name = shapeKey(key);
    return fields.has(name) || name.startsWith('layout');
  }));
  const content = shapeField(shape, 'content');
  if (content !== undefined) protectedState.content = shapeField(shape, 'type') === 'text'
    ? textLayoutGeometry(content) : content;
  const positions = shapeField(shape, 'positiondata');
  if (positions !== undefined) protectedState.positionData = textLayoutGeometry(positions);
  // Penpot touched groups identify authored local overrides. Preserve their
  // values too; an unchanged touched mask alone cannot prove override retention.
  const touched = shapeField(shape, 'touched');
  if (touched != null) {
    assert.ok(Array.isArray(touched) || touched instanceof Set,
      'Native override groups must be realised as an array or Transit set');
    // app.common.types.component/sync-attrs is the owner of these groups.
    const known = { 'fill-group': ['fills', 'hidefillonexport', 'fillcolor', 'fillopacity'],
      'stroke-group': ['strokes', 'strokewidth'], 'content-group': ['content', 'positiondata'],
      'text-font-group': ['growtype', 'fontfamily', 'fontsize', 'fontstyle', 'fontweight'],
      'text-display-group': ['letterspacing', 'lineheight', 'textalign'] };
    protectedState.localOverrides = Object.fromEntries([...touched].map(group => {
      assert.equal(typeof group, 'string', 'Unknown native override group representation');
      const keys = known[group];
      return [group, keys ? Object.fromEntries(keys.map(key => [key, shapeField(shape, key)])) : shape];
    }));
  }
  return protectedState;
}

// These hashes describe the realised values in this exact Node/V8 execution.
// They retain Transit sets and typed buffers; they are not portable JSON hashes
// and do not replace the strict geometry/override comparison below.
const nativeValueHash = value => sha256(serialize(value));
function deltaValue(value) {
  if (value === undefined) return { type: 'undefined' };
  if (typeof value === 'bigint') return { type: 'bigint', decimal: String(value) };
  if (typeof value === 'number' && !Number.isFinite(value)) return { type: 'number', value: String(value) };
  if (value instanceof Set) return { type: 'set', values: [...value].map(deltaValue) };
  if (value instanceof Map) return { type: 'map', entries: [...value].map(([key, item]) => [deltaValue(key), deltaValue(item)]) };
  if (value instanceof Date) return { type: 'date', value: value.toISOString() };
  if (ArrayBuffer.isView(value)) return { type: value.constructor.name,
    bytes: [...new Uint8Array(value.buffer, value.byteOffset, value.byteLength)] };
  if (value instanceof ArrayBuffer) return { type: 'ArrayBuffer', bytes: [...new Uint8Array(value)] };
  if (Array.isArray(value)) return value.map(deltaValue);
  if (value && typeof value === 'object')
    return Object.fromEntries(Object.entries(value).map(([key, item]) => [key, deltaValue(item)]));
  return value;
}

function nativeFieldDelta(before, after) {
  return Object.fromEntries([...new Set([...Object.keys(before), ...Object.keys(after)])]
    .filter(key => Object.hasOwn(before, key) !== Object.hasOwn(after, key) || !isDeepStrictEqual(before[key], after[key]))
    .map(key => {
      const observation = shape => {
        if (!Object.hasOwn(shape, key)) return { present: false };
        // Embedded ZUI documents remain in their original owner, not copied into
        // diagnostics. Hash the complete plugin data without emitting its body.
        return { present: true, sha256: nativeValueHash(shape[key]), ...(shapeKey(key) === 'plugindata'
          ? { valueOmitted: 'source-owner plugin data; full value hashed' } : { value: deltaValue(shape[key]) }) };
      };
      return [key, { before: observation(before), after: observation(after) }];
    }));
}

/** Called only with realised, authenticated get-file responses. SDK Library
 * objects do not expose backend revision authority. No version is inferred from
 * a color name, token-source metadata or the cached asFill value.
 */
export function sourceColorRevisionSnapshot(file, linkedFiles, pageId, boardId, document) {
  const revision = entry => {
    assert.ok(uuid.test(entry.id ?? ''), 'Source color file must have its actual UUID');
    for (const key of ['revn', 'vern']) assert.ok(Number.isSafeInteger(entry[key]) && entry[key] >= 0,
      `Source color file needs an actual ${key}`);
    return { id: entry.id, revn: entry.revn, vern: entry.vern };
  };
  const consumer = revision(file), libraries = new Map([[file.id, file]]);
  for (const linked of linkedFiles) {
    revision(linked);
    assert.ok(!libraries.has(linked.id), 'Duplicate or local file in connected color libraries');
    libraries.set(linked.id, linked);
  }
  const pages = file.data?.['pages-index'] ?? file.data?.pagesIndex;
  const objects = pages?.[pageId]?.objects;
  assert.ok(objects?.[boardId], 'Source color export needs its actual selected owner board');
  const references = [], versions = new Map(), visited = new Set();
  const rgba = value => {
    assert.ok(typeof value === 'string' && /^#[\da-f]{6}(?:[\da-f]{2})?$/i.test(value),
      'Source color must be an explicit RGB/RGBA token value');
    return { color: value.slice(0, 7).toLowerCase(), opacity: value.length === 9 ? parseInt(value.slice(7), 16) / 255 : 1 };
  };
  const sourceValue = value => {
    const seen = new Set();
    while (typeof value === 'string' && value.startsWith('$')) {
      const token = value.slice(1);
      assert.ok(!seen.has(token) && Object.hasOwn(document.tokens ?? {}, token), 'Missing or cyclic color token owner');
      seen.add(token); value = document.tokens[token];
    }
    return rgba(value);
  };
  const reference = (paint, kind, shapeId, path) => {
    const refId = shapeField(paint, kind + 'colorrefid'), refFile = shapeField(paint, kind + 'colorreffile');
    if (refId == null && refFile == null) return;
    assert.ok(uuid.test(refId ?? '') && uuid.test(refFile ?? ''), 'Partial or invalid shared color reference');
    const library = libraries.get(refFile);
    assert.ok(library, 'Shared color points to a library not connected to the actual file');
    const style = library.data?.colors?.[refId];
    assert.ok(style && style.id === refId, 'Unknown shared color UUID in its actual library');
    const encoded = shapeField(style, 'plugindata')?.['shared/dev.zircon.zui']?.['token-source'];
    assert.equal(typeof encoded, 'string', 'Shared color has no canonical token-source mapping');
    const source = JSON.parse(encoded);
    assert.equal(source.schema, 'zircon.penpot-token-source.v1'); assert.equal(source.version, 1);
    assert.equal(source.penpotStyleId, refId, 'Shared color source belongs to another UUID');
    const owner = document.penpot_token_source_owners?.[source.token];
    const pins = (document.penpot_source_fingerprints ?? []).filter(pin => pin.sourcePath === source.sourcePath);
    assert.ok(source.sourcePath === document.penpot_host_theme_source && /^[a-f\d]{64}$/i.test(source.sourceSha256 ?? '') &&
      pins.length === 1 && pins[0].sha256 === source.sourceSha256 && owner?.sourcePath === source.sourcePath &&
      owner?.sha256 === source.sourceSha256, 'Shared color source closure is stale or belongs to another owner');
    assert.equal(source.sourceTable, 'palette');
    let palette = document.penpot_host_palette?.[source.sourceKey];
    if (source.sourceIndex !== undefined) {
      assert.ok(Array.isArray(palette) && Number.isSafeInteger(source.sourceIndex) && source.sourceIndex >= 0,
        'Shared color palette index differs from its source');
      palette = palette[source.sourceIndex];
    }
    assert.equal(palette, source.originalValue, 'Shared color palette source changed');
    const expected = rgba(source.originalValue);
    assert.deepEqual(sourceValue('$' + source.token), expected, 'Shared token no longer resolves to its recorded RGBA');
    const actual = { color: style.color?.toLowerCase(), opacity: style.opacity ?? 1 };
    assert.deepEqual(actual, expected, 'Actual shared color RGBA changed outside its source owner');
    assert.ok(style.gradient == null && style.image == null, 'Shared source color is not a solid color');
    assert.deepEqual({ color: shapeField(paint, kind + 'color')?.toLowerCase(),
      opacity: shapeField(paint, kind + 'opacity') ?? 1 }, expected, 'Referenced paint RGBA is stale');
    versions.set(library.id, revision(library));
    references.push({ shapeId, path, kind, refId, refFile, ...expected,
      sourcePath: source.sourcePath, sourceSha256: source.sourceSha256, token: source.token,
      styleSha256: nativeValueHash(style) });
  };
  const scan = (value, shapeId, path) => {
    if (Array.isArray(value)) { value.forEach((entry, index) => scan(entry, shapeId, path + '/' + index)); return; }
    if (!value || typeof value !== 'object') return;
    reference(value, 'fill', shapeId, path); reference(value, 'stroke', shapeId, path);
    for (const [key, child] of Object.entries(value)) scan(child, shapeId, path + '/' + key);
  };
  const visit = id => {
    assert.ok(!visited.has(id), 'Duplicate or cyclic source-color owner subtree'); visited.add(id);
    const shape = objects[id]; assert.ok(shape, 'Source-color owner descendant disappeared');
    for (const field of ['fills', 'strokes', 'content', 'positiondata']) scan(shapeField(shape, field), id, field);
    const children = shapeField(shape, 'shapes') ?? [];
    assert.ok(Array.isArray(children), 'Source-color owner children must be realised'); children.forEach(visit);
  };
  visit(boardId);
  return { schema: 'zircon.penpot-source-color-revision.v1', consumer, pageId, boardId,
    libraries: [...versions.values()].sort((a, b) => a.id.localeCompare(b.id)), references };
}

export function assertSourceColorRevisionPreserved(before, after) {
  assert.deepEqual(after, before, 'Source color file/library revision, RGBA or canonical source changed during export');
}

/** Check every original native copy and descendant, including vector payloads
 * and text line caches. A save ACK or main UUID does not prove this invariant.
 * This check intentionally rejects geometry-changing style updates until their
 * owner supplies a separately reviewed layout migration.
 */
export function assertNativeInstanceGeometryPreserved(before, after) {
  assert.equal(after.id, before.id, 'Native copy verification needs the same actual file');
  const originalPages = before.data?.['pages-index'] ?? before.data?.pagesIndex;
  const actualPages = after.data?.['pages-index'] ?? after.data?.pagesIndex;
  assert.ok(originalPages && actualPages, 'Native copy verification requires realised page objects');
  const copies = [], nodes = [], changedNodes = [];
  let shapesChecked = 0;
  for (const [pageId, page] of Object.entries(originalPages)) {
    const original = page.objects, actual = actualPages[pageId]?.objects;
    assert.ok(original && actual, `Native copy page disappeared: ${pageId}`);
    const roots = new Set(Object.entries(original).filter(([, shape]) =>
      shapeField(shape, 'componentid') && !shapeField(shape, 'maininstance')).map(([id]) => id));
    const hasCopyAncestor = shape => {
      const parents = new Set(); let parentId = shapeField(shape, 'parentid'), found = false;
      while (parentId && original[parentId]) {
        assert.ok(!parents.has(parentId), `Cyclic native copy ancestry: ${parentId}`);
        parents.add(parentId);
        if (roots.has(parentId)) found = true;
        parentId = shapeField(original[parentId], 'parentid');
      }
      return found;
    };
    const visited = new Set();
    const visit = (id, rootId) => {
      assert.ok(!visited.has(id), `Duplicate or cyclic native copy child: ${id}`);
      visited.add(id);
      const expected = original[id], saved = actual[id];
      assert.ok(expected, `Original native copy shape is missing: ${id}`);
      const protectedStatePreserved = !!saved && isDeepStrictEqual(nativeShapeGeometry(saved), nativeShapeGeometry(expected));
      const node = { pageId, rootId, id, shapeRef: shapeField(expected, 'shaperef') ?? null,
        beforeSha256: nativeValueHash(expected), afterSha256: saved ? nativeValueHash(saved) : null,
        protectedStatePreserved };
      nodes.push(node);
      if (!saved || !isDeepStrictEqual(expected, saved)) changedNodes.push({ ...node,
        fields: nativeFieldDelta(expected, saved ?? {}) });
      shapesChecked++;
      const children = shapeField(expected, 'shapes');
      if (children !== undefined) {
        assert.ok(Array.isArray(children), `Native children are not realised: ${id}`);
        for (const child of children) visit(child, rootId);
      }
    };
    for (const [id, shape] of Object.entries(original)) {
      if (!roots.has(id) || hasCopyAncestor(shape)) continue;
      copies.push({ pageId, rootId: id }); visit(id, id);
    }
  }
  const report = { schema: 'zircon.penpot-native-copy-delta.v1', fileId: before.id,
    before: { revn: before.revn ?? null, vern: before.vern ?? null },
    after: { revn: after.revn ?? null, vern: after.vern ?? null },
    hashFormat: 'sha256/node-v8.serialize', nodeVersion: process.versions.node, v8Version: process.versions.v8,
    copies, shapesChecked, nodes, changedNodes,
    protectedStatePreserved: nodes.every(node => node.protectedStatePreserved) };
  if (!report.protectedStatePreserved) {
    const failures = nodes.filter(node => !node.protectedStatePreserved);
    throw Object.assign(new Error(`Native copy geometry, source reference or local override changed: ${failures[0].id} (${failures.length} nodes)`),
      { nativeCopyDelta: report });
  }
  return report;
}

/** Admit one source-owned import into the explicitly assigned, existing page. */
export function verifySourceOwnerAppendAdmission(file, identity, append, context) {
  assert.ok(context.sourceClosure && context.existingPlugin && context.authenticatedStorage,
    'Append requires the original source closure, existing plugin and authenticated account');
  assert.ok(!context.previousReceipt && !context.editRequested && !context.fullWorkbench,
    'Source append cannot refresh, edit or borrow a managed LevelEditor case');
  assert.equal(append.fileId, identity.fileId, 'Append file differs from the assigned workspace');
  assert.equal(append.pageId, identity.pageId, 'Append page differs from the assigned workspace');
  assert.equal(file.id, identity.fileId);
  for (const key of ['revn', 'vern']) {
    assert.ok(Number.isSafeInteger(append[key]) && append[key] >= 0, `Append needs an exact ${key}`);
    assert.equal(file[key], append[key], `Append ${key} changed; reconcile the current owner first`);
  }
  assert.ok(file.data.pages.includes(identity.pageId), 'Append page is not in the actual file');
  assert.equal(persistedZuiMetadata(file, identity.pageId).length, 0,
    'Append requires a page without an existing official source-owned asset');
  return true;
}

/** Add new roots/mains without changing any previous object, page or shared style. */
export function assertExistingPenpotContentPreserved(before, after) {
  assert.equal(after.id, before.id);
  assert.deepEqual(after.data.pages, before.data.pages, 'Existing page order changed');
  for (const key of ['colors', 'typographies'])
    assert.deepEqual(after.data[key] ?? {}, before.data[key] ?? {}, `Existing library ${key} changed`);
  for (const [id, component] of Object.entries(before.data.components ?? {}))
    assert.deepEqual(after.data.components?.[id], component, `Existing component changed: ${id}`);
  const beforePages = before.data['pages-index'], afterPages = after.data['pages-index'];
  assert.deepEqual(Object.keys(afterPages).sort(), Object.keys(beforePages).sort(), 'Existing page set changed');
  for (const [pageId, page] of Object.entries(beforePages)) {
    const saved = afterPages[pageId];
    assert.ok(saved, `Existing page disappeared: ${pageId}`);
    const { objects: beforeObjects, ...beforePage } = page;
    const { objects: afterObjects, ...afterPage } = saved;
    assert.deepEqual(afterPage, beforePage, `Existing page metadata changed: ${pageId}`);
    for (const [id, shape] of Object.entries(beforeObjects)) {
      const actual = afterObjects[id];
      assert.ok(actual, `Existing shape disappeared: ${id}`);
      if (id !== '00000000-0000-0000-0000-000000000000') {
        assert.deepEqual(actual, shape, `Existing shape changed: ${id}`);
      } else {
        const { shapes: originalChildren = [], ...originalRoot } = shape;
        const { shapes: actualChildren = [], ...actualRoot } = actual;
        assert.deepEqual(actualRoot, originalRoot, 'Existing page root geometry or metadata changed');
        assert.deepEqual(actualChildren.filter(child => originalChildren.includes(child)), originalChildren,
          'Existing page root children changed');
      }
    }
  }
}

export function persistenceClaim(evidence) {
  return evidence.status === 'passed' && evidence.backend === 'real' && evidence.saveAcknowledged === true &&
    evidence.closedOriginalContext === true && evidence.newAuthenticatedContext === true &&
    evidence.reopenedSameFile === true && evidence.metadataMatched === true &&
    evidence.exportSemanticMatch === true && evidence.sourceFingerprintMatched === true;
}

/** Admit only the known empty capture markers for an actual UI cleanup.
 * This never substitutes for the exact post-save/fresh-context comparison.
 */
export function verifyReviewScrollCleanupAdmission(acknowledgedMetadata, currentMetadata) {
  assert.ok(Array.isArray(acknowledgedMetadata) && acknowledgedMetadata.length > 0);
  assert.ok(Array.isArray(currentMetadata) && currentMetadata.length === acknowledgedMetadata.length);
  const expected = structuredClone(currentMetadata), removals = [];
  for (let index = 0; index < expected.length; index++) {
    const original = acknowledgedMetadata[index]?.metadata, current = expected[index]?.metadata;
    assert.ok(original && current && typeof original === 'object' && typeof current === 'object');
    for (const key of ['review-scroll-captured-x', 'review-scroll-captured-y', 'review-scroll-captured-positions']) {
      assert.ok(!Object.hasOwn(original, key), 'Cleanup requires an acknowledged snapshot without capture markers');
      if (Object.hasOwn(current, key)) {
        assert.equal(current[key], '', 'Only the known additive empty capture markers may be cleaned up');
        removals.push({ boardId: expected[index].boardId, key });
        delete current[key];
      }
    }
  }
  assert.deepEqual(expected, acknowledgedMetadata, 'The existing file has changes outside the known empty scroll markers');
  return removals;
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
