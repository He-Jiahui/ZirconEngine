import { parse, stringify, type TomlTable } from 'smol-toml';
import {
  cloneZuiDocument,
  cloneZuiValue,
  parseZuiDocument,
  serializeZuiDocument,
  type ZuiDocument,
  type ZuiNode,
  type ZuiChildMount,
} from './zui-document';
import type { PenpotAssetSnapshot } from './penpot-projection-model';
import {
  ZUI_SOURCE_EXPORTS_SCHEMA,
  ZUI_SOURCE_EXPORTS_VERSION,
  isSupportedVisualPath,
  type ZuiSourceExportEdit,
  type ZuiSourceExportIdentity,
  type ZuiSourceExportManifest,
  type ZuiSourceFingerprint,
  validateSourcePath,
} from './zui-export-apply-contract';
import type { ReconciledZuiDocument } from './penpot-projection-model';

type RecordValue = Record<string, unknown>;
const record = (value: unknown): value is RecordValue =>
  value !== null &&
  typeof value === 'object' &&
  !Array.isArray(value) &&
  !(value instanceof Date);
const same = (a: unknown, b: unknown): boolean => {
  if (Object.is(a, b)) return true;
  if (a instanceof Date && b instanceof Date)
    return a.getTime() === b.getTime();
  if (Array.isArray(a) && Array.isArray(b))
    return (
      a.length === b.length && a.every((value, index) => same(value, b[index]))
    );
  if (record(a) && record(b)) {
    const keys = Object.keys(a);
    return (
      keys.length === Object.keys(b).length &&
      keys.every((key) => Object.hasOwn(b, key) && same(a[key], b[key]))
    );
  }
  return false;
};

export function originalZuiSource(projection: ZuiDocument): string | undefined {
  const source = projection['penpot_original_source'];
  if (source === undefined) return undefined;
  if (typeof source !== 'string')
    throw new Error('Original ZUI source must be text');
  const original = parse(source, { integersAsBigInt: 'asNeeded' });
  if (
    !record(original['asset']) ||
    original['asset']['id'] !== projection.asset.id
  )
    throw new Error('Original ZUI source identity differs from its projection');
  return source;
}

export function restoreOriginalZuiSource(
  projection: ZuiDocument,
  result: ReconciledZuiDocument,
  snapshot?: PenpotAssetSnapshot,
): ReconciledZuiDocument {
  const source = originalZuiSource(projection);
  if (source === undefined) return result;
  const raw = parse(source, { integersAsBigInt: 'asNeeded' });
  const nativeV2 =
    record(raw['asset']) &&
    raw['asset']['version'] === 2 &&
    !['layout', 'widget'].includes(String(raw['asset']['kind']));
  if (!result.changes.length)
    return {
      ...result,
      document: nativeV2 ? parseZuiDocument(source).document : result.document,
      source,
    };
  if (!nativeV2)
    throw new Error(
      'Legacy fixture edits require an explicit legacy source mapping',
    );
  const hostProjection = projection['penpot_review_host'];
  if (record(hostProjection)) {
    if (!snapshot)
      throw new Error(
        'Edited composed host export requires its captured source identity map; re-import the host before exporting.',
      );
    return exportComposedHostEdits(projection, source, hostProjection as ZuiDocument, result, snapshot);
  }
  const original = cloneZuiDocument(parseZuiDocument(source).document);
  for (const key of new Set([
    ...Object.keys(projection),
    ...Object.keys(result.document),
  ])) {
    if (key !== 'nodes' && !same(projection[key], result.document[key]))
      throw new Error(`Unmapped source document edit: ${key}`);
  }
  const before = projection.nodes ?? {},
    after = result.document.nodes ?? {};
  for (const id of new Set([...Object.keys(before), ...Object.keys(after)])) {
    if (same(before[id], after[id])) continue;
    const target = original.nodes?.[id];
    if (!before[id] || !after[id] || !target)
      throw new Error(
        `Expanded prefab node edit requires an explicit source mapping: ${id}`,
      );
    for (const key of new Set([
      ...Object.keys(before[id]),
      ...Object.keys(after[id]),
    ])) {
      if (same(before[id][key], after[id][key])) continue;
      if (key === 'children') {
        applyMountDeltas(
          id,
          target,
          before[id].children ?? [],
          after[id].children ?? [],
        );
        continue;
      }
      if (!['props', 'layout', 'style'].includes(key))
        throw new Error(`Unmapped source node edit: ${id}.${key}`);
      target[key] = applyDelta(target[key], before[id][key], after[id][key]);
    }
  }
  return {
    ...result,
    document: original,
    source: serializeZuiDocument(original),
  };
}

