import { createHash } from 'node:crypto';
import type { Fill, LibraryColor } from '@penpot/plugin-types';
import { assertCanonicalCopyPaintUnchanged, createSourcePaintCaptureContext, singleSolidFill } from '../penpot-capture-validation';
import { restoreOriginalZuiSource } from './zui-source-export';
import { applyZuiExportToSourceFiles } from './zui-export-apply-sources';
import fixtureSource from './roundtrip-fixture.zui?raw';
import { parseZuiDocument, serializeZuiDocument, type ZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { reconcileReviewSource } from './zui-review-host';
import {
  createPenpotBridgeAsset,
  parsePenpotBridgeAsset,
  serializePenpotBridgeAsset,
  reconcilePenpotBridgeAsset,
} from './penpot-asset';

function sharedRev5OwnerFixture() {
  const sourcePath = 'zircon_editor/assets/ui/tests/style_owner.zui';
  const themePath = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const themeSha = 'dc23135b1e349773266e09cfb354c01bf047774ac0beb70a6a129e5ab4f66743';
  const file = 'b4a1e3ce-9e6a-81fe-8008-d69d607aac12';
  const id = '599586e3-022c-500b-ac80-5d9c490139af';
  const source = `# Keep comments, binding, imports, stable IDs and the original token.
[asset]
kind = "view"
id = "res://ui/tests/style_owner.zui"
version = 2
[imports]
styles = ["res://ui/editor/theme/editor_tokens.zui"]
[tokens]
"editor.surface.1" = "#242424"
[root]
node = "root"
[nodes.root]
component = "Button"
control_id = "StyleOwnerButton"
props = { text = "Save", background_color = "$editor.surface.1", value = { binding = "$document.value" } }
layout = { width = 240, height = 32 }
events = [{ kind = "Click", route = "style_owner.save" }]
`;
  const document = parseZuiDocument(source).document;
  Object.assign(document, { penpot_original_source: source, penpot_original_source_path: sourcePath,
    penpot_host_theme_source: themePath, penpot_host_palette: { surface: ['#151515', '#242424'] },
    penpot_token_source_owners: { 'editor.surface.1': { sourcePath: themePath, sha256: themeSha } },
    penpot_source_fingerprints: [{ sourcePath, sha256: createHash('sha256').update(source).digest('hex') },
      { sourcePath: themePath, sha256: themeSha }] });
  Object.assign(document.nodes!['root'], { penpot_review_source_path: sourcePath,
    penpot_review_source_node_id: 'root', penpot_review_instance_path: '[]' });
  const mapping = { schema: 'zircon.penpot-token-source.v1', version: 1,
    sourcePath: themePath, sourceSha256: themeSha, token: 'editor.surface.1',
    sourceTable: 'palette', sourceKey: 'surface', sourceIndex: 1, originalValue: '#242424', penpotStyleId: id };
  const nativeFill: Fill = { fillColor: '#242424', fillOpacity: 1, fillColorRefFile: file, fillColorRefId: id };
  const style = { id, libraryId: file, color: '#242424', opacity: 1,
    getSharedPluginData: (_namespace: string, key: string) => key === 'token-source' ? JSON.stringify(mapping) : '',
    asFill: () => ({ ...nativeFill }) };
  const libraries = [{ id: file, colors: [style as unknown as LibraryColor] }];
  const context = createSourcePaintCaptureContext(document, () => libraries);
  const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
  const identity = { nodeId: 'root', sourcePath, sourceNodeId: 'root', controlId: 'StyleOwnerButton', instancePath: '[]' };
  const capturedFill = singleSolidFill([nativeFill], 'native rev5 button', context.binding(identity, 'background_color'))!;
  const shape = snapshot.shapes.find(entry => entry.nodeId === 'root')!;
  Object.assign(shape.current.paint, { fillColor: capturedFill.fillColor, fillOpacity: capturedFill.fillOpacity });
  context.attach(snapshot);
  return { source, sourcePath, document, snapshot, context, nativeFill, capturedFill, mapping, style, libraries };
}

describe('validated shared paints in original-owner source export', () => {
  it('preserves the actual rev5 ref pair and the exact original owner bytes on a no-edit round trip', () => {
    const f = sharedRev5OwnerFixture(), before = structuredClone(f.document);
    const result = reconcileReviewSource(f.document, f.snapshot);
    expect(result.changes).toEqual([]);
    expect(result.source).toBe(f.source);
    expect(f.document).toEqual(before);
    expect(f.nativeFill).toEqual(f.capturedFill);
    const owner = parseZuiDocument(result.source!).document;
    expect(owner.nodes!['root'].props!['background_color']).toBe('$editor.surface.1');
    expect(owner.nodes!['root'].props!['value']).toEqual({ binding: '$document.value' });
    expect(owner.nodes!['root'].control_id).toBe('StyleOwnerButton');
    expect(owner.nodes!['root'].events).toEqual(f.document.nodes!['root'].events);
    expect(owner.imports).toEqual(f.document.imports);
  });

  it('keeps the reference context through a mapped width edit instead of writing a paint literal', () => {
    const f = sharedRev5OwnerFixture();
    // Make the user edit before sealing the fresh SDK capture, as production does.
    const snapshot = structuredClone(f.snapshot);
    snapshot.shapes[0].current.geometry.width += 16;
    f.context.attach(snapshot);
    const result = reconcileReviewSource(f.document, snapshot);
    const owner = parseZuiDocument(result.source!).document;
    expect(result.changes.length).toBeGreaterThan(0);
    expect(owner.nodes!['root'].props!['background_color']).toBe('$editor.surface.1');
    expect(owner.nodes!['root'].control_id).toBe('StyleOwnerButton');
    expect(owner.nodes!['root'].events).toEqual(f.document.nodes!['root'].events);
  });

  it('checks token expressions before the no-changes fast return, rejecting same-color literals and tokens', () => {
    for (const replacement of ['#242424', '$editor.surface.disabled']) {
      const f = sharedRev5OwnerFixture();
      const result = reconcileZuiDocument(f.document, f.snapshot);
      expect(result.changes).toEqual([]);
      result.document.nodes!['root'].props!['background_color'] = replacement;
      expect(() => restoreOriginalZuiSource(f.document, result, f.snapshot)).toThrow(/original visual token/);
    }
  });

  it('rejects changed SDK style records, stale dependency SHA and token resolution after capture', () => {
    for (const change of [
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.style.opacity = 0.5; },
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.mapping.sourceSha256 = '0'.repeat(64); },
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.libraries.length = 0; },
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.document.tokens!['editor.surface.1'] = '#ffffff'; },
    ]) {
      const f = sharedRev5OwnerFixture(), result = reconcileZuiDocument(f.document, f.snapshot);
      change(f);
      expect(() => restoreOriginalZuiSource(f.document, result, f.snapshot)).toThrow(/Referenced paint/);
    }
  });

  it('does not treat source metadata changes, geometry changes or topology tampering as color approval', () => {
    for (const change of [
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.snapshot.shapes[0].sourcePath = 'wrong'; },
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.snapshot.shapes[0].current.geometry.width += 1; },
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.snapshot.shapes[0].childNodeIds.push('unowned'); },
      (f: ReturnType<typeof sharedRev5OwnerFixture>) => { f.snapshot.shapes[0].current.paint.fillOpacity = 0; },
    ]) {
      const f = sharedRev5OwnerFixture(), result = reconcileZuiDocument(f.document, f.snapshot);
      change(f);
      expect(() => restoreOriginalZuiSource(f.document, result, f.snapshot)).toThrow(/capture\/source closure/);
    }
  });

  it('validates composed host ownership on the no-edit return using the outer owner pins', () => {
    const f = sharedRev5OwnerFixture();
    const outer = structuredClone(f.document), host = structuredClone(f.document);
    outer['penpot_review_host'] = host;
    expect(reconcileReviewSource(outer, f.snapshot).source).toBe(f.source);
    outer['penpot_source_fingerprints'] = [{ sourcePath: f.sourcePath, sha256: '0'.repeat(64) }];
    expect(() => reconcileReviewSource(outer, f.snapshot)).toThrow(/fingerprints differ/);
  });
});


