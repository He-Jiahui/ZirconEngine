import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  type PenpotAssetSnapshot,
  type ProjectionEditableState,
  type ProjectionSourcePart,
  type ReconciledZuiDocument,
} from './penpot-projection';
import {
  ZuiDocumentError,
  decodeZuiMetadata,
  encodeZuiMetadata,
  type ZuiDocument,
} from './zui-document';
import { reviewHost, reconcileReviewSource } from './zui-review-host';

export const PENPOT_BRIDGE_SCHEMA = 'dev.zircon.zui.penpot-asset';
export const PENPOT_BRIDGE_VERSION = 3;

export interface PenpotBridgeAsset {
  schema: typeof PENPOT_BRIDGE_SCHEMA;
  version: typeof PENPOT_BRIDGE_VERSION;
  fileName: string;
  documentMetadata: string;
  snapshot: PenpotAssetSnapshot;
}

export type PenpotBaselineIndex = Record<string, ProjectionEditableState>;

export function baselineIndex(
  snapshot: PenpotAssetSnapshot,
): PenpotBaselineIndex {
  return Object.fromEntries(
    snapshot.shapes.map(({ nodeId, baseline }) => [nodeId, baseline]),
  );
}

export function assertSnapshotBaselines(
  snapshot: PenpotAssetSnapshot,
  expected: PenpotBaselineIndex,
): void {
  const snapshotIds = snapshot.shapes.map(({ nodeId }) => nodeId);
  const expectedIds = Object.keys(expected);
  if (
    snapshotIds.length !== expectedIds.length ||
    snapshotIds.some((nodeId) => !Object.hasOwn(expected, nodeId))
  ) {
    throw bridgeError(
      'Baseline index does not match the semantic nodes; re-import the .zui asset before exporting.',
    );
  }
  for (const shape of snapshot.shapes) {
    if (!deepEqual(shape.baseline, expected[shape.nodeId])) {
      throw bridgeError(
        `Baseline for semantic node ${shape.nodeId} does not match document metadata; re-import the .zui asset before exporting.`,
      );
    }
  }
}

export function createPenpotBridgeAsset(
  document: ZuiDocument,
  fileName: string,
): PenpotBridgeAsset {
  return {
    schema: PENPOT_BRIDGE_SCHEMA,
    version: PENPOT_BRIDGE_VERSION,
    fileName,
    documentMetadata: encodeZuiMetadata(document),
    snapshot: cloneProjectionSnapshot(
      projectZuiDocument(reviewHost(document) ?? document),
    ),
  };
}

export function serializePenpotBridgeAsset(asset: PenpotBridgeAsset): string {
  validateBridgeAsset(asset);
  return `${JSON.stringify(asset, null, 2)}\n`;
}

export function parsePenpotBridgeAsset(source: string): PenpotBridgeAsset {
  let parsed: unknown;
  try {
    parsed = JSON.parse(source);
  } catch (error) {
    throw new ZuiDocumentError(
      `Failed to parse Penpot asset bridge JSON: ${errorMessage(error)}`,
      [
        {
          severity: 'error',
          code: 'penpot-bridge-json-invalid',
          message: errorMessage(error),
        },
      ],
    );
  }
  validateBridgeAsset(parsed);
  return parsed;
}

export function reconcilePenpotBridgeAsset(
  asset: PenpotBridgeAsset,
): ReconciledZuiDocument {
  validateBridgeAsset(asset);
  return reconcileReviewSource(
    decodeZuiMetadata(asset.documentMetadata),
    asset.snapshot,
  );
}

function validateBridgeAsset(
  value: unknown,
): asserts value is PenpotBridgeAsset {
  if (!isRecord(value)) {
    throw bridgeError('Penpot asset bridge root must be an object.');
  }
  if (value['schema'] !== PENPOT_BRIDGE_SCHEMA) {
    throw bridgeError(
      `Unsupported Penpot asset bridge schema ${String(value['schema'])}.`,
    );
  }
  if (value['version'] !== PENPOT_BRIDGE_VERSION) {
    throw bridgeError(
      `Unsupported Penpot asset bridge version ${String(value['version'])}.`,
    );
  }
  if (typeof value['fileName'] !== 'string' || value['fileName'].length === 0) {
    throw bridgeError(
      'Penpot asset bridge fileName must be a non-empty string.',
    );
  }
  if (typeof value['documentMetadata'] !== 'string') {
    throw bridgeError('Penpot asset bridge documentMetadata must be a string.');
  }
  validateSnapshot(value['snapshot']);
  assertCanonicalBaselines(
    decodeZuiMetadata(value['documentMetadata']),
    value['snapshot'],
  );
}

