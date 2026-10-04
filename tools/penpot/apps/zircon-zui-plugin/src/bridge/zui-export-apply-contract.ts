import { ZuiDocumentError, type ZuiDiagnostic } from './zui-document';

export const ZUI_SOURCE_EXPORTS_SCHEMA = 'dev.zircon.zui.penpot-source-exports';
export const ZUI_SOURCE_EXPORTS_VERSION = 1;

export interface ZuiSourceFingerprint {
  sourcePath: string;
  sha256: string;
}

export interface ZuiSourceExportEdit {
  exportNodeId: string;
  sourcePath: string;
  sourceNodeId: string;
  controlId: string | null;
  /** Compact JSON authored call-site list; `[]` identifies a direct root node. */
  instancePath: string;
  /** Node-relative TOML path, for example `['layout', 'container', 'gap']`. */
  fieldPath: string[];
  value: unknown;
}

export interface ZuiSourceExportIdentity {
  exportNodeId: string;
  sourcePath: string;
  sourceNodeId: string;
  controlId: string | null;
  instancePath: string;
}

export interface ZuiSourceExportManifest {
  schema: typeof ZUI_SOURCE_EXPORTS_SCHEMA;
  version: typeof ZUI_SOURCE_EXPORTS_VERSION;
  rootSourcePath: string;
  /** Serialized pre-edit host projection used to reject unlisted or contract edits. */
  baselineProjection: string;
  /** Serialized reconciled state from the edited Penpot host projection. */
  exportedProjection: string;
  /** Authored identity for every exported semantic node, captured before edits. */
  identityMap: ZuiSourceExportIdentity[];
  /** Complete owner and dependency closure captured before Penpot import. */
  sources: ZuiSourceFingerprint[];
  edits: ZuiSourceExportEdit[];
}

export interface ApplyZuiExportFileToSourceRootOptions {
  exportedPath: string;
  /** Trusted pre-import SHA-256 for the canonical root source asset. */
  expectedRootSha256: string;
  /** Canonical ZirconEngine checkout root used to resolve manifest source paths. */
  sourceRootPath: string;
  /** Separate staging root that receives only changed owner assets. */
  outputRootPath: string;
  /** Trusted source fingerprints captured before the Penpot import. */
  expectedSources: readonly ZuiSourceFingerprint[];
}

export interface AppliedZuiSourceExportOutput {
  sourcePath: string;
  outputPath: string;
  source: string;
}

