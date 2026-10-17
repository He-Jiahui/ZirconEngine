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

it('keeps the actual R74 Float32 world path reversible across layout frames', () => {
  const { root, path, translateAsset } = guardedIcon();
  Object.assign(path.parent!, {
    x: 563.0001610497911,
    y: 390.99991720935003,
    width: 17.99999678134816,
    height: 17.999997854231083,
  });
  Object.assign(path, {
    x: 566.7501603792385,
    y: 394.7499167623149,
    width: 10.499998122453235,
    height: 10.499998748301493,
    content:
      'M566.7500610351562,405.2499084472656 L566.7500610351562,394.7499084472656 M566.7500610351562,405.2499084472656 L577.2500610351562,405.2499084472656',
  });
  writeExportShapeGuard(root);

  translateAsset(
    562.9999245704907 - 563.0001610497911,
    392.9997662307896 - 390.99991720935003,
  );
  Object.assign(path.parent!, {
    width: 17.9999849796302,
    height: 17.999998927112244,
  });
  Object.assign(path, {
    x: 566.7499214412469,
    y: 396.7497660072714,
    width: 10.4999912381179,
    height: 10.499999374148786,
    content:
      'M566.7496948242188,407.249755859375 L566.7496948242188,396.749755859375 M566.7496948242188,407.249755859375 L577.2496948242188,407.249755859375',
  });

  expect(() => assertExportShapeGuard(root)).not.toThrow();
});

it('rejects content-only translation at the geometry guard precision', () => {
  const { root, path } = guardedIcon();
  for (const content of ['M 0.01 -0.01 L 1.01 0.99', 'M 1 2 L 2 3']) {
    path.content = content;
    expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
  }
});

it('still rejects a cubic control point edit above the numeric tolerance', () => {
  const { root, path } = guardedIcon();
  path.content = 'M 0 0 C 1 2 3 4 5 6 Z';
  writeExportShapeGuard(root);
  path.content = 'M 0 0 C 1.0002 2 3 4 5 6 Z';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('requires re-import for the previous version 5 coordinate baseline', () => {
  const { root } = guardedIcon();
  const stored = JSON.parse(
    root.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'export-shapes-guard'),
  );
  stored.version = 5;
  root.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'export-shapes-guard',
    JSON.stringify(stored),
  );
  expect(() => assertExportShapeGuard(root)).toThrow(/re-import/);
});

it('compares unsupported arc commands exactly even for tiny numeric changes', () => {
  const { root, path } = guardedIcon();
  path.content = 'M 0 0 A 1 1 0 0 0 1 1';
  writeExportShapeGuard(root);
  expect(() => assertExportShapeGuard(root)).not.toThrow();
  path.content = 'M 0 0 A 1 1 0 0 0 1.000030517578125 1';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('compares empty path content exactly', () => {
  const { root, path } = guardedIcon();
  path.content = '';
  writeExportShapeGuard(root);
  expect(() => assertExportShapeGuard(root)).not.toThrow();
  path.content = ' ';
  expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
});

it('compares non-finite path coordinates exactly', () => {
  for (const [before, after] of [
    ['M 0 0 L 1e999 1', 'M 0 0 L 2e999 1'],
    ['M NaN 0 L 1 1', 'M NaN 0 L 1.000030517578125 1'],
  ]) {
    const { root, path } = guardedIcon();
    path.content = before!;
    writeExportShapeGuard(root);
    expect(() => assertExportShapeGuard(root)).not.toThrow();
    path.content = after!;
    expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
  }
});

it('compares unsupported segment arities exactly', () => {
  for (const [before, after] of [
    ['M 0 0 L 1 1 2 2', 'M 0 0 L 1 1 2.000030517578125 2'],
    ['M 0 0 C 1 2 3 4', 'M 0 0 C 1.000030517578125 2 3 4'],
    ['L 0 0 L 1 1', 'L 0 0 L 1.000030517578125 1'],
  ]) {
    const { root, path } = guardedIcon();
    path.content = before!;
    writeExportShapeGuard(root);
    expect(() => assertExportShapeGuard(root)).not.toThrow();
    path.content = after!;
    expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
  }
});

it('keeps content exact when the path origin is non-finite', () => {
  for (const x of [Number.NaN, Number.POSITIVE_INFINITY]) {
    const { root, path } = guardedIcon();
    path.x = x;
    writeExportShapeGuard(root);
    expect(() => assertExportShapeGuard(root)).not.toThrow();
    path.content = 'M 0 0 L 1.000030517578125 1';
    expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
  }
});

it('compares malformed comma separators exactly', () => {
  for (const [before, after] of [
    ['M,0,0 L1 1', 'M,0,0 L1.000030517578125 1'],
    ['M0 0, L1 1', 'M0 0, L1.000030517578125 1'],
    ['M0,,0 L1 1', 'M0,,0 L1.000030517578125 1'],
    ['M0 0 L1,1,', 'M0 0 L1.000030517578125,1,'],
  ] as const) {
    const { root, path } = guardedIcon();
    path.content = before;
    writeExportShapeGuard(root);
    expect(() => assertExportShapeGuard(root)).not.toThrow();
    path.content = after;
    expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
  }
});

it('keeps arithmetic overflow during coordinate normalization exact', () => {
  for (const [x, before, after] of [
    [1e308, 'M 0 0 L 1 1', 'M 0 0 L 1.000030517578125 1'],
    [
      -1e308,
      'M -1e308 0 L 1e308 1',
      'M -1e308 0 L 1e308 1.000030517578125',
    ],
  ] as const) {
    const { root, path } = guardedIcon();
    path.x = x;
    path.content = before;
    writeExportShapeGuard(root);
    expect(() => assertExportShapeGuard(root)).not.toThrow();
    path.content = after;
    expect(() => assertExportShapeGuard(root)).toThrow(/changed content/);
  }
});
