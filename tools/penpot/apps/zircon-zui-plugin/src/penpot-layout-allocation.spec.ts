import { createPenpotBridgeAsset } from './bridge/penpot-asset';
import { parseZuiDocument, zuiNodes } from './bridge/zui-document';
import { reconcileZuiDocument } from './bridge/penpot-reconcile';
import { describe, expect, it } from 'vitest';
import type { Board } from '@penpot/plugin-types';
import { authoredGeometry } from './penpot-capture-validation';
import {
  allocateNativeLayoutSize,
  allocatedAuthoredGeometry,
} from './penpot-layout-allocation';
function board(): Board {
  const data = new Map<string, string>();
  return {
    width: 960,
    height: 313,
    getSharedPluginData: (_: string, k: string) => data.get(k) ?? '',
    setSharedPluginData: (_: string, k: string, v: string) => {
      data.set(k, v);
    },
    resize(this: { width: number; height: number }, w: number, h: number) {
      this.width = w;
      this.height = h;
    },
  } as unknown as Board;
}
const baseline = { x: 0, y: 69, width: 960, height: 313 };
const captured = (b: Board) =>
  allocatedAuthoredGeometry(
    b,
    authoredGeometry(
      { x: 0, y: 70, width: b.width, height: b.height },
      baseline,
      true,
      'fix',
      'fix',
    ),
    baseline,
  );
describe('derived native allocation stays separate from authored dimensions', () => {
  it('excludes a gap-driven allocation change from source geometry', () => {
    const b = board();
    allocateNativeLayoutSize(b, 960, 313);
    allocateNativeLayoutSize(b, 960, 310);
    expect(b.height).toBe(310);
    expect(captured(b)).toEqual(baseline);
  });
  it('retains manual dimensions independently and restores an edited height', () => {
    const b = board();
    allocateNativeLayoutSize(b, 960, 313);
    b.resize(840, 500);
    expect(captured(b)).toEqual({ ...baseline, width: 840, height: 500 });
    allocateNativeLayoutSize(b, 960, 310);
    expect(captured(b)).toEqual({ ...baseline, width: 840, height: 500 });
    allocateNativeLayoutSize(b, 960, 300);
    expect(captured(b).height).toBe(500);
    b.resize(960, 313);
    expect(captured(b).height).toBe(313);
    allocateNativeLayoutSize(b, 960, 300);
    expect(captured(b)).toEqual({ ...baseline, width: 840 });
  });
  it('captures current size until an initial baseline exists', () => {
    const b = board();
    allocateNativeLayoutSize(b, 960, 310);
    const current = { ...baseline, width: b.width, height: b.height };
    expect(allocatedAuthoredGeometry(b, current)).toEqual(current);
  });
  it('materializes the current allocation when a child leaves auto layout', () => {
    const b = board();
    allocateNativeLayoutSize(b, 960, 310);
    expect(
      allocatedAuthoredGeometry(
        b,
        { ...baseline, height: 310 },
        baseline,
        false,
      ).height,
    ).toBe(310);
  });
  it('preserves actual geometry after changing the parent', () => {
    const b = board();
    allocateNativeLayoutSize(b, 960, 310);
    Object.assign(b, { parent: { id: 'new-parent' } });
    expect(captured(b).height).toBe(310);
  });
  it('excludes intermediate native geometry that was never captured as an authored edit', () => {
    const b = board();
    allocateNativeLayoutSize(b, 960, 313);
    b.resize(940, 320);
    allocateNativeLayoutSize(b, 960, 310);
    expect(captured(b)).toEqual(baseline);
  });
});

