import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { LayoutDependencies } from '../../tools/zui-layout-dependencies';
import { parseZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { propertyRowGeometry } from './zui-property-row-geometry';
import { parsePropertyAxes } from './zui-property-row-values';

describe('Editor retained property rows', () => {
  const root = (process.env['ZUI_LAYOUT_REPO_ROOT'] ?? fileURLToPath(new URL('../../../../../../', import.meta.url)));
  const asset =
    'zircon_editor/assets/ui/editor/components/workbench/primitives/data/workbench_property_row.zui';
  const theme = 'zircon_editor/assets/ui/editor/theme/editor_tokens.zui';
  const dependencies = new LayoutDependencies();
  beforeAll(async () => dependencies.load(root, [asset, theme]));
  function source(value = 'Value') {
    const document = parseZuiDocument(
      readFileSync(`${root}/${asset}`, 'utf8'),
    ).document;
    dependencies.embed(document, asset, theme);
    document.nodes!['root'].props!['value'] = value;
    document.nodes!['root'].events = [
      { id: 'commit', event: 'Change', route: 'property.commit' },
    ];
    document.nodes!['root']['extension'] = { retained: true };
    return document;
  }

  it('projects distinct label and value fields using host theme metrics and keeps no-edit semantics', () => {
    const document = source();
    const shape = projectZuiDocument(document).shapes[0];
    expect(shape.text).toBeNull();
    expect(shape.textFragments!['property-label'].characters).toBe('Property');
    expect(shape.textFragments!['property-value']).toMatchObject({
      characters: 'Value',
      property: 'value',
      fontSize: 14,
    });
    expect(shape.propertyRow).toMatchObject({
      labelWidth: 98,
      fieldInsetY: 3,
      fieldRadius: 4,
      fieldBorderWidth: 1,
      fieldFill: '#0f0f0f',
    });
    expect(propertyRowGeometry(shape.propertyRow!, 360, 28)).toMatchObject({
      parts: {
        'field-property-value': { x: 98, y: 3, width: 257, height: 22 },
      },
      texts: {
        'property-label': { x: 5, y: 4, width: 90.5, height: 20 },
        'property-value': { x: 103, y: 7, width: 247, height: 14 },
      },
    });
    expect(
      reconcileZuiDocument(
        document,
        cloneProjectionSnapshot(projectZuiDocument(document)),
      ).document,
    ).toEqual(document);
  });

  it('uses native identities and never adds property fields solely from a class name', () => {
    const document = source();
    document.nodes!['root'].component = 'InputField';
    expect(projectZuiDocument(document).shapes[0].propertyRow).toBeUndefined();
    document.nodes!['root'].control_id = 'WorkbenchMeshRow';
    document.nodes!['root'].props!['layout_label_width'] = 92;
    document.nodes!['root'].props!['border_width'] = 0;
    expect(projectZuiDocument(document).shapes[0].propertyRow).toMatchObject({
      labelWidth: 108,
      fieldBorderWidth: 1,
    });
    document.nodes!['root'].control_id = 'WorkbenchComponentPropertyRowRoot';
    expect(projectZuiDocument(document).shapes[0].propertyRow).toMatchObject({
      labelWidth: 108,
    });
    delete document['penpot_host_theme_source'];
    expect(projectZuiDocument(document).shapes[0].propertyRow).toBeUndefined();
  });

  it('writes scalar and label edits to their authored layer without losing events or unknown fields', () => {
    const document = source();
    document.nodes!['root'].state = { value_text: '  Current  ' };
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.textFragments!['property-value'].characters =
      'Updated';
    snapshot.shapes[0].current.textFragments!['property-label'].characters =
      'Asset';
    const expected = structuredClone(document);
    expected.nodes!['root'].state!['value_text'] = 'Updated';
    expected.nodes!['root'].props!['text'] = 'Asset';
    expect(reconcileZuiDocument(document, snapshot).document).toEqual(expected);
  });

  it('preserves skipped tokens, repeated axes, units and undisplayed groups during an axis edit', () => {
    const raw = ' ignored X  10 px auto  Y Z -2.5 rem W 1 2 3 X 4 Y 5 ';
    const document = source(raw);
    const projection = projectZuiDocument(document);
    const shape = projection.shapes[0];
    expect(shape.propertyRow!.axisKeys).toEqual([
      'axis-0',
      'axis-1',
      'axis-2',
      'axis-3',
    ]);
    expect(shape.textFragments!['axis-0'].characters).toBe('10 px auto');
    expect(shape.textFragments!['axis-1-label'].characters).toBe('Z');
    expect(parsePropertyAxes(raw)).toHaveLength(5);
    expect(
      reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(document);
    const snapshot = cloneProjectionSnapshot(projection);
    snapshot.shapes[0].current.textFragments!['axis-1'].characters = '-3 rem';
    const result = reconcileZuiDocument(document, snapshot).document;
    expect(result.nodes!['root'].props!['value']).toBe(
      raw.replace('-2.5 rem', '-3 rem'),
    );
  });

  it('rejects unmapped host typography, axis labels and parser-changing edits', () => {
    const document = source('X 0 Y 1');
    for (const [key, property, value] of [
      ['axis-0', 'fontSize', 18],
      ['axis-0-label', 'characters', 'Z'],
      ['axis-0', 'characters', '3 Z 4'],
    ] as const) {
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
      Object.assign(snapshot.shapes[0].current.textFragments![key], {
        [property]: value,
      });
      expect(() => reconcileZuiDocument(document, snapshot)).toThrow(
        /retained-host|axis label|axis grouping/,
      );
    }
  });

  it('clamps native geometry at narrow widths without inventing larger text boxes', () => {
    const shape = projectZuiDocument(source('X 0 Y 1 Z 2 W 3')).shapes[0];
    for (const width of [1, 32, 240, 360, 480]) {
      const geometry = propertyRowGeometry(shape.propertyRow!, width, 28);
      for (const rect of [
        ...Object.values(geometry.parts),
        ...Object.values(geometry.texts),
      ]) {
        expect(rect.width).toBeGreaterThan(0);
        expect(rect.height).toBeGreaterThan(0);
        expect(rect.x + rect.width).toBeLessThanOrEqual(width + 0.0001);
        expect(rect.y + rect.height).toBeLessThanOrEqual(28);
      }
    }
    expect(propertyRowGeometry(shape.propertyRow!, 0, 0)).toEqual({
      parts: {},
      texts: {},
    });
  });

  it('uses focus borders for scalar fields while selection and axis groups stay neutral', () => {
    const document = source();
    for (const state of ['selected', 'focused', 'pressed']) {
      document.nodes!['root'].state = { [state]: true };
      expect(
        projectZuiDocument(document).shapes[0].propertyRow!.fieldBorder,
      ).toBe(state === 'selected' ? '#484848' : '#66b2ff');
    }
    document.nodes!['root'].props!['value'] = 'X 0 Y 1';
    expect(
      projectZuiDocument(document).shapes[0].propertyRow!.fieldBorder,
    ).toBe('#484848');
  });
});
