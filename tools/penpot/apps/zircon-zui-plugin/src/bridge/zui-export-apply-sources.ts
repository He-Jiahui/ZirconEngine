import { createHash } from 'node:crypto';
import { patchCanonicalZuiSource } from './zui-export-apply-source';
import {
  cloneZuiDocument,
  parseZuiDocument,
  type ZuiDiagnostic,
  type ZuiDocument,
} from './zui-document';
import {
  assertFingerprintClosure,
  applyError,
  isRecord,
  isSupportedVisualPath,
  parseSourceExportManifest,
  same,
  staleSourceError,
  type AppliedZuiSourceExportOutput,
  type AppliedZuiSourceExports,
  type ZuiSourceExportEdit,
  type ZuiSourceExportManifest,
  type ZuiSourceFingerprint,
} from './zui-export-apply-contract';

export type {
  ApplyZuiExportFileToSourceRootOptions,
  AppliedZuiSourceExportOutput,
  AppliedZuiSourceExports,
  ZuiSourceExportEdit,
  ZuiSourceExportIdentity,
  ZuiSourceExportManifest,
  ZuiSourceFingerprint,
} from './zui-export-apply-contract';
export {
  ZUI_SOURCE_EXPORTS_SCHEMA,
  ZUI_SOURCE_EXPORTS_VERSION,
} from './zui-export-apply-contract';

interface DocumentChange {
  path: string[];
  after: unknown;
}

interface ResolvedEdit extends ZuiSourceExportEdit {
  fieldPathKey: string;
}

/** Apply and verify source-owner deltas in memory without filesystem writes. */
export function applyZuiExportToSourceFiles(
  exportedSource: string,
  canonicalSources: Readonly<Record<string, string>>,
  expectedSources: readonly ZuiSourceFingerprint[],
): AppliedZuiSourceExports {
  const exported = parseZuiDocument(exportedSource);
  const rawManifest = exported.document['penpot_source_exports'];
  if (rawManifest === undefined) {
    return {
      appliedPathsBySource: {},
      outputs: [],
      diagnostics: exported.diagnostics,
    };
  }
  const manifest = parseSourceExportManifest(rawManifest);
  assertFingerprintClosure(manifest.sources, expectedSources);
  for (const fingerprint of manifest.sources) {
    const source = canonicalSources[fingerprint.sourcePath];
    if (source === undefined)
      throw applyError(
        'source-missing',
        `Canonical source ${fingerprint.sourcePath} is missing from the explicit source set.`,
      );
    if (sha256(source) !== fingerprint.sha256)
      throw staleSourceError(fingerprint.sourcePath);
  }
  const canonicalKeys = Object.keys(canonicalSources);
  if (
    canonicalKeys.length !== manifest.sources.length ||
    canonicalKeys.some(
      (sourcePath) => !manifest.sources.some((item) => item.sourcePath === sourcePath),
    )
  )
    throw applyError(
      'source-set-mismatch',
      'The explicit canonical source set must match the export dependency closure exactly.',
    );

  const rootSource = canonicalSources[manifest.rootSourcePath];
  if (rootSource === undefined)
    throw applyError(
      'root-source-unlisted',
      `The canonical root source ${manifest.rootSourcePath} is missing from the fingerprint closure.`,
    );
  const exportedRoot = cloneZuiDocument(exported.document);
  delete exportedRoot['penpot_source_exports'];
  const canonicalRoot = parseZuiDocument(rootSource).document;
  if (!same(exportedRoot, canonicalRoot))
    throw applyError(
      'exported-root-source-mismatch',
      'The downloaded Penpot .zui body, after removing its source-export envelope, does not match the canonical root source asset.',
    );
  if (manifest.edits.length === 0)
    throw applyError(
      'source-export-empty',
      'A changed Penpot host projection must include at least one explicitly mapped visual edit.',
    );
  const baseline = parseZuiDocument(manifest.baselineProjection);
  const exportedProjection = parseZuiDocument(manifest.exportedProjection);
  assertAssetIdentity(baseline.document, exportedProjection.document);
  assertStableNodeSet(baseline.document, exportedProjection.document);
  validateProjectionIdentityMap(manifest, baseline.document, canonicalSources);

  const documentChanges = collectDocumentChanges(
    baseline.document,
    exportedProjection.document,
  );
  const resolvedEdits = resolveEdits(
    manifest,
    exportedProjection.document,
    documentChanges,
    canonicalSources,
  );
  const byOwner = new Map<string, ResolvedEdit[]>();
  for (const edit of resolvedEdits) {
    const entries = byOwner.get(edit.sourcePath) ?? [];
    entries.push(edit);
    byOwner.set(edit.sourcePath, entries);
  }

  const outputs: AppliedZuiSourceExportOutput[] = [];
  const appliedPathsBySource: Record<string, string[]> = {};
  const diagnostics: ZuiDiagnostic[] = [
    ...baseline.diagnostics,
    ...exportedProjection.diagnostics,
    ...exported.diagnostics,
  ];
  for (const [sourcePath, edits] of byOwner) {
    const originalSource = canonicalSources[sourcePath];
    const canonical = parseZuiDocument(originalSource);
    const updatedDocument = cloneZuiDocument(canonical.document);
    const uniqueEdits = deduplicateOwnerEdits(edits);
    const changedPaths: string[][] = [];
    for (const edit of uniqueEdits) {
      if (!updatedDocument.nodes?.[edit.sourceNodeId])
        throw applyError(
          'source-node-unmapped',
          `Canonical source ${sourcePath} has no raw node ${edit.sourceNodeId}.`,
        );
      const mappedPath = [
        'nodes',
        edit.sourceNodeId,
        ...edit.fieldPath,
      ];
      setDocumentValue(updatedDocument, mappedPath, edit.value);
      changedPaths.push(mappedPath);
      const displayPath = mappedPath.join('.');
      const paths = appliedPathsBySource[sourcePath] ?? [];
      if (!paths.includes(displayPath)) paths.push(displayPath);
      appliedPathsBySource[sourcePath] = paths;
    }
    const updatedSource = patchCanonicalZuiSource(
      originalSource,
      canonical.document,
      updatedDocument,
      changedPaths,
    );
    const reparsed = parseZuiDocument(updatedSource);
    if (!same(reparsed.document, updatedDocument))
      throw applyError(
        'source-patch-verification-failed',
        `Mapped edits for ${sourcePath} could not be written without changing unrelated source fields.`,
      );
    diagnostics.push(...canonical.diagnostics, ...reparsed.diagnostics);
    diagnostics.push({
      severity: 'info',
      code: 'canonical-owner-visual-edits-applied',
      message: `${changedPaths.length} visual fields were mapped to ${sourcePath}.`,
    });
    outputs.push({ sourcePath, outputPath: '', source: updatedSource });
  }

  return { appliedPathsBySource, outputs, diagnostics };
}