describe('r63 local paint export boundary', () => {
  it('keeps a literal override local when a view source actually owns the edited node', () => {
    // This direct view-owner fixture has a real raw source node. The archived
    // r63 Assets copy has only inherited component-root metadata instead.
    const f = sharedRev5OwnerFixture();
    const sourceBefore = structuredClone(f.document);
    const snapshot = structuredClone(f.snapshot);
    const shape = snapshot.shapes[0];
    const fill: Fill = { fillColor: '#ffb800', fillOpacity: 1 };
    const beforeFill = structuredClone(fill);
    const captured = singleSolidFill([fill], 'owned view button')!;
    shape.current.paint.fillColor = captured.fillColor!;
    shape.current.paint.fillOpacity = captured.fillOpacity!;
    assertCanonicalCopyPaintUnchanged(f.document, shape.nodeId, true,
      shape.baseline, shape.current);
    const context = createSourcePaintCaptureContext(f.document, () => f.libraries);
    context.attach(snapshot);
    const result = reconcileReviewSource(f.document, snapshot);
    const owner = parseZuiDocument(result.source!).document;
    const expected = parseZuiDocument(f.source).document;
    expected.nodes!['root'].props!['background_color'] = '#ffb800';
    expect(owner).toEqual(expected);
    expect(owner.tokens!['editor.surface.1']).toBe('#242424');
    expect(f.document).toEqual(sourceBefore);
    expect(f.style.color).toBe('#242424');
    expect(f.nativeFill.fillColorRefFile).toBe('b4a1e3ce-9e6a-81fe-8008-d69d607aac12');
    expect(fill).toEqual(beforeFill);
  });

  it('requires a consumer owner before an inherited component-root override can reach reconciliation', () => {
    const f = sharedRev5OwnerFixture();
    f.document.asset.kind = 'component';
    delete f.document.root;
    f.document.components = { WorkbenchButton: { root: 'root' } };
    const snapshot = structuredClone(f.snapshot);
    const shape = snapshot.shapes[0];
    shape.current.paint.fillColor = '#ffb800';
    const sourceBefore = structuredClone(f.document);
    expect(() => assertCanonicalCopyPaintUnchanged(f.document, shape.nodeId,
      true, shape.baseline, shape.current)).toThrow(/consumer source owner.*canonical component/);
    expect(f.document).toEqual(sourceBefore);
    expect(f.style.color).toBe('#242424');
    expect(shape.current.paint.fillColor).toBe('#ffb800');
    // No fake owner mapping, baseline rewrite or canonical reconcile follows.
  });
});

