import type { Board, Fill, LibraryColor, Stroke, Text } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { cloneProjectionSnapshot, projectZuiDocument } from './bridge/penpot-projection';
import { captureNodeTexts, unsupportedTextStyle } from './penpot-text-capture';
import { guardControlTextGeometry } from './penpot-control-guard';
import {
  EMPTY_TEXT_SENTINEL,
  authoredGeometry,
  assertCanonicalCopyPaintUnchanged,
  capturedHeightMin,
  capturedMappedTextStyle,
  capturedFontTypography,
  capturedSemanticText,
  gridCellForChildIndex,
  layoutSizingIsCompatible,
  authoredAnchoredPosition,
  resolvedAnchoredPosition,
  resolvedParentContainerKinds,
  singleSolidFill,
  singleSolidStroke,
  createSourcePaintCaptureContext,
  stableSemanticChildOrder,
  symmetricLayoutGap,
} from './penpot-capture-validation';

const rev5File = 'b4a1e3ce-9e6a-81fe-8008-d69d607aac12';
const rev5Style = '599586e3-022c-500b-ac80-5d9c490139af';
const codecTheme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
const codecOwner = 'zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui';
const codecThemeSha = 'dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743';

function referencedPaintFixture() {
  const identity = { nodeId: 'root', sourcePath: codecOwner, sourceNodeId: 'root',
    controlId: 'WorkbenchButtonRoot', instancePath: '[]' };
  const document: ZuiDocument = {
    asset: { kind: 'view', id: 'res://codec-test.zui', version: 2 }, root: { node: 'root' },
    nodes: { root: { component: 'Button', control_id: identity.controlId,
      props: { text: 'Save', background_color: '$editor.surface.1', border_color: '$editor.surface.1',
        foreground_color: '$editor.surface.1', placeholder_color: '$editor.surface.1' },
      penpot_review_source_path: codecOwner, penpot_review_source_node_id: 'root',
      penpot_review_instance_path: '[]' } },
    tokens: { 'editor.surface.1': '#242424', 'editor.surface.disabled': '#242424',
      'editor.surface.tab.hover': '#242424cc' },
    penpot_source_fingerprints: [{ sourcePath: codecOwner, sha256: '1'.repeat(64) },
      { sourcePath: codecTheme, sha256: codecThemeSha }],
    penpot_host_theme_source: codecTheme,
    penpot_host_palette: { surface: ['#151515', '#242424'], surface_disabled: '#242424',
      surface_tab_hover: '#242424cc' },
    penpot_token_source_owners: Object.fromEntries(['editor.surface.1', 'editor.surface.disabled',
      'editor.surface.tab.hover'].map(token => [token, { sourcePath: codecTheme, sha256: codecThemeSha }])),
  };
  const mapping = { schema: 'zircon.penpot-token-source.v1', version: 1, sourcePath: codecTheme,
    sourceSha256: codecThemeSha, token: 'editor.surface.1', sourceTable: 'palette',
    sourceKey: 'surface', sourceIndex: 1 as number | null, originalValue: '#242424', penpotStyleId: rev5Style };
  const fill: Fill = { fillColor: '#242424', fillOpacity: 1,
    fillColorRefFile: rev5File, fillColorRefId: rev5Style };
  const stroke: Stroke = { strokeColor: '#242424', strokeOpacity: 1, strokeWidth: 2,
    strokeStyle: 'solid', strokeAlignment: 'inner', strokeColorRefFile: rev5File, strokeColorRefId: rev5Style };
  const color = { id: rev5Style, libraryId: rev5File, color: '#242424', opacity: 1,
    gradient: undefined, image: undefined,
    getSharedPluginData: (_namespace: string, key: string) => key === 'token-source' ? JSON.stringify(mapping) : '',
    asFill: () => ({ ...fill }), asStroke: () => ({ ...stroke }) };
  const libraries = [{ id: rev5File, colors: [color as unknown as LibraryColor] }];
  const context = () => createSourcePaintCaptureContext(document, () => libraries);
  return { document, identity, mapping, color, fill, stroke, libraries, context };
}

