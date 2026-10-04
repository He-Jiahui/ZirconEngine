import { createHash, randomUUID } from 'node:crypto';
import { readFile, rename, rm, writeFile } from 'node:fs/promises';
import { basename, dirname, resolve } from 'node:path';
import { patchCanonicalZuiSource } from './zui-export-apply-source';
export { applyZuiExportFileToSourceRoot } from './zui-export-apply-files';
export { applyZuiExportToSourceFiles } from './zui-export-apply-sources';
export {
  ZUI_SOURCE_EXPORTS_SCHEMA,
  ZUI_SOURCE_EXPORTS_VERSION,
} from './zui-export-apply-contract';
export type {
  AppliedZuiSourceExports,
  AppliedZuiSourceExportOutput,
  ApplyZuiExportFileToSourceRootOptions,
  ZuiSourceExportEdit,
  ZuiSourceExportIdentity,
  ZuiSourceExportManifest,
  ZuiSourceFingerprint,
} from './zui-export-apply-contract';
import {
  cloneZuiDocument,
  ZuiDocumentError,
  parseZuiDocument,
  type ZuiDiagnostic,
  type ZuiDocument,
} from './zui-document';

export interface ApplyZuiExportOptions {
  expectedCanonicalSha256: string;
}

export interface AppliedZuiExport {
  source: string;
  appliedPaths: string[];
  diagnostics: ZuiDiagnostic[];
}

export interface ApplyZuiExportFileOptions extends ApplyZuiExportOptions {
  exportedPath: string;
  canonicalPath: string;
  outputPath?: string;
}

interface DocumentChange {
  path: Array<string | number>;
  afterExists: boolean;
  after: unknown;
}

const VISUAL_PROP_KEYS = new Set([
  'background',
  'background_color',
  'border',
  'border_color',
  'border_width',
  'color',
  'corner_radius',
  'disabled_foreground_color',
  'fg',
  'font',
  'font_family',
  'font_size',
  'font_weight',
  'foreground',
  'foreground_color',
  'label_color',
  'line_height',
  'line_height_ratio',
  'opacity',
  'outline',
  'radius',
  'selected_foreground_color',
  'tab_font_size',
  'tab_line_height',
  'text_align',
  'text_font_weight',
]);

const PAINT_STYLE_SUBKEYS = new Set([
  'align',
  'color',
  'family',
  'line_height',
  'line_height_ratio',
  'radius',
  'size',
  'weight',
  'width',
]);

const COLOR_PROP_KEYS = new Set([
  'background',
  'background_color',
  'border',
  'border_color',
  'color',
  'disabled_foreground_color',
  'fg',
  'foreground',
  'foreground_color',
  'label_color',
  'outline',
  'selected_foreground_color',
]);

/** Apply a downloaded Penpot .zui result as visual deltas onto its canonical source. */
export function applyZuiExportToCanonicalSource(
  exportedSource: string,
  canonicalSource: string,
  options: ApplyZuiExportOptions,
): AppliedZuiExport {
  assertExpectedHash(options.expectedCanonicalSha256);
  const actualHash = sha256(canonicalSource);
  if (actualHash !== options.expectedCanonicalSha256.toLowerCase()) {
    throw applyError(
      'canonical-source-stale',
      `Canonical source hash changed: expected ${options.expectedCanonicalSha256.toLowerCase()}, found ${actualHash}. Re-import the current source and export again.`,
    );
  }

  const canonical = parseZuiDocument(canonicalSource);
  const exported = parseZuiDocument(exportedSource);
  assertAssetIdentity(canonical.document, exported.document);
  assertStableNodeSet(canonical.document, exported.document);

  const changes = collectDocumentChanges(canonical.document, exported.document);
  if (changes.length === 0) {
    return {
      source: canonicalSource,
      appliedPaths: [],
      diagnostics: [...canonical.diagnostics, ...exported.diagnostics],
    };
  }

  for (const change of changes) {
    if (!isSupportedVisualPath(change.path)) {
      const displayPath = change.path.map(String).join('.');
      const nodeId =
        change.path[0] === 'nodes' && typeof change.path[1] === 'string'
          ? change.path[1]
          : undefined;
      const knownNode = nodeId && canonical.document.nodes?.[nodeId];
      if (nodeId && !knownNode) {
        throw applyError(
          'expanded-node-unmapped',
          `Expanded prefab node edit requires an explicit source mapping: ${nodeId}`,
          displayPath,
        );
      }
      throw applyError(
        'unsupported-nonvisual-export-change',
        `Unsupported nonvisual export change: ${displayPath}`,
        displayPath,
      );
    }
    if (!change.afterExists) {
      throw applyError(
        'visual-property-removal-unsupported',
        `Removing mapped visual properties is not supported: ${change.path.map(String).join('.')}`,
        change.path.map(String).join('.'),
      );
    }
  }

  const updatedDocument = cloneZuiDocument(canonical.document);
  for (const change of changes) {
    setDocumentValue(
      updatedDocument,
      change.path,
      change.after,
      change.afterExists,
    );
  }

  const source = patchCanonicalZuiSource(
    canonicalSource,
    canonical.document,
    exported.document,
    changes.map(({ path }) => path),
  );
  const reparsed = parseZuiDocument(source);
  if (!same(reparsed.document, updatedDocument)) {
    throw applyError(
      'source-patch-verification-failed',
      'The edited source could not be mapped back to the canonical TOML without changing unrelated fields. No output was written.',
    );
  }

  return {
    source,
    appliedPaths: changes.map(({ path }) => path.map(String).join('.')),
    diagnostics: [
      ...canonical.diagnostics,
      ...exported.diagnostics,
      {
        severity: 'info',
        code: 'canonical-visual-edits-applied',
        message: `${changes.length} supported visual source fields were applied.`,
      },
    ],
  };
}

