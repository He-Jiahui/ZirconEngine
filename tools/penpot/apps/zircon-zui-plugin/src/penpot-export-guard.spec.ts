import type { Board, Shape } from '@penpot/plugin-types';
import {
  assertExportShapeGuard,
  writeExportShapeGuard,
} from './penpot-export-guard';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_AUXILIARY,
} from './metadata';

function guardedIcon(): {
  root: Board;
  path: Shape & { content: string };
  setGenerated: (enabled: boolean) => void;
  translateAsset: (x: number, y: number) => void;
} {
  const data = new Map<string, string>();
  let generated = true;
  const root = {
    id: 'asset',
    name: 'Review board',
    type: 'board',
    x: 0,
    y: 0,
    width: 24,
    height: 24,
    children: [] as Shape[],
    getSharedPluginData: (namespace: string, key: string) =>
      namespace === ZUI_METADATA_NAMESPACE
        ? key === ZUI_METADATA_ROLE
          ? ZUI_ROLE_ASSET
          : data.get(key)
        : undefined,
    setSharedPluginData: (_namespace: string, key: string, value: string) =>
      data.set(key, value),
  } as unknown as Board;
  const group = {
    id: 'icon',
    name: 'Icon: chevron-right',
    type: 'group',
    x: 0,
    y: 0,
    width: 24,
    height: 24,
    parent: root,
    children: [] as Shape[],
    getSharedPluginData: (namespace: string, key: string) =>
      generated &&
      namespace === ZUI_METADATA_NAMESPACE &&
      key === ZUI_METADATA_ROLE
        ? ZUI_ROLE_AUXILIARY
        : undefined,
  } as unknown as Shape & { children: Shape[] };
  const path = {
    id: 'icon-path',
    name: 'arrow outline',
    type: 'path',
    x: 0,
    y: 0,
    width: 1,
    height: 1,
    parent: group,
    content: 'M 0 0 L 1 1',
    getSharedPluginData: () => undefined,
  } as unknown as Shape & { content: string };
  root.children.push(group);
  group.children.push(path);
  writeExportShapeGuard(root);
  return {
    root,
    path,
    setGenerated: (enabled) => (generated = enabled),
    translateAsset: (x, y) => {
      for (const shape of [root, group, path]) {
        shape.x += x;
        shape.y += y;
      }
    },
  };
}

it('identifies the unmapped SVG path without permitting its changed content', () => {
  const { root, path } = guardedIcon();
  path.content = 'M 0 0 L 2 2';

  expect(() => assertExportShapeGuard(root)).toThrow(
    /Icon: chevron-right.*arrow outline.*changed content without a ZUI source mapping.*numeric tokens 4\/4, max numeric delta 1, same command skeleton true/,
  );
});

it('allows only Penpot numeric normalization on generated icon paths', () => {
  const { root, path } = guardedIcon();
  path.content = 'M 0 0 L 1.000030517578125 1';
  expect(() => assertExportShapeGuard(root)).not.toThrow();
});

it('still rejects the same tiny change on a path without generated-icon ownership', () => {
  const { root, path, setGenerated } = guardedIcon();
  setGenerated(false);
  writeExportShapeGuard(root);
  path.content = 'M 0 0 L 1.000030517578125 1';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('rejects a generated icon path change above the sub-pixel tolerance', () => {
  const { root, path } = guardedIcon();
  path.content = 'M 0 0 L 1.0002 1';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('still rejects altered path commands even with unchanged numeric coordinates', () => {
  const { root, path } = guardedIcon();
  path.content = 'M 0 0 Q 1 1';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('reports both measured geometries when an unmapped helper moves', () => {
  const { root, path } = guardedIcon();
  Object.assign(path.parent!, { x: 0, y: 0 });
  Object.assign(path, { x: 0, y: 0, width: 24, height: 24 });
  writeExportShapeGuard(root);
  path.x = 2;
  expect(() => assertExportShapeGuard(root)).toThrow(
    /changed geometry without a ZUI source mapping.*before \{"x":0,"y":0,"width":24,"height":24\}.*after \{"x":2,"y":0,"width":24,"height":24\}/,
  );
});

it('keeps a generated icon reversible when its ancestor layout translates', () => {
  const { root, path, translateAsset } = guardedIcon();
  translateAsset(10, 20);
  path.content = 'M 10 20 L 11 21';
  expect(() => assertExportShapeGuard(root)).not.toThrow();
});

it('compares cubic control points in the translated icon coordinate space', () => {
  const { root, path, translateAsset } = guardedIcon();
  path.content = 'M 0 0 C 1 2 3 4 5 6 Z';
  writeExportShapeGuard(root);
  translateAsset(10, -20);
  path.content = 'M 10 -20 C 11 -18 13 -16 15 -14 Z';
  expect(() => assertExportShapeGuard(root)).not.toThrow();
});

it('uses generated auxiliary ownership for field paths regardless of layer name', () => {
  const { root, path, translateAsset } = guardedIcon();
  path.parent!.name = 'Field search';
  writeExportShapeGuard(root);
  translateAsset(0, 2);
  path.content = 'M 0 2 L 1 3';
  expect(() => assertExportShapeGuard(root)).not.toThrow();
});

it('keeps a directly generated auxiliary path reversible after layout translation', () => {
  const { root, path, setGenerated, translateAsset } = guardedIcon();
  setGenerated(false);
  path.getSharedPluginData = (namespace, key) =>
    namespace === ZUI_METADATA_NAMESPACE && key === ZUI_METADATA_ROLE
      ? ZUI_ROLE_AUXILIARY
      : '';
  writeExportShapeGuard(root);
  translateAsset(10, 20);
  path.content = 'M 10 20 L 11 21';
  expect(() => assertExportShapeGuard(root)).not.toThrow();
  path.content = 'M 10 20 L 11 22';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('rejects a path edit even when its ancestor layout also translates', () => {
  const { root, path, translateAsset } = guardedIcon();
  translateAsset(10, 20);
  path.content = 'M 10 20 L 11 22';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('keeps world-coordinate changes guarded for paths without generated ownership', () => {
  const { root, path, setGenerated, translateAsset } = guardedIcon();
  setGenerated(false);
  writeExportShapeGuard(root);
  translateAsset(10, 20);
  path.content = 'M 10 20 L 11 21';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('requires re-import when an asset has a world-coordinate guard baseline', () => {
  const { root } = guardedIcon();
  const stored = JSON.parse(
    root.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard'),
  );
  root.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'export-shapes-guard',
    JSON.stringify(stored.shapes ?? stored),
  );
  expect(() => assertExportShapeGuard(root)).toThrow(/re-import/);
});