// Archived Widget rev17/18/20 fields, pinned in widget-r03-consumer/fixture.json.
// The expected-after paragraph is only the exact Parent-reviewed key move.
// SDK contract doubles below read shared namespaces as the official API does;
// they are not live SDK, typography-provider or original-owner export evidence.
const widgetR03Archived = {
  "fileId": "b4a1e3ce-9e6a-81fe-8008-d69d60d24223",
  "pageId": "b4a1e3ce-9e6a-81fe-8008-d69d60d24224",
  "textId": "485c980b-5100-806a-8008-d6e3f9f2db07",
  "ownerMetadata": {
    "shared/dev.zircon.zui": {
      "baseline": "{\"geometry\":{\"x\":0,\"y\":0,\"width\":240.00000953674316,\"height\":31.999999284744263},\"paint\":{\"fillColor\":\"#242424\",\"fillOpacity\":1,\"strokeColor\":\"#383838\",\"strokeOpacity\":1,\"strokeWidth\":1,\"borderRadius\":4,\"opacity\":1},\"text\":{\"characters\":\"Button\",\"property\":\"text\",\"color\":\"#c0c0c0\",\"colorOpacity\":1,\"fontSize\":13.333333,\"fontWeight\":\"700\",\"align\":\"center\",\"fontFamily\":\"Roboto\",\"lineHeight\":1.2},\"container\":{\"kind\":\"free\",\"direction\":\"column\",\"wrap\":false,\"gap\":0,\"rowGap\":0,\"columnGap\":0,\"columns\":1,\"rows\":1,\"padding\":{\"top\":0,\"right\":0,\"bottom\":0,\"left\":0},\"alignItems\":\"start\",\"justifyContent\":\"start\",\"clip\":false}}",
      "child-index": "0",
      "component": "Button",
      "controlId": "WorkbenchButtonRoot",
      "instancePath": "[]",
      "layout-mode": "free",
      "layout-sizing-guard": "{\"horizontal\":null,\"vertical\":null}",
      "native-layout-allocation": "{\"kind\":\"anchored\",\"width\":240.00000953674316,\"height\":32,\"parent\":\"6ddc248b-bd35-80c5-8008-d6c8bd5da088\"}",
      "node-id": "root",
      "prefab-role": "button",
      "role": "node",
      "sourceNodeId": "root",
      "sourcePath": "zircon_editor/assets/ui/editor/components/workbench/primitives/inputs/button.zui"
    },
    "shared/zircon-widget-shared-consumer": {
      "consumer": "{\"revision\":\"shared-library-consumer-r01\",\"sourceFile\":\"b4a1e3ce-9e6a-81fe-8008-d69d607aac12\",\"componentId\":\"6ddc248b-bd35-80c5-8008-d6c8c01096f3\",\"libraryRevision\":1,\"role\":\"Button / Palette\",\"state\":\"Shared default\",\"target\":{\"sourcePath\":\"zircon_editor/assets/ui/editor/ui_asset_editor.zui\",\"sourceNodeId\":\"palette_label\",\"controlId\":\"PaletteLabel\",\"note\":\"actual Label palette descriptor content; no insertion execution claim\"},\"sourceCodecVerified\":false,\"productBehaviorVerified\":false}"
    }
  },
  "textMetadata": {
    "shared/dev.zircon.zui": {
      "empty-text": "false",
      "node-id": "root",
      "role": "text",
      "text-base-weight": "700",
      "text-property": "text",
      "text-style-guard": "{\"fontStyle\":\"normal\",\"letterSpacing\":\"0\",\"textTransform\":\"none\",\"direction\":\"ltr\",\"textDecoration\":\"none\",\"verticalAlign\":\"center\",\"strokes\":[]}"
    }
  },
  "typographyMetadata": {
    "shared/dev.zircon.zui": {
      "token-source": "{\"schema\":\"zircon.penpot-token-source.v1\",\"version\":1,\"sourcePath\":\"zircon_editor/assets/ui/editor/theme/editor_tokens.zui\",\"sourceSha256\":\"dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743\",\"sourceTable\":\"typography\",\"token\":\"editor.typography.body\",\"sourceBindings\":{\"font-family\":\"editor.typography.ui.family\",\"font-size\":\"editor.typography.body.size\",\"font-weight\":\"editor.typography.body.weight\",\"line-height\":\"editor.typography.line_height\"},\"penpotStyleId\":\"99eb86f9-12ff-5935-96ad-3d1a985e61f0\"}"
    }
  },
  "paragraphMetadataBefore": {
    "dev.zircon.zui": {
      "token-source": "{\"schema\":\"zircon.penpot-token-source.v1\",\"version\":1,\"sourcePath\":\"zircon_editor/assets/ui/editor/theme/editor_tokens.zui\",\"sourceSha256\":\"dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743\",\"sourceTable\":\"typography\",\"token\":\"editor.typography.body\",\"sourceBindings\":{\"font-family\":\"editor.typography.ui.family\",\"font-size\":\"editor.typography.body.size\",\"font-weight\":\"editor.typography.body.weight\",\"line-height\":\"editor.typography.line_height\"},\"penpotStyleId\":\"99eb86f9-12ff-5935-96ad-3d1a985e61f0\"}"
    }
  },
  "paragraphMetadataAfter": {
    "shared/dev.zircon.zui": {
      "token-source": "{\"schema\":\"zircon.penpot-token-source.v1\",\"version\":1,\"sourcePath\":\"zircon_editor/assets/ui/editor/theme/editor_tokens.zui\",\"sourceSha256\":\"dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743\",\"sourceTable\":\"typography\",\"token\":\"editor.typography.body\",\"sourceBindings\":{\"font-family\":\"editor.typography.ui.family\",\"font-size\":\"editor.typography.body.size\",\"font-weight\":\"editor.typography.body.weight\",\"line-height\":\"editor.typography.line_height\"},\"penpotStyleId\":\"99eb86f9-12ff-5935-96ad-3d1a985e61f0\"}"
    }
  },
  "sdkBefore": {
    "id": "485c980b-5100-806a-8008-d6e3f9f2db07",
    "characters": "Button",
    "fontId": "gfont-roboto",
    "fontFamily": "Roboto",
    "fontSize": "13.333333",
    "fontWeight": "400",
    "lineHeight": "1.2",
    "fills": [
      {
        "fillColor": "#c0c0c0",
        "fillOpacity": 1,
        "fillColorGradient": null,
        "fillColorRefFile": null,
        "fillColorRefId": null,
        "fillImage": null
      }
    ],
    "x": 6245,
    "y": 144,
    "width": 220.00001525878906,
    "height": 32,
    "textBounds": {
      "x": 6335.390625,
      "y": 152,
      "width": 39.21875,
      "height": 16
    }
  },
  "sdkAfter": {
    "id": "485c980b-5100-806a-8008-d6e3f9f2db07",
    "characters": "Button",
    "fontId": "gfont-roboto",
    "fontFamily": "Roboto",
    "fontSize": "13.333333",
    "fontWeight": "400",
    "lineHeight": "1.2",
    "fills": [
      {
        "fillColor": "#c0c0c0",
        "fillOpacity": 1,
        "fillColorGradient": null,
        "fillColorRefFile": null,
        "fillColorRefId": null,
        "fillImage": null
      }
    ],
    "x": 6245,
    "y": 144,
    "width": 220.00001525878906,
    "height": 32,
    "textBounds": {
      "x": 6335.390625,
      "y": 152,
      "width": 39.21875,
      "height": 16
    }
  },
  "boardBefore": {
    "id": "485c980b-5100-806a-8008-d6e3f9f2db06",
    "name": "Shared default / WorkbenchButton",
    "x": 6235,
    "y": 144,
    "width": 240.00000953674316,
    "height": 31.999999284744263
  }
};

