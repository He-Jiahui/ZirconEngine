import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Board, Shape } from '@penpot/plugin-types';
import {
  createAuthoringLayoutRefresh,
  type AuthoringShapeEvents,
} from './penpot-authoring-layout';
import { applyWeightedFlexLayout } from './penpot-weighted-flex';
import type { ProjectionContainer } from './bridge/penpot-projection-model';
import type { ZuiDocument } from './bridge/zui-document';
function setup() {
  const data = new Map<string, string>();
  const child = {
    id: 'main',
    type: 'board',
    width: 960,
    height: 637,
    hidden: false,
    children: [],
    layoutChild: {
      absolute: false,
      horizontalSizing: 'fix',
      verticalSizing: 'fix',
    },
    getSharedPluginData: (_: string, k: string) =>
      data.get(k) ?? (k === 'node-id' ? 'main' : k === 'role' ? 'node' : ''),
    setSharedPluginData: (_: string, k: string, v: string) => {
      data.set(k, v);
    },
    resize(this: { width: number; height: number }, w: number, h: number) {
      this.width = w;
      this.height = h;
    },
  } as unknown as Board;
  const root = {
    id: 'asset',
    type: 'board',
    width: 960,
    height: 640,
    children: [child],
    getSharedPluginData: (_: string, k: string) =>
      k === 'role' ? 'asset' : k === 'node-id' ? 'root' : '',
    parent: null,
  } as unknown as Board;
  Object.assign(child, { parent: root });
  const document: ZuiDocument = {
    asset: { kind: 'view', id: 'test', version: 2 },
    nodes: {
      root: {
        component: 'VerticalBox',
        children: [
          {
            node: 'main',
            slot: { layout: { linear_size: { rule: 'Stretch' } } },
          },
          {
            node: 'tail',
            slot: {
              layout: { linear_size: { rule: 'Auto', shrink_value: 0 } },
            },
          },
        ],
      },
      main: { component: 'Overlay' },
      tail: { component: 'Space' },
    },
  };
  const tailData = new Map<string, string>();
  const status = {
    ...child,
    id: 'tail',
    height: 3,
    getSharedPluginData: (_: string, k: string) =>
      tailData.get(k) ??
      (k === 'node-id' ? 'tail' : k === 'role' ? 'node' : ''),
    setSharedPluginData: (_: string, k: string, v: string) => {
      tailData.set(k, v);
    },
  } as unknown as Board;
  Object.assign(root, { children: [child, status] });
  let gap = 0,
    blocked = false,
    release: (() => void) | undefined;
  const errors: unknown[] = [];
  const listeners = new Map<
    symbol,
    { id: string; callback: (shape: Shape) => void }
  >();
  const events: AuthoringShapeEvents = {
    selection: [child],
    on: (_type, callback, props) => {
      const id = Symbol();
      listeners.set(id, { id: props.shapeId, callback });
      return id;
    },
    off: (id) => {
      listeners.delete(id);
    },
  };
  const emit = () => {
    for (const listener of listeners.values())
      if (listener.id === child.id) listener.callback(child);
  };
  const refresh = createAuthoringLayoutRefresh(events, {
    asset: () => root,
    draft: () => {
      if (blocked) throw new Error('Unmapped shape edit');
      const current = gap;
      return {
        key: String(current),
        refresh: async () => {
          if (release !== undefined)
            await new Promise<void>((resolve) => {
              release = resolve;
            });
          const flow: ProjectionContainer = {
            kind: 'flex',
            direction: 'column',
            wrap: false,
            gap: current,
            rowGap: current,
            columnGap: current,
            columns: 1,
            rows: 1,
            padding: { left: 0, right: 0, top: 0, bottom: 0 },
            alignItems: 'stretch',
            justifyContent: 'start',
            clip: false,
          };
          applyWeightedFlexLayout(
            root,
            document,
            flow,
            new Map([
              ['main', { width: 960, height: 0 }],
              ['tail', { width: 960, height: 3 }],
            ]),
          );
          emit();
        },
      };
    },
    onError: (error) => errors.push(error),
  });
  return {
    root,
    child,
    refresh,
    emit,
    errors,
    setGap: (value: number) => {
      gap = value;
    },
    block: () => {
      blocked = true;
    },
    unblock: () => {
      blocked = false;
    },
    hold: () => {
      release = () => {};
    },
    release: () => {
      const resolve = release;
      release = undefined;
      resolve?.();
    },
  };
}
afterEach(() => vi.useRealTimers());
describe('live native layout refresh', () => {
  it('reflows a normal shape edit and restores spacing while self events remain bounded', async () => {
    vi.useFakeTimers();
    const s = setup();
    s.refresh.bindSelection();
    s.setGap(4);
    s.emit();
    await vi.advanceTimersByTimeAsync(160);
    await s.refresh.flush(s.root);
    expect(s.child.height).toBe(633);
    await vi.advanceTimersByTimeAsync(500);
    expect(vi.getTimerCount()).toBe(0);
    s.setGap(3);
    s.emit();
    await vi.advanceTimersByTimeAsync(160);
    await s.refresh.flush(s.root);
    expect(s.child.height).toBe(634);
    s.refresh.dispose();
  });
  it('flushes edits made without a selection callback before capture', async () => {
    const s = setup();
    s.setGap(4);
    await s.refresh.flush(s.root);
    expect(s.child.height).toBe(633);
    s.setGap(3);
    await s.refresh.flush(s.root);
    expect(s.child.height).toBe(634);
    s.refresh.dispose();
  });
  it('serializes the next authored edit behind a pending native refresh', async () => {
    const s = setup();
    s.setGap(4);
    s.hold();
    const first = s.refresh.flush(s.root);
    await Promise.resolve();
    await Promise.resolve();
    s.setGap(5);
    const second = s.refresh.flush(s.root);
    s.release();
    await first;
    await second;
    expect(s.child.height).toBe(632);
    s.refresh.dispose();
  });
  it('rejects unsupported edits before resizing and allows a later repaired draft', async () => {
    const s = setup();
    s.block();
    await expect(s.refresh.flush(s.root)).rejects.toThrow(
      'Unmapped shape edit',
    );
    expect(s.child.height).toBe(637);
    s.unblock();
    s.setGap(4);
    await s.refresh.flush(s.root);
    expect(s.child.height).toBe(633);
    s.refresh.dispose();
  });
  it('removes old selection observers while import or review owns the board', async () => {
    vi.useFakeTimers();
    const s = setup();
    s.refresh.bindSelection();
    s.setGap(4);
    s.emit();
    const resume = await s.refresh.suspend();
    await vi.advanceTimersByTimeAsync(500);
    expect(s.child.height).toBe(637);
    resume();
    s.emit();
    await vi.advanceTimersByTimeAsync(160);
    await s.refresh.flush(s.root);
    expect(s.child.height).toBe(633);
    s.refresh.dispose();
  });
});
