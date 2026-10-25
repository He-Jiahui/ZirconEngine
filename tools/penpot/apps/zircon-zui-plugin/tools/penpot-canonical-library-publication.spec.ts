import { chromium } from 'playwright';
import { assertCanonicalLibraryPreserved, publishCanonicalLibrary } from './penpot-canonical-library-publication';

describe('canonical publication requires the existing plugin and strict worker build', () => {
  it('rejects an absent existing registration before launching a browser', async () => {
    const launch = vi.spyOn(chromium, 'launch');
    try {
      await expect(publishCanonicalLibrary({} as Parameters<typeof publishCanonicalLibrary>[0]))
        .rejects.toThrow('requires the existing registered plugin');
      expect(launch).not.toHaveBeenCalled();
    } finally { launch.mockRestore(); }
  });

  it('rejects an absent strict worker receipt before launching a browser', async () => {
    const launch = vi.spyOn(chromium, 'launch');
    try {
      await expect(publishCanonicalLibrary({ existingPlugin: {
        host: 'http://127.0.0.1:19002/', pluginId: 'cce74285-44c0-8011-8008-d6ea3345bdbe',
        registryEntrySha256: 'a'.repeat(64), manifestSha256: 'b'.repeat(64),
      } } as Parameters<typeof publishCanonicalLibrary>[0]))
        .rejects.toThrow('requires a pinned strict worker build receipt');
      expect(launch).not.toHaveBeenCalled();
    } finally { launch.mockRestore(); }
  });
});

describe('append canonical sources without replacing library owners', () => {
  function fixture() {
    return { id: 'shared-file', data: {
      components: { existing: { 'main-instance-id': 'original-main', name: 'WorkbenchButton' } },
      colors: { primary: { id: 'style-original', color: '#0070e0' } },
      typographies: { body: { id: 'font-original', 'font-size': '14' } },
      'pages-index': { page: { name: 'Shared', objects: {
        '00000000-0000-0000-0000-000000000000': { id: 'root', shapes: ['original-main'], x: 0 },
        'original-main': { id: 'original-main', x: 12, y: 8, 'component-id': 'existing' },
      } } },
    } };
  }
  it('permits a new main without changing the original IDs, styles or page objects', () => {
    const before = fixture(), after = structuredClone(before);
    Object.assign(after.data.components, { added: { 'main-instance-id': 'added-main', name: 'WorkbenchField' } });
    Object.assign(after.data['pages-index'].page.objects, { 'added-main': { id: 'added-main', x: 300, y: 8 } });
    after.data['pages-index'].page.objects['00000000-0000-0000-0000-000000000000'].shapes.push('added-main');
    expect(() => assertCanonicalLibraryPreserved(before, after)).not.toThrow();
  });
  it('rejects replacement of an original main despite a matching component name', () => {
    const before = fixture(), after = structuredClone(before);
    after.data.components.existing['main-instance-id'] = 'replacement';
    expect(() => assertCanonicalLibraryPreserved(before, after)).toThrow('Existing component changed');
  });
  it('rejects hidden geometry and style edits in an append request', () => {
    const before = fixture(), after = structuredClone(before);
    after.data['pages-index'].page.objects['original-main'].x += 1;
    expect(() => assertCanonicalLibraryPreserved(before, after)).toThrow('Existing shape changed');
    const styles = structuredClone(before);
    styles.data.colors.primary.color = '#ff0000';
    expect(() => assertCanonicalLibraryPreserved(before, styles)).toThrow('Existing library colors changed');
  });
  it('rejects page-root changes and removal of existing sources', () => {
    const before = fixture(), after = structuredClone(before);
    after.data['pages-index'].page.objects['00000000-0000-0000-0000-000000000000'].shapes.length = 0;
    expect(() => assertCanonicalLibraryPreserved(before, after)).toThrow('root children changed');
    const removed = structuredClone(before);
    Reflect.deleteProperty(removed.data['pages-index'].page.objects, 'original-main');
    expect(() => assertCanonicalLibraryPreserved(before, removed)).toThrow('Existing shape disappeared');
  });
});
