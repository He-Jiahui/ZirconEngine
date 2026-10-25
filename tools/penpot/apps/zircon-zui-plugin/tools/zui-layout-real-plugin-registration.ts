import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';

type Entry = Record<string, unknown>;

export const pluginRegistrationFingerprintProtocol = 'penpot-plugin-registry-v2';

function record(value: unknown): Entry {
  assert.ok(value && typeof value === 'object' &&
    (Object.getPrototypeOf(value) === Object.prototype || Object.getPrototypeOf(value) === null),
  'Missing actual plugin registry record');
  return value as Entry;
}

/** Only the decoded registry permission set and manifest string vector are accepted. */
export function pluginRegistrationPermissions(value: unknown): string[] {
  assert.ok(Array.isArray(value) || value instanceof Set, 'Missing actual plugin permission collection');
  const permissions: unknown[] = Array.from(value);
  assert.ok(permissions.every(item => typeof item === 'string' && item.length > 0),
    'Actual plugin permissions must be nonempty strings');
  assert.equal(new Set(permissions).size, permissions.length, 'Duplicate actual plugin permissions');
  return (permissions as string[]).sort();
}

export function pluginRegistrationFingerprint(value: unknown): string {
  // Preserve transport kinds: a permission Set must not collapse to {} or alias an array/marker.
  // Version 1 pins must be re-observed through this decoder; never accept a legacy hash fallback.
  const stable = (entry: unknown): unknown => {
    if (entry === null || typeof entry === 'string' || typeof entry === 'boolean') return entry;
    if (typeof entry === 'number') {
      assert.ok(Number.isFinite(entry), 'Invalid plugin registration fingerprint number');
      return entry;
    }
    if (entry instanceof Set) return ['set', pluginRegistrationPermissions(entry)];
    if (Array.isArray(entry)) return ['array', Array.from(entry, stable)];
    const fields = record(entry);
    assert.equal(Object.getOwnPropertySymbols(fields).length, 0, 'Invalid plugin registration fingerprint keys');
    return ['record', Object.keys(fields).sort().map(key => [key, stable(fields[key])])];
  };
  const serialized = JSON.stringify(stable(value));
  assert.ok(typeof serialized === 'string', 'Invalid plugin registration fingerprint input');
  return createHash('sha256').update(pluginRegistrationFingerprintProtocol + '\0')
    .update(serialized).digest('hex');
}

/** Fingerprint the entire account registry, including unrelated entries and ordered IDs. */
export function pluginRegistrySnapshot(profile: unknown): {
  sha256: string; entryCount: number; orderedIdCount: number;
  fingerprintProtocol: typeof pluginRegistrationFingerprintProtocol;
} {
  const registry = record(record(record(profile)['props'])['plugins']);
  const data = record(registry['data']);
  const ids = registry['ids'];
  assert.ok(Array.isArray(ids) && ids.every(id => typeof id === 'string'), 'Missing actual plugin registry order');
  return { sha256: pluginRegistrationFingerprint(registry), entryCount: Object.keys(data).length,
    orderedIdCount: ids.length, fingerprintProtocol: pluginRegistrationFingerprintProtocol };
}

export function verifyPluginRegistryUnchanged(profile: unknown, expectedSha256: string): ReturnType<typeof pluginRegistrySnapshot> {
  assert.match(expectedSha256, /^[0-9a-f]{64}$/);
  const snapshot = pluginRegistrySnapshot(profile);
  assert.equal(snapshot.sha256, expectedSha256, 'The account plugin registry changed during reuse');
  return snapshot;
}

/** Read only: the caller supplies a previously observed registration and manifest. */
export function verifyExistingPluginRegistration(profile: unknown, manifestUrl: string,
  permissions: unknown, expected: { pluginId: string; registryEntrySha256: string; registrySha256?: string }): {
    entry: Entry; index: number; fingerprint: string; registrySha256: string; entryCount: number;
    fingerprintProtocol: typeof pluginRegistrationFingerprintProtocol;
  } {
  const registry = record(record(record(profile)['props'])['plugins']);
  const entries = Object.values(record(registry['data'])).map(record);
  const matches = entries.filter(entry => entry['url'] === manifestUrl);
  assert.equal(matches.length, 1, 'Existing plugin manifest must resolve to one actual registration');
  const entry = matches[0];
  assert.equal(entry['plugin-id'], expected.pluginId, 'Existing plugin ID differs');
  assert.match(expected.registryEntrySha256, /^[0-9a-f]{64}$/);
  const fingerprint = pluginRegistrationFingerprint(entry);
  assert.equal(fingerprint, expected.registryEntrySha256, 'Existing plugin registration changed');
  assert.ok(Array.isArray(permissions), 'Manifest permissions must be the actual JSON array');
  assert.deepEqual(pluginRegistrationPermissions(entry['permissions']), pluginRegistrationPermissions(permissions),
    'Existing plugin permissions differ');
  const ids = registry['ids'];
  assert.ok(Array.isArray(ids) && ids.every(id => typeof id === 'string'));
  assert.equal(ids.filter(id => id === expected.pluginId).length, 1, 'Existing plugin registry order is ambiguous');
  const snapshot = expected.registrySha256
    ? verifyPluginRegistryUnchanged(profile, expected.registrySha256) : pluginRegistrySnapshot(profile);
  return { entry, index: ids.indexOf(expected.pluginId), fingerprint,
    registrySha256: snapshot.sha256, entryCount: snapshot.entryCount,
    fingerprintProtocol: snapshot.fingerprintProtocol };
}
