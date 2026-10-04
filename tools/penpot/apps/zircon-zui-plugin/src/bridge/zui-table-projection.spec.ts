import type { ZuiDocument } from './zui-document';
import {
  cloneProjectionSnapshot,
  projectZuiDocument,
  reconcileZuiDocument,
} from './penpot-projection';
import { tableCells } from './zui-table-cells';
import { tableColumnWidths, tableGeometry } from './zui-table-geometry';

function fixture(): ZuiDocument {
  return {
    asset: { id: 'res://table.zui', kind: 'component', version: 2 },
    components: { Row: { root: 'root' } },
    nodes: {
      root: {
        component: 'Table',
        control_id: 'WorkbenchTableRow',
        classes: ['workbench-table-row'],
        props: {
          text: 'Source',
          options: ['Mesh.asset', '', 'tex', '12K', 'r42'],
        },
        layout: { width: { preferred: 480 }, height: { preferred: 32 } },
        events: [{ id: 'Select', event: 'Click', route: 'asset.select' }],
        extension: { unknown: { value: true } },
      },
    },
  };
}

describe('Workbench table projection', () => {
  it('maps native display cells to source indices and preserves a no-edit roundtrip', () => {
    const source = fixture();
    const projection = projectZuiDocument(source);
    const row = projection.shapes[0];
    expect(row.text).toBeNull();
    expect(Object.keys(row.textFragments!)).toEqual([
      'cell-0',
      'cell-2',
      'cell-3',
      'cell-4',
    ]);
    expect(
      Object.values(row.textFragments!).map((text) => text.characters),
    ).toEqual(['Mesh.asset', 'Texture', '12 KB', 'rev 42']);
    expect(row.textFragments!['cell-0'].fontSize).toBe(14);
    expect(
      reconcileZuiDocument(source, cloneProjectionSnapshot(projection))
        .document,
    ).toEqual(source);
  });

  it('writes one explicit cell without changing events, unknown fields or filtered options', () => {
    const source = fixture();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
    snapshot.shapes[0].current.textFragments!['cell-2'].characters = 'Material';
    const expected = structuredClone(source);
    (expected.nodes!['root'].props!['options'] as string[])[2] = 'Material';
    expect(reconcileZuiDocument(source, snapshot).document).toEqual(expected);
  });

  it('rejects style and archived-cell edits the native row cannot represent', () => {
    const source = fixture();
    const snapshot = cloneProjectionSnapshot(projectZuiDocument(source));
    snapshot.shapes[0].current.textFragments!['cell-0'].fontSize = 18;
    expect(() => reconcileZuiDocument(source, snapshot)).toThrow(
      /retained-host fontSize/,
    );
    source.nodes!['root'].props = { text: 'Host UI 12K r42' };
    const archived = cloneProjectionSnapshot(projectZuiDocument(source));
    archived.shapes[0].current.textFragments!['archived-cell-0'].characters =
      'Other';
    expect(() => reconcileZuiDocument(source, archived)).toThrow(
      /declare options/,
    );
  });

  it('uses declared cell data or native archived parsing without interpreting value_text as cells', () => {
    expect(
      tableCells({
        text: 'Name Type Size Rev',
        value_text: 'Not native cells',
      }).map((cell) => cell.value),
    ).toEqual(['Name', 'Type', 'Size', 'Revision']);
    expect(
      tableCells({
        text: 'Host UI 12K r42',
        options: ['Whole row with four words', 'Another row with four words'],
      }).map((cell) => cell.value),
    ).toEqual(['Host', 'UI', '12 KB', 'rev 42']);
  });

  it('allocates only native columns within finite bounds, including narrow and tiny frames', () => {
    const table = projectZuiDocument(fixture()).shapes[0].table!;
    const measure = (text: string) => text.length * 7;
    expect(tableColumnWidths(table, 500, measure)).toEqual([180, 135, 95, 90]);
    for (const width of [240, 360, 480]) {
      const columns = tableColumnWidths(table, width - 40, measure);
      expect(columns.reduce((sum, value) => sum + value, 0)).toBeCloseTo(
        width - 40,
      );
      const geometry = tableGeometry(table, width, 32, measure);
      for (const rect of Object.values(geometry.texts)) {
        expect(rect.x).toBeGreaterThanOrEqual(0);
        expect(rect.x + rect.width).toBeLessThanOrEqual(width);
        expect(rect.height).toBe(24);
      }
    }
    expect(
      tableColumnWidths({ ...table, narrow: true }, 500, measure).slice(2),
    ).toEqual([0, 0]);
    expect(tableGeometry(table, 1, 1, measure).texts).toEqual({});
  });

  it('reserves a column for the widest authored cell before hiding lower-priority columns', () => {
    const table = projectZuiDocument(fixture()).shapes[0].table!;
    const authored = {
      ...table,
      cells: [
        ...table.cells,
        {
          key: 'cell-static-mesh',
          column: 1,
          sourceIndex: 5,
          value: 'Static Mesh',
        },
      ],
    };
    const measure = (text: string) => text.length * 7;
    const widths = tableColumnWidths(authored, 328, measure);
    expect(widths[1]).toBeGreaterThanOrEqual(
      measure('Static Mesh') + authored.insetX * 2 + authored.textClipGuard,
    );
    expect(widths[3]).toBe(0);
  });

  it('paints a selected Workbench table row from review state without changing source props', () => {
    const source = fixture();
    source.tokens = {
      'editor.surface.1': '#242424',
      'editor.surface.selected': '#243f5a',
      'editor.surface.hover': '#454545',
      'editor.surface.3': '#383838',
      'editor.surface.disabled': '#2b2b2b',
      'editor.accent': '#60aeff',
      'editor.focus.ring': '#66b2ff',
    };
    source.nodes!['root'].props = {
      ...source.nodes!['root'].props,
      selected: false,
      background_color: '$editor.surface.1',
      selected_background_color: '$editor.surface.selected',
      hover_background_color: '$editor.surface.hover',
      pressed_background_color: '$editor.surface.3',
      disabled_background_color: '$editor.surface.disabled',
      focus_border_color: '$editor.focus.ring',
      border_width: 1,
    };
    const reviewed = structuredClone(source);
    reviewed.nodes!['root'].state = { selected: true };
    const base = projectZuiDocument(source).shapes[0];
    const selected = projectZuiDocument(reviewed).shapes[0];

    expect(base.paint.fillColor).toBe('#242424');
    expect(selected.paint.fillColor).toBe('#243f5a');
    expect(selected.paint.strokeColor).toBe('#60aeff');
    for (const [flag, color] of [
      ['hovered', '#454545'],
      ['pressed', '#383838'],
      ['disabled', '#2b2b2b'],
    ] as const) {
      const changed = structuredClone(source);
      changed.nodes!['root'].state = { [flag]: true };
      expect(projectZuiDocument(changed).shapes[0].paint.fillColor).toBe(color);
    }
    const focused = structuredClone(source);
    focused.nodes!['root'].state = { focused: true };
    expect(projectZuiDocument(focused).shapes[0].paint.strokeColor).toBe(
      '#66b2ff',
    );
    expect(reviewed.nodes!['root'].props?.['selected']).toBe(false);
    expect(
      reconcileZuiDocument(
        source,
        cloneProjectionSnapshot(projectZuiDocument(source)),
      ).document,
    ).toEqual(source);
  });
});
