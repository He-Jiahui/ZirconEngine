import type { Board, Shape } from '@penpot/plugin-types';
import {
  copyNativeComponentAppearance,
  inheritedTreeRowPainterForCopy,
  nativeComponentStructure,
} from './penpot-native-component-copy';
import { ZUI_METADATA_NAMESPACE as NS } from './metadata';
import { allocateNativeLayoutSize } from './penpot-layout-allocation';

function shape(minWidth: number | null): Shape {
  return {
    type: 'board',
    name: 'Button',
    width: 120,
    height: 32,
    children: [],
    layoutChild: { minWidth },
    getSharedPluginData: () => '',
    getSharedPluginDataKeys: () => [],
  } as unknown as Board;
}

describe('native instance size constraints', () => {
  it('does not share a master when a later instance needs a cleared bound', () => {
    expect(nativeComponentStructure(shape(100))).not.toEqual(
      nativeComponentStructure(shape(null)),
    );
    expect(nativeComponentStructure(shape(null))).toEqual(
      nativeComponentStructure(shape(null)),
    );
  });

  it('rejects a host that silently ignores a nullable constraint override', () => {
    const target = shape(100);
    Object.defineProperty(target.layoutChild, 'minWidth', {
      get: () => 100,
      set: () => undefined,
    });
    expect(() => copyNativeComponentAppearance(shape(null), target)).toThrow(
      'did not preserve native instance size constraints',
    );
  });
});

function inheritedCopyFixture() {
  const f = allocatedCopyFixture();
  const component = { id: 'canonical-component', libraryId: 'canonical-library', mainInstance: () => f.source };
  for (const original of [f.source, f.original])
    Object.assign(original, { component: () => component, isComponentMainInstance: () => true });
  for (const [copy, original] of [[f.target, f.source], [f.copy, f.original]])
    Object.assign(copy, { component: () => component, isComponentCopyInstance: () => true,
      componentRefShape: () => new Proxy(original, {}) });
  const state = JSON.parse(f.source.getSharedPluginData(NS, 'tree-row-authoring-painter'));
  Object.assign(state, { originalPaint: { 'source-part': { fills: [], strokes: [], opacity: 1 } },
    originalBounds: { 'source-part': { width: 240, height: 32 } }, originalWidths: { 'source-part': [2] } });
  const inherited = JSON.stringify(state);
  f.source.setSharedPluginData(NS, 'tree-row-authoring-painter', inherited);
  f.target.setSharedPluginData(NS, 'tree-row-authoring-painter', inherited);
  return { ...f, component, inherited };
}