export interface AppliedZuiSourceExports {
  appliedPathsBySource: Record<string, string[]>;
  outputs: AppliedZuiSourceExportOutput[];
  diagnostics: ZuiDiagnostic[];
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

export function parseSourceExportManifest(
  value: unknown,
): ZuiSourceExportManifest {
  if (typeof value !== 'string')
    throw applyError(
      'source-export-envelope-invalid',
      'The penpot_source_exports field must be a JSON string.',
    );
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch (error) {
    throw applyError(
      'source-export-envelope-invalid',
      `The Penpot source-export JSON is invalid: ${errorMessage(error)}`,
    );
  }
  if (!isRecord(parsed))
    throw applyError(
      'source-export-envelope-invalid',
      'The Penpot source-export envelope must be an object.',
    );
  assertExactKeys(parsed, [
    'schema',
    'version',
    'rootSourcePath',
    'baselineProjection',
    'exportedProjection',
    'identityMap',
    'sources',
    'edits',
  ]);
  if (parsed['schema'] !== ZUI_SOURCE_EXPORTS_SCHEMA)
    throw applyError(
      'source-export-schema-unsupported',
      `Unsupported Penpot source-export schema ${String(parsed['schema'])}.`,
    );
  if (parsed['version'] !== ZUI_SOURCE_EXPORTS_VERSION)
    throw applyError(
      'source-export-version-unsupported',
      `Unsupported Penpot source-export version ${String(parsed['version'])}.`,
    );
  const rootSourcePath = requireNonemptyString(
    parsed['rootSourcePath'],
    'source-export rootSourcePath',
  );
  validateSourcePath(rootSourcePath);
  if (typeof parsed['baselineProjection'] !== 'string')
    throw applyError(
      'source-export-envelope-invalid',
      'The Penpot source-export envelope must contain a serialized baselineProjection.',
    );
  if (typeof parsed['exportedProjection'] !== 'string')
    throw applyError(
      'source-export-envelope-invalid',
      'The Penpot source-export envelope must contain a serialized exportedProjection.',
    );
  const identityMap = parseIdentityMap(parsed['identityMap']);
  const sources = parseFingerprintList(
    parsed['sources'],
    'source-export sources',
  );
  const edits = parseEditList(parsed['edits']);
  if (sources.length === 0)
    throw applyError(
      'source-export-sources-empty',
      'The Penpot source-export envelope must include every owner and dependency source.',
    );
  return {
    schema: ZUI_SOURCE_EXPORTS_SCHEMA,
    version: ZUI_SOURCE_EXPORTS_VERSION,
    rootSourcePath,
    baselineProjection: parsed['baselineProjection'],
    exportedProjection: parsed['exportedProjection'],
    identityMap,
    sources,
    edits,
  };
}

export function parseFingerprintList(
  value: unknown,
  label: string,
): ZuiSourceFingerprint[] {
  if (!Array.isArray(value))
    throw applyError(
      'source-fingerprints-invalid',
      `${label} must be an array.`,
    );
  const result: ZuiSourceFingerprint[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    if (!isRecord(item))
      throw applyError(
        'source-fingerprints-invalid',
        `${label}[${index}] must be an object.`,
      );
    assertExactKeys(item, ['sourcePath', 'sha256']);
    const sourcePath = requireNonemptyString(
      item['sourcePath'],
      `${label}[${index}].sourcePath`,
    );
    validateSourcePath(sourcePath);
    const hash = requireNonemptyString(
      item['sha256'],
      `${label}[${index}].sha256`,
    );
    if (!/^[\da-f]{64}$/i.test(hash))
      throw applyError(
        'source-fingerprints-invalid',
        `${label}[${index}].sha256 must be a SHA-256 hex digest.`,
      );
    if (seen.has(sourcePath))
      throw applyError(
        'source-fingerprints-duplicate',
        `${label} contains duplicate source path ${sourcePath}.`,
      );
    seen.add(sourcePath);
    result.push({ sourcePath, sha256: hash.toLowerCase() });
  }
  return result;
}

export function assertFingerprintClosure(
  manifestSources: readonly ZuiSourceFingerprint[],
  expectedSources: readonly ZuiSourceFingerprint[],
): void {
  const expected = parseFingerprintList(expectedSources, 'expected sources');
  const actualByPath = new Map(
    manifestSources.map((item) => [item.sourcePath, item.sha256]),
  );
  const expectedByPath = new Map(
    expected.map((item) => [item.sourcePath, item.sha256]),
  );
  if (
    actualByPath.size !== expectedByPath.size ||
    [...actualByPath].some(
      ([sourcePath, hash]) => expectedByPath.get(sourcePath) !== hash,
    )
  )
    throw applyError(
      'source-baseline-mismatch',
      'The Penpot export source fingerprint closure differs from the trusted pre-import source baseline.',
    );
}

export function assertExpectedRootHash(
  expectedRootSha256: string,
  manifestSources: readonly ZuiSourceFingerprint[],
  rootSourcePath: string,
): void {
  if (!/^[\da-f]{64}$/i.test(expectedRootSha256))
    throw applyError(
      'expected-root-hash-invalid',
      'A trusted root-source SHA-256 hash is required for source-owner apply.',
    );
  const fingerprint = manifestSources.find(
    ({ sourcePath }) => sourcePath === rootSourcePath,
  );
  if (!fingerprint || fingerprint.sha256 !== expectedRootSha256.toLowerCase())
    throw applyError(
      'source-root-baseline-mismatch',
      `The expected root SHA-256 does not match the source-export fingerprint for ${rootSourcePath}.`,
    );
}

export function validateSourcePath(sourcePath: string): string[] {
  if (
    sourcePath.trim() !== sourcePath ||
    sourcePath.length === 0 ||
    sourcePath.includes('\\') ||
    sourcePath.includes(':') ||
    sourcePath.startsWith('/') ||
    /^[a-zA-Z]:/.test(sourcePath)
  )
    throw invalidSourcePath(sourcePath);
  const segments = sourcePath.split('/');
  if (
    segments.some(
      (segment) => segment.length === 0 || segment === '.' || segment === '..',
    ) ||
    !sourcePath.toLowerCase().endsWith('.zui') ||
    !containsUiAssetDirectory(segments)
  )
    throw invalidSourcePath(sourcePath);
  return segments;
}

export function isSupportedVisualPath(path: string[]): boolean {
  if (
    path[0] !== 'nodes' ||
    !path[1] ||
    !['layout', 'props', 'style'].includes(path[2]) ||
    path.length < 4
  )
    return false;
  const propertyPath = path.slice(3);
  if (path[2] === 'layout') return isMappedLayoutPath(propertyPath);
  if (path[2] === 'style')
    return (
      propertyPath[0] === 'self' && isMappedPaintPath(propertyPath.slice(1))
    );
  return isMappedPaintPath(propertyPath);
}

export function applyError(
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

export function isRecord(value: unknown): value is Record<string, unknown> {
  return (
    value !== null &&
    typeof value === 'object' &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

export function same(left: unknown, right: unknown): boolean {
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

export function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function staleSourceError(sourcePath: string): ZuiDocumentError {
  return applyError(
    'canonical-source-stale',
    `Canonical source ${sourcePath} is stale compared with the pre-import fingerprint. Re-import the current sources and export again.`,
  );
}

function parseIdentityMap(value: unknown): ZuiSourceExportIdentity[] {
  if (!Array.isArray(value))
    throw applyError(
      'source-export-identities-invalid',
      'identityMap must be an array.',
    );
  const result: ZuiSourceExportIdentity[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    if (!isRecord(item))
      throw applyError(
        'source-export-identities-invalid',
        `identityMap[${index}] must be an object.`,
      );
    assertExactKeys(item, [
      'exportNodeId',
      'sourcePath',
      'sourceNodeId',
      'controlId',
      'instancePath',
    ]);
    const exportNodeId = requireNonemptyString(
      item['exportNodeId'],
      `identityMap[${index}].exportNodeId`,
    );
    const sourcePath = requireNonemptyString(
      item['sourcePath'],
      `identityMap[${index}].sourcePath`,
    );
    validateSourcePath(sourcePath);
    const sourceNodeId = requireNonemptyString(
      item['sourceNodeId'],
      `identityMap[${index}].sourceNodeId`,
    );
    const controlId = item['controlId'];
    if (controlId !== null && typeof controlId !== 'string')
      throw applyError(
        'source-export-identities-invalid',
        `identityMap[${index}].controlId must be a string or null.`,
      );
    const instancePath = requireNonemptyString(
      item['instancePath'],
      `identityMap[${index}].instancePath`,
    );
    validateInstancePath(instancePath, index);
    if (seen.has(exportNodeId))
      throw applyError(
        'source-export-identities-duplicate',
        `identityMap contains duplicate exported node ${exportNodeId}.`,
      );
    seen.add(exportNodeId);
    result.push({
      exportNodeId,
      sourcePath,
      sourceNodeId,
      controlId,
      instancePath,
    });
  }
  return result;
}

function parseEditList(value: unknown): ZuiSourceExportEdit[] {
  if (!Array.isArray(value))
    throw applyError(
      'source-export-edits-invalid',
      'source-export edits must be an array.',
    );
  const result: ZuiSourceExportEdit[] = [];
  const seen = new Set<string>();
  for (const [index, item] of value.entries()) {
    if (!isRecord(item))
      throw applyError(
        'source-export-edits-invalid',
        `source-export edits[${index}] must be an object.`,
      );
    assertExactKeys(item, [
      'exportNodeId',
      'sourcePath',
      'sourceNodeId',
      'controlId',
      'instancePath',
      'fieldPath',
      'value',
    ]);
    const exportNodeId = requireNonemptyString(
      item['exportNodeId'],
      `source-export edits[${index}].exportNodeId`,
    );
    const sourcePath = requireNonemptyString(
      item['sourcePath'],
      `source-export edits[${index}].sourcePath`,
    );
    validateSourcePath(sourcePath);
    const sourceNodeId = requireNonemptyString(
      item['sourceNodeId'],
      `source-export edits[${index}].sourceNodeId`,
    );
    const controlId = item['controlId'];
    if (controlId !== null && typeof controlId !== 'string')
      throw applyError(
        'source-export-edits-invalid',
        `source-export edits[${index}].controlId must be a string or null.`,
      );
    const instancePath = requireNonemptyString(
      item['instancePath'],
      `source-export edits[${index}].instancePath`,
    );
    validateInstancePath(instancePath, index);
    if (
      !Array.isArray(item['fieldPath']) ||
      item['fieldPath'].length === 0 ||
      item['fieldPath'].some((part) => typeof part !== 'string' || !part.trim())
    )
      throw applyError(
        'source-export-edits-invalid',
        `source-export edits[${index}].fieldPath must be a nonempty string array.`,
      );
    const fieldPath = item['fieldPath'] as string[];
    if (!isSupportedVisualPath(['nodes', exportNodeId, ...fieldPath]))
      throw applyError(
        'unsupported-nonvisual-export-change',
        `Unsupported nonvisual export change: nodes.${exportNodeId}.${fieldPath.join('.')}.`,
      );
    const exportKey = JSON.stringify([exportNodeId, ...fieldPath]);
    if (seen.has(exportKey))
      throw applyError(
        'source-export-edits-duplicate',
        `Penpot export contains duplicate mappings for ${exportKey}.`,
      );
    seen.add(exportKey);
    result.push({
      exportNodeId,
      sourcePath,
      sourceNodeId,
      controlId,
      instancePath,
      fieldPath: [...fieldPath],
      value: item['value'],
    });
  }
  return result;
}

function validateInstancePath(value: string, index: number): void {
  let parsed: unknown;
  try {
    parsed = JSON.parse(value);
  } catch {
    throw applyError(
      'source-export-instance-path-invalid',
      `source-export edits[${index}].instancePath must be canonical JSON.`,
    );
  }
  if (!Array.isArray(parsed) || JSON.stringify(parsed) !== value)
    throw applyError(
      'source-export-instance-path-invalid',
      `source-export edits[${index}].instancePath must be a compact JSON array.`,
    );
  for (const [stepIndex, step] of parsed.entries()) {
    if (
      !isRecord(step) ||
      Object.keys(step).length !== 2 ||
      Object.keys(step)[0] !== 'sourcePath' ||
      Object.keys(step)[1] !== 'sourceNodeId' ||
      typeof step['sourcePath'] !== 'string' ||
      !step['sourcePath'].trim() ||
      typeof step['sourceNodeId'] !== 'string' ||
      !step['sourceNodeId'].trim()
    )
      throw applyError(
        'source-export-instance-path-invalid',
        `source-export edits[${index}].instancePath[${stepIndex}] must contain sourcePath and sourceNodeId.`,
      );
    validateSourcePath(step['sourcePath']);
  }
}

function isMappedLayoutPath(path: string[]): boolean {
  if (['width', 'height'].includes(path[0]) && path.length === 2)
    return ['min', 'preferred', 'max', 'stretch'].includes(path[1]);
  if (path[0] === 'position' && path.length === 2)
    return ['x', 'y'].includes(path[1]);
  if (path[0] === 'padding' && path.length === 2)
    return ['top', 'right', 'bottom', 'left'].includes(path[1]);
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
    ].includes(path[1])
  );
}

function isMappedPaintPath(path: string[]): boolean {
  const property = path[0];
  if (VISUAL_PROP_KEYS.has(property) && path.length === 1) return true;
  if (path.length !== 2) return false;
  const subproperty = path[1];
  if (property === 'font') return PAINT_STYLE_SUBKEYS.has(subproperty);
  if (property === 'border')
    return ['color', 'radius', 'width'].includes(subproperty);
  return COLOR_PROP_KEYS.has(property) && subproperty === 'color';
}

function containsUiAssetDirectory(segments: string[]): boolean {
  return segments.some(
    (segment, index) =>
      segment.toLowerCase() === 'assets' &&
      segments[index + 1]?.toLowerCase() === 'ui' &&
      index + 2 < segments.length,
  );
}

function invalidSourcePath(sourcePath: string): ZuiDocumentError {
  return applyError(
    'source-path-invalid',
    `Source path must be a repository-relative UI asset path ending in .zui: ${sourcePath}.`,
  );
}

function assertExactKeys(
  value: Record<string, unknown>,
  expected: readonly string[],
): void {
  const keys = Object.keys(value);
  if (
    keys.length !== expected.length ||
    keys.some((key) => !expected.includes(key))
  )
    throw applyError(
      'source-export-envelope-invalid',
      `Unexpected or missing source-export fields; expected ${expected.join(', ')}.`,
    );
}

function requireNonemptyString(value: unknown, label: string): string {
  if (typeof value !== 'string' || value.trim() === '')
    throw applyError(
      'source-export-envelope-invalid',
      `${label} must be a non-empty string.`,
    );
  return value;
}
