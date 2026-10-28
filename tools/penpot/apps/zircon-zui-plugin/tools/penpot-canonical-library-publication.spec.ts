import { chromium } from 'playwright';
import { createHash } from 'node:crypto';
import { assertCanonicalLibraryPreserved, assertCanonicalPublicationSavedReadback, canonicalPublicationEmbeddedSvgPins, canonicalPublicationExportEvidence, canonicalPublicationSourcePins,
  closeCanonicalPublicationSaveContext, verifyCanonicalPublicationExportReadback,
  collectCanonicalOwnerExportEvidence, collectCanonicalPublicationExportEvidence,
  publishCanonicalLibrary, requestCanonicalPluginResource, withCanonicalPublicationSourcePins,
} from './penpot-canonical-library-publication';
import { withSourceColorRevisionFence } from './zui-layout-real-session';
import { parseZuiDocument, type ZuiDocument } from '../src/bridge/zui-document';
import { FIELD_ICON_PATHS } from '../src/bridge/zui-field-projection';
import { verifySaveResponse } from './zui-layout-real-session-contract.mjs';

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

describe('canonical publication settles the original save observer before acknowledging persistence', () => {
  function fixture() {
    const identity = {fileId: '11111111-1111-4111-8111-111111111111'};
    const request = {id: identity.fileId, 'session-id': '22222222-2222-4222-8222-222222222222',
      revn: 5, changes: [{type: 'mod-obj', id: 'original-shape'}]};
    const response = {revn: 6, lagged: [{id: '33333333-3333-4333-8333-333333333333',
      'file-id': identity.fileId, 'session-id': request['session-id'], revn: 6, changes: request.changes}]};
    return {identity, request, response, saves: [verifySaveResponse(response, identity, 5, request)],
      errors: [] as string[], pending: new Set<Promise<void>>()};
  }

  // Actual observer tasks catch into onError and remove themselves in finally.
  function observe(state: ReturnType<typeof fixture>, operation: Promise<void>) {
    const handled: Promise<void> = operation.catch(error => { state.errors.push(String(error)); })
      .finally(() => state.pending.delete(handled));
    state.pending.add(handled);
    return handled;
  }

  it('rejects a late failed body capture despite an earlier exact file/session/change ACK', async () => {
    const state = fixture();
    let complete!: () => void;
    observe(state, new Promise<void>(resolve => { complete = resolve; }).then(() => {
      throw new Error('actual save response body unavailable');
    }));
    const close = vi.fn(async () => 'existing-authentication');
    const result = closeCanonicalPublicationSaveContext(state.pending, state.errors, close);
    complete();
    await expect(result).rejects.toThrow('actual save response body unavailable');
    expect(state.saves).toEqual([6]);
    expect(state.pending.size).toBe(0);
    expect(close).not.toHaveBeenCalled();
  });

  it('retains a later same-file response that does not acknowledge the actual session and changes', async () => {
    const state = fixture();
    const otherSession = structuredClone(state.response);
    otherSession.lagged[0]['session-id'] = '44444444-4444-4444-8444-444444444444';
    observe(state, Promise.resolve().then(() => {
      state.saves.push(verifySaveResponse(otherSession, state.identity, 5, state.request));
    }));
    const close = vi.fn(async () => 'existing-authentication');
    await expect(closeCanonicalPublicationSaveContext(state.pending, state.errors, close))
      .rejects.toThrow('update-file did not acknowledge this file');
    expect(state.saves).toEqual([6]);
    expect(close).not.toHaveBeenCalled();
  });

  it('waits for another response task registered while the first observation settles', async () => {
    const state = fixture();
    let complete!: () => void;
    const later = new Promise<void>(resolve => { complete = resolve; });
    const first = observe(state, Promise.resolve().then(() => {
      observe(state, later.then(() => { throw new Error('later actual save capture failed'); }));
    }));
    const close = vi.fn(async () => 'existing-authentication');
    const result = closeCanonicalPublicationSaveContext(state.pending, state.errors, close);
    await first;
    await Promise.resolve();
    await Promise.resolve();
    expect(close).not.toHaveBeenCalled();
    complete();
    await expect(result).rejects.toThrow('later actual save capture failed');
    expect(state.pending.size).toBe(0);
    expect(close).not.toHaveBeenCalled();
  });

  it.each(['loading-failed', 'response-capture'])('rejects %s reported while closing before fresh-context work', async phase => {
    const state = fixture();
    const failure = new Error(`actual save ${phase} during close`);
    const close = vi.fn(async () => {
      if (phase === 'loading-failed') state.errors.push(String(failure));
      else observe(state, Promise.resolve().then(() => { throw failure; }));
      return 'existing-authentication';
    });
    const reopen = vi.fn();
    await expect(closeCanonicalPublicationSaveContext(state.pending, state.errors, close).then(reopen))
      .rejects.toThrow(failure.message);
    expect(close).toHaveBeenCalledTimes(1);
    expect(reopen).not.toHaveBeenCalled();
    expect(state.saves).toEqual([6]);
    expect(state.pending.size).toBe(0);
  });

  it('preserves the original context-close failure instead of admitting a fresh context', async () => {
    const state = fixture();
    const failure = new Error('original context close failed');
    const reopen = vi.fn();
    await expect(closeCanonicalPublicationSaveContext(state.pending, state.errors, async () => {
      throw failure;
    }).then(reopen)).rejects.toBe(failure);
    expect(reopen).not.toHaveBeenCalled();
    expect(state.errors).toEqual([]);
  });

  it.each([false, true])('returns unchanged authentication only after clean observation and close (pending=%s)', async hasPending => {
    const state = fixture();
    const events: string[] = [];
    const authentication = {cookies: [], origins: []};
    if (hasPending) observe(state, Promise.resolve().then(() => { events.push('observed'); }));
    const close = vi.fn(async () => { events.push('closed'); return authentication; });
    await expect(closeCanonicalPublicationSaveContext(state.pending, state.errors, close)).resolves.toBe(authentication);
    expect(events).toEqual(hasPending ? ['observed', 'closed'] : ['closed']);
    expect(close).toHaveBeenCalledTimes(1);
    expect(state.pending.size).toBe(0);
    expect(state.errors).toEqual([]);
    expect(state.saves).toEqual([6]);
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

  function fencedExportFixture() {
    const identity = {origin: 'http://127.0.0.1:19001',
      fileId: '11111111-1111-4111-8111-111111111111', pageId: '22222222-2222-4222-8222-222222222222'};
    const boardId = '33333333-3333-4333-8333-333333333333';
    const libraryId = '44444444-4444-4444-8444-444444444444';
    const colorId = '55555555-5555-4555-8555-555555555555';
    const sourcePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
    const sourceSha256 = 'a'.repeat(64);
    const mapping = {schema: 'zircon.penpot-token-source.v1', version: 1, penpotStyleId: colorId,
      sourcePath, sourceSha256, token: 'editor.text.primary', sourceTable: 'palette', sourceKey: 'text',
      originalValue: '#c0c0c0'};
    const document: ZuiDocument = {...original(), tokens: {'editor.text.primary': '#c0c0c0'},
      penpot_host_theme_source: sourcePath, penpot_host_palette: {text: '#c0c0c0'},
      penpot_source_fingerprints: [{sourcePath, sha256: sourceSha256}],
      penpot_token_source_owners: {'editor.text.primary': {sourcePath, sha256: sourceSha256}}};
    const file = {id: identity.fileId, revn: 5, vern: 0, data: {'pages-index': {
      [identity.pageId]: {objects: {[boardId]: {id: boardId, fills: [{'fill-color': '#c0c0c0',
        'fill-color-ref-id': colorId, 'fill-color-ref-file': libraryId}]}}},
    }}};
    const library = {id: libraryId, revn: 2, vern: 0, data: {colors: {[colorId]: {
      id: colorId, color: '#c0c0c0', opacity: 1,
      'plugin-data': {'shared/dev.zircon.zui': {'token-source': JSON.stringify(mapping)}},
    }}}};
    const events: string[] = [];
    const context = {request: {post: vi.fn(async (url: string, options: {data: Record<string, unknown>}) => {
      let value: unknown;
      if (url.endsWith('/get-file-libraries')) {
        expect(options.data).toEqual({'file-id': identity.fileId});
        events.push('read-connected-libraries'); value = [{id: libraryId}];
      } else {
        expect(url).toBe(`${identity.origin}/api/rpc/command/get-file`);
        expect([identity.fileId, libraryId]).toContain(options.data['id']);
        const consumer = options.data['id'] === identity.fileId;
        events.push(consumer ? 'read-consumer' : 'read-library'); value = consumer ? file : library;
      }
      return {ok: () => true, status: () => 200, headers: () => ({'content-type': 'application/json'}),
        json: async () => structuredClone(value)};
    })}} as unknown as Parameters<typeof withSourceColorRevisionFence>[0];
    const receipt: Record<string, unknown> = {};
    const sourceBytes = new Map([[sourcePath, sourceSha256]]);
    const readSourceSha256 = vi.fn(async (path: string) => {
      expect(sourceBytes.has(path)).toBe(true); return sourceBytes.get(path)!;
    });
    const sourceColorFence: Parameters<typeof collectCanonicalPublicationExportEvidence>[2] = operation =>
      withSourceColorRevisionFence(context, identity, boardId, document, JSON.parse, receipt, operation);
    const collect = (requested: Parameters<typeof collectCanonicalPublicationExportEvidence>[4],
      exportSource: Parameters<typeof collectCanonicalPublicationExportEvidence>[5],
      capturePng: Parameters<typeof collectCanonicalPublicationExportEvidence>[6]) =>
      collectCanonicalPublicationExportEvidence([{sourcePath, sha256: sourceSha256}], readSourceSha256,
        sourceColorFence, original(), requested, exportSource, capturePng);
    return {file, library, events, receipt, sourcePath, sourceBytes, readSourceSha256, collect};
  }

  it.each(['source-owner', 'source-owner-and-png'] as const)
    ('finishes authenticated revision reads after all requested %s evidence is persisted', async requested => {
      const fixture = fencedExportFixture();
      const capture = vi.fn(async () => {
        fixture.events.push('png-start'); await Promise.resolve(); fixture.events.push('png-persisted');
        return {pngPath: 'actual-sdk-download.png', canvas: {source: 'official-sdk'}};
      });
      const result = await fixture.collect(requested, async () => {
        fixture.events.push('download-event'); await Promise.resolve(); fixture.events.push('source-persisted');
        return {exportPath: 'actual-owner.zui', exported: originalSource};
      }, capture);
      const reads = ['read-consumer', 'read-connected-libraries', 'read-library'];
      expect(fixture.events).toEqual([...reads, 'download-event', 'source-persisted',
        ...(requested === 'source-owner-and-png' ? ['png-start', 'png-persisted'] : []), ...reads]);
      expect(result.exportSha256).toBe(createHash('sha256').update(originalSource).digest('hex'));
      expect(result.pngEvidence).toBe(requested === 'source-owner' ? 'not-requested' : 'captured');
      expect(capture).toHaveBeenCalledTimes(requested === 'source-owner' ? 0 : 1);
      expect(fixture.readSourceSha256).toHaveBeenCalledTimes(2);
      expect(fixture.receipt['sourceColorRevisionFences']).toEqual([
        expect.objectContaining({status: 'exported-with-stable-source-color-revisions'}),
      ]);
    });

  it.each([
    ['source-persistence', 'consumer'], ['source-persistence', 'library'],
    ['png-persistence', 'consumer'], ['png-persistence', 'library'],
  ] as const)('rejects a late %s %s revision change even when source and cached RGBA match', async (stage, target) => {
    const fixture = fencedExportFixture();
    const advance = () => { (target === 'consumer' ? fixture.file : fixture.library).revn++; };
    const capture = vi.fn(async () => {
      advance(); return {pngPath: 'actual-sdk-download.png', canvas: {source: 'official-sdk'}};
    });
    await expect(fixture.collect(stage === 'source-persistence' ? 'source-owner' : 'source-owner-and-png', async () => {
      await Promise.resolve();
      if (stage === 'source-persistence') advance();
      return {exportPath: 'actual-owner.zui', exported: originalSource};
    }, capture)).rejects.toThrow('revision, RGBA or canonical source changed during export');
    expect(capture).toHaveBeenCalledTimes(stage === 'source-persistence' ? 0 : 1);
    expect(fixture.receipt['sourceColorRevisionFences']).toEqual([expect.objectContaining({status: 'failed'})]);
  });

  it.each(['source-persistence', 'png-persistence'] as const)
    ('rejects source dependency drift during %s without accepting completed evidence', async stage => {
      const fixture = fencedExportFixture();
      const changeBytes = () => { fixture.sourceBytes.set(fixture.sourcePath, 'b'.repeat(64)); };
      const capture = vi.fn(async () => {
        changeBytes(); return {pngPath: 'actual-sdk-download.png', canvas: {source: 'official-sdk'}};
      });
      await expect(fixture.collect(stage === 'source-persistence' ? 'source-owner' : 'source-owner-and-png', async () => {
        if (stage === 'source-persistence') changeBytes();
        return {exportPath: 'actual-owner.zui', exported: originalSource};
      }, capture)).rejects.toThrow(`Canonical source bytes changed: ${fixture.sourcePath}`);
    });

  it('preserves a source persistence failure after the download event and skips PNG', async () => {
    const fixture = fencedExportFixture(), failure = new Error('original source download save failed');
    const capture = vi.fn(async () => ({pngPath: 'unused.png', canvas: {}}));
    await expect(fixture.collect('source-owner-and-png', async () => {
      fixture.events.push('download-event'); await Promise.resolve(); throw failure;
    }, capture)).rejects.toBe(failure);
    expect(capture).not.toHaveBeenCalled();
    expect(fixture.events).toEqual(['read-consumer', 'read-connected-libraries', 'read-library', 'download-event']);
    expect(fixture.receipt['sourceColorRevisionFences']).toEqual([expect.objectContaining({status: 'failed'})]);
  });

  it('retains an actual PNG failure and leaves its authenticated export fence failed', async () => {
    const fixture = fencedExportFixture(), failure = new Error('original official PNG save failed');
    await expect(fixture.collect('source-owner-and-png', async () =>
      ({exportPath: 'actual-owner.zui', exported: originalSource}), async () => {
        await Promise.resolve(); throw failure;
      })).rejects.toBe(failure);
    expect(fixture.receipt['sourceColorRevisionFences']).toEqual([expect.objectContaining({status: 'failed'})]);
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

describe('canonical publication fences the SVG bytes embedded in its actual projection', () => {
  const owner = {sourcePath: 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/field.zui',
    sha256: 'a'.repeat(64)};
  const digest = (text: string) => createHash('sha256').update(text).digest('hex');
  const projection = (): ZuiDocument => ({
    asset: {kind: 'component', id: 'res://ui/editor/components/workbench/primitives/inputs/field.zui', version: 2},
    components: {WorkbenchField: {root: 'root'}},
    nodes: {root: {component: 'InputField', control_id: 'WorkbenchFieldRoot'}},
  });
  const search = '<svg viewBox="0 0 16 16">\r\n<path d="M1 1L2 2"/>\r\n</svg>';

  it('pins every loaded Field SVG with exact bytes without changing source metadata or nodes', () => {
    const document = projection();
    const icons = {search, clear: '<svg><path d="M1 1L3 3"/></svg>', stepper: '<svg><path d="M2 1L1 2"/></svg>'};
    document['penpot_field_icons'] = icons;
    document['penpot_source_fingerprints'] = [owner];
    const before = structuredClone(document);
    expect(canonicalPublicationEmbeddedSvgPins(document, [owner.sourcePath, ...Object.values(FIELD_ICON_PATHS)]))
      .toEqual(Object.entries(icons).map(([key, svg]) => ({
        sourcePath: FIELD_ICON_PATHS[key as keyof typeof FIELD_ICON_PATHS], sha256: digest(svg),
      })));
    expect(digest(search)).not.toBe(digest(search.replaceAll('\r\n', '\n')));
    expect(document).toEqual(before);
  });

  it('keeps explicit callee-owned SVG and native tab sources separate from the caller', () => {
    const document = projection();
    document.nodes!['icon'] = {component: 'Icon', props: {source: 'res://icons/tools/pin.svg'},
      penpot_review_source_path: 'zircon_plugins/fixture/assets/ui/primitives/icon.zui', penpot_icon_svg: search};
    document.nodes!['tab'] = {component: 'Button', props: {icon: 'res://icons/old-authored.svg'},
      penpot_review_source_path: owner.sourcePath, penpot_review_native_tab_icon: {assetPath: 'ionicons/grid-outline.svg'},
      penpot_icon_svg: '<svg><path d="M0 0H4"/></svg>'};
    document['penpot_native_icon_svgs'] = {'ionicons/grid-outline.svg': '<svg><path d="M0 0H4"/></svg>'};
    expect(canonicalPublicationEmbeddedSvgPins(document, [])).toEqual([
      {sourcePath: 'zircon_editor/assets/icons/ionicons/grid-outline.svg', sha256: digest('<svg><path d="M0 0H4"/></svg>')},
      {sourcePath: 'zircon_plugins/fixture/assets/icons/tools/pin.svg', sha256: digest(search)},
    ]);
  });

  it.each([false, true])('rejects an actual SVG revision change around SDK work; changed before=%j', async changedBefore => {
    const document = projection();
    document['penpot_field_icons'] = {search};
    const pins = canonicalPublicationSourcePins([{...owner,
      sourceFingerprints: [owner, ...canonicalPublicationEmbeddedSvgPins(document, [FIELD_ICON_PATHS.search])]}]);
    const current = new Map(pins.map(pin => [pin.sourcePath, pin.sha256]));
    const changed = digest('<svg><path d="M9 9L8 8"/></svg>');
    if (changedBefore) current.set(FIELD_ICON_PATHS.search, changed);
    const writes: string[] = [];
    await expect(withCanonicalPublicationSourcePins(pins, async path => current.get(path)!, async () => {
      writes.push('original-main');
      await Promise.resolve();
      current.set(FIELD_ICON_PATHS.search, changed);
    })).rejects.toThrow('Canonical source bytes changed: ' + FIELD_ICON_PATHS.search);
    expect(writes).toEqual(changedBefore ? [] : ['original-main']);
    expect(current.get(FIELD_ICON_PATHS.search)).toBe(changed);
  });

  it.each(['zircon_editor/assets/icons/not-embedded.svg', 'zircon_editor/assets/icons/not-supported.png'])
    ('refuses a declared resource without loaded byte provenance: %s', sourcePath => {
      expect(() => canonicalPublicationEmbeddedSvgPins(projection(), [owner.sourcePath, sourcePath]))
        .toThrow('no embedded SVG provenance');
    });

  it('refuses conflicting bytes for the same physical SVG source', () => {
    const document = projection();
    document['penpot_field_icons'] = {search};
    document['penpot_native_icon_svgs'] = {'zircon_editor_shell/controls/search.svg': '<svg/>'};
    expect(() => canonicalPublicationEmbeddedSvgPins(document, [])).toThrow('Conflicting embedded SVG bytes');
  });

  it('refuses unknown Field mappings and unowned or escaping per-node SVGs', () => {
    const unknown = projection();
    unknown['penpot_field_icons'] = {invented: search};
    expect(() => canonicalPublicationEmbeddedSvgPins(unknown, [])).toThrow('Unknown embedded field SVG');
    for (const props of [
      {source: 'res://icons/../outside.svg', owner: 'zircon_editor/assets/ui/editor/icon.zui'},
      {source: 'res://icons/pin.svg', owner: undefined},
      {source: 'res://icons/pin.svg', owner: 'outside/assets/ui/icon.zui'},
    ]) {
      const document = projection();
      document.nodes!['icon'] = {component: 'Icon', props: {source: props.source}, penpot_icon_svg: search};
      if (props.owner) document.nodes!['icon']['penpot_review_source_path'] = props.owner;
      expect(() => canonicalPublicationEmbeddedSvgPins(document, [])).toThrow();
    }
    const missing = projection();
    missing.nodes!['icon'] = {component: 'Icon', penpot_icon_svg: search};
    expect(() => canonicalPublicationEmbeddedSvgPins(missing, [])).toThrow('no authored or native tab source');
  });
});

describe('canonical publication verifies the complete library after the read-only export workspace closes', () => {
  function fixture() {
    return {id: 'shared-file', revn: 8, vern: 0, data: {
      components: {
        original: {'main-instance-id': 'original-main', name: 'WorkbenchButton'},
        added: {'main-instance-id': 'added-main', name: 'WorkbenchField'},
      },
      colors: {primary: {id: 'original-color', color: '#0070e0'}},
      typographies: {body: {id: 'original-typography', 'font-size': '14'}},
      'pages-index': {page: {name: 'Shared', objects: {
        root: {id: 'root', shapes: ['original-main', 'added-main']},
        'original-main': {id: 'original-main', 'component-id': 'original', x: 12},
        'added-main': {id: 'added-main', 'component-id': 'added', x: 240},
        'added-caption': {id: 'added-caption', 'parent-id': 'added-main', text: 'Value'},
      }}},
    }};
  }

  it('reads an independent snapshot only after close and returns it without changing the baseline', async () => {
    const before = fixture();
    const original = structuredClone(before);
    const after = structuredClone(before);
    const events: string[] = [];
    const read = vi.fn(async () => { events.push('read'); return after; });
    await expect(verifyCanonicalPublicationExportReadback(before, async () => { events.push('closed'); }, read))
      .resolves.toBe(after);
    expect(events).toEqual(['closed', 'read']);
    expect(read).toHaveBeenCalledTimes(1);
    expect(before).toEqual(original);
  });

  it.each(['id', 'revn', 'vern'] as const)('rejects a changed %s after final export close', async key => {
    const before = fixture();
    const after = structuredClone(before);
    if (key === 'id') after.id = 'other-file';
    else after[key] += 1;
    await expect(verifyCanonicalPublicationExportReadback(before, async () => {}, async () => after)).rejects.toThrow();
    expect(before).toEqual(fixture());
  });

  it.each(['original-main', 'added-main', 'added-caption', 'color', 'typography', 'membership', 'root-order'])
    ('rejects complete data drift in %s even if the readback revision is unchanged', async changed => {
      const before = fixture();
      const after = structuredClone(before);
      const objects = after.data['pages-index'].page.objects;
      switch (changed) {
        case 'original-main': objects['original-main'].x += 1; break;
        case 'added-main': after.data.components.added['main-instance-id'] = 'original-main'; break;
        case 'added-caption': objects['added-caption'].text = 'Changed'; break;
        case 'color': after.data.colors.primary.color = '#ffffff'; break;
        case 'typography': after.data.typographies.body['font-size'] = '13'; break;
        case 'membership': Reflect.deleteProperty(objects, 'added-caption'); break;
        case 'root-order': objects.root.shapes.reverse(); break;
      }
      await expect(verifyCanonicalPublicationExportReadback(before, async () => {}, async () => after))
        .rejects.toThrow('Read-only canonical export changed file data');
      expect(before).toEqual(fixture());
    });

  it('rejects a saved change that materializes during workspace close after the last export fence', async () => {
    const before = fixture();
    const persisted = structuredClone(before);
    const read = vi.fn(async () => structuredClone(persisted));
    await expect(verifyCanonicalPublicationExportReadback(before, async () => {
      persisted.revn += 1;
      persisted.data['pages-index'].page.objects['added-caption'].text = 'Late native layout';
    }, read)).rejects.toThrow('Read-only canonical export changed file revision');
    expect(read).toHaveBeenCalledTimes(1);
    expect(before).toEqual(fixture());
  });

  it.each(['close', 'fresh-read'])('preserves the original %s failure and admits no result', async phase => {
    const failure = new Error(`${phase} failed`);
    const read = vi.fn(async () => {
      if (phase === 'fresh-read') throw failure;
      return fixture();
    });
    const close = vi.fn(async () => { if (phase === 'close') throw failure; });
    await expect(verifyCanonicalPublicationExportReadback(fixture(), close, read)).rejects.toBe(failure);
    expect(close).toHaveBeenCalledTimes(1);
    expect(read).toHaveBeenCalledTimes(phase === 'close' ? 0 : 1);
  });

  it('rejects an export save-observer error during close before starting the independent read', async () => {
    const pending = new Set<Promise<void>>();
    const errors: string[] = [];
    const read = vi.fn(async () => fixture());
    await expect(verifyCanonicalPublicationExportReadback(fixture(),
      () => closeCanonicalPublicationSaveContext(pending, errors, async () => {
        const observed = Promise.resolve().then(() => { errors.push('actual export save body unavailable'); })
          .finally(() => { pending.delete(observed); });
        pending.add(observed);
      }), read)).rejects.toThrow('actual export save body unavailable');
    expect(read).not.toHaveBeenCalled();
    expect(pending.size).toBe(0);
  });
});

describe('canonical publication preserves the complete saved snapshot before admitting owner export', () => {
  function fixture() {
    const original = {id: 'shared-file', revn: 7, vern: 0, data: {
      pages: ['page'],
      components: {original: {'main-instance-id': 'original-main', name: 'WorkbenchButton'}},
      colors: {primary: {id: 'original-color', color: '#0070e0'}},
      typographies: {body: {id: 'original-typography', 'font-size': '14'}},
      'pages-index': {page: {name: 'Shared', objects: {
        '00000000-0000-0000-0000-000000000000': {id: 'root', shapes: ['original-main']},
        'original-main': {id: 'original-main', 'component-id': 'original', x: 12},
      }}},
      options: {nativeTextCacheVersion: 1},
    }};
    const saved = { ...structuredClone(original), revn: 8, data: {
      ...structuredClone(original.data),
      components: {...original.data.components,
        added: {'main-instance-id': 'added-main', name: 'WorkbenchField'}},
      'pages-index': {page: {name: 'Shared', objects: {
        ...structuredClone(original.data['pages-index'].page.objects),
        '00000000-0000-0000-0000-000000000000': {id: 'root', shapes: ['original-main', 'added-main']},
        'added-main': {id: 'added-main', 'component-id': 'added', x: 240},
        'added-caption': {id: 'added-caption', 'parent-id': 'added-main', content: {text: 'Value'},
          fills: [{color: '#c0c0c0'}], positionData: [{width: 48, text: 'Value'}]},
        'added-vector': {id: 'added-vector', 'parent-id': 'added-main', content: 'M0 0L1 1'},
      }}},
    }};
    return {original, saved};
  }

  it('admits an independent exact saved readback without changing either snapshot', () => {
    const {saved} = fixture();
    const before = structuredClone(saved), reopened = structuredClone(saved);
    expect(() => assertCanonicalPublicationSavedReadback(saved, reopened)).not.toThrow();
    expect(saved).toEqual(before);
    expect(reopened).toEqual(before);
  });

  it.each(['id', 'revn', 'vern'] as const)('rejects a changed %s before adopting the reopened export baseline', key => {
    const {saved} = fixture();
    const reopened = structuredClone(saved);
    if (key === 'id') reopened.id = 'other-file';
    else reopened[key] += 1;
    expect(() => assertCanonicalPublicationSavedReadback(saved, reopened)).toThrow();
    expect(saved).toEqual(fixture().saved);
  });

  it.each(['new-main-geometry', 'new-caption-paint', 'new-position-data', 'new-vector', 'new-membership', 'file-options'])
    ('rejects %s that the pre-append original-content guard does not protect', changed => {
      const {original, saved} = fixture();
      const reopened = structuredClone(saved);
      const objects = reopened.data['pages-index'].page.objects;
      switch (changed) {
        case 'new-main-geometry': objects['added-main'].x += 1; break;
        case 'new-caption-paint': objects['added-caption'].fills[0].color = '#ffffff'; break;
        case 'new-position-data': objects['added-caption'].positionData[0].width += 1.0625; break;
        case 'new-vector': objects['added-vector'].content = 'M0 0L2 2'; break;
        case 'new-membership': Reflect.deleteProperty(objects, 'added-caption'); break;
        case 'file-options': reopened.data.options.nativeTextCacheVersion += 1; break;
      }
      expect(() => assertCanonicalLibraryPreserved(original, reopened)).not.toThrow();
      expect(reopened.data.components).toEqual(saved.data.components);
      expect(() => assertCanonicalPublicationSavedReadback(saved, reopened))
        .toThrow('Canonical save readback changed file data');
      expect(saved).toEqual(fixture().saved);
    });

  it('rejects a late save during original workspace close even when an exact change ACK was observed', async () => {
    const {saved} = fixture();
    const request = {id: saved.id, 'session-id': '22222222-2222-4222-8222-222222222222',
      revn: 7, changes: [{type: 'mod-obj', id: 'added-caption'}]};
    const acknowledged = verifySaveResponse({revn: 8, lagged: [{
      id: '33333333-3333-4333-8333-333333333333', 'file-id': saved.id,
      'session-id': request['session-id'], revn: 8, changes: request.changes,
    }]}, {fileId: saved.id}, 7, request);
    expect(acknowledged).toBe(saved.revn);
    const persisted = structuredClone(saved);
    const pending = new Set<Promise<void>>();
    const errors: string[] = [];
    const events: string[] = [];
    const exportOwner = vi.fn();
    const readback = async () => {
      await closeCanonicalPublicationSaveContext(pending, errors, async () => {
        events.push('closed');
        persisted.revn += 1;
        persisted.data['pages-index'].page.objects['added-caption'].positionData[0].width += 1.0625;
      });
      events.push('fresh-read');
      const reopened = structuredClone(persisted);
      assertCanonicalPublicationSavedReadback(saved, reopened);
      exportOwner();
    };
    await expect(readback()).rejects.toThrow('Canonical save readback changed file revision');
    expect(events).toEqual(['closed', 'fresh-read']);
    expect(exportOwner).not.toHaveBeenCalled();
    expect(saved).toEqual(fixture().saved);
  });
});
