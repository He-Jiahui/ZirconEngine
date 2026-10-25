import { createHash } from 'node:crypto';
import { serialize } from 'node:v8';
import { pluginRegistrationFingerprint, pluginRegistrationFingerprintProtocol, pluginRegistrationPermissions,
  verifyExistingPluginRegistration, pluginRegistrySnapshot, verifyPluginRegistryUnchanged } from './zui-layout-real-plugin-registration';

describe('preserve the whole actual account plugin registry', () => {
  function profile() {
    return { props: { plugins: { ids: ['existing', 'unrelated'], data: {
      existing: { 'plugin-id': 'existing', url: 'http://127.0.0.1:19002/manifest.json', permissions: ['content:read'] },
      unrelated: { 'plugin-id': 'unrelated', url: 'http://127.0.0.1:19005/manifest.json', permissions: ['library:read'] },
    } } } };
  }
  it('records all entries and ordered IDs without changing them', () => {
    const actual = profile(), before = JSON.stringify(actual), snapshot = pluginRegistrySnapshot(actual);
    expect(snapshot.entryCount).toBe(2);
    expect(verifyPluginRegistryUnchanged(actual, snapshot.sha256)).toEqual(snapshot);
    expect(JSON.stringify(actual)).toBe(before);
  });
  it('rejects deletion, installation, unrelated mutation and ordering changes during reuse', () => {
    const baseline = pluginRegistrySnapshot(profile()).sha256;
    const mutations = [
      (value: ReturnType<typeof profile>) => { delete (value.props.plugins.data as Record<string, unknown>).unrelated; },
      (value: ReturnType<typeof profile>) => { value.props.plugins.ids.reverse(); },
      (value: ReturnType<typeof profile>) => { value.props.plugins.data.unrelated.permissions.push('content:write'); },
      (value: ReturnType<typeof profile>) => {
        Object.assign(value.props.plugins.data, { extra: { 'plugin-id': 'extra', permissions: [] } });
        value.props.plugins.ids.push('extra');
      },
    ];
    for (const mutate of mutations) {
      const actual = profile(); mutate(actual);
      expect(() => verifyPluginRegistryUnchanged(actual, baseline)).toThrow('registry changed');
    }
  });
});

describe('reuse an actual registered Penpot plugin', () => {
  const url = 'http://127.0.0.1:19002/manifest.json';
  function fixture() {
    const entry = { 'plugin-id': 'plugin-actual', url, permissions: ['content:read', 'library:read'] };
    return { entry, profile: { props: { plugins: { ids: ['another', 'plugin-actual'], data: { actual: entry } } } },
      expected: { pluginId: 'plugin-actual', registryEntrySha256: pluginRegistrationFingerprint(entry) } };
  }
  it('returns the original UI index and leaves the profile unchanged', () => {
    const f = fixture(), before = JSON.stringify(f.profile);
    expect(verifyExistingPluginRegistration(f.profile, url, [...f.entry.permissions].reverse(), f.expected).index).toBe(1);
    expect(JSON.stringify(f.profile)).toBe(before);
  });
  it('fingerprints nested records independently of object insertion order', () => {
    expect(pluginRegistrationFingerprint({ b: { d: 1, c: 2 }, a: 3 }))
      .toBe(pluginRegistrationFingerprint({ a: 3, b: { c: 2, d: 1 } }));
  });
  it('rejects missing or duplicate manifests before opening a plugin', () => {
    const f = fixture();
    expect(() => verifyExistingPluginRegistration(f.profile, url + '?other', f.entry.permissions, f.expected)).toThrow('one actual registration');
    Object.assign(f.profile.props.plugins.data, { duplicate: { ...f.entry } });
    expect(() => verifyExistingPluginRegistration(f.profile, url, f.entry.permissions, f.expected)).toThrow('one actual registration');
  });
  it('rejects registration replacement, permission drift, and ambiguous order', () => {
    const f = fixture();
    expect(() => verifyExistingPluginRegistration(f.profile, url, f.entry.permissions, { ...f.expected, pluginId: 'replaced' })).toThrow('ID differs');
    expect(() => verifyExistingPluginRegistration(f.profile, url, ['content:write'], f.expected)).toThrow('permissions differ');
    f.profile.props.plugins.ids.push('plugin-actual');
    expect(() => verifyExistingPluginRegistration(f.profile, url, f.entry.permissions, f.expected)).toThrow('order is ambiguous');
  });
  it('rejects mutation since the recorded preflight', () => {
    const f = fixture();
    f.entry.permissions.push('content:write');
    expect(() => verifyExistingPluginRegistration(f.profile, url, f.entry.permissions, f.expected)).toThrow('registration changed');
  });
});

