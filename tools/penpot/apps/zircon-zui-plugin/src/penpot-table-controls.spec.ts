import type { Board, Shape, Text } from '@penpot/plugin-types';
import { describe, expect, it } from 'vitest';
import { projectWorkbenchTable } from './bridge/zui-table-projection';
import { TABLE_COLUMN_SAMPLES } from './bridge/zui-table-geometry';
import { assertControlTextGeometry } from './penpot-control-guard';
import { layoutTableText, TABLE_MEASUREMENT } from './penpot-table-controls';
import { assertExportShapeGuard, writeExportShapeGuard } from './penpot-export-guard';

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

// Proposed global-guard coverage; fixtures use the real owner identity, not
// standalone text snapshots. No guard baseline is rewritten after a reflow.
function sourceMappedTableFixture(fragment = 'cell-0') {
  const namespace = 'dev.zircon.zui';
  const assetData = new Map([['role', 'asset']]);
  const rowData = new Map([
    ['role', 'node'], ['node-id', 'retarget_root_chain_table_row'],
    ['component', 'Table'],
    ['sourcePath', 'zircon_editor/assets/ui/editor/workbench/primitives/data/table_row.zui'],
    ['sourceNodeId', 'root'],
    ['instancePath', JSON.stringify([{sourcePath:'zircon_editor/assets/ui/editor/animation/extension_retarget_workspace.zui',sourceNodeId:'retarget_root_chain_table_row'}])],
    ['controlId', 'WorkbenchExtensionRetargetRootChainTableRow'],
  ]);
  const textData = new Map([['role', 'text'], ['text-fragment', fragment]]);
  const getter = (data: Map<string, string>) => (ns: string, key: string) =>
    ns === namespace ? data.get(key) ?? '' : '';
  const setter = (data: Map<string, string>) => (_ns: string, key: string, value: string) => data.set(key, value);
  const table = projectWorkbenchTable(
    { asset: { id: 'res://ui/editor/animation/extension_retarget_workspace.zui', kind: 'component', version: 2 } },
    { component: 'Table', control_id: 'WorkbenchExtensionRetargetRootChainTableRow',
      props: { options: ['Root', 'Bone', '0', '0'] } },
  )!.table;
  const asset = { id: 'asset', name: 'Retarget owner', type: 'board',
    x: 0, y: 0, width: 720, height: 48, children: [] as Shape[],
    getSharedPluginData: getter(assetData), setSharedPluginData: setter(assetData),
  } as unknown as Board;
  const row = { id: 'row', name: 'retarget_root_chain_table_row', type: 'board',
    parent: asset, x: 0, y: 0, width: 720, height: 48, children: [] as Shape[],
    getSharedPluginData: getter(rowData), setSharedPluginData: setter(rowData),
  } as unknown as Board;
  const text = { id: 'cell', name: 'ZUI text '+fragment, type: 'text', parent: row,
    x: 0, y: 0, width: 1, height: 1, hidden: false,
    getSharedPluginData: getter(textData), setSharedPluginData: setter(textData),
    resize(width: number, height: number) { Object.assign(this, {width, height}); },
  } as unknown as Text;
  row.children.push(text);
  const labels = [...new Set([...TABLE_COLUMN_SAMPLES, ...table.cells.map(cell => cell.value)])];
  asset.children.push(row, ...labels.map((label, index) => ({
    id: 'measurement-'+index, name: label, type: 'text', parent: asset,
    x: 0, y: 0, width: Math.max(1, label.length*7), height: 16,
    getSharedPluginData: getter(new Map([['role','auxiliary'], [TABLE_MEASUREMENT,label]])),
  }) as unknown as Text));
  layoutTableText(row, text, table);
  writeExportShapeGuard(asset);
  const saved = asset.getSharedPluginData(namespace, 'export-shapes-guard');
  const reflow = () => { row.width += 6; layoutTableText(row, text, table); };
  return {asset,row,text,rowData,textData,reflow,
    savedGuard: () => asset.getSharedPluginData(namespace, 'export-shapes-guard'),saved};
}

describe('source-owned Table global geometry guard', () => {
  it('accepts measured cell reflow without replacing the export baseline', () => {
    const f = sourceMappedTableFixture();
    const width = f.text.width;
    f.reflow();
    expect(f.text.width).not.toBe(width);
    expect(() => assertExportShapeGuard(f.asset)).not.toThrow();
    expect(f.savedGuard()).toBe(f.saved);
  });
  it('rejects a direct cell resize after source reflow', () => {
    const f = sourceMappedTableFixture(); f.reflow(); f.text.width += 0.125;
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
    expect(f.savedGuard()).toBe(f.saved);
  });
  it('rejects changed source owner identity after source reflow', () => {
    const f = sourceMappedTableFixture(); f.reflow(); f.rowData.set('sourceNodeId', 'other');
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
  });
  it('rejects reassigned cell fragment after source reflow', () => {
    const f = sourceMappedTableFixture(); f.reflow(); f.textData.set('text-fragment', 'cell-1');
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
  });
  it('does not sign an unknown fragment as mapped cell geometry', () => {
    const f = sourceMappedTableFixture();
    f.textData.set('text-fragment', 'unknown-cell'); f.reflow();
    f.text.hidden = false; f.text.width += 0.125;
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
  });
  it('requires complete source identity before accepting reflow', () => {
    const f = sourceMappedTableFixture(); f.rowData.delete('instancePath'); f.reflow();
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed geometry/);
  });
  it('keeps topology immutable after a valid mapped reflow', () => {
    const f = sourceMappedTableFixture(); f.reflow(); f.row.children = [];
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed children|added or removed/);
  });
  it('leaves unmapped column visibility changes rejected', () => {
    const f = sourceMappedTableFixture('cell-3');
    f.row.width = 120; layoutTableText(f.row, f.text, projectWorkbenchTable(
      {asset:{id:'res://row.zui',kind:'component',version:2}},
      {component:'Table',control_id:'row',props:{options:['Root','Bone','0','0']}},
    )!.table);
    expect(f.text.hidden).toBe(true);
    expect(() => assertExportShapeGuard(f.asset)).toThrow(/changed hidden/);
  });
});