function widgetR03SdkInput(stage: 'before' | 'after') {
  const calls: Array<[string, string]> = [];
  const readShared = (data: Record<string, Record<string, string>>) =>
    (namespace: string, key: string): string => {
      calls.push([namespace, key]);
      return data['shared/' + namespace]?.[key] ?? '';
    };
  const readOnly = () => { throw new Error('Archived guards and metadata must not be rewritten'); };
  const privateRead = () => { throw new Error('Private metadata is not a shared-source fallback'); };
  const native = stage === 'before' ? widgetR03Archived.sdkBefore : widgetR03Archived.sdkAfter;
  const text = {
    ...native, type: 'text', name: 'ZUI text · root', strokes: [],
    fontVariantId: 'regular', fontStyle: 'normal', letterSpacing: '0',
    textTransform: 'none', direction: 'ltr', textDecoration: 'none',
    verticalAlign: 'center', align: 'center',
    getSharedPluginData: readShared(widgetR03Archived.textMetadata),
    getPluginData: privateRead, setSharedPluginData: readOnly,
    // Raw rich-content attributes are not shape-source metadata.
    paragraphPluginData: stage === 'before'
      ? widgetR03Archived.paragraphMetadataBefore : widgetR03Archived.paragraphMetadataAfter,
  } as unknown as Text;
  const board = {
    ...widgetR03Archived.boardBefore, type: 'board', children: [text],
    getSharedPluginData: readShared(widgetR03Archived.ownerMetadata),
    getPluginData: privateRead, setSharedPluginData: readOnly,
  } as unknown as Board;
  const typography = {
    getSharedPluginData: readShared(widgetR03Archived.typographyMetadata),
    getPluginData: privateRead,
  };
  return { board, text, typography, calls };
}

describe('archived Widget r03 shared metadata', () => {
  it.each(['before', 'after'] as const)(
    'keeps %s source identity through SDK shared access without accepting the archived font guard',
    (stage) => {
      const input = widgetR03SdkInput(stage), f = referencedPaintFixture();
      const original = structuredClone(widgetR03Archived);
      const context = f.context();
      const bindings: Parameters<typeof context.binding>[] = [];
      const paintContext = { ...context, binding: (...args: Parameters<typeof context.binding>) => {
        bindings.push(args);
        return context.binding(...args);
      } };
      const baseline = JSON.parse(widgetR03Archived.ownerMetadata['shared/dev.zircon.zui'].baseline);
      expect(() => captureNodeTexts(input.board, baseline, f.document, paintContext))
        .toThrow(/changed a text style outside the .zui bridge profile/);
      expect(bindings).toEqual([[
        { nodeId: 'root', sourcePath: codecOwner, sourceNodeId: 'root',
          controlId: 'WorkbenchButtonRoot', instancePath: '[]' },
        'foreground_color', 'text', 'primary',
      ]]);
      const encoded = input.typography.getSharedPluginData('dev.zircon.zui', 'token-source');
      expect(encoded).toBe(widgetR03Archived.paragraphMetadataBefore['dev.zircon.zui']['token-source']);
      expect(encoded).toBe(widgetR03Archived.paragraphMetadataAfter['shared/dev.zircon.zui']['token-source']);
      expect(JSON.parse(encoded)).toMatchObject({
        sourcePath: codecTheme, sourceSha256: codecThemeSha, sourceTable: 'typography',
        token: 'editor.typography.body', penpotStyleId: '99eb86f9-12ff-5935-96ad-3d1a985e61f0',
      });
      expect(input.calls.every(([namespace]) => namespace === 'dev.zircon.zui')).toBe(true);
      // This old guard lacks provider IDs and the body record is weight400.
      // Do not rewrite it to turn a metadata-storage observation into approval.
      expect(JSON.parse(widgetR03Archived.textMetadata['shared/dev.zircon.zui']['text-style-guard']).fontId)
        .toBeUndefined();
      expect(input.text.fontId).toBe('gfont-roboto');
      expect(input.text.fontWeight).toBe('400');
      expect(widgetR03Archived).toEqual(original);
    },
  );

  it.each(['unknown namespace', 'conflicting shared record', 'typography record'] as const)(
    'rejects %s instead of normalizing or falling back during source-paint capture', (kind) => {
      const f = referencedPaintFixture();
      const correct = JSON.stringify(f.mapping);
      const data: Record<string, Record<string, string>> = {
        'dev.zircon.zui': { 'token-source': correct },
        'shared/unrecognized.zircon.zui': { 'token-source': correct },
      };
      if (kind === 'conflicting shared record')
        data['shared/dev.zircon.zui'] = {
          'token-source': JSON.stringify({ ...f.mapping, sourceSha256: '0'.repeat(64) }),
        };
      if (kind === 'typography record')
        data['shared/dev.zircon.zui'] = { ...widgetR03Archived.typographyMetadata['shared/dev.zircon.zui'] };
      const before = structuredClone(data), calls: Array<[string, string]> = [];
      f.color.getSharedPluginData = (namespace: string, key: string) => {
        calls.push([namespace, key]);
        return data['shared/' + namespace]?.[key] ?? '';
      };
      expect(() => singleSolidFill([f.fill], 'Widget shared metadata adversary',
        f.context().binding(f.identity, 'background_color')))
        .toThrow(kind === 'unknown namespace' ? /no valid token-source/ : /token\/source/);
      expect(calls).toEqual([['dev.zircon.zui', 'token-source']]);
      expect(data).toEqual(before);
      expect(f.fill.fillColorRefFile).toBe(rev5File);
      expect(f.fill.fillColorRefId).toBe(rev5Style);
    },
  );
});