/** Apply a downloaded export to an explicit path, using a sibling temp file for atomic replacement. */
export async function applyZuiExportFileToCanonicalSource(
  options: ApplyZuiExportFileOptions,
): Promise<AppliedZuiExport> {
  const [exportedSource, canonicalSource] = await Promise.all([
    readFile(options.exportedPath, 'utf8'),
    readFile(options.canonicalPath, 'utf8'),
  ]);
  const result = applyZuiExportToCanonicalSource(
    exportedSource,
    canonicalSource,
    options,
  );
  const targetPath = resolve(options.outputPath ?? options.canonicalPath);
  const canonicalPath = resolve(options.canonicalPath);
  const expectedHash = options.expectedCanonicalSha256.toLowerCase();
  const temporaryPath = resolve(
    dirname(targetPath),
    `.${basename(targetPath)}.zui-apply-${process.pid}-${randomUUID()}.tmp`,
  );

  await writeFile(temporaryPath, result.source, {
    encoding: 'utf8',
    flag: 'wx',
  });
  try {
    const currentCanonical = await readFile(options.canonicalPath, 'utf8');
    if (sha256(currentCanonical) !== expectedHash) {
      throw applyError(
        'canonical-source-stale',
        'Canonical source changed while the export was being prepared. No output was written.',
      );
    }
    await rename(temporaryPath, targetPath);
  } catch (error) {
    await rm(temporaryPath, { force: true });
    throw error;
  }

  if (samePath(targetPath, canonicalPath)) {
    const written = await readFile(canonicalPath, 'utf8');
    if (written !== result.source) {
      throw applyError(
        'canonical-source-write-verification-failed',
        'The canonical source did not match the verified apply result after replacement.',
      );
    }
  }
  return result;
}

function assertExpectedHash(value: string): void {
  if (!/^[\da-f]{64}$/i.test(value)) {
    throw applyError(
      'expected-canonical-hash-invalid',
      'An expected canonical SHA-256 hash is required (64 hexadecimal characters).',
    );
  }
}

function assertAssetIdentity(
  canonical: ZuiDocument,
  exported: ZuiDocument,
): void {
  if (
    canonical.asset.id !== exported.asset.id ||
    canonical.asset.kind !== exported.asset.kind ||
    canonical.asset.version !== exported.asset.version
  ) {
    throw applyError(
      'asset-identity-changed',
      `Asset identity changed: expected ${canonical.asset.kind} ${canonical.asset.id} v${canonical.asset.version}, found ${exported.asset.kind} ${exported.asset.id} v${exported.asset.version}.`,
    );
  }
}

function assertStableNodeSet(
  canonical: ZuiDocument,
  exported: ZuiDocument,
): void {
  const before = Object.keys(canonical.nodes ?? {});
  const after = Object.keys(exported.nodes ?? {});
  const beforeSet = new Set(before);
  const afterSet = new Set(after);
  const added = after.filter((nodeId) => !beforeSet.has(nodeId));
  if (added.length > 0) {
    throw applyError(
      'expanded-node-unmapped',
      `Expanded prefab node edit requires an explicit source mapping: ${added.join(', ')}`,
    );
  }
  const removed = before.filter((nodeId) => !afterSet.has(nodeId));
  if (removed.length > 0) {
    throw applyError(
      'source-node-removed',
      `Canonical source nodes cannot be removed by an exported design edit: ${removed.join(', ')}`,
    );
  }
}

function collectDocumentChanges(
  before: unknown,
  after: unknown,
  path: Array<string | number> = [],
): DocumentChange[] {
  if (same(before, after)) return [];
  const beforeRecord = isRecord(before)
    ? before
    : before === undefined
      ? {}
      : undefined;
  const afterRecord = isRecord(after)
    ? after
    : after === undefined
      ? {}
      : undefined;
  if (beforeRecord && afterRecord) {
    const changes: DocumentChange[] = [];
    for (const key of new Set([
      ...Object.keys(beforeRecord),
      ...Object.keys(afterRecord),
    ])) {
      changes.push(
        ...collectDocumentChanges(beforeRecord[key], afterRecord[key], [
          ...path,
          key,
        ]),
      );
    }
    return changes.length > 0 ? changes : [{ path, afterExists: true, after }];
  }
  return [{ path, afterExists: after !== undefined, after }];
}