function exportComposedHostEdits(
  projection: ZuiDocument,
  rootSource: string,
  baselineProjection: ZuiDocument,
  result: ReconciledZuiDocument,
  snapshot: PenpotAssetSnapshot,
): ReconciledZuiDocument {
  const rootSourcePath = stringValue(projection['penpot_original_source_path']);
  if (!rootSourcePath)
    throw new Error(
      'Edited composed host export is missing penpot_original_source_path; re-import with its canonical owner path.',
    );
  validateSourcePath(rootSourcePath);
  if (parse(rootSource)['penpot_source_exports'] !== undefined)
    throw new Error(
      'Canonical source already declares penpot_source_exports; remove the reserved export metadata before applying.',
    );
  const expectedValue = projection['penpot_source_fingerprints'];
  const sources = normalizeFingerprints(expectedValue, rootSourcePath);
  const identityMap = collectSourceIdentities(
    baselineProjection,
    snapshot,
    rootSourcePath,
  );
  const baseline = cloneZuiDocument(baselineProjection);
  const exported = cloneZuiDocument(result.document);
  const edits = collectProjectionEdits(baseline, exported, identityMap);
  if (!edits.length)
    return {
      ...result,
      document: parseZuiDocument(rootSource).document,
      source: rootSource,
    };
  const manifest: ZuiSourceExportManifest = {
    schema: ZUI_SOURCE_EXPORTS_SCHEMA,
    version: ZUI_SOURCE_EXPORTS_VERSION,
    rootSourcePath,
    baselineProjection: serializeZuiDocument(baseline),
    exportedProjection: serializeZuiDocument(exported),
    identityMap,
    sources,
    edits,
  };
  const envelope = JSON.stringify(manifest);
  const encoded = stringify({ penpot_source_exports: envelope } as unknown as TomlTable)
    .trimEnd();
  const source = insertRootTomlAssignment(rootSource, encoded);
  const exportedRoot = parseZuiDocument(source).document;
  exportedRoot['penpot_source_exports'] = envelope;
  return {
    ...result,
    document: exportedRoot,
    source,
    diagnostics: [
      ...result.diagnostics,
      {
        severity: 'info',
        code: 'penpot-source-owner-map-exported',
        message: `${edits.length} visual field changes include source-owner metadata for ${new Set(edits.map(({ sourcePath }) => sourcePath)).size} source asset(s).`,
      },
    ],
  };
}