describe('strict source-owned shared paint capture', () => {
  it('retains the actual native Shared rev5 UUID pair for fill, stroke and text bindings', () => {
    const f = referencedPaintFixture(), context = f.context();
    expect(singleSolidFill([f.fill], 'button', context.binding(f.identity, 'background_color'))).toEqual(f.fill);
    expect(singleSolidStroke([f.stroke], 'button', context.binding(f.identity, 'border_color'))).toEqual(f.stroke);
    expect(singleSolidFill([f.fill], 'caption', context.binding(f.identity, 'foreground_color', 'text', 'primary')))
      .toEqual(f.fill);
    expect(f.document.nodes!['root'].props!['background_color']).toBe('$editor.surface.1');
  });

  it('passes that same operation through production text capture without dropping native references', () => {
    const f = referencedPaintFixture(), context = f.context();
    const boardData: Record<string, string> = { 'node-id': 'root', sourcePath: codecOwner,
      sourceNodeId: 'root', controlId: f.identity.controlId, instancePath: '[]' };
    const textData: Record<string, string> = { role: 'text', 'node-id': 'root', 'text-property': 'text' };
    const board = { type: 'board', name: 'source root', x: 0, y: 0, children: [],
      getSharedPluginData: (_namespace: string, key: string) => boardData[key] ?? '' } as unknown as Board;
    const text = { type: 'text', name: 'caption', characters: 'Save', x: 4, y: 4, width: 120, height: 24,
      fontId: 'custom-source-roboto', fontVariantId: 'regular', fontFamily: 'Roboto', fontSize: '13.333333',
      fontWeight: '400', align: 'center', lineHeight: '1.2', fills: [f.fill], strokes: [],
      getSharedPluginData: (_namespace: string, key: string) => textData[key] ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) => { textData[key] = value; },
    } as unknown as Text;
    textData['text-style-guard'] = JSON.stringify(unsupportedTextStyle(text));
    guardControlTextGeometry(board, text);
    (board.children as Text[]).push(text);
    singleSolidFill([f.fill], 'source root', context.binding(f.identity, 'background_color'));
    expect(captureNodeTexts(board, undefined, f.document, context).text)
      .toMatchObject({ characters: 'Save', property: 'text', color: '#242424', colorOpacity: 1 });
    expect(text.fills).toEqual([f.fill]);
    text.fontId = 'gfont-roboto';
    expect(() => captureNodeTexts(board, undefined, f.document, context)).toThrow(/outside the .zui bridge profile/);
    text.fontId = 'custom-source-roboto'; text.x += 1;
    expect(() => captureNodeTexts(board, undefined, f.document, context)).toThrow(/native geometry/);
  });

  it('rejects missing and forged contexts, partial or unknown reference pairs', () => {
    const f = referencedPaintFixture(), binding = f.context().binding(f.identity, 'background_color');
    expect(() => singleSolidFill([f.fill], 'button')).toThrow(/context/);
    expect(() => singleSolidFill([f.fill], 'button', { kind: 'source-paint-binding' })).toThrow(/context/);
    for (const change of [{ fillColorRefId: undefined }, { fillColorRefFile: undefined },
      { fillColorRefId: '' }, { fillColorRefFile: '' },
      { fillColorRefId: '00000000-0000-0000-0000-000000000000' },
      { fillColorRefFile: '00000000-0000-0000-0000-000000000000' }])
      expect(() => singleSolidFill([{ ...f.fill, ...change }], 'button', binding)).toThrow(/Referenced paint/);
  });

  it.each(['sourcePath', 'sourceNodeId', 'controlId', 'instancePath'] as const)(
    'rejects a different original %s even when the color matches', (key) => {
      const f = referencedPaintFixture();
      expect(() => singleSolidFill([f.fill], 'button', f.context().binding(
        { ...f.identity, [key]: 'wrong' }, 'background_color'))).toThrow(/identity|instancePath/);
    },
  );

  it('requires every invocation to be canonical and source-pinned', () => {
    const f = referencedPaintFixture();
    for (const instancePath of [' [ ] ', '[{"sourcePath":"unowned.zui","sourceNodeId":"root"}]']) {
      f.document.nodes!['root']['penpot_review_instance_path'] = instancePath;
      expect(() => singleSolidFill([f.fill], 'button', f.context().binding(
        { ...f.identity, instancePath }, 'background_color'))).toThrow(/instancePath|invocation/);
    }
  });

  it('rejects stale dependency SHA, missing owners, duplicate pins and wrong actual library ownership', () => {
    const changes: Array<(f: ReturnType<typeof referencedPaintFixture>) => void> = [
      f => { f.mapping.sourceSha256 = '2'.repeat(64); },
      f => { f.document['penpot_token_source_owners'] = {}; },
      f => { (f.document['penpot_source_fingerprints'] as unknown[]).push(
        { sourcePath: codecTheme, sha256: codecThemeSha }); },
      f => { f.color.libraryId = '00000000-0000-0000-0000-000000000000'; },
      f => { f.libraries.length = 0; },
    ];
    for (const change of changes) {
      const f = referencedPaintFixture(); change(f);
      expect(() => singleSolidFill([f.fill], 'button', f.context().binding(f.identity, 'background_color')))
        .toThrow(/Referenced paint/);
    }
  });

  it('rejects another same-color token or palette key instead of inferring ownership from RGB', () => {
    const f = referencedPaintFixture();
    f.document.nodes!['root'].props!['background_color'] = '$editor.surface.disabled';
    expect(() => singleSolidFill([f.fill], 'button', f.context().binding(f.identity, 'background_color')))
      .toThrow(/token\/source/);
    f.mapping.token = 'editor.surface.disabled';
    // Claiming that token while retaining surface[1] is still a mismatch.
    expect(() => singleSolidFill([f.fill], 'button', f.context().binding(f.identity, 'background_color')))
      .toThrow(/token\/source/);
    f.mapping.sourceKey = 'surface_disabled'; f.mapping.sourceIndex = null;
    expect(singleSolidFill([f.fill], 'button', f.context().binding(f.identity, 'background_color'))).toEqual(f.fill);
    f.document.nodes!['root'].props!['background_color'] = '#242424';
    expect(() => singleSolidFill([f.fill], 'button', f.context().binding(f.identity, 'background_color')))
      .toThrow(/original visual property token/);
  });

  it('preserves exact RGBA alpha and the placeholder token, including scalar null sourceIndex', () => {
    const f = referencedPaintFixture();
    f.document.nodes!['root'].props!['placeholder_color'] = '$editor.surface.tab.hover';
    Object.assign(f.mapping, { token: 'editor.surface.tab.hover', sourceKey: 'surface_tab_hover',
      sourceIndex: null, originalValue: '#242424cc' });
    f.fill.fillOpacity = f.color.opacity = 204 / 255;
    const binding = f.context().binding(f.identity, 'foreground_color', 'placeholder', 'primary');
    expect(singleSolidFill([f.fill], 'placeholder', binding)).toEqual(f.fill);
    expect(() => singleSolidFill([{ ...f.fill, fillOpacity: 1 }], 'placeholder', binding)).toThrow(/opacity/);
  });

  it('checks actual SDK record values as well as asFill, and rechecks source mutations', () => {
    for (const change of [{ color: '#242425' }, { opacity: 0 }, { opacity: Number.NaN },
      { gradient: {} }, { image: {} }]) {
      const f = referencedPaintFixture(); Object.assign(f.color, change);
      expect(() => singleSolidFill([f.fill], 'button', f.context().binding(f.identity, 'background_color')))
        .toThrow(/SDK library record/);
    }
    const f = referencedPaintFixture(), context = f.context();
    (f.document['penpot_host_palette'] as Record<string, unknown>)['surface_disabled'] = '#ffffff';
    expect(() => singleSolidFill([f.fill], 'button', context.binding(f.identity, 'background_color')))
      .toThrow(/closure changed/);
    const changed = referencedPaintFixture(), changedContext = changed.context();
    const binding = changedContext.binding(changed.identity, 'background_color');
    singleSolidFill([changed.fill], 'button', binding);
    changed.document.nodes!['root'].props!['background_color'] = '#242424';
    expect(() => singleSolidFill([changed.fill], 'button', binding)).toThrow(/closure changed/);
  });

  it('continues rejecting mixed, multi-paint, gradient/image and unsupported strokes with a valid context', () => {
    const f = referencedPaintFixture(), context = f.context();
    const fillBinding = context.binding(f.identity, 'background_color');
    for (const fills of ['mixed', [f.fill, f.fill], [{ ...f.fill, fillColorGradient: {} }],
      [{ ...f.fill, fillImage: {} }]] as unknown as Array<readonly Fill[] | 'mixed'>)
      expect(() => singleSolidFill(fills, 'button', fillBinding)).toThrow(/bridge only exports/);
    for (const change of [{ strokeStyle: 'dashed' }, { strokeAlignment: 'center' },
      { strokeCapStart: 'round' }, { strokeCapEnd: 'round' }, { strokeColorGradient: {} }, { strokeImage: {} }])
      expect(() => singleSolidStroke([{ ...f.stroke, ...change } as Stroke], 'button',
        context.binding(f.identity, 'border_color'))).toThrow(/unsupported stroke/);
    expect(() => singleSolidStroke([f.stroke], 'button', fillBinding)).toThrow(/wrong visual property/);
    expect(() => singleSolidFill([{ ...f.fill, fillColor: '#ffffff' }], 'button', fillBinding)).toThrow(/value\/opacity/);
  });

  it('keeps same-family provider/variant changes outside the supported text style guard', () => {
    const text = { fontId: 'custom-495ff285-source', fontVariantId: 'regular', fontFamily: 'Roboto' } as Text;
    const original = JSON.stringify(unsupportedTextStyle(text));
    expect(JSON.stringify(unsupportedTextStyle({ ...text, fontId: 'gfont-roboto' } as Text))).not.toBe(original);
    expect(JSON.stringify(unsupportedTextStyle({ ...text, fontVariantId: '700' } as Text))).not.toBe(original);
  });
});