describe('actual decoded Transit plugin permission sets', () => {
  const url = 'http://127.0.0.1:19002/manifest.json';
  function fixture() {
    // get-profile :permissions is [:set :string]; real-session's decoder returns native Set.
    const entry = { 'plugin-id': 'plugin-actual', url,
      permissions: new Set(['library:read', 'content:read']) };
    const unrelated = { 'plugin-id': 'another', url: 'http://127.0.0.1:19005/manifest.json',
      permissions: new Set(['content:read']) };
    const profile = { props: { plugins: { ids: ['another', 'plugin-actual'], data: { unrelated, actual: entry } } } };
    return { entry, profile, expected: { pluginId: 'plugin-actual',
      registryEntrySha256: pluginRegistrationFingerprint(entry), registrySha256: pluginRegistrySnapshot(profile).sha256 } };
  }
  it('compares a decoded Set to manifest permissions, records v2 and preserves the full registry', () => {
    const f = fixture(), before = serialize(f.profile);
    const result = verifyExistingPluginRegistration(f.profile, url, ['content:read', 'library:read'], f.expected);
    expect(result.index).toBe(1);
    expect(result.entry).toBe(f.entry);
    expect(result.fingerprintProtocol).toBe(pluginRegistrationFingerprintProtocol);
    expect(result.registrySha256).toBe(f.expected.registrySha256);
    expect(pluginRegistrationPermissions(f.entry.permissions)).toEqual(['content:read', 'library:read']);
    expect(serialize(f.profile)).toEqual(before);
    expect([...f.entry.permissions]).toEqual(['library:read', 'content:read']);
  });
  it('ignores Set iteration order while retaining ordinary array and ordered registry ID order', () => {
    const f = fixture(), before = pluginRegistrySnapshot(f.profile);
    f.entry.permissions = new Set(['content:read', 'library:read']);
    expect(pluginRegistrySnapshot(f.profile)).toEqual(before);
    expect(verifyExistingPluginRegistration(f.profile, url, ['library:read', 'content:read'], f.expected).index).toBe(1);
    expect(pluginRegistrationFingerprint(['a', 'b'])).not.toBe(pluginRegistrationFingerprint(['b', 'a']));
    f.profile.props.plugins.ids.reverse();
    expect(() => verifyPluginRegistryUnchanged(f.profile, before.sha256)).toThrow('registry changed');
  });
  it('fingerprints permission content and transport kind without empty-object or marker collisions', () => {
    const read = new Set(['content:read']);
    const write = new Set(['content:write']);
    expect(pluginRegistrationFingerprint({ permissions: read })).not.toBe(pluginRegistrationFingerprint({ permissions: write }));
    for (const other of [{}, ['content:read'], ['set', ['content:read']], { set: ['content:read'] }]) {
      expect(pluginRegistrationFingerprint(read)).not.toBe(pluginRegistrationFingerprint(other));
    }
  });
  it('rejects the legacy fingerprint which omitted actual permission Set content', () => {
    const f = fixture();
    const legacy = createHash('sha256').update(JSON.stringify({ permissions: {},
      'plugin-id': f.entry['plugin-id'], url })).digest('hex');
    expect(() => verifyExistingPluginRegistration(f.profile, url, ['content:read', 'library:read'],
      { ...f.expected, registryEntrySha256: legacy })).toThrow('registration changed');
  });
  it('rejects actual permission additions/removals and unrelated Set mutations after preflight', () => {
    for (const permission of ['content:write', 'library:read']) {
      const f = fixture();
      if (permission === 'content:write') f.entry.permissions.add(permission);
      else f.entry.permissions.delete(permission);
      expect(() => verifyExistingPluginRegistration(f.profile, url, ['content:read', 'library:read'], f.expected))
        .toThrow('registration changed');
    }
    const f = fixture(); f.profile.props.plugins.data.unrelated.permissions.add('content:write');
    expect(() => verifyExistingPluginRegistration(f.profile, url, ['content:read', 'library:read'], f.expected))
      .toThrow('registry changed');
  });
  it('rejects manifest permission drift and a Set substituted for the manifest JSON array', () => {
    const f = fixture();
    expect(() => verifyExistingPluginRegistration(f.profile, url, ['content:read', 'content:write'], f.expected))
      .toThrow('permissions differ');
    expect(() => verifyExistingPluginRegistration(f.profile, url, f.entry.permissions, f.expected))
      .toThrow('actual JSON array');
  });
  it.each([undefined, null, {}, { tag: 'set', value: ['content:read'] }, 'content:read',
    ['content:read', 'content:read'], ['content:read', null], new Set([1]), new Set([''])])(
    'rejects a malformed or lossy permission collection: %p', (permissions) => {
      expect(() => pluginRegistrationPermissions(permissions)).toThrow();
    });
  it('keeps the actual kebab-case plugin ID contract and exact manifest URL', () => {
    const f = fixture();
    const jsonAlias = { pluginId: f.entry['plugin-id'], url, permissions: ['content:read', 'library:read'] };
    const profile = { props: { plugins: { ids: ['plugin-actual'], data: { actual: jsonAlias } } } };
    expect(() => verifyExistingPluginRegistration(profile, url, jsonAlias.permissions,
      { pluginId: 'plugin-actual', registryEntrySha256: pluginRegistrationFingerprint(jsonAlias) })).toThrow('ID differs');
    expect(() => verifyExistingPluginRegistration(f.profile, url + '?alias', ['content:read', 'library:read'], f.expected))
      .toThrow('one actual registration');
  });
  it('does not treat the ordered ID vector as a Set', () => {
    const f = fixture();
    const profile = { props: { plugins: { ...f.profile.props.plugins, ids: new Set(f.profile.props.plugins.ids) } } };
    expect(() => pluginRegistrySnapshot(profile)).toThrow('registry order');
  });
  it.each([undefined, () => 'content:read', new Map([['permissions', 'content:read']]), new Date(0), NaN, Infinity])(
    'rejects unsupported fingerprint values rather than silently omitting them: %p', (value) => {
      expect(() => pluginRegistrationFingerprint({ value })).toThrow();
    });
});
