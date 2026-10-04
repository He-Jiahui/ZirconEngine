import type { Board, Text } from '@penpot/plugin-types';
import {
  assertExportShapeGuard,
  writeExportShapeGuard,
} from './penpot-export-guard';
function fixture() {
  const data = new Map<string, string>();
  const probeData = new Map<string, string>();
  const root = {
    id: 'asset',
    type: 'board',
    name: 'Asset',
    x: 0,
    y: 0,
    width: 960,
    height: 640,
    children: [] as Text[],
    getSharedPluginData: (_: string, k: string) =>
      k === 'role' ? 'asset' : (data.get(k) ?? ''),
    setSharedPluginData: (_: string, k: string, v: string) => {
      data.set(k, v);
    },
  } as unknown as Board;
  const probe = {
    id: 'probe',
    type: 'text',
    name: 'Content font measurement',
    x: 0,
    y: 0,
    width: 100,
    height: 18,
    parent: root,
    opacity: 0,
    growType: 'auto-width',
    layoutChild: { absolute: true },
    fontFamily: 'Fira Sans',
    fontSize: '14',
    fontWeight: '400',
    lineHeight: '1.2',
    characters: 'Before',
    getSharedPluginData: (_: string, k: string) =>
      k === 'role' ? 'auxiliary' : (probeData.get(k) ?? ''),
  } as unknown as Text;
  root.children.push(probe);
  probeData.set('linear-content-measurement', 'title');
  const update = (text: string) => {
    probe.characters = text;
    probeData.set(
      'linear-content-measurement-signature',
      JSON.stringify({
        text,
        style: {
          family: 'Fira Sans',
          size: 14,
          weight: '400',
          lineHeight: 1.2,
        },
        runs: [],
      }),
    );
  };
  update('Before');
  writeExportShapeGuard(root);
  return { root, probe, update, data };
}
it('accepts a regenerated invisible intrinsic probe without rewriting its immutable guard', () => {
  const f = fixture();
  const original = f.data.get('export-shapes-guard');
  f.update('After supported text edit');
  f.probe.resize = () => {};
  Object.assign(f.probe, { width: 220, height: 22 });
  expect(() => assertExportShapeGuard(f.root)).not.toThrow();
  expect(f.data.get('export-shapes-guard')).toBe(original);
});
it('rejects a probe made visible or changed outside its source signature', () => {
  const f = fixture();
  f.probe.opacity = 1;
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
  f.probe.opacity = 0;
  f.probe.characters = 'Unmapped probe edit';
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
});
it('continues guarding measurements with an unsupported font or grow policy', () => {
  const f = fixture();
  f.probe.fontFamily = 'Other Font';
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
  f.probe.fontFamily = 'Fira Sans';
  f.probe.growType = 'fixed';
  expect(() => assertExportShapeGuard(f.root)).toThrow(
    /without a ZUI source mapping/,
  );
});