// Minimal SDK/source fixtures derived from r63's five archived states.
// Native persistence is archived evidence; these tests do not replay the SDK.
describe('r63 native copy paint ownership', () => {
  const stages = [
    { name: 'default', override: false },
    { name: 'override', override: true },
    { name: 'override-reopen', override: true },
    { name: 'reset', override: false },
    { name: 'reset-reopen', override: false },
  ] as const;
  const sourceHash = '05c1245c915f75b1f3b1be025942f2d05bc985e88d8a75db1f168c554563f3e5';

  it.each(stages)('preserves the archived $name paint and rejects promotion of local overrides', (stage) => {
    const f = referencedPaintFixture();
    f.document.asset.kind = 'component';
    delete f.document.root;
    f.document.components = { WorkbenchButton: { root: 'root' } };
    (f.document['penpot_source_fingerprints'] as Array<{ sha256: string }>)[0].sha256 = sourceHash;
    const native = {
      id: '4d7f131f-5bed-803b-8008-d72e9bbaae38',
      shapeRef: '6ddc248b-bd35-80c5-8008-d6c8bd9b057f',
      fills: [stage.override ? { fillColor: '#ffb800', fillOpacity: 1 } : { ...f.fill }] as Fill[],
      touched: stage.override ? ['fill-group'] : [],
      instancePath: '[]',
    };
    const before = structuredClone(native);
    const context = f.context();
    const captured = singleSolidFill(native.fills, native.id,
      context.binding(f.identity, 'background_color'))!;
    expect(captured).toEqual(native.fills[0]);
    const shape = cloneProjectionSnapshot(projectZuiDocument(f.document)).shapes[0];
    Object.assign(shape.current.paint, {
      fillColor: captured.fillColor, fillOpacity: captured.fillOpacity,
    });
    const capture = () => assertCanonicalCopyPaintUnchanged(
      f.document, 'root', true, shape.baseline, shape.current);
    if (stage.override) expect(capture).toThrow(/consumer source owner.*canonical component/);
    else expect(capture).not.toThrow();
    expect(native).toEqual(before);
    expect(f.document.nodes!['root'].props!['background_color']).toBe('$editor.surface.1');
  });

  it('does not block an owned main edit or manufacture a new baseline for a copy', () => {
    const f = referencedPaintFixture();
    f.document.asset.kind = 'component';
    delete f.document.root;
    f.document.components = { WorkbenchButton: { root: 'root' } };
    const shape = cloneProjectionSnapshot(projectZuiDocument(f.document)).shapes[0];
    shape.current.paint.fillColor = '#ffb800';
    expect(() => assertCanonicalCopyPaintUnchanged(f.document, 'root', false,
      shape.baseline, shape.current)).not.toThrow();
    expect(() => assertCanonicalCopyPaintUnchanged(f.document, 'root', true,
      undefined, shape.current)).not.toThrow();
    expect(shape.baseline.paint.fillColor).toBe('#242424');
  });


  it('leaves an authored nested copy under the existing consumer-owner checks', () => {
    const f = referencedPaintFixture();
    f.document.asset.kind = 'component';
    delete f.document.root;
    f.document.components = { ConsumerPanel: { root: 'panel' } };
    f.document.nodes!['panel'] = { component: 'Container', children: [{ node: 'root' }] };
    const shape = cloneProjectionSnapshot(projectZuiDocument(f.document)).shapes
      .find(entry => entry.nodeId === 'root')!;
    shape.current.paint.fillColor = '#ffb800';
    expect(() => assertCanonicalCopyPaintUnchanged(f.document, 'root', true,
      shape.baseline, shape.current)).not.toThrow();
    expect(shape.current.paint.fillColor).toBe('#ffb800');
  });

  it.each(['strokeColor', 'strokeOpacity', 'strokeWidth', 'borderRadius', 'opacity'] as const)(
    'rejects a canonical-copy %s override as well as fill changes', (property) => {
      const f = referencedPaintFixture();
      f.document.asset.kind = 'component';
      delete f.document.root;
      f.document.components = { WorkbenchButton: { root: 'root' } };
      const shape = cloneProjectionSnapshot(projectZuiDocument(f.document)).shapes[0];
      if (property === 'strokeColor') shape.current.paint[property] = '#ffb800';
      else shape.current.paint[property] += 0.25;
      expect(() => assertCanonicalCopyPaintUnchanged(f.document, 'root', true,
        shape.baseline, shape.current)).toThrow(/consumer source owner/);
    },
  );

  it('rejects local caption and fragment paint promotion without mutating their source', () => {
    for (const fragment of [false, true]) {
      const f = referencedPaintFixture();
      f.document.asset.kind = 'component';
      delete f.document.root;
      f.document.components = { WorkbenchButton: { root: 'root' } };
      const shape = cloneProjectionSnapshot(projectZuiDocument(f.document)).shapes[0];
      expect(shape.baseline.text).not.toBeNull();
      if (fragment) {
        shape.baseline.textFragments = { value: structuredClone(shape.baseline.text!) };
        shape.current.textFragments = { value: { ...shape.baseline.text!, color: '#ffb800' } };
      } else shape.current.text!.colorOpacity = 0.5;
      const before = structuredClone(shape);
      expect(() => assertCanonicalCopyPaintUnchanged(f.document, 'root', true,
        shape.baseline, shape.current)).toThrow(/consumer source owner/);
      expect(shape).toEqual(before);
    }
  });
});

