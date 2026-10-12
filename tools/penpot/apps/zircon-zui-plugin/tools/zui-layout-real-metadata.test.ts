import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import test from 'node:test';
import { decodeZuiMetadata, encodeZuiMetadata, parseZuiDocument, zuiNodes } from '../src/bridge/zui-document';
import { compareStoredZuiMetadata } from './zui-layout-real-metadata';

const fixture = parseZuiDocument(await readFile(new URL('../src/bridge/roundtrip-fixture.zui', import.meta.url), 'utf8')).document;

test('wire equality permits only the owning codec serialization of an undefined optional key', () => {
  const expected = decodeZuiMetadata(encodeZuiMetadata(fixture));
  const expectedRoot = zuiNodes(expected)['root'];
  assert.ok(expectedRoot);
  expectedRoot['optional_runtime_field'] = undefined;
  const actual = decodeZuiMetadata(encodeZuiMetadata(expected));
  const report = compareStoredZuiMetadata(actual, expected);
  assert.equal(report.wireMatched, true);
  assert.equal(report.rawDifferencesOnlyUndefinedObjectOmissions, true);
  assert.deepEqual(report.rawDifferences.map(item => item.path), ['/nodes/root/optional_runtime_field']);
  assert.deepEqual(report.wireDifferences, []);
});

test('wire equality rejects lost IDs, events, refs, bindings and an explicit null value', () => {
  const expected = decodeZuiMetadata(encodeZuiMetadata(fixture));
  const expectedRoot = zuiNodes(expected)['root'];
  assert.ok(expectedRoot);
  expectedRoot['protected_ref'] = { asset_ref: 'res://protected/reference' };
  expectedRoot['protected_binding'] = { binding: 'vm.ProtectedValue' };
  expectedRoot['nullable_runtime_value'] = null;
  for (const field of ['control_id', 'events', 'protected_ref', 'protected_binding', 'nullable_runtime_value']) {
    assert.ok(Object.hasOwn(expectedRoot, field), `${field} must be present in this protected fixture`);
    const actual = decodeZuiMetadata(encodeZuiMetadata(expected));
    const actualRoot = zuiNodes(actual)['root'];
    assert.ok(actualRoot);
    delete actualRoot[field];
    const report = compareStoredZuiMetadata(actual, expected);
    assert.equal(report.wireMatched, false, `${field} loss must fail the complete wire equality gate`);
    assert.equal(report.rawDifferencesOnlyUndefinedObjectOmissions, false);
    assert.ok(report.wireDifferences.some(item => item.path === `/nodes/root/${field}`));
  }
});