function validateSnapshot(
  value: unknown,
): asserts value is PenpotAssetSnapshot {
  if (!isRecord(value) || typeof value['assetId'] !== 'string') {
    throw bridgeError('Penpot asset bridge snapshot must declare assetId.');
  }
  assertStringArray(value['rootNodeIds'], 'snapshot.rootNodeIds');
  assertStringArray(value['detachedNodeIds'], 'snapshot.detachedNodeIds');
  if (!Array.isArray(value['shapes'])) {
    throw bridgeError('Penpot asset bridge snapshot.shapes must be an array.');
  }
  value['shapes'].forEach((shape, index) => validateShape(shape, index));
}

function validateShape(value: unknown, index: number): void {
  const path = `snapshot.shapes[${index}]`;
  if (!isRecord(value)) {
    throw bridgeError(`${path} must be an object.`);
  }
  if (
    typeof value['nodeId'] !== 'string' ||
    typeof value['component'] !== 'string'
  ) {
    throw bridgeError(`${path} must declare nodeId and component.`);
  }
  if (
    value['parentNodeId'] !== null &&
    typeof value['parentNodeId'] !== 'string'
  ) {
    throw bridgeError(`${path}.parentNodeId must be a string or null.`);
  }
  assertStringArray(value['childNodeIds'], `${path}.childNodeIds`);
  if (value['sourceParts'] !== undefined)
    validateSourceParts(value['sourceParts'], value['nodeId'], `${path}.sourceParts`);
  if (!isRecord(value['baseline']) || !isRecord(value['current'])) {
    throw bridgeError(
      `${path} must declare baseline and current editable states.`,
    );
  }
  validateEditableState(value['baseline'], `${path}.baseline`);
  validateEditableState(value['current'], `${path}.current`);
}

function validateSourceParts(
  value: unknown,
  nodeId: string,
  path: string,
): asserts value is ProjectionSourcePart[] {
  if (!Array.isArray(value)) throw bridgeError(`${path} must be an array.`);
  const seen = new Set<string>();
  for (const [index, part] of value.entries()) {
    if (!isRecord(part)) throw bridgeError(`${path}[${index}] must be an object.`);
    if (part['nodeId'] !== nodeId)
      throw bridgeError(`${path}[${index}].nodeId must match ${nodeId}.`);
    if (typeof part['property'] !== 'string' || !part['property'].trim())
      throw bridgeError(`${path}[${index}].property must be nonempty.`);
    if (seen.has(part['property']))
      throw bridgeError(`${path} contains duplicate property ${part['property']}.`);
    seen.add(part['property']);
  }
}

function validateEditableState(
  value: Record<string, unknown>,
  path: string,
): void {
  const geometry = recordAt(value, 'geometry', path);
  assertFiniteNumbers(
    geometry,
    ['x', 'y', 'width', 'height'],
    `${path}.geometry`,
  );

  const paint = recordAt(value, 'paint', path);
  assertNullableString(paint['fillColor'], `${path}.paint.fillColor`);
  assertNullableString(paint['strokeColor'], `${path}.paint.strokeColor`);
  assertFiniteNumbers(
    paint,
    ['fillOpacity', 'strokeOpacity', 'strokeWidth', 'borderRadius', 'opacity'],
    `${path}.paint`,
  );

  const fragments = value['textFragments'];
  if (fragments !== undefined && !isRecord(fragments))
    throw bridgeError(`${path}.textFragments must be an object.`);
  for (const text of [value['text'], ...Object.values(fragments ?? {})]) {
    if (text === null) continue;
    if (!isRecord(text))
      throw bridgeError(`${path}.text must be an object or null.`);
    if (typeof text['characters'] !== 'string') {
      throw bridgeError(`${path}.text.characters must be a string.`);
    }
    assertEnum(
      text['property'],
      [
        'text',
        'value_text',
        'query',
        'value',
        'placeholder',
        'title',
        'message',
        'label',
        'label_text',
        'group_label',
        'options',
        null,
      ],
      `${path}.text.property`,
    );
    assertNullableString(text['color'], `${path}.text.color`);
    assertFiniteNumber(text['colorOpacity'], `${path}.text.colorOpacity`);
    assertNullableFiniteNumber(text['fontSize'], `${path}.text.fontSize`);
    assertNullableString(text['fontWeight'], `${path}.text.fontWeight`);
    if (typeof text['fontFamily'] !== 'string' || !text['fontFamily'].trim())
      throw bridgeError(`${path}.text.fontFamily must be a nonempty string.`);
    assertFiniteNumber(text['lineHeight'], `${path}.text.lineHeight`);
    if (Number(text['lineHeight']) <= 0)
      throw bridgeError(`${path}.text.lineHeight must be positive.`);
    assertEnum(
      text['align'],
      ['left', 'center', 'right', 'justify', null],
      `${path}.text.align`,
    );
  }

  const container = recordAt(value, 'container', path);
  if (value['slotPadding'] !== undefined)
    assertFiniteNumbers(
      recordAt(value, 'slotPadding', path),
      ['top', 'right', 'bottom', 'left'],
      `${path}.slotPadding`,
    );
  assertEnum(
    container['kind'],
    ['free', 'flex', 'grid'],
    `${path}.container.kind`,
  );
  assertEnum(
    container['direction'],
    ['row', 'column'],
    `${path}.container.direction`,
  );
  if (typeof container['wrap'] !== 'boolean') {
    throw bridgeError(`${path}.container.wrap must be a boolean.`);
  }
  assertFiniteNumbers(
    container,
    ['gap', 'rowGap', 'columnGap', 'columns', 'rows'],
    `${path}.container`,
  );
  for (const track of ['columns', 'rows'] as const) {
    const value = container[track];
    if (typeof value !== 'number' || !Number.isInteger(value) || value <= 0) {
      throw bridgeError(
        `${path}.container.${track} must be a positive integer.`,
      );
    }
  }
  const padding = recordAt(container, 'padding', `${path}.container`);
  assertFiniteNumbers(
    padding,
    ['top', 'right', 'bottom', 'left'],
    `${path}.container.padding`,
  );
  assertEnum(
    container['alignItems'],
    ['start', 'center', 'end', 'stretch'],
    `${path}.container.alignItems`,
  );
  assertEnum(
    container['justifyContent'],
    ['start', 'center', 'end', 'space-between', 'space-around', 'space-evenly'],
    `${path}.container.justifyContent`,
  );
  if (typeof container['clip'] !== 'boolean') {
    throw bridgeError(`${path}.container.clip must be a boolean.`);
  }
}