function prepared() {
  const source = `# Original comments must survive a no-edit export\r\n${fixtureSource}`;
  const projection = parseZuiDocument(source).document;
  projection['penpot_original_source'] = source;
  projection['penpot_original_imports'] = projection.imports;
  projection.imports = { widgets: [], styles: [] };
  projection.nodes!['submit'].component = 'Button';
  projection.nodes!['submit'].props!['inherited_default'] = 'do not inline';
  projection.nodes!['submit']['penpot_prefab_source'] =
    'components/button.zui#RoundtripButton';
  return { source, projection };
}


function preparedComposedHostWithGeneratedSeparator() {
  const { source, projection } = prepared();
  const sourcePath = 'zircon_editor/assets/ui/workbench_window.zui';
  const host = structuredClone(parseZuiDocument(source).document);
  for (const [nodeId, node] of Object.entries(host.nodes ?? {})) {
    node['penpot_review_source_path'] = sourcePath;
    node['penpot_review_source_node_id'] = nodeId;
    node['penpot_review_instance_path'] = '[]';
  }
  host.nodes!['generated_header_separator'] = {
    component: 'Separator',
    props: { penpot_generated_host_structure: true, background_color: '#141414' },
    layout: { width: 100, height: 1 },
  };
  host.nodes!['root'].children!.push({ node: 'generated_header_separator' });
  const fingerprints = [{ sourcePath, sha256: createHash('sha256').update(source).digest('hex') }];
  projection['penpot_original_source_path'] = sourcePath;
  projection['penpot_source_fingerprints'] = fingerprints;
  projection['penpot_review_host'] = host;
  return { source, projection, host, sourcePath, fingerprints };
}

