import type { ZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import {
  createPenpotBridgeAsset,
  parsePenpotBridgeAsset,
  serializePenpotBridgeAsset,
  reconcilePenpotBridgeAsset,
} from './penpot-asset';
import { segmentedGeometry } from './zui-segmented-geometry';

function fixture(): ZuiDocument {
  return {
    asset: { id: 'res://segments.zui', kind: 'component', version: 2 },
    components: { Segments: { root: 'root' } },
    tokens: { selection: '#243f5a', body: 14 },
    nodes: {
      root: {
        component: 'SegmentedControl',
        props: {
          value: 'center',
          options: ['left', 'center', 'right'],
          selected_background_color: '$selection',
          font_size: '$body',
        },
        layout: { width: { preferred: 360 }, height: { preferred: 32 } },
        events: [
          { id: 'Mode/Change', event: 'Change', route: 'test.select_mode' },
        ],
        extension: { preserve: ['binding', 'selection'] },
      },
    },
  };
}

describe('native segmented and tab projection', () => {
  it('projects native option labels and selected paint without rewriting values', () => {
    const source = fixture();
    const projection = projectZuiDocument(source);
    const node = projection.shapes[0];
    expect(node.text).toBeNull();
    expect(
      Object.values(node.textFragments!).map((text) => text.characters),
    ).toEqual(['Left', 'Center', 'Right']);
    expect(node.textFragments!['option-1']).toMatchObject({
      property: 'options',
      color: '#e8e8e8',
      fontSize: 14,
      fontWeight: '400',
    });
    const geometry = segmentedGeometry(node.segmented!, 360, 32);
    expect(geometry.parts['selected-option-1']).toMatchObject({
      x: 122,
      y: 2,
      width: 116,
      height: 28,
      fill: '#243f5a',
      radius: 3,
    });
    expect(geometry.parts['underline-option-1']).toMatchObject({
      x: 122,
      y: 28,
      width: 116,
      height: 2,
      fill: '#60aeff',
    });
    expect(
      reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(source);
  });

  it('keeps option lanes, dividers and text insets stable across required widths', () => {
    const node = projectZuiDocument(fixture()).shapes[0];
    for (const width of [240, 360, 480]) {
      const geometry = segmentedGeometry(node.segmented!, width, 32);
      expect(geometry.parts['divider-1']).toMatchObject({
        x: width / 3,
        y: 4,
        width: 1,
        height: 24,
      });
      expect(geometry.texts['option-1']).toEqual({
        x: width / 3 + 8,
        y: 5,
        width: width / 3 - 16,
        height: 22,
      });
    }
    expect(segmentedGeometry(node.segmented!, 1, 32)).toEqual({
      parts: {},
      texts: {},
    });
  });

  it('edits a selected option and its selection key while retaining source contracts', () => {
    const source = fixture();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
    snapshot.shapes[0].current.textFragments!['option-1'].characters = 'Middle';
    const result = reconcileZuiDocument(source, snapshot).document;
    const expected = structuredClone(source);
    expected.nodes!['root'].props!['options'] = ['left', 'Middle', 'right'];
    expected.nodes!['root'].props!['value'] = 'Middle';
    expect(result).toEqual(expected);
    expect(
      projectZuiDocument(result)
        .shapes[0].segmented!.options.filter((option) => option.selected)
        .map((option) => option.key),
    ).toEqual(['option-1']);
  });

  it('maps structured and filtered options to original array indices', () => {
    const source = fixture();
    source.nodes!['root'].props!['options'] = [
      '',
      19,
      { label: 'center', value: 'stable-id', extra: true },
      'right',
    ];
    const asset = parsePenpotBridgeAsset(
      serializePenpotBridgeAsset(
        createPenpotBridgeAsset(source, 'options.zui'),
      ),
    );
    expect(
      Object.keys(asset.snapshot.shapes[0].current.textFragments!),
    ).toEqual(['option-2', 'option-3']);
    asset.snapshot.shapes[0].current.textFragments!['option-2'].characters =
      'Middle';
    expect(
      reconcilePenpotBridgeAsset(asset).document.nodes!['root'].props,
    ).toMatchObject({
      value: 'Middle',
      options: [
        '',
        19,
        { label: 'Middle', value: 'stable-id', extra: true },
        'right',
      ],
    });
    expect(source.nodes!['root'].props!['value']).toBe('center');
  });

  it('maps selected, idle, disabled and group label colors to their actual source roles', () => {
    const source = fixture();
    source.nodes!['root'].props!['group_label'] = 'Mode';
    let snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
    snapshot.shapes[0].current.text!.characters = 'Alignment';
    snapshot.shapes[0].current.text!.color = '#aaaaaa';
    snapshot.shapes[0].current.textFragments!['option-1'].color = '#ffffff';
    expect(
      reconcileZuiDocument(source, snapshot).document.nodes!['root'].props,
    ).toMatchObject({
      group_label: 'Alignment',
      label_color: '#aaaaaa',
      selected_foreground_color: '#ffffff',
    });
    source.nodes!['root'].state = { disabled: true };
    snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
    snapshot.shapes[0].current.text!.color = '#888888';
    for (const text of Object.values(snapshot.shapes[0].current.textFragments!))
      text.color = '#888888';
    expect(
      reconcileZuiDocument(source, snapshot).document.nodes!['root'].props![
        'disabled_foreground_color'
      ],
    ).toBe('#888888');
  });

  it('rejects independent shared styles, invalid display labels and ambiguous selection', () => {
    const source = fixture();
    for (const edit of [
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['option-1'].fontSize = 16;
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['option-0'].color = '#ffffff';
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['option-0'].characters = 'lowercase';
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['option-1'].characters = 'Right';
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        s.shapes[0].current.textFragments!['option-1'].property = 'value';
      },
      (s: ReturnType<typeof cloneProjectionSnapshot>) => {
        delete s.shapes[0].current.textFragments!['option-1'];
      },
    ]) {
      const snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
      edit(snapshot);
      expect(() => reconcileZuiDocument(source, snapshot)).toThrow();
    }
  });

  it('places group labels and applies unavailable precedence without dropping options', () => {
    const source = fixture();
    source.nodes!['root'].props!['label'] = 'Alignment';
    source.nodes!['root'].state = {
      disabled: true,
      pressed: true,
      hovered: true,
    };
    const node = projectZuiDocument(source).shapes[0];
    const geometry = segmentedGeometry(node.segmented!, 240, 56);
    expect(geometry.texts['primary']).toEqual({
      x: 0,
      y: 0,
      width: 240,
      height: 14,
    });
    expect(geometry.parts['surface']).toMatchObject({
      y: 18,
      height: 38,
      fill: '#2b2b2b',
      stroke: '#363636',
    });
    expect(geometry.parts['underline-option-1'].fill).toBe('#737373');
    expect(Object.keys(geometry.texts)).toHaveLength(4);
  });

  it('maps Tab text to its own typography and preserves selected underline semantics', () => {
    const source = fixture();
    source.nodes!['root'].component = 'Tab';
    source.nodes!['root'].props = {
      text: 'Scene',
      selected: true,
      tab_font_size: 14,
      tab_line_height_ratio: 1.4,
    };
    const projection = projectZuiDocument(source);
    const node = projection.shapes[0];
    expect(node.text).toMatchObject({
      characters: 'Scene',
      property: 'text',
      fontSize: 14,
      color: '#e8e8e8',
    });
    expect(
      segmentedGeometry(node.segmented!, 120, 32).parts['underline'],
    ).toMatchObject({ x: 0, y: 30, width: 120, height: 2 });
    const snapshot = cloneProjectionSnapshot(projection);
    snapshot.shapes[0].current.text!.fontSize = 16;
    snapshot.shapes[0].current.text!.characters = 'Assets';
    expect(
      reconcileZuiDocument(source, snapshot).document.nodes!['root'].props,
    ).toMatchObject({
      text: 'Assets',
      selected: true,
      tab_font_size: 16,
      tab_line_height: 22.4,
    });
  });

  it('preserves authored owner paint and refuses unresolved painter tokens', () => {
    const source = fixture();
    source.nodes!['root'].props!['background_color'] = '#242424';
    expect(projectZuiDocument(source).shapes[0].paint.fillColor).toBe(
      '#242424',
    );
    source.nodes!['root'].props!['selected_inset'] = '$missing';
    expect(() => projectZuiDocument(source)).toThrow(
      /Unresolved segmented control token/,
    );
  });

  it('keeps transparent Tab surfaces absent while retaining their hover surface', () => {
    const source = fixture();
    source.nodes!['root'].component = 'Tab';
    source.nodes!['root'].props = {
      text: 'Scene',
      background_color: 'transparent',
      border_color: 'transparent',
    };
    let node = projectZuiDocument(source).shapes[0];
    expect(node.paint.fillOpacity).toBe(0);
    expect(
      segmentedGeometry(node.segmented!, 120, 32).parts['surface'],
    ).toBeUndefined();
    expect(
      reconcileZuiDocument(
        source,
        cloneProjectionSnapshot(projectZuiDocument(source)),
      ).document,
    ).toEqual(source);
    source.nodes!['root'].state = { hovered: true };
    node = projectZuiDocument(source).shapes[0];
    expect(
      segmentedGeometry(node.segmented!, 120, 32).parts['surface'].fill,
    ).toBe('#454545');
  });
});
