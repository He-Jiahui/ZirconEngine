import type { Fill, LibraryColor, Stroke } from '@penpot/plugin-types';
import { parseZuiDocument, serializeZuiDocument, type ZuiDocument } from './bridge/zui-document';
import { projectZuiDocument, cloneProjectionSnapshot } from './bridge/penpot-projection';
import { reconcileZuiDocument } from './bridge/penpot-reconcile';
import { restoreOriginalZuiSource } from './bridge/zui-source-export';
import { reviewHost } from './bridge/zui-review-host';
import {
  bindSourceFill, bindSourceStroke, captureSourceFill, captureSourceStroke,
  resolveSourceColor, sourceColorProperty, sourceColorProperties,
} from './penpot-source-colors';

const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const token = '$editor.surface.1';
const source = `[asset]
kind = "component"
id = "res://ui/button.zui"
version = 2
[imports]
styles = ["res://ui/editor/theme/editor_tokens.zui"]
[components.WorkbenchButton]
root = "root"
[nodes.root]
component = "Button"
control_id = "OriginalButton"
props = { text = "Save", background_color = "$editor.surface.1" }
events = [{ kind = "Click", route = "original.save" }]
layout = { width = { preferred = 120 }, height = { preferred = 32 } }
`;

function fixture() {
  const document = parseZuiDocument(source).document;
  Object.assign(document, {
    tokens: { 'editor.surface.1': '#242424' },
    penpot_host_theme_source: themePath, penpot_host_palette: { surface: ['#151515', '#242424'] },
    penpot_source_fingerprints: [{ sourcePath: themePath, sha256: 'a'.repeat(64) }],
    penpot_token_source_owners: { 'editor.surface.1': { sourcePath: themePath, sha256: 'a'.repeat(64) } },
  });
  const mapping = { schema: 'zircon.penpot-token-source.v1', version: 1,
    sourcePath: themePath, sourceSha256: 'a'.repeat(64), sourceTable: 'palette',
    sourceKey: 'surface', sourceIndex: 1, token: token.slice(1),
    originalValue: '#242424', penpotStyleId: 'source-color' };
  const color = { id: 'source-color', libraryId: 'shared-file',
    getSharedPluginData: (_namespace: string, key: string) =>
      key === 'token-source' ? JSON.stringify(mapping) : '',
    asFill: () => ({ fillColor: '#242424', fillOpacity: 1,
      fillColorRefId: color.id, fillColorRefFile: color.libraryId }),
    asStroke: () => ({ strokeColor: '#242424', strokeOpacity: 1,
      strokeColorRefId: color.id, strokeColorRefFile: color.libraryId }),
  } as unknown as LibraryColor;
  const library = { id: 'shared-file', colors: [color] };
  vi.stubGlobal('penpot', { library: { local: library, connected: [library] } });
  return { document, color, mapping, library };
}

