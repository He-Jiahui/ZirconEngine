import { chromium } from 'playwright';
import { assertCanonicalLibraryPreserved, canonicalPublicationExportEvidence, canonicalPublicationSourcePins,
  collectCanonicalOwnerExportEvidence, publishCanonicalLibrary, requestCanonicalPluginResource, withCanonicalPublicationSourcePins,
} from './penpot-canonical-library-publication';
import { parseZuiDocument } from '../src/bridge/zui-document';

describe('canonical publication checks the manifest and worker response CORS before native Open', () => {
  const origin = 'http://127.0.0.1:19001';
  const manifestUrl = 'http://127.0.0.1:19002/manifest.json';
  const workerUrl = 'http://127.0.0.1:19002/assets/plugin.js';
  const response = (url: string, allowOrigin?: string) => ({
    status: () => 200, url: () => url,
    headers: (): Record<string, string> => allowOrigin === undefined ? {} : {'access-control-allow-origin': allowOrigin},
  });

  it.each([manifestUrl, workerUrl])('rejects HTTP200 without CORS and retains its exact response: %s', async url => {
    const get = vi.fn(async () => response(url));
    const record = vi.fn();
    await expect(requestCanonicalPluginResource(origin, url, get, record)).rejects.toThrow('CORS rejected');
    expect(get).toHaveBeenCalledWith(url, {headers: {Origin: origin}, maxRedirects: 0});
    expect(record).toHaveBeenCalledTimes(1);
    expect(record).toHaveBeenCalledWith({resourceUrl: url, responseUrl: url, httpStatus: 200,
      requestOrigin: origin, crossOrigin: true, accessControlAllowOrigin: null});
  });

  it.each(['*', origin, '  *  '])('admits the response header for the existing noncredentialed cross-origin fetch: %j', async header => {
    const actual = response(workerUrl, header);
    const record = vi.fn();
    await expect(requestCanonicalPluginResource(origin, workerUrl, async () => actual, record)).resolves.toBe(actual);
    expect(record).toHaveBeenCalledTimes(1);
    expect(record).toHaveBeenCalledWith({resourceUrl: workerUrl, responseUrl: workerUrl, httpStatus: 200,
      requestOrigin: origin, crossOrigin: true, accessControlAllowOrigin: header});
  });

  it.each(['http://localhost:19001', 'http://127.0.0.1:19002', 'null', `${origin}, http://localhost:19001`, ''])
    ('rejects an absent or different authorized origin header: %j', async header => {
      await expect(requestCanonicalPluginResource(origin, manifestUrl, async () => response(manifestUrl, header), vi.fn()))
        .rejects.toThrow('CORS rejected');
    });

  it('does not demand a CORS header on a same-origin resource', async () => {
    const url = `${origin}/manifest.json`;
    const actual = response(url);
    await expect(requestCanonicalPluginResource(origin, url, async () => actual, vi.fn())).resolves.toBe(actual);
  });

  it.each([302, 404])('preserves HTTP%s as failure even with an allowed origin', async status => {
    const actual = {...response(manifestUrl, '*'), status: () => status};
    await expect(requestCanonicalPluginResource(origin, manifestUrl, async () => actual, vi.fn()))
      .rejects.toThrow('Plugin resource HTTP status');
  });

  it('rejects a response from a different resource instead of qualifying redirected bytes', async () => {
    await expect(requestCanonicalPluginResource(origin, workerUrl, async () => response(`${workerUrl}?other=1`, '*'), vi.fn()))
      .rejects.toThrow('Plugin resource redirected');
  });

  it('retains a transport failure without fabricating an HTTP or SDK result', async () => {
    const failure = new Error('original connection refused');
    const record = vi.fn();
    await expect(requestCanonicalPluginResource(origin, manifestUrl, async () => { throw failure; }, record))
      .rejects.toBe(failure);
    expect(record).not.toHaveBeenCalled();
  });
});

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