describe('Penpot height.min capture capability', () => {
  it.each(['host', 'free-child'])(
    'leaves the authored minimum of %s unmapped without an auto-layout parent', (nodeId) => {
      expect(capturedHeightMin(nodeId, 800, false, undefined)).toBeUndefined();
      expect(capturedHeightMin(nodeId, 800, false, undefined, {})).toBeUndefined();
    },
  );

  it.each([0, 96, 128])('captures the actual supported minimum %s', (heightMin) => {
    expect(capturedHeightMin('outliner', 96, true, { minHeight: heightMin })).toBe(heightMin);
    expect(capturedHeightMin('outliner', 96, true, { minHeight: heightMin }, { heightMin: 96 }))
      .toBe(heightMin);
  });

  it.each([undefined, null, {}, { minHeight: undefined }, { minHeight: null },
    { minHeight: Number.NaN }, { minHeight: Number.POSITIVE_INFINITY },
    { minHeight: -1 }, { minHeight: '96' }])(
    'rejects a missing or invalid supported SDK minimum %j', (layoutChild) => {
      expect(() => capturedHeightMin('outliner', 96, true, layoutChild))
        .toThrow(/no finite nonnegative Penpot minHeight/);
      expect(() => capturedHeightMin('outliner', 96, true, layoutChild, { heightMin: 96 }))
        .toThrow(/no finite nonnegative Penpot minHeight/);
    },
  );

  it.each([undefined, null, { minHeight: 96 }])(
    'rejects loss of an existing mapping even when its last SDK value is %j', (layoutChild) => {
      expect(() => capturedHeightMin('outliner', 96, false, layoutChild, { heightMin: 96 }))
        .toThrow(/lost its Penpot minHeight capability/);
    },
  );

  it('rejects a newly introduced mapping or an inconsistent parent capability', () => {
    expect(() => capturedHeightMin('outliner', 96, true, { minHeight: 128 }, {}))
      .toThrow(/introduced a Penpot minHeight mapping/);
    expect(() => capturedHeightMin('host', 800, false, { minHeight: 800 }))
      .toThrow(/Penpot minHeight capability/);
    expect(() => capturedHeightMin('outliner', null, true, { minHeight: 96 }, { heightMin: 96 }))
      .toThrow(/lost its authored height.min mapping/);
  });

  it('does not make an SDK-only constraint an authored editable field', () => {
    expect(capturedHeightMin('unmapped', null, true, { minHeight: 96 })).toBeUndefined();
    expect(capturedHeightMin('unmapped', null, true, { minHeight: 96 }, {})).toBeUndefined();
  });
});