function resolveEdits(
  manifest: ZuiSourceExportManifest,
  exported: ZuiDocument,
  changes: DocumentChange[],
  canonicalSources: Readonly<Record<string, string>>,
): ResolvedEdit[] {
  const sourcePaths = new Set(manifest.sources.map(({ sourcePath }) => sourcePath));
  const identityByNode = new Map(
    manifest.identityMap.map((identity) => [identity.exportNodeId, identity]),
  );
  const editsByPath = new Map(
    manifest.edits.map((edit) => [
      JSON.stringify([edit.exportNodeId, ...edit.fieldPath]),
      edit,
    ]),
  );
  const used = new Set<string>();
  const resolved: ResolvedEdit[] = [];
  for (const change of changes) {
    const [root, exportNodeId, ...fieldPath] = change.path;
    const displayPath = change.path.join('.');
    if (
      root !== 'nodes' ||
      exportNodeId === undefined ||
      !isSupportedVisualPath(change.path)
    )
      throw applyError(
        'unsupported-nonvisual-export-change',
        `Unsupported nonvisual export change: ${displayPath}.`,
        displayPath,
      );
    const key = JSON.stringify([exportNodeId, ...fieldPath]);
    const edit = editsByPath.get(key);
    if (!edit)
      throw applyError(
        'source-export-unmapped-change',
        `Changed visual path has no owner mapping: ${displayPath}.`,
        displayPath,
      );
    if (!sourcePaths.has(edit.sourcePath))
      throw applyError(
        'source-export-owner-unlisted',
        `Source owner ${edit.sourcePath} is missing from the dependency fingerprint closure.`,
      );
    const identity = identityByNode.get(edit.exportNodeId);
    if (
      !identity ||
      identity.sourcePath !== edit.sourcePath ||
      identity.sourceNodeId !== edit.sourceNodeId ||
      identity.controlId !== edit.controlId ||
      identity.instancePath !== edit.instancePath
    )
      throw applyError(
        'source-export-owner-identity-mismatch',
        `Edit owner identity for ${edit.exportNodeId} does not match the captured projection identity map.`,
      );
    const ownerSource = canonicalSources[edit.sourcePath];
    const ownerDocument = parseZuiDocument(ownerSource).document;
    if (!ownerDocument.nodes?.[edit.sourceNodeId])
      throw applyError(
        'source-node-unmapped',
        `Canonical source ${edit.sourcePath} has no raw node ${edit.sourceNodeId}.`,
      );
    const value = getDocumentValue(exported, change.path);
    if (
      !value.exists ||
      !same(value.value, change.after) ||
      !same(edit.value, value.value)
    )
      throw applyError(
        'source-export-value-unmapped',
        `Mapped Penpot field ${displayPath} has no matching exported value.`,
      );
    used.add(key);
    resolved.push({ ...edit, fieldPathKey: JSON.stringify(edit.fieldPath) });
  }
  for (const [key, edit] of editsByPath) {
    if (!used.has(key))
      throw applyError(
        'source-export-edit-unmatched',
        `Source mapping ${edit.exportNodeId}.${edit.fieldPath.join('.')} does not correspond to a changed visual field.`,
      );
  }
  const byOwnerField = new Map<string, ResolvedEdit>();
  for (const edit of resolved) {
    const key = JSON.stringify([
      edit.sourcePath,
      edit.sourceNodeId,
      edit.fieldPath,
    ]);
    const previous = byOwnerField.get(key);
    if (previous && !same(previous.value, edit.value))
      throw applyError(
        'source-export-instance-conflict',
        `Conflicting Penpot instances changed the same source field ${edit.sourcePath}#${edit.sourceNodeId}.${edit.fieldPath.join('.')}.`,
      );
    if (!previous) byOwnerField.set(key, edit);
  }
  return resolved;
}