function isSupportedVisualPath(path: Array<string | number>): boolean {
  if (
    path[0] !== 'nodes' ||
    typeof path[1] !== 'string' ||
    !['layout', 'props', 'style'].includes(String(path[2]))
  )
    return false;

  const section = path[2];
  const propertyPath = path.slice(3);
  if (
    propertyPath.length === 0 ||
    propertyPath.some((part) => typeof part === 'number')
  )
    return false;

  if (section === 'layout') return isMappedLayoutPath(propertyPath);
  if (section === 'style') {
    return (
      propertyPath[0] === 'self' && isMappedPaintPath(propertyPath.slice(1))
    );
  }
  return isMappedPaintPath(propertyPath);
}

function isMappedLayoutPath(path: Array<string | number>): boolean {
  if (path.some((part) => typeof part !== 'string')) return false;
  if (['width', 'height'].includes(String(path[0])) && path.length === 2)
    return ['min', 'preferred', 'max', 'stretch'].includes(String(path[1]));
  if (path[0] === 'position' && path.length === 2)
    return ['x', 'y'].includes(String(path[1]));
  if (path[0] === 'padding' && path.length === 2)
    return ['top', 'right', 'bottom', 'left'].includes(String(path[1]));
  if (path[0] === 'clip' && path.length === 1) return true;
  return (
    path[0] === 'container' &&
    path.length === 2 &&
    [
      'kind',
      'gap',
      'row_gap',
      'column_gap',
      'horizontal_gap',
      'vertical_gap',
      'columns',
      'rows',
      'align_items',
      'justify_content',
    ].includes(String(path[1]))
  );
}

function isMappedPaintPath(path: Array<string | number>): boolean {
  if (path.some((part) => typeof part !== 'string')) return false;
  const property = String(path[0]);
  if (VISUAL_PROP_KEYS.has(property) && path.length === 1) return true;
  if (path.length !== 2) return false;
  const subproperty = String(path[1]);
  if (property === 'font') return PAINT_STYLE_SUBKEYS.has(subproperty);
  if (property === 'border')
    return ['color', 'radius', 'width'].includes(subproperty);
  return COLOR_PROP_KEYS.has(property) && subproperty === 'color';
}

function setDocumentValue(
  document: ZuiDocument,
  path: Array<string | number>,
  value: unknown,
  exists: boolean,
): void {
  let parent: unknown = document;
  for (const segment of path.slice(0, -1)) {
    if (typeof segment === 'number') {
      if (!Array.isArray(parent) || !parent[segment])
        throw applyError(
          'source-path-invalid',
          `Cannot locate exported source path ${path.map(String).join('.')}.`,
        );
      parent = parent[segment];
      continue;
    }
    if (!isRecord(parent))
      throw applyError(
        'source-path-invalid',
        `Cannot locate exported source path ${path.map(String).join('.')}.`,
      );
    if (parent[segment] === undefined) parent[segment] = {};
    parent = parent[segment];
  }
  const leaf = path.at(-1);
  if (typeof leaf === 'number') {
    if (!Array.isArray(parent))
      throw applyError(
        'source-path-invalid',
        `Cannot locate exported source path ${path.map(String).join('.')}.`,
      );
    if (exists) parent[leaf] = value;
    else parent.splice(leaf, 1);
    return;
  }
  if (!isRecord(parent))
    throw applyError(
      'source-path-invalid',
      `Cannot locate exported source path ${path.map(String).join('.')}.`,
    );
  if (exists) parent[leaf!] = value;
  else delete parent[leaf!];
}

function sha256(source: string): string {
  return createHash('sha256').update(source, 'utf8').digest('hex');
}

function samePath(left: string, right: string): boolean {
  const normalizedLeft = resolve(left);
  const normalizedRight = resolve(right);
  return process.platform === 'win32'
    ? normalizedLeft.toLowerCase() === normalizedRight.toLowerCase()
    : normalizedLeft === normalizedRight;
}

function applyError(
  code: string,
  message: string,
  path?: string,
): ZuiDocumentError {
  const diagnostic: ZuiDiagnostic = {
    severity: 'error',
    code,
    message,
    ...(path ? { path } : {}),
  };
  return new ZuiDocumentError(message, [diagnostic]);
}

function same(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true;
  if (left instanceof Date && right instanceof Date)
    return left.getTime() === right.getTime();
  if (Array.isArray(left) && Array.isArray(right))
    return (
      left.length === right.length &&
      left.every((value, index) => same(value, right[index]))
    );
  if (isRecord(left) && isRecord(right)) {
    const keys = Object.keys(left);
    return (
      keys.length === Object.keys(right).length &&
      keys.every(
        (key) => Object.hasOwn(right, key) && same(left[key], right[key]),
      )
    );
  }
  return false;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return (
    value !== null &&
    typeof value === 'object' &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}
