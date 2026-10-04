import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  layoutSettlementOptionsForNodes,
  layoutAuditStabilitySnapshot,
  tableMeasurementsReady,
  settleAfterLayoutRefresh,
  waitForStableLayout,
} from './penpot-layout-settlement';

afterEach(() => vi.useRealTimers());

describe('Penpot layout settlement', () => {
  it('budgets two complete layout samples for large native boards without unbounded waits', () => {
    expect(layoutSettlementOptionsForNodes(34).timeoutMs).toBe(60_000);
    expect(layoutSettlementOptionsForNodes(355).timeoutMs).toBe(248_500);
    expect(layoutSettlementOptionsForNodes(1_429).timeoutMs).toBe(360_000);
  });

  it('reflows late text measurements and waits for the updated layout', async () => {
    vi.useFakeTimers();
    let measuredWidth = 1;
    let cellWidth = 1;
    let complete = false;
    setTimeout(() => (measuredWidth = 60), 250);
    const pending = waitForStableLayout(
      () => (cellWidth = measuredWidth),
      () => ({ measuredWidth, cellWidth }),
    ).then(() => (complete = true));
    await vi.advanceTimersByTimeAsync(600);
    expect(complete).toBe(false);
    expect(cellWidth).toBe(60);
    await vi.advanceTimersByTimeAsync(400);
    await pending;
    expect(complete).toBe(true);
  });

  it('fails explicitly when geometry never stabilizes', async () => {
    vi.useFakeTimers();
    let width = 1;
    const pending = waitForStableLayout(
      () => width++,
      () => ({ width }),
      { timeoutMs: 1_000 },
    );
    const rejection = expect(pending).rejects.toThrow('did not settle');
    await vi.advanceTimersByTimeAsync(1_000);
    await rejection;
  });

  it('ignores subpixel measurement jitter while retaining meaningful changes', async () => {
    vi.useFakeTimers();
    let width = 100;
    let tick = 0;
    const pending = waitForStableLayout(
      () => {
        tick += 1;
        width = 100 + (tick % 2 === 0 ? 0.0004 : -0.0004);
      },
      () => ({ width }),
      { timeoutMs: 1_000, numericPrecision: 3 },
    );
    await vi.advanceTimersByTimeAsync(1_000);
    await pending;
  });

  it('includes a concrete changed path when settlement times out', async () => {
    vi.useFakeTimers();
    let width = 1;
    const pending = waitForStableLayout(
      () => {
        width += 1;
      },
      () => ({ frame: { width } }),
      { timeoutMs: 500, intervalMs: 50, quietMs: 100 },
    );
    const rejection = expect(pending).rejects.toThrow(/frame\.width/);
    await vi.advanceTimersByTimeAsync(500);
    await rejection;
  });

  it('samples geometry between bounded layout refreshes', async () => {
    vi.useFakeTimers();
    let width = 1;
    let refreshes = 0;
    setTimeout(() => (width = 64), 250);

    const pending = settleAfterLayoutRefresh(
      () => {
        refreshes += 1;
      },
      () => ({ width }),
      { intervalMs: 100, quietMs: 100, timeoutMs: 1_000 },
    );

    await vi.advanceTimersByTimeAsync(1_000);
    await pending;
    expect(refreshes).toBe(2);
  });

  it('settles layout after a native content mutation between refreshes', async () => {
    vi.useFakeTimers();
    let y = 0;
    let mutations = 0;
    const pending = settleAfterLayoutRefresh(
      () => {},
      () => ({ y }),
      { intervalMs: 100, quietMs: 100, timeoutMs: 1_000 },
      async () => {
        mutations += 1;
        setTimeout(() => (y = 684), 100);
      },
    );

    await vi.advanceTimersByTimeAsync(1_000);
    await pending;
    expect(mutations).toBe(1);
    expect(y).toBe(684);
  });

  it('does not accept a quiet layout until its readiness predicate is true', async () => {
    vi.useFakeTimers();
    let positions = [0, 0];
    let complete = false;
    setTimeout(() => (positions = [0, 684]), 750);
    const pending = waitForStableLayout(
      () => {},
      () => ({ positions }),
      {
        intervalMs: 100,
        quietMs: 100,
        timeoutMs: 1_500,
        isReady: (snapshot) => new Set(snapshot.positions).size > 1,
      },
    ).then(() => (complete = true));

    await vi.advanceTimersByTimeAsync(600);
    expect(complete).toBe(false);
    await vi.advanceTimersByTimeAsync(900);
    await pending;
    expect(complete).toBe(true);
  });

  it('settles against semantic geometry instead of native component identity', () => {
    const base = {
      totalNodes: 1,
      checkedNodes: 1,
      overflowCount: 0,
      overflowNodeIds: [],
      overflowDetails: [],
      invalidGeometryCount: 0,
      invalidGeometryNodeIds: [],
      invalidGeometryDetails: [],
      assetBounds: { x: 0, y: 0, width: 320, height: 240 },
      semanticNodes: [
        {
          nodeId: 'root',
          shapeId: 'shape-one',
          parentNodeId: null,
          component: 'Label',
          visible: true,
          detached: false,
          nativeComponent: { id: 'native-one', source: 'source', copy: true },
          bounds: { x: 0, y: 0, width: 320, height: 24 },
          text: 'Caption',
          textParts: [
            {
              shapeId: 'text-one',
              text: 'Caption',
              displayText: 'Caption',
              bounds: { x: 8, y: 2, width: 80, height: 20 },
            },
          ],
        },
      ],
    } as const;
    const rematerialized = {
      ...base,
      semanticNodes: base.semanticNodes.map((node) => ({
        ...node,
        shapeId: 'shape-two',
        nativeComponent: { ...node.nativeComponent, id: 'native-two' },
        textParts: node.textParts.map((part) => ({
          ...part,
          shapeId: 'text-two',
        })),
      })),
    };

    expect(layoutAuditStabilitySnapshot(base)).toEqual(
      layoutAuditStabilitySnapshot(rematerialized),
    );
  });

  it('waits for shaped table font measurements before capturing an export baseline', () => {
    const incomplete = {
      totalNodes: 1,
      checkedNodes: 1,
      overflowCount: 0,
      overflowNodeIds: [],
      overflowDetails: [],
      invalidGeometryCount: 0,
      invalidGeometryNodeIds: [],
      invalidGeometryDetails: [],
      semanticNodes: [
        {
          nodeId: 'row',
          shapeId: 'row-shape',
          parentNodeId: null,
          component: 'Table',
          visible: true,
          detached: false,
          bounds: { x: 0, y: 0, width: 240, height: 28 },
          text: '',
          tableMeasurements: [
            { shapeId: 'measure-1', text: 'Name', width: 1, height: 1 },
          ],
        },
      ],
    };
    const shaped = {
      ...incomplete,
      semanticNodes: incomplete.semanticNodes.map((node) => ({
        ...node,
        tableMeasurements: [
          { shapeId: 'measure-2', text: 'Name', width: 37, height: 20 },
        ],
      })),
    };
    const before = layoutAuditStabilitySnapshot(incomplete);
    const after = layoutAuditStabilitySnapshot(shaped);
    expect(tableMeasurementsReady(before)).toBe(false);
    expect(tableMeasurementsReady(after)).toBe(true);
    expect(before).not.toEqual(after);
    expect(after).toEqual(
      layoutAuditStabilitySnapshot({
        ...shaped,
        semanticNodes: shaped.semanticNodes.map((node) => ({
          ...node,
          shapeId: 'another-row-shape',
        })),
      }),
    );
  });
});