describe('native Assets-panel TreeRow reference interpretation', () => {
  it('derives only copy references and never captures current geometry, paint or allocation', () => {
    const f = inheritedCopyFixture();
    f.copy.width = 158;
    f.copy.fills = [{ fillColor: '#ff0000' }];
    f.target.setSharedPluginData(NS, 'native-layout-allocation', '{"parent":"stale-source-parent","width":240}');
    const before = [f.source, f.original, f.target, f.copy].map((shape) => ({
      width: shape.width, fills: shape.fills,
      metadata: shape.getSharedPluginDataKeys(NS).map((key) => [key, shape.getSharedPluginData(NS, key)]),
    }));
    const state = JSON.parse(inheritedTreeRowPainterForCopy(f.source, f.target));
    expect(state.parts).toEqual(['copy-part']);
    expect(state.generated['copy-row']).toEqual({ type: 'board', parent: 'consumer-host', children: ['copy-part'] });
    expect(state.originalGeometry['copy-part']).toEqual({ x: 0, y: 0, width: 240, height: 32 });
    expect(state.originalPaint['copy-part']).toEqual({ fills: [], strokes: [], opacity: 1 });
    expect(state.originalWidths['copy-part']).toEqual([2]);
    expect([f.source, f.original, f.target, f.copy].map((shape) => ({
      width: shape.width, fills: shape.fills,
      metadata: shape.getSharedPluginDataKeys(NS).map((key) => [key, shape.getSharedPluginData(NS, key)]),
    }))).toEqual(before);
  });

  it.each(['missing', 'duplicate', 'foreign', 'type', 'parent', 'extra'])('rejects %s SDK source mappings without metadata writes', (failure) => {
    const f = inheritedCopyFixture();
    if (failure === 'missing') Object.assign(f.copy, { componentRefShape: () => null });
    if (failure === 'duplicate') Object.assign(f.copy, { componentRefShape: () => f.source });
    if (failure === 'foreign') Object.assign(f.copy, { component: () => ({ ...f.component, libraryId: 'other-library' }) });
    if (failure === 'type') Object.assign(f.copy, { type: 'path' });
    if (failure === 'parent') Object.assign(f.copy, { parent: { id: 'other-parent' } });
    if (failure === 'extra') (f.target.children as Shape[]).push({ ...f.copy, id: 'extra' } as Shape);
    expect(() => inheritedTreeRowPainterForCopy(f.source, f.target)).toThrow(/TreeRow native/);
    expect(f.target.getSharedPluginData(NS, 'tree-row-authoring-painter')).toBe(f.inherited);
  });

  it.each(['generated', 'originalPaint', 'originalGeometry', 'originalBounds', 'originalWidths'])('rejects an absent %s instead of blessing the copy readback', (field) => {
    const f = inheritedCopyFixture(), state = JSON.parse(f.inherited);
    delete state[field];
    const raw = JSON.stringify(state);
    f.source.setSharedPluginData(NS, 'tree-row-authoring-painter', raw);
    f.target.setSharedPluginData(NS, 'tree-row-authoring-painter', raw);
    expect(() => inheritedTreeRowPainterForCopy(f.source, f.target)).toThrow('immutable original painter baseline');
    expect(f.target.getSharedPluginData(NS, 'tree-row-authoring-painter')).toBe(raw);
  });

  it('rejects stale inherited state and an out-of-main generated reference', () => {
    const f = inheritedCopyFixture();
    f.target.setSharedPluginData(NS, 'tree-row-authoring-painter', f.inherited + ' ');
    expect(() => inheritedTreeRowPainterForCopy(f.source, f.target)).toThrow('differs from its canonical main');
    const state = JSON.parse(f.inherited);
    state.generated['source-part'].children = ['not-in-main'];
    const raw = JSON.stringify(state);
    f.source.setSharedPluginData(NS, 'tree-row-authoring-painter', raw);
    f.target.setSharedPluginData(NS, 'tree-row-authoring-painter', raw);
    expect(() => inheritedTreeRowPainterForCopy(f.source, f.target)).toThrow('outside its source copy');
    expect(f.target.getSharedPluginData(NS, 'tree-row-authoring-painter')).toBe(raw);
  });
});

function allocatedCopyFixture(width = 240) {
  function make(id: string, values: Record<string, string> = {}): Board {
    return {
      id, type: 'board', name: id, x: 10, y: 20, width, height: 32,
      children: [], hidden: false, fills: [], strokes: [], opacity: 1, rotation: 0,
      constraintsHorizontal: 'left', constraintsVertical: 'top',
      getSharedPluginData: (_: string, key: string) => values[key] ?? '',
      getSharedPluginDataKeys: () => Object.keys(values),
      setSharedPluginData: (_: string, key: string, value: string) => { values[key] = value; },
      resize(w: number, h: number) { this.width = w; this.height = h; },
    } as unknown as Board;
  }
  const source = make('source-row'), target = make('copy-row');
  Object.assign(source, { parent: { id: 'source-host' } });
  Object.assign(target, { parent: { id: 'consumer-host' }, nativeShapeRef: 'canonical-main' });
  const original = make('source-part', { sourceNodeId: 'source-part',
    sourcePath: 'zircon_editor/assets/ui/editor/component.zui',
    instancePath: 'owner/slot/value', 'text-property': 'caption', events: '["Click","source-part"]' });
  const copy = make('copy-part');
  Object.assign(original, { parent: source, constraintsHorizontal: 'leftright' });
  Object.assign(copy, { parent: target, nativeShapeRef: 'canonical-caption' });
  (source.children as Shape[]).push(original);
  (target.children as Shape[]).push(copy);
  allocateNativeLayoutSize(source, width, 32, 'anchored');
  source.setSharedPluginData(NS, 'control-part-list', JSON.stringify([original.id]));
  source.setSharedPluginData(NS, 'tree-row-authoring-painter', JSON.stringify({ version: 1,
    parts: [original.id], size: { width, height: 32 }, panels: [], icons: [], iconSources: [],
    generated: { [source.id]: { type: 'board', parent: 'source-host', children: [original.id] },
      [original.id]: { type: 'board', parent: source.id, children: [] } },
    originalGeometry: { [original.id]: { x: 0, y: 0, width, height: 32 } },
  }));
  return { source, target, original, copy };
}

