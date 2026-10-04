import { parseZuiLayoutSource } from './zui-layout-optimizer';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { projectZuiDocument } from './penpot-projection';

describe('layout source preparation order', () => {
  it('converts a legacy parametric slider before resolving its defaults and measuring it', () => {
    const source = `[asset]
kind="widget"
id="res://parametric.zui"
version=1
[components.Slider]
root="slider"
params={min={type="number",default=0.0},max={type="number",default=100.0},value={type="number",default=25.0}}
[nodes.slider]
kind="native"
type="RangeField"
props={min="$param.min",max="$param.max",value="$param.value"}
layout={width={preferred=240},height={preferred=32}}
`;
    const parsed = parseZuiLayoutSource(source, 'test.zui');
    expect(parsed.sourceFormat).toBe('legacy');
    expect(parsed.changes).toEqual(['migrate-legacy-document']);
    expect(parsed.document.nodes!['slider'].props!['min']).toBe('$param.min');
    const projectionDocument = structuredClone(parsed.document);
    new LayoutDependencies().embed(projectionDocument, 'test.zui');
    const projection = projectZuiDocument(projectionDocument);
    expect(projection.shapes[0].slider?.percent).toBe(0.25);
    expect(parsed.document.nodes!['slider'].props!['min']).toBe('$param.min');
    expect(parsed.document['legacy_migration']).toMatchObject({
      source_kind: 'widget',
      source_version: 1,
    });
  });

  it('does not apply presentation defaults or enlarge an authored v2 layout', () => {
    const source = `[asset]
kind="view"
id="res://view.zui"
version=2
[root]
node="root"
[nodes.root]
component="Label"
props={text="Long label",font_size=12}
layout={width={preferred=40},height={preferred=20}}
`;
    const parsed = parseZuiLayoutSource(source, 'test.zui');
    expect(parsed.changes).toEqual([]);
    expect(parsed.document.tokens).toBeUndefined();
    expect(parsed.document.nodes!['root'].props!['font_size']).toBe(12);
    expect(parsed.document.nodes!['root'].layout!['width']).toEqual({
      preferred: 40,
    });
  });

  it('wraps multi-root legacy fixtures in a responsive authored scroll owner', () => {
    const source = `[asset]
kind="widget"
id="res://multi-root.zui"
version=1
[components.One]
root="one"
[components.Two]
root="two"
[nodes.one]
kind="native"
type="Label"
props={text="One"}
[nodes.two]
kind="native"
type="Label"
props={text="Two"}
`;
    const parsed = parseZuiLayoutSource(source, 'test.zui');
    const root = parsed.document.nodes!.__penpot_root;
    expect(root.component).toBe('ScrollableBox');
    expect(root.layout).toMatchObject({
      clip: true,
      container: { kind: 'ScrollableBox', axis: 'Vertical' },
      width: { min: 0, stretch: 'Stretch' },
      height: { min: 0, stretch: 'Stretch' },
    });
  });

  it('preserves a legacy root reference instead of creating an empty wrapper', () => {
    const source = `[asset]
kind="layout"
id="res://root-ref.zui"
version=2
[root]
node="root_node"
[nodes.root_node]
kind="native"
type="Label"
props={text="Visible"}
`;
    const parsed = parseZuiLayoutSource(source, 'test.zui');
    expect(parsed.document.root).toEqual({ node: 'root_node' });
    expect(parsed.document.nodes!['root_node'].component).toBe('Label');
  });

  it('reattaches legacy child references during migration', () => {
    const source = `[asset]
kind="layout"
id="res://child-ref.zui"
version=2
[root]
node="root_node"
[nodes.root_node]
kind="native"
type="VerticalBox"
children=[{child="title"}]
[nodes.title]
kind="native"
type="Label"
props={text="Visible"}
`;
    const parsed = parseZuiLayoutSource(source, 'test.zui');
    expect(parsed.document.nodes!['root_node'].children).toEqual([
      { node: 'title' },
    ]);
  });
});
