import { describe, expect, it } from 'vitest';
import type { ZuiDocument } from './bridge/zui-document';
import { projectZuiDocument } from './bridge/penpot-projection';
import { measureLinearDesiredSizes } from './penpot-linear-content';

function document(
  kind: string,
  container: Record<string, unknown>,
  mounts = [{ node: 'a' }, { node: 'b' }],
): ZuiDocument {
  return {
    asset: { kind: 'view', id: 'test', version: 2 },
    root: { node: 'root' },
    nodes: {
      root: {
        component: kind,
        layout: { container: { kind, ...container } },
        children: mounts,
      },
      a: { component: 'Label' },
      b: { component: 'Label' },
    },
  };
}
function measure(doc: ZuiDocument, width = 30, height = 20) {
  return measureLinearDesiredSizes(doc, projectZuiDocument(doc), () => ({
    width,
    height,
  })).get('root');
}
describe('native intrinsic container contracts', () => {
  it('ignores slot padding while measuring scroll content', () => {
    const doc = document('ScrollableBox', { axis: 'Vertical', gap: 4 });
    doc.nodes!['root'].children![0].slot = {
      layout: { padding: { left: 5, right: 7, top: 8, bottom: 9 } },
    };
    expect(measure(doc)).toEqual({ width: 30, height: 44 });
  });
  it('applies wrap item_min_width before padding and line breaks', () => {
    const doc = document('WrapBox', {
      item_min_width: 60,
      horizontal_gap: 5,
      vertical_gap: 5,
    });
    doc.nodes!['root'].layout!['width'] = {
      min: 0,
      preferred: 100,
      max: 100,
      stretch: 'Fixed',
    };
    for (const mount of doc.nodes!['root'].children!)
      mount.slot = { layout: { padding: { left: 5, right: 5 } } };
    expect(measure(doc)).toEqual({ width: 100, height: 45 });
  });
  it('expands grid dimensions and distributes desired extents across explicit spans', () => {
    const doc = document(
      'GridBox',
      { columns: 1, rows: 1, column_gap: 5, row_gap: 3 },
      [{ node: 'a' }],
    );
    doc.nodes!['root'].children![0].slot = {
      layout: {
        column: 2,
        row: 1,
        column_span: 2,
        row_span: 2,
        padding: { left: 10, right: 10, top: 2, bottom: 2 },
      },
    };
    expect(measure(doc, 100, 40)).toEqual({ width: 135, height: 50 });
  });
  it('uses MUI size and offset only when slot.layout is absent', () => {
    const doc = document('GridBox', { columns: 2, rows: 1, column_gap: 5 }, [
      { node: 'a' },
    ]);
    doc.nodes!['a'].props = { size: 2, offset: 3 };
    expect(measure(doc, 120, 40)).toEqual({ width: 140, height: 40 });
    doc.nodes!['root'].children![0].slot = { layout: {} };
    expect(measure(doc, 120, 40)).toEqual({ width: 125, height: 40 });
  });
});
