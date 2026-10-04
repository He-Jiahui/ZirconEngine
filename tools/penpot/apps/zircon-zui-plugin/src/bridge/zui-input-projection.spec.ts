import type { ZuiDocument } from './zui-document';
import { inputControlGeometry } from './zui-input-projection';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';

const checkbox = (): ZuiDocument => ({
  asset: { kind: 'component', id: 'res://checkbox.zui', version: 2 },
  components: { CheckboxExample: { root: 'root' } },
  tokens: { 'label.color': '#b3b3b3' },
  nodes: {
    root: {
      component: 'Checkbox',
      props: {
        text: 'Checkbox',
        checked: false,
        label_color: '$label.color',
        foreground_color: '#ffffff',
        font_size: 14,
        layout_spacing: 8,
      },
    },
  },
});

describe('native input projection', () => {
  it('uses the selection label color and regular left-aligned text', () => {
    const document = checkbox();
    const original = structuredClone(document);
    const projection = projectZuiDocument(document);
    expect(projection.shapes[0].text).toMatchObject({
      color: '#b3b3b3',
      fontSize: 14,
      fontWeight: '400',
      align: 'left',
    });
    expect(
      reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(original);
  });

  it('writes a label color edit to the property consumed by the engine', () => {
    const document = checkbox();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(document));
    snapshot.shapes[0].current.text!.color = '#cccccc';
    const result = reconcileZuiDocument(document, snapshot);
    expect(result.document.nodes!['root'].props).toMatchObject({
      label_color: '#cccccc',
      foreground_color: '#ffffff',
    });
    expect(result.document.tokens).toEqual(document.tokens);
  });

  it('keeps the resolved label style above metadata paint aliases', () => {
    const document = checkbox();
    document.nodes!['root'].state = { foreground_color: '#000000' };
    document.nodes!['root'].style = { self: { label_color: '#cccccc' } };
    expect(projectZuiDocument(document).shapes[0].text?.color).toBe('#cccccc');
    document.nodes!['root'].state!['disabled'] = true;
    expect(projectZuiDocument(document).shapes[0].text?.color).toBe('#737373');
  });

  it('keeps checkbox labels beyond the native mark across component widths', () => {
    const control = projectZuiDocument(checkbox()).shapes[0].inputControl!;
    for (const width of [240, 360, 480]) {
      const geometry = inputControlGeometry(control, width, 28);
      expect(geometry.mark).toEqual({ x: 10, y: 6, width: 16, height: 16 });
      expect(geometry.text).toMatchObject({ x: 34, y: 5, width: width - 44 });
    }
  });

  it('uses a centered selected radio dot and preserves authored state on export', () => {
    const document = checkbox();
    document.nodes!['root'].component = 'Radio';
    document.nodes!['root'].state = { selected: true };
    const projection = projectZuiDocument(document);
    const control = projection.shapes[0].inputControl!;
    expect(control.kind).toBe('radio');
    expect(control.active).toBe(true);
    const geometry = inputControlGeometry(control, 360, 28);
    expect(geometry.mark).toEqual({ x: 10, y: 6, width: 16, height: 16 });
    expect(geometry.secondary).toEqual({
      x: 14.5,
      y: 10.5,
      width: 7,
      height: 7,
    });
    expect(
      reconcileZuiDocument(document, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(document);
  });

  it('keeps toggle text clear of a right-aligned track and moves only its thumb', () => {
    const document = checkbox();
    document.nodes!['root'].component = 'Toggle';
    const off = projectZuiDocument(document).shapes[0].inputControl!;
    const offGeometry = inputControlGeometry(off, 360, 28);
    expect(offGeometry.mark).toEqual({ x: 318, y: 5, width: 34, height: 18 });
    expect(offGeometry.text).toMatchObject({ x: 10, width: 300 });
    expect(offGeometry.secondary).toEqual({
      x: 320,
      y: 8,
      width: 12,
      height: 12,
    });
    document.nodes!['root'].state = { selected: true };
    const on = projectZuiDocument(document).shapes[0].inputControl!;
    expect(inputControlGeometry(on, 360, 28).secondary).toEqual({
      x: 338,
      y: 8,
      width: 12,
      height: 12,
    });
    document.nodes!['root'].state!['disabled'] = true;
    const disabled = projectZuiDocument(document).shapes[0].inputControl!;
    expect(disabled.secondary?.color).toBe('#737373');
  });

  it('reserves the authored caret gap and right inset for dropdown text', () => {
    const document = checkbox();
    document.nodes!['root'] = {
      component: 'Dropdown',
      props: {
        value_text: 'Default',
        font_size: 14,
        caret_size: 12,
        caret_right_inset: 12,
        caret_gap: 4,
        horizontal_inset: 8,
      },
    };
    const control = projectZuiDocument(document).shapes[0].inputControl!;
    const geometry = inputControlGeometry(control, 360, 32);
    expect(geometry.mark).toEqual({ x: 336, y: 10, width: 12, height: 12 });
    expect(geometry.text).toMatchObject({ x: 8, width: 324 });
    expect(geometry.text.y).toBeCloseTo(6.2);
    expect(geometry.text.height).toBeCloseTo(19.6);
    document.nodes!['root'].state = { hovered: true };
    expect(projectZuiDocument(document).shapes[0].paint.strokeColor).toBe(
      '#60aeff',
    );
    document.nodes!['root'].props!['label'] = 'Quality';
    expect(() => projectZuiDocument(document)).toThrow(
      'separate editable label and value',
    );
  });
});