describe('canonical owner export retains source guards independently of PNG evidence', () => {
  const originalSource = `[asset]
kind = "component"
id = "res://ui/editor/components/workbench/primitives/inputs/field.zui"
version = 2

[imports]
widgets = []
styles = ["res://ui/editor/theme/editor_tokens.zui"]

[components.WorkbenchField]
root = "root"

[nodes.root]
component = "InputField"
control_id = "WorkbenchFieldRoot"
props = { value = "18446744073709551615", editable_text = true, foreground_color = "$editor.text.primary" }
`;
  const original = () => parseZuiDocument(originalSource).document;

  it('exports the exact decimal-string owner source without requesting PNG in explicit source-owner mode', async () => {
    const capture = vi.fn(async () => { throw new Error('PNG must be independently admitted'); });
    const evidence = await collectCanonicalOwnerExportEvidence(original(), originalSource, 'source-owner', capture);
    expect(evidence).toEqual({sourceSemanticMatch: true, pngEvidence: 'not-requested'});
    expect(capture).not.toHaveBeenCalled();
  });

  it.each([
    ['stable control identity', 'WorkbenchFieldRoot', 'DifferentFieldRoot'],
    ['source import', 'res://ui/editor/theme/editor_tokens.zui', 'res://ui/editor/theme/different_tokens.zui'],
    ['token expression', '$editor.text.primary', '$editor.text.secondary'],
    ['typed input state', 'editable_text = true', 'editable_text = false'],
  ])('rejects a changed %s even when PNG was not requested', async (_name, from, to) => {
    const capture = vi.fn(async () => ({pngPath: 'unused.png', canvas: {}}));
    await expect(collectCanonicalOwnerExportEvidence(
      original(), originalSource.replace(from, to), 'source-owner', capture)).rejects.toThrow();
    expect(capture).not.toHaveBeenCalled();
  });

  it('requests PNG by default only after the owner source matches and waits for its downloader', async () => {
    const capture = vi.fn(async () => ({pngPath: 'actual-sdk-download.png', canvas: {source: 'official-sdk'}}));
    const evidence = await collectCanonicalOwnerExportEvidence(
      original(), originalSource, canonicalPublicationExportEvidence(undefined), capture);
    expect(capture).toHaveBeenCalledTimes(1);
    expect(evidence).toEqual({sourceSemanticMatch: true, pngEvidence: 'captured',
      pngPath: 'actual-sdk-download.png', canvas: {source: 'official-sdk'}});
  });

  it('preserves an actual PNG downloader failure in the default evidence mode', async () => {
    const capture = vi.fn(async () => { throw new Error('official PNG download timed out'); });
    await expect(collectCanonicalOwnerExportEvidence(
      original(), originalSource, canonicalPublicationExportEvidence(undefined), capture))
      .rejects.toThrow('official PNG download timed out');
    expect(capture).toHaveBeenCalledTimes(1);
  });

  it('rejects an unknown evidence mode before browser launch or worker input reads', async () => {
    const launch = vi.spyOn(chromium, 'launch');
    try {
      await expect(publishCanonicalLibrary({existingPlugin: {}, exportEvidence: 'skip-source-guards'}
        as unknown as Parameters<typeof publishCanonicalLibrary>[0]))
        .rejects.toThrow('exportEvidence must explicitly select');
      expect(launch).not.toHaveBeenCalled();
    } finally { launch.mockRestore(); }
  });
});

describe('canonical publication pins current owner and dependency bytes', () => {
  const owner = {sourcePath: 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/field.zui',
    sha256: 'a'.repeat(64)};
  const number = {sourcePath: 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/number_field.zui',
    sha256: 'b'.repeat(64)};
  const theme = {sourcePath: 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui', sha256: 'c'.repeat(64)};

  it('shares the same dependency pin across two independently prepared owners', () => {
    expect(canonicalPublicationSourcePins([
      {...owner, sourceFingerprints: [owner, theme]},
      {...number, sourceFingerprints: [number, theme]},
    ])).toEqual([owner, theme, number]);
  });

  it('rejects an owner read that differs from the bytes parsed for its projection', () => {
    expect(() => canonicalPublicationSourcePins([
      {...owner, sha256: 'd'.repeat(64), sourceFingerprints: [owner, theme]},
    ])).toThrow('changed during dependency preparation');
  });

  it('rejects conflicting dependency revisions instead of replacing a previous pin', () => {
    expect(() => canonicalPublicationSourcePins([
      {...owner, sourceFingerprints: [owner, theme]},
      {...number, sourceFingerprints: [number, {...theme, sha256: 'd'.repeat(64)}]},
    ])).toThrow('Conflicting canonical dependency fingerprints');
  });

  it.each([owner, theme])('blocks an import when $sourcePath changed before its write', async changed => {
    const current = new Map([owner, theme].map(pin => [pin.sourcePath, pin.sha256]));
    current.set(changed.sourcePath, 'd'.repeat(64));
    const write = vi.fn(async () => 'imported');
    await expect(withCanonicalPublicationSourcePins([owner, theme], async path => current.get(path)!, write))
      .rejects.toThrow(`Canonical source bytes changed: ${changed.sourcePath}`);
    expect(write).not.toHaveBeenCalled();
  });

  it('rejects drift during an awaited import without claiming its partial write was undone', async () => {
    const current = new Map([owner, theme].map(pin => [pin.sourcePath, pin.sha256]));
    const actualWrites: string[] = [];
    await expect(withCanonicalPublicationSourcePins([owner, theme], async path => current.get(path)!, async () => {
      actualWrites.push('actual-main');
      await Promise.resolve();
      current.set(theme.sourcePath, 'd'.repeat(64));
      return 'imported';
    })).rejects.toThrow(`Canonical source bytes changed: ${theme.sourcePath}`);
    expect(actualWrites).toEqual(['actual-main']);
    expect(current.get(theme.sourcePath)).toBe('d'.repeat(64));
  });

  it('returns an official operation result when all prepared source bytes remain current', async () => {
    const current = new Map([owner, theme].map(pin => [pin.sourcePath, pin.sha256]));
    const write = vi.fn(async () => ({mainInstanceId: 'actual-main'}));
    await expect(withCanonicalPublicationSourcePins([owner, theme], async path => current.get(path)!, write))
      .resolves.toEqual({mainInstanceId: 'actual-main'});
    expect(write).toHaveBeenCalledTimes(1);
  });
});