function preparedHeightMinCallsite() {
  const rootPath = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
  const componentPath = 'zircon_editor/assets/ui/editor/unreal/components/outliner.zui';
  const shellPath = 'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
  const source = `# Preserve the original weighted dock and its comments.
[asset]
kind = "view"
id = "res://ui/editor/unreal/level_editor_window.zui"
version = 2
[root]
node = "right_dock"
[imports]
widgets = ["res://ui/editor/unreal/components/outliner.zui#UnrealOutlinerPanel"]
[nodes.right_dock]
component = "VerticalGroup"
layout = { container = { kind = "VerticalBox", gap = 4 }, height = { preferred = 343, stretch = "Fixed" } }
children = [{ node = "outliner", slot = { layout = { linear_size = { rule = "Stretch", value = 0.4 } } } }, { node = "details", slot = { layout = { linear_size = { rule = "Stretch", value = 1.0 } } } }]
[nodes.outliner]
component = "UnrealOutlinerPanel"
control_id = "WorkbenchMainBandSceneTreePanel"
layout = { clip = true, width = { stretch = "Stretch" }, height = { min = 96, stretch = "Stretch" } }
[nodes.details]
component = "VerticalGroup"
layout = { height = { min = 96, stretch = "Stretch" } }
`;
  const component = `[asset]
kind = "component"
id = "res://ui/editor/unreal/components/outliner.zui"
version = 2
[components.UnrealOutlinerPanel]
root = "panel"
[nodes.panel]
component = "VerticalGroup"
control_id = "WorkbenchSceneTreePanel"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }
`;
  const shell = `[asset]
kind = "view"
id = "res://ui/editor/unreal/editor_shell.zui"
version = 2
[root]
node = "host"
[nodes.host]
component = "UiHostWindow"
control_id = "UiHostWindowRoot"
layout = { container = { kind = "Overlay" }, width = { min = 1280, preferred = 1280, max = 1280, stretch = "Fixed" }, height = { min = 800, preferred = 800, max = 800, stretch = "Fixed" } }
children = [{ node = "pane_surface" }]
[nodes.pane_surface]
component = "PaneSurface"
`;
  const projection = parseZuiDocument(source).document;
  const host = structuredClone(projection);
  for (const [id, node] of Object.entries(host.nodes!)) {
    node['penpot_review_source_path'] = rootPath;
    node['penpot_review_source_node_id'] = id;
    node['penpot_review_instance_path'] = '[]';
  }
  const outliner = host.nodes!.outliner;
  outliner.component = 'VerticalGroup';
  outliner['penpot_review_source_path'] = componentPath;
  outliner['penpot_review_source_node_id'] = 'panel';
  outliner['penpot_review_instance_path'] = JSON.stringify([
    { sourcePath: shellPath, sourceNodeId: 'pane_surface' },
    { sourcePath: rootPath, sourceNodeId: 'outliner' },
  ]);
  outliner['penpot_prefab_source'] = `${componentPath}#UnrealOutlinerPanel`;
  host.root = { node: 'host' };
  host.nodes!.host = {
    ...structuredClone(parseZuiDocument(shell).document.nodes!.host),
    children: [{ node: 'right_dock' }],
    penpot_review_source_path: shellPath,
    penpot_review_source_node_id: 'host',
    penpot_review_instance_path: '[]',
  };
  const sources = { [rootPath]: source, [componentPath]: component, [shellPath]: shell };
  const fingerprints = Object.entries(sources).map(([sourcePath, text]) => ({
    sourcePath, sha256: createHash('sha256').update(text).digest('hex'),
  }));
  projection['penpot_original_source'] = source;
  projection['penpot_original_source_path'] = rootPath;
  projection['penpot_source_fingerprints'] = fingerprints;
  projection['penpot_review_host'] = host;
  return { projection, host, source, sources, fingerprints, rootPath, componentPath };
}