function validateProjectionIdentityMap(
  manifest: ZuiSourceExportManifest,
  baseline: ZuiDocument,
  canonicalSources: Readonly<Record<string, string>>,
): void {
  const nodeIds = Object.keys(baseline.nodes ?? {});
  const identityIds = manifest.identityMap.map(({ exportNodeId }) => exportNodeId);
  if (
    nodeIds.length !== identityIds.length ||
    nodeIds.some((nodeId) => !identityIds.includes(nodeId))
  )
    throw applyError(
      'source-export-identity-map-incomplete',
      'The source identity map must cover every authored node in the imported host projection.',
    );
  const sourcePathSet = new Set(manifest.sources.map(({ sourcePath }) => sourcePath));
  const parsedSources = new Map<string, ZuiDocument>();
  const sourceDocument = (sourcePath: string): ZuiDocument => {
    const existing = parsedSources.get(sourcePath);
    if (existing) return existing;
    const parsed = parseZuiDocument(canonicalSources[sourcePath]).document;
    parsedSources.set(sourcePath, parsed);
    return parsed;
  };
  for (const identity of manifest.identityMap) {
    if (!sourcePathSet.has(identity.sourcePath))
      throw applyError(
        'source-export-owner-unlisted',
        `Identity owner ${identity.sourcePath} is missing from the dependency fingerprint closure.`,
      );
    const owner = sourceDocument(identity.sourcePath);
    if (!owner.nodes?.[identity.sourceNodeId])
      throw applyError(
        'source-node-unmapped',
        `Canonical source ${identity.sourcePath} has no raw node ${identity.sourceNodeId}.`,
      );
    const projectedNode = baseline.nodes?.[identity.exportNodeId];
    const projectedSourcePath = projectedNode?.['penpot_review_source_path'];
    const projectedSourceNodeId =
      projectedNode?.['penpot_review_source_node_id'];
    const projectedInstancePath =
      projectedNode?.['penpot_review_instance_path'];
    const projectedControlId =
      typeof projectedNode?.control_id === 'string'
        ? projectedNode.control_id
        : null;
    if (
      projectedSourcePath !== identity.sourcePath ||
      projectedSourceNodeId !== identity.sourceNodeId ||
      projectedInstancePath !== identity.instancePath ||
      projectedControlId !== identity.controlId
    )
      throw applyError(
        'source-export-owner-identity-mismatch',
        `Owner identity for ${identity.exportNodeId} does not match the imported projection identity.`,
      );
    for (const step of JSON.parse(identity.instancePath) as Array<{
      sourcePath: string;
      sourceNodeId: string;
    }>) {
      if (!sourcePathSet.has(step.sourcePath))
        throw applyError(
          'source-export-instance-owner-unlisted',
          `Instance call site ${step.sourcePath} is missing from the dependency fingerprint closure.`,
        );
      const instanceOwner = sourceDocument(step.sourcePath);
      if (!instanceOwner.nodes?.[step.sourceNodeId])
        throw applyError(
          'source-export-instance-node-unmapped',
          `Canonical source ${step.sourcePath} has no instance call-site node ${step.sourceNodeId}.`,
        );
    }
  }
}

