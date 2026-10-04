import fixtureSource from './roundtrip-fixture.zui?raw';
import {
  assertSnapshotBaselines,
  baselineIndex,
  createPenpotBridgeAsset,
  parsePenpotBridgeAsset,
  reconcilePenpotBridgeAsset,
  serializePenpotBridgeAsset,
} from './penpot-asset';
import {
  normalizeZuiDocument,
  parseZuiDocument,
  zuiNodes,
  type ZuiTable,
} from './zui-document';
import { reconcileZuiDocument } from './penpot-reconcile';

describe('serializable Penpot asset bridge', () => {
  it('round-trips an unchanged asset without losing ZUI semantics', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const encoded = serializePenpotBridgeAsset(
      createPenpotBridgeAsset(document, 'penpot_roundtrip.zui'),
    );
    const asset = parsePenpotBridgeAsset(encoded);
    const result = reconcilePenpotBridgeAsset(asset);

    expect(asset.schema).toBe('dev.zircon.zui.penpot-asset');
    expect(asset.version).toBe(3);
    expect(asset.snapshot.rootNodeIds).toEqual(['root']);
    expect(asset.snapshot.detachedNodeIds).toEqual(['detached_template']);
    expect(result.changes).toEqual([]);
    expect(normalizeZuiDocument(result.document)).toEqual(
      normalizeZuiDocument(document),
    );
  });

  it('serializes native painter source-part mappings and rejects mapping edits', () => {
    const { document } = parseZuiDocument(`
[asset]
kind = "component"
id = "res://review/dialog.zui"
version = 2
[components.DialogReview]
root = "root"
[nodes.root]
component = "Dialog"
props = { open = true, popup_open = true, title = "Settings", message = "Review" }
`);
    const asset = createPenpotBridgeAsset(document, 'dialog.zui');
    const shape = asset.snapshot.shapes.find(({ nodeId }) => nodeId === 'root');
    expect(shape?.sourceParts).toEqual([
      { nodeId: 'root', property: 'title' },
      { nodeId: 'root', property: 'message' },
      { nodeId: 'root', property: 'open' },
      { nodeId: 'root', property: 'popup_open' },
    ]);
    shape!.sourceParts![0].property = 'unknown';
    expect(() => serializePenpotBridgeAsset(asset)).toThrow(
      /Source-part mapping for semantic node root/i,
    );
  });

  it('applies editable snapshot changes while retaining runtime-only fields', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const asset = createPenpotBridgeAsset(document, 'penpot_roundtrip.zui');
    const title = asset.snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    );
    const root = asset.snapshot.shapes.find(({ nodeId }) => nodeId === 'root');

    expect(title?.current.text).not.toBeNull();
    title!.current.text!.characters = 'Headless Penpot edit';
    root!.current.paint.borderRadius = 10;

    const result = reconcilePenpotBridgeAsset(
      parsePenpotBridgeAsset(serializePenpotBridgeAsset(asset)),
    );
    const resultNodes = zuiNodes(result.document);
    const sourceNodes = zuiNodes(document);

    expect(resultNodes['title'].props?.['text']).toBe('Headless Penpot edit');
    expect(resultNodes['root'].props?.['corner_radius']).toBe(10);
    expect(resultNodes['root'].events).toEqual(sourceNodes['root'].events);
    expect(resultNodes['root']['zircon_extension']).toEqual(
      sourceNodes['root']['zircon_extension'],
    );
  });

  it('materializes fixed child dimensions when a free parent becomes auto-layout', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const sourceNodes = zuiNodes(document);
    const rootContainer = sourceNodes['root'].layout?.['container'] as ZuiTable;
    rootContainer['kind'] = 'Overlay';
    expect(sourceNodes['root'].children).toBeDefined();
    const sourceActionsMount = sourceNodes['root'].children!.find(
      ({ node }) => node === 'actions',
    )!;
    sourceActionsMount.slot!['runtime_hint'] = 'preserve-me';
    const asset = createPenpotBridgeAsset(document, 'penpot_roundtrip.zui');
    const root = asset.snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    const title = asset.snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;
    const actions = asset.snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'actions',
    )!;
    root.current.container.kind = 'flex';
    root.current.container.direction = 'column';

    const result = reconcilePenpotBridgeAsset(asset);
    const resultNodes = zuiNodes(result.document);
    const titleLayout = resultNodes['title'].layout!;
    expect(resultNodes['root'].children).toBeDefined();
    const actionsMount = resultNodes['root'].children!.find(
      ({ node }) => node === 'actions',
    )!;
    const actionsSlotLayout = actionsMount.slot?.['layout'] as ZuiTable;

    expect(titleLayout['width']).toEqual({
      min: title.current.geometry.width,
      preferred: title.current.geometry.width,
      max: title.current.geometry.width,
      stretch: 'Fixed',
    });
    expect(titleLayout['height']).toEqual({
      min: title.current.geometry.height,
      preferred: title.current.geometry.height,
      max: title.current.geometry.height,
      stretch: 'Fixed',
    });
    expect(actionsSlotLayout['width']).toEqual({
      min: actions.current.geometry.width,
      preferred: actions.current.geometry.width,
      max: actions.current.geometry.width,
      stretch: 'Fixed',
    });
    expect(actionsMount.slot?.['runtime_hint']).toBe('preserve-me');
    expect(result.changes).toEqual(
      expect.arrayContaining([
        'nodes.root.layout.container.kind',
        'nodes.title.layout.width',
        'nodes.title.layout.height',
        'nodes.root.children.actions.slot.layout.width',
      ]),
    );
  });

  it('materializes arranged child geometry when an auto-layout parent becomes free', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const asset = createPenpotBridgeAsset(document, 'penpot_roundtrip.zui');
    const root = asset.snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    const title = asset.snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'title',
    )!;
    root.current.container.kind = 'free';
    title.baseline.geometry = {
      x: 20,
      y: 16,
      width: 380,
      height: 32,
    };
    title.current.geometry = { ...title.baseline.geometry };

    const result = reconcileZuiDocument(document, asset.snapshot);
    const titleLayout = zuiNodes(result.document)['title'].layout!;

    expect(titleLayout['position']).toEqual({ x: 20, y: 16 });
    expect(titleLayout['width']).toEqual({
      min: 380,
      preferred: 380,
      max: 380,
      stretch: 'Fixed',
    });
    expect(titleLayout['height']).toEqual({
      min: 32,
      preferred: 32,
      max: 32,
      stretch: 'Fixed',
    });
    expect(result.changes).toEqual(
      expect.arrayContaining([
        'nodes.root.layout.container.kind',
        'nodes.title.layout.position.x',
        'nodes.title.layout.position.y',
        'nodes.title.layout.width',
        'nodes.title.layout.height',
      ]),
    );
  });

  it('rejects bridge files with an unsupported contract version', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const encoded = serializePenpotBridgeAsset(
      createPenpotBridgeAsset(document, 'penpot_roundtrip.zui'),
    );

    expect(() =>
      parsePenpotBridgeAsset(encoded.replace('"version": 3', '"version": 4')),
    ).toThrow(/unsupported Penpot asset bridge version 4/i);
  });

  it('rejects a baseline that no longer matches document metadata', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const asset = createPenpotBridgeAsset(document, 'penpot_roundtrip.zui');
    const root = asset.snapshot.shapes.find(({ nodeId }) => nodeId === 'root')!;
    root.baseline.geometry.width = 999;
    root.current.geometry.width = 999;

    expect(() => serializePenpotBridgeAsset(asset)).toThrow(
      /baseline for semantic node root does not match document metadata/i,
    );
  });

  it('reports malformed nested snapshot state at the contract boundary', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const asset = createPenpotBridgeAsset(document, 'penpot_roundtrip.zui');
    const malformed = asset as unknown as {
      snapshot: { shapes: Array<{ current: { geometry?: unknown } }> };
    };
    delete malformed.snapshot.shapes[0].current.geometry;

    expect(() => serializePenpotBridgeAsset(asset)).toThrow(
      /snapshot\.shapes\[0\]\.current\.geometry must be an object/i,
    );
  });

  it('indexes a legal __proto__ semantic node without prototype loss', () => {
    const { document } = parseZuiDocument(fixtureSource);
    const snapshot = createPenpotBridgeAsset(
      document,
      'penpot_roundtrip.zui',
    ).snapshot;
    snapshot.shapes[0].nodeId = '__proto__';
    const index = baselineIndex(snapshot);

    expect(Object.hasOwn(index, '__proto__')).toBe(true);
    expect(JSON.parse(JSON.stringify(index))).toHaveProperty('__proto__');
    expect(() => assertSnapshotBaselines(snapshot, index)).not.toThrow();
  });
});