describe('root callsite height.min source export', () => {
  it('exports an unchanged actual host minimum of 800 without inventing a Penpot layoutChild mapping', () => {
    const s = preparedHeightMinCallsite();
    const original = structuredClone(s.projection);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(s.host));
    const host = snapshot.shapes.find(({ nodeId }) => nodeId === 'host')!;
    expect(host.baseline).not.toHaveProperty('heightMin');
    expect(host.current).not.toHaveProperty('heightMin');
    expect(s.host.nodes!.host.layout!['height']).toMatchObject({ min: 800 });
    const result = reconcileReviewSource(s.projection, snapshot);
    expect(result.source).toBe(s.source);
    expect(result.changes).toEqual([]);
    expect(s.projection).toEqual(original);
  });

  it('exports only the existing root override while preserving rendered identity and weighted layout', () => {
    const s = preparedHeightMinCallsite();
    const original = structuredClone(s.projection);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(s.host));
    expect(snapshot.shapes.find(({ nodeId }) => nodeId === 'host')!.baseline)
      .not.toHaveProperty('heightMin');
    const outliner = snapshot.shapes.find(({ nodeId }) => nodeId === 'outliner')!;
    expect(outliner.baseline.heightMin).toBe(96);
    outliner.current.heightMin = 128;
    const result = reconcileReviewSource(s.projection, snapshot);
    const exported = parseZuiDocument(result.source!).document;
    const manifest = JSON.parse(exported['penpot_source_exports'] as string);
    expect(manifest.edits).toEqual([{
      exportNodeId: 'outliner', sourcePath: s.componentPath, sourceNodeId: 'panel',
      controlId: 'WorkbenchMainBandSceneTreePanel', instancePath: outliner.instancePath,
      fieldPath: ['layout', 'height', 'min'], value: 128,
      heightMinOwner: { sourcePath: s.rootPath, sourceNodeId: 'outliner', baselineValue: 96,
        callee: `${s.componentPath}#UnrealOutlinerPanel` },
    }]);
    expect(manifest.identityMap.find((row: { exportNodeId: string }) => row.exportNodeId === 'outliner'))
      .toMatchObject({ sourcePath: s.componentPath, sourceNodeId: 'panel', instancePath: outliner.instancePath });
    delete exported['penpot_source_exports'];
    expect(exported).toEqual(parseZuiDocument(s.source).document);
    const applied = applyZuiExportToSourceFiles(result.source!, s.sources, s.fingerprints);
    expect(applied.appliedPathsBySource).toEqual({ [s.rootPath]: ['nodes.outliner.layout.height.min'] });
    expect(applied.outputs).toHaveLength(1);
    const expected = parseZuiDocument(s.source).document;
    (expected.nodes!.outliner.layout!['height'] as Record<string, unknown>)['min'] = 128;
    expect(parseZuiDocument(applied.outputs[0].source).document).toEqual(expected);
    expect(applied.outputs[0].source).toContain('# Preserve the original weighted dock');
    expect(outliner.current.geometry).toEqual(outliner.baseline.geometry);
    expect(s.projection).toEqual(original);
  });

  it('exports an existing direct root minimum without inventing a callsite owner', () => {
    const s = preparedHeightMinCallsite();
    const original = structuredClone(s.projection);
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(s.host));
    snapshot.shapes.find(({ nodeId }) => nodeId === 'details')!.current.heightMin = 128;
    const result = reconcileReviewSource(s.projection, snapshot);
    const exported = parseZuiDocument(result.source!).document;
    const manifest = JSON.parse(exported['penpot_source_exports'] as string);
    expect(manifest.edits).toEqual([{
      exportNodeId: 'details', sourcePath: s.rootPath, sourceNodeId: 'details',
      controlId: null, instancePath: '[]',
      fieldPath: ['layout', 'height', 'min'], value: 128,
    }]);
    const applied = applyZuiExportToSourceFiles(result.source!, s.sources, s.fingerprints);
    expect(applied.appliedPathsBySource).toEqual({ [s.rootPath]: ['nodes.details.layout.height.min'] });
    expect(applied.outputs).toHaveLength(1);
    const expected = parseZuiDocument(s.source).document;
    (expected.nodes!.details.layout!['height'] as Record<string, unknown>)['min'] = 128;
    expect(parseZuiDocument(applied.outputs[0].source).document).toEqual(expected);
    expect(s.projection).toEqual(original);
  });

  it.each(['missing invocation', 'missing leaf', 'ambiguous binding', 'rebound callee', 'stale baseline'])(
    'rejects a %s before emitting a root override', (kind) => {
      const s = preparedHeightMinCallsite();
      const root = parseZuiDocument(s.source).document;
      if (kind === 'missing invocation')
        s.host.nodes!.outliner['penpot_review_instance_path'] = '[]';
      if (kind === 'missing leaf')
        delete (root.nodes!.outliner.layout!['height'] as Record<string, unknown>)['min'];
      if (kind === 'ambiguous binding')
        (root.imports!['widgets'] as string[]).push('res://ui/editor/unreal/components/other.zui#UnrealOutlinerPanel');
      if (kind === 'rebound callee')
        s.host.nodes!.outliner['penpot_prefab_source'] = `${s.componentPath}#DifferentPanel`;
      if (kind === 'stale baseline')
        (root.nodes!.outliner.layout!['height'] as Record<string, unknown>)['min'] = 95;
      s.projection['penpot_original_source'] = serializeZuiDocument(root);
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(s.host));
      snapshot.shapes.find(({ nodeId }) => nodeId === 'outliner')!.current.heightMin = 128;
      expect(() => reconcileReviewSource(s.projection, snapshot)).toThrow(/height.min owner/);
    },
  );
});