function deduplicateOwnerEdits(edits: readonly ResolvedEdit[]): ResolvedEdit[] {
  const unique = new Map<string, ResolvedEdit>();
  for (const edit of edits) {
    const key = JSON.stringify([edit.sourceNodeId, edit.fieldPath]);
    const previous = unique.get(key);
    if (previous && !same(previous.value, edit.value))
      throw applyError(
        'source-export-instance-conflict',
        `Conflicting Penpot instances changed the same source field ${edit.sourcePath}#${edit.sourceNodeId}.${edit.fieldPath.join('.')}.`,
      );
    if (!previous) unique.set(key, edit);
  }
  return [...unique.values()];
}

function collectDocumentChanges(
  before: unknown,
  after: unknown,
  path: string[] = [],
): DocumentChange[] {
  if (same(before, after)) return [];
  const beforeRecord = isRecord(before) ? before : before === undefined ? {} : undefined;
  const afterRecord = isRecord(after) ? after : after === undefined ? {} : undefined;
  if (beforeRecord && afterRecord) {
    const changes: DocumentChange[] = [];
    for (const key of new Set([
      ...Object.keys(beforeRecord),
      ...Object.keys(afterRecord),
    ]))
      changes.push(
        ...collectDocumentChanges(
          beforeRecord[key],
          afterRecord[key],
          [...path, key],
        ),
      );
    return changes.length > 0 ? changes : [{ path, after }];
  }
  return [{ path, after }];
}

function assertAssetIdentity(before: ZuiDocument, after: ZuiDocument): void {
  if (
    before.asset.id !== after.asset.id ||
    before.asset.kind !== after.asset.kind ||
    before.asset.version !== after.asset.version
  )
    throw applyError(
      'asset-identity-changed',
      'The exported projection changed its root asset identity or schema version.',
    );
}

function assertStableNodeSet(before: ZuiDocument, after: ZuiDocument): void {
  const beforeIds = Object.keys(before.nodes ?? {});
  const afterIds = Object.keys(after.nodes ?? {});
  const beforeSet = new Set(beforeIds);
  const afterSet = new Set(afterIds);
  const added = afterIds.filter((id) => !beforeSet.has(id));
  if (added.length)
    throw applyError(
      'expanded-node-unmapped',
      `Expanded or generated nodes must exist in the imported baseline before they can be mapped: ${added.join(', ')}.`,
    );
  const removed = beforeIds.filter((id) => !afterSet.has(id));
  if (removed.length)
    throw applyError(
      'source-node-removed',
      `Source nodes cannot be removed by an exported Penpot edit: ${removed.join(', ')}.`,
    );
}

function setDocumentValue(
  document: ZuiDocument,
  path: string[],
  value: unknown,
): void {
  let parent: unknown = document;
  for (const segment of path.slice(0, -1)) {
    if (!isRecord(parent))
      throw applyError(
        'source-path-invalid',
        `Cannot locate source path ${path.join('.')}.`,
      );
    if (parent[segment] === undefined) parent[segment] = {};
    parent = parent[segment];
  }
  if (!isRecord(parent))
    throw applyError(
      'source-path-invalid',
      `Cannot locate source path ${path.join('.')}.`,
    );
  parent[path.at(-1)!] = value;
}

function getDocumentValue(
  document: ZuiDocument,
  path: string[],
): { exists: boolean; value: unknown } {
  let value: unknown = document;
  for (const segment of path) {
    if (!isRecord(value) || !Object.hasOwn(value, segment))
      return { exists: false, value: undefined };
    value = value[segment];
  }
  return { exists: true, value };
}

function sha256(source: string): string {
  return createHash('sha256').update(source, 'utf8').digest('hex');
}