describe('anchored allocations retain source constraints', () => {
  it('keeps a gap-driven Overlay child height separate from authored dimensions', () => {
    const b = board();
    Object.assign(b, { parent: { id: 'overlay-parent' } });
    allocateNativeLayoutSize(b, 960, 310, 'anchored');
    expect(
      allocatedAuthoredGeometry(
        b,
        { ...baseline, height: 310 },
        baseline,
        false,
      ),
    ).toEqual(baseline);
  });

  it('keeps Stretch in the exported source after a free child allocation changes', () => {
    const { document } = parseZuiDocument(`
[asset]
kind = "view"
id = "anchored-stretch"
version = 2
[root]
node = "root"
[nodes.root]
component = "Overlay"
layout = { container = { kind = "Overlay" }, width = { preferred = 960.0 }, height = { preferred = 313.0 } }
children = [{ node = "workspace" }]
[nodes.workspace]
component = "Container"
layout = { width = { stretch = "Stretch" }, height = { stretch = "Stretch" } }
`);
    const asset = createPenpotBridgeAsset(document, 'anchored-stretch.zui');
    const workspace = asset.snapshot.shapes.find(
      ({ nodeId }) => nodeId === 'workspace',
    )!;
    const sourceGeometry = workspace.baseline.geometry;
    expect(sourceGeometry.height).toBeGreaterThan(3);
    const b = board();
    Object.assign(b, { parent: { id: 'overlay-parent' } });
    allocateNativeLayoutSize(
      b,
      sourceGeometry.width,
      sourceGeometry.height - 3,
      'anchored',
    );
    workspace.current.geometry = allocatedAuthoredGeometry(
      b,
      { ...sourceGeometry, height: b.height },
      sourceGeometry,
      false,
    );
    const result = reconcileZuiDocument(document, asset.snapshot);
    expect(result.changes).toEqual([]);
    expect(zuiNodes(result.document)['workspace'].layout?.['height']).toEqual({
      stretch: 'Stretch',
    });
  });

  it('retains a manual anchored height across subsequent native refreshes', () => {
    const b = board();
    Object.assign(b, { parent: { id: 'overlay-parent' } });
    allocateNativeLayoutSize(b, 960, 310, 'anchored');
    b.resize(960, 500);
    expect(
      allocatedAuthoredGeometry(
        b,
        { ...baseline, height: 500 },
        baseline,
        false,
      ).height,
    ).toBe(500);
    allocateNativeLayoutSize(b, 960, 300, 'anchored');
    expect(
      allocatedAuthoredGeometry(
        b,
        { ...baseline, height: 300 },
        baseline,
        false,
      ).height,
    ).toBe(500);
  });
});

describe('allocation ownership follows the current parent layout', () => {
  it('materializes actual dimensions when an anchored child joins auto layout', () => {
    const b = board();
    Object.assign(b, { parent: { id: 'overlay-parent' } });
    allocateNativeLayoutSize(b, 960, 310, 'anchored');
    expect(
      allocatedAuthoredGeometry(b, { ...baseline, height: 310 }, baseline, true)
        .height,
    ).toBe(310);
  });

  it('uses actual dimensions after an anchored child is reparented', () => {
    const b = board();
    Object.assign(b, { parent: { id: 'overlay-parent' } });
    allocateNativeLayoutSize(b, 960, 310, 'anchored');
    Object.assign(b, { parent: { id: 'another-overlay' } });
    expect(
      allocatedAuthoredGeometry(
        b,
        { ...baseline, height: 310 },
        baseline,
        false,
      ).height,
    ).toBe(310);
  });

  it('captures a designer restore of an anchored height', () => {
    const b = board();
    Object.assign(b, { parent: { id: 'overlay-parent' } });
    allocateNativeLayoutSize(b, 960, 310, 'anchored');
    b.resize(960, 500);
    allocatedAuthoredGeometry(b, { ...baseline, height: 500 }, baseline, false);
    allocateNativeLayoutSize(b, 960, 300, 'anchored');
    b.resize(960, 313);
    expect(allocatedAuthoredGeometry(b, baseline, baseline, false).height).toBe(
      313,
    );
    allocateNativeLayoutSize(b, 960, 300, 'anchored');
    expect(
      allocatedAuthoredGeometry(
        b,
        { ...baseline, height: 300 },
        baseline,
        false,
      ).height,
    ).toBe(313);
  });
});

describe('authored dimensions follow allocation ownership changes', () => {
  it.each([
    ['flow', 'anchored', 'original-parent'],
    ['anchored', 'flow', 'original-parent'],
    ['flow', 'flow', 'new-parent'],
    ['anchored', 'anchored', 'new-parent'],
  ] as const)(
    'preserves the materialized geometry after %s to %s in %s',
    (initialKind, nextKind, parentId) => {
      const b = board();
      Object.assign(b, { parent: { id: 'original-parent' } });
      allocateNativeLayoutSize(b, 960, 313, initialKind);
      b.resize(840, 500);
      expect(
        allocatedAuthoredGeometry(
          b,
          { ...baseline, width: 840, height: 500 },
          baseline,
          initialKind === 'flow',
        ),
      ).toEqual({ ...baseline, width: 840, height: 500 });
      allocateNativeLayoutSize(b, 960, 310, initialKind);
      Object.assign(b, { parent: { id: parentId } });
      const materialized = { ...baseline, width: b.width, height: b.height };
      expect(
        allocatedAuthoredGeometry(
          b,
          materialized,
          baseline,
          nextKind === 'flow',
        ),
      ).toEqual(materialized);
      allocateNativeLayoutSize(b, 930, 300, nextKind);
      expect(
        allocatedAuthoredGeometry(
          b,
          { ...baseline, width: b.width, height: b.height },
          materialized,
          nextKind === 'flow',
        ),
      ).toEqual(materialized);
      allocateNativeLayoutSize(b, 920, 290, nextKind);
      expect(
        allocatedAuthoredGeometry(
          b,
          { ...baseline, width: b.width, height: b.height },
          materialized,
          nextKind === 'flow',
        ),
      ).toEqual(materialized);
    },
  );
});