function insertRootTomlAssignment(source: string, assignment: string): string {
  const bom = source.startsWith('\uFEFF') ? '\uFEFF' : '';
  const body = source.slice(bom.length);
  const firstTable = /^[ \t]*\[[^\r\n]*\][ \t]*(?:#.*)?$/m.exec(body);
  const index = firstTable?.index ?? body.length;
  const before = body.slice(0, index);
  const after = body.slice(index);
  const newline = body.includes('\r\n') ? '\r\n' : '\n';
  const separator = before.endsWith('\n') ? '' : newline;
  return `${bom}${before}${separator}${assignment}${newline}${after}`;
}

function normalizeFingerprints(
  value: unknown,
  rootSourcePath: string,
): ZuiSourceFingerprint[] {
  if (!Array.isArray(value))
    throw new Error('penpot_source_fingerprints must be an array of source paths and hashes');
  const fingerprints: ZuiSourceFingerprint[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    if (!record(item) || typeof item['sourcePath'] !== 'string' || typeof item['sha256'] !== 'string')
      throw new Error(`penpot_source_fingerprints[${index}] must declare sourcePath and sha256`);
    validateSourcePath(item['sourcePath']);
    if (seen.has(item['sourcePath']))
      throw new Error(`penpot_source_fingerprints contains duplicate ${item['sourcePath']}`);
    if (!/^[\da-f]{64}$/i.test(item['sha256']))
      throw new Error(`penpot_source_fingerprints[${index}].sha256 must be a SHA-256 digest`);
    seen.add(item['sourcePath']);
    fingerprints.push({ sourcePath: item['sourcePath'], sha256: item['sha256'].toLowerCase() });
  }
  if (!seen.has(rootSourcePath))
    throw new Error(`penpot_source_fingerprints must include root source ${rootSourcePath}`);
  return fingerprints;
}

function collectSourceIdentities(
  baselineProjection: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
  rootSourcePath: string,
): ZuiSourceExportIdentity[] {
  const shapes = new Map(snapshot.shapes.map((shape) => [shape.nodeId, shape]));
  const result: ZuiSourceExportIdentity[] = [];
  for (const [exportNodeId, node] of Object.entries(baselineProjection.nodes ?? {})) {
    const shape = shapes.get(exportNodeId);
    const sourcePath =
      shape?.sourcePath ?? stringValue(node['penpot_review_source_path']);
    const sourceNodeId =
      shape?.sourceNodeId ?? stringValue(node['penpot_review_source_node_id']);
    const instancePath =
      shape?.instancePath ?? stringValue(node['penpot_review_instance_path']);
    const controlId = shape && Object.hasOwn(shape, 'controlId')
      ? shape.controlId ?? null
      : typeof node.control_id === 'string'
        ? node.control_id
        : null;
    if (!sourcePath || !sourceNodeId || !instancePath || !shape)
      throw new Error(
        `Host projection node ${exportNodeId} has no captured source ownership identity; re-import its source provenance.`,
      );
    validateSourcePath(sourcePath);
    validateCompactInstancePath(instancePath, exportNodeId);
    result.push({
      exportNodeId,
      sourcePath,
      sourceNodeId,
      controlId,
      instancePath,
    });
  }
  if (shapes.size !== result.length)
    throw new Error(
      'Penpot semantic shape identity map does not match the host projection node set; re-import the host before exporting.',
    );
  return result;
}

function collectProjectionEdits(
  before: ZuiDocument,
  after: ZuiDocument,
  identityMap: ZuiSourceExportIdentity[],
): ZuiSourceExportEdit[] {
  const identityByNode = new Map(
    identityMap.map((identity) => [identity.exportNodeId, identity]),
  );
  const changes = sourceChanges(before, after);
  return changes.map(({ path, value, exists }) => {
    const [root, exportNodeId, ...fieldPath] = path;
    if (
      root !== 'nodes' ||
      !exportNodeId ||
      !exists ||
      !isSupportedVisualPath(path)
    )
      throw new Error(
        `Composed host export changed nonvisual or removed data at ${path.join('.')}; re-import the host and edit a supported visual field.`,
      );
    const identity = identityByNode.get(exportNodeId);
    if (!identity)
      throw new Error(
        `Changed host node ${exportNodeId} has no captured source owner; re-import the host before exporting.`,
      );
    return { ...identity, fieldPath, value };
  });
}

function sourceChanges(
  before: unknown,
  after: unknown,
  path: string[] = [],
): Array<{ path: string[]; value: unknown; exists: boolean }> {
  if (same(before, after)) return [];
  if (record(before) && record(after)) {
    const changes: Array<{ path: string[]; value: unknown; exists: boolean }> = [];
    for (const key of new Set([...Object.keys(before), ...Object.keys(after)]))
      changes.push(
        ...sourceChanges(before[key], after[key], [...path, key]),
      );
    return changes;
  }
  return [{ path, value: after, exists: after !== undefined }];
}

function validateCompactInstancePath(value: string, nodeId: string): void {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    throw new Error(`Host node ${nodeId} has invalid instancePath JSON`);
  }
  if (!Array.isArray(parsed) || JSON.stringify(parsed) !== value)
    throw new Error(`Host node ${nodeId} instancePath must be compact canonical JSON`);
  for (const [index, step] of parsed.entries()) {
    if (
      !record(step) ||
      Object.keys(step).length !== 2 ||
      Object.keys(step)[0] !== 'sourcePath' ||
      Object.keys(step)[1] !== 'sourceNodeId' ||
      !stringValue(step['sourcePath']) ||
      !stringValue(step['sourceNodeId'])
    )
      throw new Error(`Host node ${nodeId} instancePath[${index}] is invalid`);
    validateSourcePath(step['sourcePath'] as string);
  }
}

function stringValue(value: unknown): string | undefined {
  return typeof value === 'string' && value.trim() ? value : undefined;
}

function applyMountDeltas(
  id: string,
  target: ZuiNode,
  before: ZuiChildMount[],
  after: ZuiChildMount[],
): void {
  if (
    before.length !== after.length ||
    before.some((mount, index) => mount.node !== after[index].node)
  )
    throw new Error(
      `Source hierarchy edit requires an explicit mapping: ${id}`,
    );
  for (const [index, mount] of before.entries()) {
    if (same(mount, after[index])) continue;
    const originalIndex =
      target.children?.findIndex((child) => child.node === mount.node) ?? -1;
    if (originalIndex < 0 || !target.children)
      throw new Error(
        `Expanded prefab slot edit requires an explicit source mapping: ${id}/${mount.node}`,
      );
    target.children[originalIndex] = applyDelta(
      target.children[originalIndex],
      mount,
      after[index],
    ) as ZuiChildMount;
  }
}

function applyDelta(target: unknown, before: unknown, after: unknown): unknown {
  if (!record(before) || !record(after)) return cloneZuiValue(after);
  const result = record(target) ? { ...target } : {};
  for (const key of new Set([...Object.keys(before), ...Object.keys(after)])) {
    if (same(before[key], after[key])) continue;
    if (!Object.hasOwn(after, key)) delete result[key];
    else result[key] = applyDelta(result[key], before[key], after[key]);
  }
  return result;
}
