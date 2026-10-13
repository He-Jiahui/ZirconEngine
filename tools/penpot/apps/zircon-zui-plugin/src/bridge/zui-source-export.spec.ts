import { createHash } from 'node:crypto';
import { applyZuiExportToSourceFiles } from './zui-export-apply-sources';
import fixtureSource from './roundtrip-fixture.zui?raw';
import { parseZuiDocument, type ZuiDocument } from './zui-document';
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
