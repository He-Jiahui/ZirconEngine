import { isDeepStrictEqual } from 'node:util';
import { decodeZuiMetadata, encodeZuiMetadata, type ZuiDocument } from '../src/bridge/zui-document';
import { sha256 } from './zui-layout-real-session-contract.mjs';

interface Difference {
  path: string; reason: string; actual: Record<string, unknown>; expected: Record<string, unknown>;
  undefinedObjectPropertyOmitted: boolean;
}
const describe = (value: unknown, present = true): Record<string, unknown> => {
  if (!present) return { type: 'absent' };
  if (value === null) return { type: 'null', value: null };
  if (typeof value === 'string') return { type: 'string', length: value.length, sha256: sha256(value) };
  if (typeof value === 'bigint') return { type: 'bigint', value: String(value) };
  if (typeof value !== 'object') return { type: typeof value, value };
  return { type: Object.prototype.toString.call(value),
    ...(Array.isArray(value) ? { length: value.length } : { keys: Object.keys(value).length }) };
};

/** Enumerate every differing property; this report never filters the equality gate. */
export function metadataDifferencePaths(actual: unknown, expected: unknown): Difference[] {
  const differences: Difference[] = [];
  const walk = (left: unknown, right: unknown, path: string) => {
    if (isDeepStrictEqual(left, right)) return;
    const arrays = Array.isArray(left) && Array.isArray(right);
    const objects = left !== null && right !== null && typeof left === 'object' && typeof right === 'object' &&
      Object.prototype.toString.call(left) === '[object Object]' && Object.prototype.toString.call(right) === '[object Object]';
    if (!arrays && !objects) {
      differences.push({ path, reason: 'value-or-type', actual: describe(left), expected: describe(right),
        undefinedObjectPropertyOmitted: false });
      return;
    }
    const leftObject = left as Record<string, unknown>;
    const rightObject = right as Record<string, unknown>;
    for (const key of new Set([...Object.keys(leftObject), ...Object.keys(rightObject)])) {
      const childPath = `${path}/${key.replace(/~/g, '~0').replace(/\//g, '~1')}`;
      const leftHas = Object.hasOwn(leftObject, key);
      const rightHas = Object.hasOwn(rightObject, key);
      if (leftHas && rightHas) walk(leftObject[key], rightObject[key], childPath);
      else differences.push({ path: childPath, reason: 'property-presence',
        actual: describe(leftObject[key], leftHas), expected: describe(rightObject[key], rightHas),
        undefinedObjectPropertyOmitted: objects && !leftHas && rightHas && rightObject[key] === undefined });
    }
    if (arrays && (left as unknown[]).length !== (right as unknown[]).length)
      differences.push({ path: `${path}/length`, reason: 'array-length', actual: describe((left as unknown[]).length),
        expected: describe((right as unknown[]).length), undefinedObjectPropertyOmitted: false });
    if (Object.getPrototypeOf(left) !== Object.getPrototypeOf(right))
      differences.push({ path, reason: 'prototype', actual: describe(left), expected: describe(right),
        undefinedObjectPropertyOmitted: false });
  };
  walk(actual, expected, '');
  return differences;
}

export function compareStoredZuiMetadata(actual: ZuiDocument, expected: ZuiDocument) {
  // Use the same typed-value metadata codec as the production importer. It
  // preserves all serializable fields and naturally omits JS undefined keys.
  const expectedWire = decodeZuiMetadata(encodeZuiMetadata(expected));
  const rawDifferences = metadataDifferencePaths(actual, expected);
  const wireDifferences = metadataDifferencePaths(actual, expectedWire);
  return { wireMatched: isDeepStrictEqual(actual, expectedWire),
    rawDifferences, rawDifferencesOnlyUndefinedObjectOmissions:
      rawDifferences.every(item => item.undefinedObjectPropertyOmitted), wireDifferences,
    expectedCodec: 'decodeZuiMetadata(encodeZuiMetadata(expected))' };
}