describe('source-owned shared color bindings', () => {
  afterEach(() => vi.unstubAllGlobals());

  it('uses the native UUID and deduplicates the same local/connected library identity', () => {
    const { document, color } = fixture();
    const bound = bindSourceFill({ fillColor: '#242424', fillOpacity: 1 }, document, token);
    expect(bound).toEqual(color.asFill());
    expect(captureSourceFill([bound], 'button', document, token)).toBe(bound);
    expect(sourceColorProperty(document, 'root', 'background_color')).toBe(token);
  });

  it('keeps stroke metrics while binding and validating its source color', () => {
    const { document } = fixture();
    const original: Stroke = { strokeColor: '#242424', strokeWidth: 3,
      strokeStyle: 'solid', strokeAlignment: 'inner' };
    const bound = bindSourceStroke(original, document, token);
    expect(bound).toMatchObject({ ...original, strokeColorRefId: 'source-color', strokeColorRefFile: 'shared-file' });
    expect(captureSourceStroke([bound], 'button', document, token)).toBe(bound);
  });

  it('preserves the original token, imports, IDs and route in an unchanged official source round trip', () => {
    const { document } = fixture();
    document['penpot_original_source'] = source;
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    const bound = bindSourceFill({ fillColor: '#242424' }, document, token);
    const captured = captureSourceFill([bound], 'button', document, token)!;
    snapshot.shapes[0].current.paint.fillColor = captured.fillColor!;
    snapshot.shapes[0].current.paint.fillOpacity = captured.fillOpacity ?? 1;
    const result = reconcileZuiDocument(document, snapshot);
    expect(result.changes).toEqual([]);
    expect(restoreOriginalZuiSource(document, result, snapshot).source).toBe(source);
    const restored = parseZuiDocument(serializeZuiDocument(result.document)).document;
    expect(restored.nodes!['root'].props!['background_color']).toBe(token);
    expect(restored.nodes!['root'].control_id).toBe('OriginalButton');
    expect(restored.nodes!['root'].events).toEqual(document.nodes!['root'].events);
    expect(restored.imports).toEqual(document.imports);
  });

  it('does not assign a matching color to an authored literal or a different state paint', () => {
    const { document } = fixture();
    const fill: Fill = { fillColor: '#242424' };
    expect(bindSourceFill(fill, document, '#242424')).toBe(fill);
    const hover: Fill = { fillColor: '#383838' };
    expect(bindSourceFill(hover, document, token)).toBe(hover);
    expect(captureSourceFill([fill], 'literal')).toBe(fill);
    document.nodes!['root'].props!['selected'] = true;
    expect(sourceColorProperty(document, 'root', 'background_color')).toBeUndefined();
  });

  it('preserves source color spelling when the SDK normalises the shared style value', () => {
    const { document, mapping, color } = fixture();
    document.tokens!['editor.surface.1'] = '#ABCDEF';
    (document['penpot_host_palette'] as { surface: string[] }).surface[1] = '#ABCDEF';
    mapping.originalValue = '#ABCDEF';
    color.asFill = () => ({ fillColor: '#abcdef', fillOpacity: 1,
      fillColorRefId: color.id, fillColorRefFile: color.libraryId });
    expect(captureSourceFill([color.asFill()], 'button', document, token)?.fillColor).toBe('#ABCDEF');
  });

  it.each(['00', '80', 'ff'])('retains source RGBA alpha %s and rejects one-byte opacity drift', alpha => {
    const { document, mapping, color } = fixture();
    const rgba = '#ABCDEF' + alpha, opacity = parseInt(alpha, 16) / 255;
    document.tokens!['editor.surface.1'] = rgba;
    (document['penpot_host_palette'] as { surface: string[] }).surface[1] = rgba;
    mapping.originalValue = rgba;
    color.asFill = () => ({ fillColor: '#abcdef', fillOpacity: opacity,
      fillColorRefId: color.id, fillColorRefFile: color.libraryId });
    color.asStroke = () => ({ strokeColor: '#abcdef', strokeOpacity: opacity,
      strokeColorRefId: color.id, strokeColorRefFile: color.libraryId });
    const captured = captureSourceFill([color.asFill()], 'button', document, token)!;
    expect(captured).toMatchObject({ fillColor: '#ABCDEF', fillOpacity: opacity,
      fillColorRefId: color.id, fillColorRefFile: color.libraryId });
    expect(captureSourceStroke([color.asStroke()], 'button', document, token)?.strokeOpacity).toBe(opacity);
    const wrongOpacity = opacity === 1 ? 254 / 255 : opacity + 1 / 255;
    expect(() => captureSourceFill([{ ...color.asFill(), fillOpacity: wrongOpacity }], 'button', document, token))
      .toThrow('unmapped shared color');
    expect(() => captureSourceStroke([{ ...color.asStroke(), strokeOpacity: wrongOpacity }], 'button', document, token))
      .toThrow('unmapped shared color');
    expect(document.nodes!['root'].props!['background_color']).toBe(token);
  });

  it.each(['fillColorRefId', 'fillColorRefFile'] as const)('rejects partial and foreign references: %s', key => {
    const { document, color } = fixture();
    expect(() => captureSourceFill([{ ...color.asFill(), [key]: undefined }], 'button', document, token))
      .toThrow('unmapped shared color');
    expect(() => captureSourceFill([{ ...color.asFill(), [key]: 'foreign' }], 'button', document, token))
      .toThrow('unmapped shared color');
  });

  it('rejects missing closure, source hash drift and a falsely assigned palette owner', () => {
    const { document, mapping } = fixture();
    const pins = document['penpot_source_fingerprints'];
    delete document['penpot_source_fingerprints'];
    expect(() => resolveSourceColor(document, token)).toThrow('closure differs');
    document['penpot_source_fingerprints'] = pins;
    mapping.sourceSha256 = 'b'.repeat(64);
    expect(() => resolveSourceColor(document, token)).toThrow('closure differs');
    mapping.sourceSha256 = 'a'.repeat(64); mapping.sourceIndex = 0;
    expect(() => resolveSourceColor(document, token)).toThrow('closure differs');
  });

  it('rejects a caller shadowing the theme token even when both literal colors are equal', () => {
    const { document } = fixture();
    document['penpot_token_source_owners'] = { 'editor.surface.1': {
      sourcePath: 'zircon_editor/assets/ui/button.zui', sha256: 'b'.repeat(64),
    } };
    expect(() => resolveSourceColor(document, token)).toThrow('closure differs');
  });

  it('rejects an edited global color and an edited linked fill rather than exporting local literals', () => {
    const { document, color } = fixture();
    const original = color.asFill.bind(color);
    color.asFill = () => ({ ...original(), fillColor: '#555555' });
    expect(() => resolveSourceColor(document, token)).toThrow('outside its source owner');
    color.asFill = original;
    expect(() => captureSourceFill([{ ...original(), fillColor: '#555555' }], 'button', document, token))
      .toThrow('unmapped shared color');
  });

  it('keeps gradients, images, multiple fills and stroke caps rejected after reference validation', () => {
    const { document, color } = fixture();
    expect(() => captureSourceFill([color.asFill(), color.asFill()], 'button', document, token)).toThrow();
    expect(() => captureSourceFill('mixed', 'button', document, token)).toThrow();
    expect(() => captureSourceFill([{ ...color.asFill(), fillImage: { id: 'image' } } as Fill], 'button', document, token))
      .toThrow('non-solid');
    expect(() => captureSourceStroke([{ ...color.asStroke(), strokeCapEnd: 'round' } as Stroke], 'button', document, token))
      .toThrow('unsupported stroke');
  });

  it('rejects two distinct published identities for one token', () => {
    const { document, color, library } = fixture();
    const other = { ...color, id: 'other-color', libraryId: 'other-file' } as LibraryColor;
    const metadata = JSON.parse(color.getSharedPluginData('dev.zircon.zui', 'token-source'));
    other.getSharedPluginData = () => JSON.stringify({ ...metadata, penpotStyleId: other.id });
    other.asFill = () => ({ ...color.asFill(), fillColorRefId: other.id, fillColorRefFile: other.libraryId });
    expect(() => resolveSourceColor(document, token, [library, { id: other.libraryId, colors: [other] }]))
      .toThrow('Ambiguous shared color');
  });

  it('uses the stylesheet cascade and explicit placeholder source without mutating either', () => {
    const { document } = fixture();
    document.nodes!['root'].props!['placeholder_color'] = '$editor.text.secondary';
    document.stylesheets = [{ rules: [{ selector: '#OriginalButton',
      set: { self: { background_color: '$editor.surface.2' } } }] }];
    const before = JSON.stringify(document);
    expect(sourceColorProperty(document, 'root', 'background_color')).toBe('$editor.surface.2');
    expect(sourceColorProperty(document, 'root', 'foreground_color', 'placeholder')).toBe('$editor.text.secondary');
    expect(JSON.stringify(document)).toBe(before);
    document.nodes!['root'].props!['background'] = { color: '#242424' };
    document.stylesheets = [];
    expect(sourceColorProperty(document, 'root', 'background_color')).toBe('#242424');
  });

  it('indexes one import operation and resolves edits again for the next operation', () => {
    const { document } = fixture();
    const lookup = sourceColorProperties(document);
    expect(lookup('root', 'background_color')).toBe(token);
    document.nodes!['root'].props!['background_color'] = '#121212';
    expect(sourceColorProperties(document)('root', 'background_color')).toBe('#121212');
  });

  it('uses the outer source closure in a composed review host without mutating the source', () => {
    const { document } = fixture();
    const host: ZuiDocument = { ...document, asset: { ...document.asset, kind: 'view' },
      root: { node: 'root' } };
    delete host['penpot_source_fingerprints'];
    document['penpot_review_host'] = host;
    const before = JSON.stringify(document);
    const prepared = reviewHost(document)!;
    expect(resolveSourceColor(prepared, token)?.id).toBe('source-color');
    expect(JSON.stringify(document)).toBe(before);
    host['penpot_source_fingerprints'] = [{ sourcePath: themePath, sha256: 'b'.repeat(64) }];
    expect(() => reviewHost(document)).toThrow('differ from its source owner');
    host['penpot_source_fingerprints'] = [
      { sourcePath: themePath, sha256: 'a'.repeat(64) },
      { sourcePath: themePath, sha256: 'a'.repeat(64) },
    ];
    expect(() => reviewHost(document)).toThrow('invalid or duplicated');
  });
});