describe('Penpot capture paint validation', () => {
  it('captures editable family and line height while rejecting mixed or invalid text', () => {
    expect(capturedFontTypography('Fira Sans', '1.5', 'title')).toEqual({
      fontFamily: 'Fira Sans',
      lineHeight: 1.5,
    });
    expect(() => capturedFontTypography('mixed', '1.5', 'title')).toThrow(
      'font family',
    );
    expect(() => capturedFontTypography('Fira Sans', 'mixed', 'title')).toThrow(
      'line height',
    );
    expect(() => capturedFontTypography('Fira Sans', '0', 'title')).toThrow(
      'line height',
    );
  });
  it('places semantic grid children with Penpot one-based coordinates', () => {
    expect(gridCellForChildIndex(0)).toEqual({ row: 1, column: 1 });
    expect(gridCellForChildIndex(8)).toEqual({ row: 9, column: 1 });
    expect(gridCellForChildIndex(0, 3)).toEqual({ row: 1, column: 1 });
    expect(gridCellForChildIndex(2, 3)).toEqual({ row: 1, column: 3 });
    expect(gridCellForChildIndex(3, 3)).toEqual({ row: 2, column: 1 });
    expect(() => gridCellForChildIndex(-1)).toThrow(/non-negative integer/i);
    expect(() => gridCellForChildIndex(0, 0)).toThrow(/positive integer/i);
  });

  it('restores semantic sibling order after Penpot grid traversal', () => {
    expect(
      stableSemanticChildOrder(
        ['warlock', 'warrior', 'mage'],
        new Map([
          ['warrior', 0],
          ['mage', 1],
          ['warlock', 2],
        ]),
      ),
    ).toEqual(['warrior', 'mage', 'warlock']);
  });

  it('restores a Penpot placeholder to the authored empty text', () => {
    expect(capturedSemanticText(EMPTY_TEXT_SENTINEL, true)).toBe('');
    expect(capturedSemanticText(`${EMPTY_TEXT_SENTINEL}Edited`, true)).toBe(
      'Edited',
    );
    expect(capturedSemanticText(EMPTY_TEXT_SENTINEL, false)).toBe(
      EMPTY_TEXT_SENTINEL,
    );
  });

  it('rejects mixed mapped text styles and asymmetric layout gaps', () => {
    expect(() =>
      capturedMappedTextStyle('mixed', '400', 'left', 'Editable text'),
    ).toThrow(/mixed or invalid font size/i);
    expect(() =>
      capturedMappedTextStyle('12', 'mixed', 'left', 'Editable text'),
    ).toThrow(/mixed or invalid font weight/i);
    expect(() =>
      capturedMappedTextStyle('12', '400', 'mixed', 'Editable text'),
    ).toThrow(/mixed or missing text alignment/i);
    expect(() => symmetricLayoutGap(8, 12, 'Semantic node root')).toThrow(
      /asymmetric row and column gaps/i,
    );
    expect(symmetricLayoutGap(8, 8, 'Semantic node root')).toBe(8);
  });

  it('suppresses auto-layout geometry while retaining fixed-size edits', () => {
    const baseline = { x: 10, y: 20, width: 100, height: 40 };
    const laidOut = { x: 30, y: 50, width: 180, height: 48 };

    expect(authoredGeometry(laidOut, baseline, true, 'fill', 'fix')).toEqual({
      x: 10,
      y: 20,
      width: 100,
      height: 48,
    });
    expect(authoredGeometry(laidOut, baseline, false, 'fill', 'fill')).toEqual(
      laidOut,
    );
  });

  it('resolves and reverses anchor, pivot, and offset positioning', () => {
    const resolved = resolvedAnchoredPosition(
      { width: 960, height: 640 },
      { width: 260, height: 106 },
      { x: 1, y: 0 },
      { x: 1, y: 0 },
      { x: -24, y: 226 },
    );

    expect(resolved).toEqual({ x: 676, y: 226 });
    expect(
      authoredAnchoredPosition(
        { width: 960, height: 640 },
        { width: 260, height: 106 },
        { x: 1, y: 0 },
        { x: 1, y: 0 },
        resolved,
      ),
    ).toEqual({ x: -24, y: 226 });
  });

  it('accepts only derived sizing availability across parent mode transitions', () => {
    const unavailable = { horizontal: null, vertical: null };
    const fixed = { horizontal: 'fix', vertical: 'fix' };
    const edited = { horizontal: 'fill', vertical: 'fix' };

    expect(layoutSizingIsCompatible(unavailable, fixed, 'free', 'flex')).toBe(
      true,
    );
    expect(layoutSizingIsCompatible(fixed, unavailable, 'grid', 'free')).toBe(
      true,
    );
    expect(layoutSizingIsCompatible(unavailable, edited, 'free', 'grid')).toBe(
      false,
    );
    expect(layoutSizingIsCompatible(fixed, edited, 'flex', 'grid')).toBe(false);
  });

  it('resolves reparented layout contexts from source and current hierarchies', () => {
    const baselineKinds = new Map([
      ['source-free', 'free' as const],
      ['source-auto', 'flex' as const],
    ]);
    const currentKinds = new Map([
      ['target-auto', 'grid' as const],
      ['target-free', 'free' as const],
    ]);

    expect(
      resolvedParentContainerKinds(
        'source-free',
        'target-auto',
        baselineKinds,
        currentKinds,
      ),
    ).toEqual({ baseline: 'free', current: 'grid' });
    expect(
      resolvedParentContainerKinds(
        'source-auto',
        'target-free',
        baselineKinds,
        currentKinds,
      ),
    ).toEqual({ baseline: 'flex', current: 'free' });
  });

  it('accepts the supported single-solid paint profile', () => {
    expect(
      singleSolidFill([{ fillColor: '#112233', fillOpacity: 0.5 }], 'node'),
    ).toMatchObject({ fillColor: '#112233' });
    expect(
      singleSolidStroke(
        [
          {
            strokeColor: '#445566',
            strokeStyle: 'solid',
            strokeAlignment: 'inner',
          },
        ],
        'node',
      ),
    ).toMatchObject({ strokeColor: '#445566' });
  });

  it('accepts null stroke caps returned by the Penpot StrokeProxy', () => {
    expect(
      singleSolidStroke(
        [
          {
            strokeColor: '#445566',
            strokeStyle: 'solid',
            strokeAlignment: 'inner',
            strokeCapStart: null,
            strokeCapEnd: null,
          } as never,
        ],
        'node',
      ),
    ).toMatchObject({ strokeColor: '#445566' });
  });

  it('rejects paint structures that .zui cannot represent', () => {
    expect(() =>
      singleSolidFill(
        [{ fillColor: '#112233' }, { fillColor: '#445566' }],
        'node',
      ),
    ).toThrow(/multiple fills/i);
    expect(() =>
      singleSolidFill([{ fillColorGradient: {} as never }], 'node'),
    ).toThrow(/non-solid/i);
    expect(() =>
      singleSolidStroke(
        [{ strokeColor: '#112233', strokeStyle: 'dashed' }],
        'node',
      ),
    ).toThrow(/unsupported stroke/i);
  });
});