function assertCanonicalBaselines(
  document: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
): void {
  const canonical = cloneProjectionSnapshot(
    projectZuiDocument(reviewHost(document) ?? document),
  );
  assertSnapshotBaselines(snapshot, baselineIndex(canonical));
  const canonicalByNode = new Map(
    canonical.shapes.map((shape) => [shape.nodeId, shape.sourceParts ?? []]),
  );
  for (const shape of snapshot.shapes) {
    const expected = canonicalByNode.get(shape.nodeId);
    if (!expected) continue;
    const actual = shape.sourceParts ?? [];
    if (!deepEqual(actual, expected))
      throw bridgeError(
        `Source-part mapping for semantic node ${shape.nodeId} does not match document metadata; re-import the .zui asset before exporting.`,
      );
  }
}

function recordAt(
  value: Record<string, unknown>,
  key: string,
  path: string,
): Record<string, unknown> {
  const record = value[key];
  if (!isRecord(record)) throw bridgeError(`${path}.${key} must be an object.`);
  return record;
}

function assertFiniteNumbers(
  value: Record<string, unknown>,
  keys: string[],
  path: string,
): void {
  for (const key of keys) assertFiniteNumber(value[key], `${path}.${key}`);
}

function assertFiniteNumber(
  value: unknown,
  path: string,
): asserts value is number {
  if (typeof value !== 'number' || !Number.isFinite(value)) {
    throw bridgeError(`${path} must be a finite number.`);
  }
}

function assertNullableFiniteNumber(
  value: unknown,
  path: string,
): asserts value is number | null {
  if (value !== null) assertFiniteNumber(value, path);
}

function assertNullableString(
  value: unknown,
  path: string,
): asserts value is string | null {
  if (value !== null && typeof value !== 'string') {
    throw bridgeError(`${path} must be a string or null.`);
  }
}

function assertEnum<T>(
  value: unknown,
  allowed: readonly T[],
  path: string,
): asserts value is T {
  if (!allowed.some((candidate) => Object.is(candidate, value))) {
    throw bridgeError(`${path} has unsupported value ${String(value)}.`);
  }
}

function deepEqual(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true;
  if (Array.isArray(left) && Array.isArray(right)) {
    return (
      left.length === right.length &&
      left.every((value, index) => deepEqual(value, right[index]))
    );
  }
  if (isRecord(left) && isRecord(right)) {
    const leftKeys = Object.keys(left);
    const rightKeys = Object.keys(right);
    return (
      leftKeys.length === rightKeys.length &&
      leftKeys.every(
        (key) => Object.hasOwn(right, key) && deepEqual(left[key], right[key]),
      )
    );
  }
  return false;
}

function assertStringArray(
  value: unknown,
  path: string,
): asserts value is string[] {
  if (!Array.isArray(value) || value.some((item) => typeof item !== 'string')) {
    throw bridgeError(`${path} must be an array of strings.`);
  }
}

function bridgeError(message: string): ZuiDocumentError {
  return new ZuiDocumentError(message, [
    { severity: 'error', code: 'penpot-bridge-invalid', message },
  ]);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}
