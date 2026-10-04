import type { Board, Text } from '@penpot/plugin-types';
import { describe, expect, it } from 'vitest';
import { projectWorkbenchTable } from './bridge/zui-table-projection';
import { TABLE_COLUMN_SAMPLES } from './bridge/zui-table-geometry';
import { assertControlTextGeometry } from './penpot-control-guard';
import { layoutTableText, TABLE_MEASUREMENT } from './penpot-table-controls';

describe('native table text layout', () => {
  it('keeps the no-edit guard current when measured columns disappear and return', () => {
    const table = projectWorkbenchTable(
      { asset: { id: 'res://row.zui', kind: 'component', version: 2 } },
      {
        component: 'Table',
        control_id: 'WorkbenchTableRow',
        props: { options: ['Latin', '512 glyphs', '0 missing', 'Page 0'] },
      },
    )!.table;
    const board = {
      x: 0,
      y: 0,
      width: 480,
      height: 48,
      children: [
        ...TABLE_COLUMN_SAMPLES,
        ...table.cells.map((cell) => cell.value),
      ].map((label) => ({
        width: label.length * 7,
        getSharedPluginData: (_namespace: string, key: string) =>
          key === TABLE_MEASUREMENT ? label : '',
      })),
    } as unknown as Board;
    const metadata = new Map([['text-fragment', 'cell-3']]);
    const text = {
      name: 'Page cell',
      x: 0,
      y: 0,
      width: 1,
      height: 1,
      hidden: false,
      getSharedPluginData: (_namespace: string, key: string) =>
        metadata.get(key) ?? '',
      setSharedPluginData: (_namespace: string, key: string, value: string) =>
        metadata.set(key, value),
      resize(width: number, height: number) {
        this.width = width;
        this.height = height;
      },
    } as unknown as Text;
    layoutTableText(board, text, table);
    expect(text.hidden).toBe(false);
    board.resize = (width: number, height: number) => {
      Object.assign(board, { width, height });
    };
    board.resize(120, 48);
    layoutTableText(board, text, table);
    expect(text.hidden).toBe(true);
    expect(() => assertControlTextGeometry(board, text)).not.toThrow();
    board.resize(480, 48);
    layoutTableText(board, text, table);
    expect(text.hidden).toBe(false);
    expect(() => assertControlTextGeometry(board, text)).not.toThrow();
  });
});
