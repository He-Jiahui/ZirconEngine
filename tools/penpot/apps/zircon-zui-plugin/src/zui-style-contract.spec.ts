import { describe, expect, it } from 'vitest';
import { checkZuiStyleContract } from '../tools/zui-style-contract';

const source = (body: string) =>
  `[asset]\nkind = "view"\nid = "test"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "WorkbenchButton"\nprops = { ${body} }\n`;

describe('zui style contract', () => {
  it('accepts the Workbench baseline values', () => {
    expect(
      checkZuiStyleContract(
        source('font_size = 14, height = 32, gap = 8, corner_radius = 4'),
        'test.zui',
      ).issues,
    ).toEqual([]);
  });
  it('reports source, node and property for violations', () => {
    const result = checkZuiStyleContract(
      source('font_size = 11, height = 40, gap = 6, corner_radius = 12'),
      'bad.zui',
    );
    expect(
      result.issues.map((item) => `${item.source}:${item.node}:${item.path}`),
    ).toEqual([
      'bad.zui:root:nodes.root.props.font_size',
      'bad.zui:root:nodes.root.props.height',
      'bad.zui:root:nodes.root.props.gap',
      'bad.zui:root:nodes.root.props.corner_radius',
    ]);
  });
  it('keeps WoC geometry exempt from Editor visual values', () => {
    expect(
      checkZuiStyleContract(
        source('height = 40, corner_radius = 12'),
        'examples-woc/menu.zui',
      ).issues,
    ).toEqual([]);
  });
  it('checks nested layout values and preserves field paths', () => {
    const text =
      '[asset]\nkind = "view"\nid = "nested"\nversion = 2\n\n[root]\nnode = "root"\n\n[nodes.root]\ncomponent = "Button"\nlayout = { height = { preferred = 40 }, padding = { top = 6 }, container = { gap = 7 } }\n';
    const paths = checkZuiStyleContract(
      text,
      'zircon_editor/assets/ui/editor/button.zui',
    ).issues.map((item) => item.path);
    expect(paths).toEqual([
      'nodes.root.layout.height.preferred',
      'nodes.root.layout.container.gap',
      'nodes.root.layout.padding.top',
    ]);
  });
  it('reserves 48px only for a segmented control with an authored two-line label', () => {
    const segmented = (label: string, height: number) => `[asset]
kind = "view"
id = "segments"
version = 2

[root]
node = "root"

[nodes.root]
component = "WorkbenchSegmentedControl"
props = { options = ["left", "center", "right"]${label ? `, label_text = "${label}"` : ''} }
layout = { height = { min = ${height}, preferred = ${height}, max = ${height}, stretch = "Fixed" } }
`;
    const path = 'zircon_editor/assets/ui/editor/component_drawer.zui';
    expect(
      checkZuiStyleContract(segmented('Segmented Control', 48), path).issues,
    ).toEqual([]);
    expect(
      checkZuiStyleContract(
        segmented('Segmented Control', 32),
        path,
      ).issues.map((item) => item.path),
    ).toEqual([
      'nodes.root.layout.height.preferred',
      'nodes.root.layout.height.min',
      'nodes.root.layout.height.max',
    ]);
    expect(checkZuiStyleContract(segmented('', 48), path).issues).toHaveLength(
      3,
    );
  });
  it('keeps intentional decorative and fully rounded geometry out of the ordinary-control cap', () => {
    const text = `[asset]
kind = "view"
id = "decorative"
version = 2

[root]
node = "root"

[nodes.root]
component = "Overlay"
children = [{ node = "glow" }, { node = "filter_chip" }]

[nodes.glow]
component = "Space"
props = { background_color = "#8ca0b633", border_width = 0, corner_radius = 72 }

[nodes.filter_chip]
component = "Button"
props = { corner_radius = 999 }
`;
    expect(
      checkZuiStyleContract(
        text,
        'zircon_editor/assets/ui/editor/decorative.zui',
      ).issues,
    ).toEqual([]);
  });
});