describe('native component copy ownership and generated references', () => {
  it.each([592, 852, 1232])('keeps the linked identity and actual %i allocation in the importer copy', (width) => {
    const f = allocatedCopyFixture(width);
    const sourceState = f.source.getSharedPluginData(NS, 'tree-row-authoring-painter');
    copyNativeComponentAppearance(f.source, f.target);
    expect(JSON.parse(f.target.getSharedPluginData(NS, 'native-layout-allocation'))).toEqual({
      kind: 'anchored', width, height: 32, parent: 'consumer-host',
    });
    expect(f.target.id).toBe('copy-row');
    expect(f.copy.id).toBe('copy-part');
    expect((f.target as unknown as { nativeShapeRef: string }).nativeShapeRef).toBe('canonical-main');
    expect((f.copy as unknown as { nativeShapeRef: string }).nativeShapeRef).toBe('canonical-caption');
    expect(f.copy.constraintsHorizontal).toBe('leftright');
    expect(f.copy.getSharedPluginData(NS, 'sourceNodeId')).toBe('source-part');
    expect(f.copy.getSharedPluginData(NS, 'events')).toBe('["Click","source-part"]');
    expect(f.copy.getSharedPluginData(NS, 'instancePath')).toBe('owner/slot/value');
    expect(JSON.parse(f.target.getSharedPluginData(NS, 'control-part-list'))).toEqual(['copy-part']);
    const painter = JSON.parse(f.target.getSharedPluginData(NS, 'tree-row-authoring-painter'));
    expect(painter.parts).toEqual(['copy-part']);
    expect(painter.generated['copy-row']).toEqual({ type: 'board', parent: 'consumer-host', children: ['copy-part'] });
    expect(painter.generated['copy-part'].parent).toBe('copy-row');
    expect(Object.keys(painter.originalGeometry)).toEqual(['copy-part']);
    expect(f.source.getSharedPluginData(NS, 'tree-row-authoring-painter')).toBe(sourceState);
  });

  it('rejects a stale source parent before changing any target appearance or metadata', () => {
    const f = allocatedCopyFixture();
    Object.assign(f.source, { parent: { id: 'unknown-new-parent' } });
    const before = [f.target.name, f.target.width, f.target.height, f.copy.constraintsHorizontal];
    expect(() => copyNativeComponentAppearance(f.source, f.target)).toThrow('allocation is stale');
    expect([f.target.name, f.target.width, f.target.height, f.copy.constraintsHorizontal]).toEqual(before);
    expect(f.target.getSharedPluginDataKeys(NS)).toEqual([]);
  });

  it('uses stable IDs when SDK child getters return fresh proxy objects', () => {
    const f = allocatedCopyFixture();
    Object.defineProperty(f.source, 'children', { get: () => [new Proxy(f.original, {})] });
    Object.defineProperty(f.target, 'children', { get: () => [new Proxy(f.copy, {})] });
    expect(() => copyNativeComponentAppearance(f.source, f.target)).not.toThrow();
    expect(JSON.parse(f.target.getSharedPluginData(NS, 'control-part-list'))).toEqual(['copy-part']);
    expect(f.copy.constraintsHorizontal).toBe('leftright');
  });

  it('rejects an unmapped generated part before copying the root', () => {
    const f = allocatedCopyFixture();
    f.source.setSharedPluginData(NS, 'control-part-list', '["outside-source-copy"]');
    expect(() => copyNativeComponentAppearance(f.source, f.target)).toThrow('outside its source copy');
    expect(f.target.getSharedPluginDataKeys(NS)).toEqual([]);
  });

  it('remaps only field paint map keys, leaving literal paint values intact', () => {
    const f = allocatedCopyFixture();
    f.original.setSharedPluginData(NS, 'field-icon-paint', JSON.stringify({ version: 1,
      sourceSize: { width: 24, height: 24 }, sourceWidths: [2],
      generated: { 'source-part': { fills: [], strokes: [] } }, originalWidths: { 'source-part': [2] },
    }));
    copyNativeComponentAppearance(f.source, f.target);
    const paint = JSON.parse(f.copy.getSharedPluginData(NS, 'field-icon-paint'));
    expect(paint.generated).toEqual({ 'copy-part': { fills: [], strokes: [] } });
    expect(paint.originalWidths).toEqual({ 'copy-part': [2] });
    expect(paint.sourceWidths).toEqual([2]);
  });
});