describe('projection export to runtime source', () => {
  it('returns the exact unexpanded source and comments on no-edit export', () => {
    const { source, projection } = prepared();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(projection));
    const result = reconcileReviewSource(projection, snapshot);
    expect(result.source).toBe(source);
    expect(result.document).toEqual(parseZuiDocument(source).document);
    const asset = parsePenpotBridgeAsset(
      serializePenpotBridgeAsset(
        createPenpotBridgeAsset(projection, 'source.zui'),
      ),
    );
    expect(reconcilePenpotBridgeAsset(asset).source).toBe(source);
  });

  it('writes an instance text override while preserving imports, bindings, events and unknown fields', () => {
    const { source, projection } = prepared();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(projection));
    snapshot.shapes.find(
      (shape) => shape.nodeId === 'submit',
    )!.current.text!.characters = 'Build';
    const result = reconcileReviewSource(projection, snapshot);
    const expected = parseZuiDocument(source).document;
    expected.nodes!['submit'].props!['text'] = 'Build';
    expect(result.document).toEqual(expected);
    expect(parseZuiDocument(result.source!).document).toEqual(expected);
  });

  it('writes a known slot edit without inlining the expanded node layout', () => {
    const { source, projection } = prepared();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(projection));
    snapshot.shapes.find(
      (shape) => shape.nodeId === 'actions',
    )!.current.slotPadding!.right = 24;
    const result = reconcileReviewSource(projection, snapshot);
    const expected = parseZuiDocument(source).document;
    expected.nodes!['root'].children![1].slot!['layout']['padding'] = {
      right: 24,
    };
    expect(result.document).toEqual(expected);
  });

  it('exports a changed composed host as the original .zui plus a verifiable source-owner map', () => {
    const { source, projection, host, sourcePath, fingerprints } =
      preparedComposedHostWithGeneratedSeparator();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
    snapshot.shapes.find((shape) => shape.nodeId === 'root')!.current.container.gap += 1;

    const rawResult = reconcileZuiDocument(host, snapshot);
    expect(
      (rawResult.document.nodes!.root.layout!['container'] as Record<
        string,
        unknown
      >)['gap'],
    ).toBe(snapshot.shapes.find((shape) => shape.nodeId === 'root')!.current.container.gap);
    const result = reconcileReviewSource(projection, snapshot);
    expect(result.changes).toContain('nodes.root.layout.container.gap');
    const exported = parseZuiDocument(result.source!).document;
    const manifest = JSON.parse(
      exported['penpot_source_exports'] as string,
    ) as {
      rootSourcePath: string;
      identityMap: Array<{
        exportNodeId: string;
        sourcePath: string;
        sourceNodeId: string;
        controlId: string | null;
        instancePath: string;
      }>;
      edits: Array<{
        sourcePath: string;
        sourceNodeId: string;
        fieldPath: string[];
        value: unknown;
      }>;
    };
    delete exported['penpot_source_exports'];

    const editedIdentity = manifest.identityMap.find(
      ({ exportNodeId }) => exportNodeId === 'root',
    );
    expect(exported).toEqual(parseZuiDocument(source).document);
    expect(manifest.rootSourcePath).toBe(sourcePath);
    expect(editedIdentity).toMatchObject({
      sourcePath,
      sourceNodeId: 'root',
      controlId: 'RoundtripRoot',
      instancePath: '[]',
    });
    expect(manifest.identityMap.some(({ exportNodeId }) =>
      exportNodeId === 'generated_header_separator')).toBe(false);
    const applied = applyZuiExportToSourceFiles(
      result.source!, { [sourcePath]: source }, fingerprints,
    );
    expect(applied.outputs).toHaveLength(1);
    expect(applied.outputs[0].sourcePath).toBe(sourcePath);
    const appliedDocument = parseZuiDocument(applied.outputs[0].source).document;
    expect(appliedDocument.nodes!['generated_header_separator']).toBeUndefined();
    expect((appliedDocument.nodes!.root.layout!['container'] as Record<string, unknown>)['gap'])
      .toBe(snapshot.shapes.find((shape) => shape.nodeId === 'root')!.current.container.gap);
    expect(manifest.edits).toEqual([
      {
        exportNodeId: 'root',
        sourcePath,
        sourceNodeId: 'root',
        controlId: 'RoundtripRoot',
        instancePath: '[]',
        fieldPath: ['layout', 'container', 'gap'],
        value: snapshot.shapes.find((shape) => shape.nodeId === 'root')!.current
          .container.gap,
      },
    ]);
  });

  it('rejects a visual edit to a marked generated Header separator', () => {
    const { projection, host } = preparedComposedHostWithGeneratedSeparator();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
    snapshot.shapes.find((shape) => shape.nodeId === 'generated_header_separator')!
      .current.paint.fillColor = '#444444';
    expect(() => reconcileReviewSource(projection, snapshot)).toThrow(
      'Changed host node generated_header_separator has no captured source owner',
    );
  });

  it('still rejects an authored host node with missing captured ownership', () => {
    const { projection, host } = preparedComposedHostWithGeneratedSeparator();
    delete host.nodes!.root['penpot_review_source_path'];
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(host));
    snapshot.shapes.find((shape) => shape.nodeId === 'root')!.current.container.gap += 1;
    expect(() => reconcileReviewSource(projection, snapshot)).toThrow(
      'Host projection node root has no captured source ownership identity',
    );
  });

  it('rejects edits to generated prefab internals without a source target', () => {
    const { projection } = prepared();
    projection.nodes!['expanded_internal'] = {
      component: 'Label',
      props: { text: 'Inherited' },
    };
    projection.nodes!['actions'].children!.push({ node: 'expanded_internal' });
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(projection));
    snapshot.shapes.find(
      (shape) => shape.nodeId === 'expanded_internal',
    )!.current.text!.characters = 'Changed';
    expect(() => reconcileReviewSource(projection, snapshot)).toThrow(
      'Expanded prefab node edit requires an explicit source mapping',
    );
  });

  it('restores a legacy fixture exactly and refuses unsupported legacy edits', () => {
    const source =
      '# Legacy parser contract\r\n[asset]\r\nkind="layout"\r\nid="res://legacy.zui"\r\nversion=1\r\n';
    const projection: ZuiDocument = {
      asset: { id: 'res://legacy.zui', kind: 'view', version: 2 },
      root: { node: 'root' },
      nodes: { root: { component: 'Label', props: { text: 'Fixture' } } },
      penpot_original_source: source,
    };
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(projection));
    expect(reconcileReviewSource(projection, snapshot).source).toBe(source);
    snapshot.shapes[0].current.text!.characters = 'Edited';
    expect(() => reconcileReviewSource(projection, snapshot)).toThrow(
      'Legacy fixture edits require an explicit legacy source mapping',
    );
  });

  it('does not export source metadata for another asset identity', () => {
    const { projection } = prepared();
    projection.asset.id = 'res://wrong.zui';
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(projection));
    expect(() => reconcileReviewSource(projection, snapshot)).toThrow(
      'Original ZUI source identity differs',
    );
  });
});
