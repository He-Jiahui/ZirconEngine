import { describe, expect, it } from 'vitest';
import type { Board } from '@penpot/plugin-types';
import {
  auditRenderedAssetBoard,
  scrollAxisForNode,
} from './penpot-asset-renderer';
import { ZUI_METADATA_NAMESPACE } from './metadata';

function measuredBoard(id: string, metadata: Record<string, string>): Board {
  return {
    id,
    name: id,
    type: 'board',
    x: 0,
    y: 0,
    width: 100,
    height: 100,
    hidden: false,
    clipContent: false,
    children: [],
    getSharedPluginData: (namespace: string, key: string) =>
      namespace === ZUI_METADATA_NAMESPACE ? (metadata[key] ?? '') : '',
  } as unknown as Board;
}

describe('Penpot rendered source identity', () => {
  it('preserves owner and parent tuples for repeated prefab instances', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const parent = measuredBoard('parent', {
      role: 'node',
      'node-id': 'host',
      sourcePath: 'host.zui',
      sourceNodeId: 'host_root',
      instancePath: '[]',
    });
    const instances = ['first', 'second'].map((callSite) => {
      const instancePath = JSON.stringify([
        { sourcePath: 'host.zui', sourceNodeId: callSite },
      ]);
      return measuredBoard(callSite, {
        role: 'node',
        'node-id': `${callSite}/button`,
        sourcePath: 'button.zui',
        sourceNodeId: 'button',
        controlId: 'SharedButton',
        instancePath,
      });
    });
    Object.assign(asset, { children: [parent] });
    Object.assign(parent, { parent: asset, children: instances });
    for (const instance of instances) Object.assign(instance, { parent });
    const measured = auditRenderedAssetBoard(asset).semanticNodes!;
    for (const [index, callSite] of ['first', 'second'].entries()) {
      expect(measured[index + 1]).toMatchObject({
        sourcePath: 'button.zui',
        sourceNodeId: 'button',
        controlId: 'SharedButton',
        instancePath: JSON.stringify([
          { sourcePath: 'host.zui', sourceNodeId: callSite },
        ]),
        parentSourcePath: 'host.zui',
        parentSourceNodeId: 'host_root',
        parentInstancePath: '[]',
      });
    }
  });

  it('leaves missing owner metadata incomplete for strict acceptance', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const child = measuredBoard('child', { role: 'node', 'node-id': 'child' });
    Object.assign(asset, { children: [child] });
    Object.assign(child, { parent: asset });
    expect(auditRenderedAssetBoard(asset).semanticNodes![0]).toMatchObject({
      sourcePath: null,
      sourceNodeId: null,
      controlId: null,
      instancePath: '',
      parentSourcePath: null,
      parentSourceNodeId: null,
      parentInstancePath: '',
    });
  });
});

describe('Penpot asset scroll semantics', () => {
  it('checks each nested viewport against its own scroll axes', () => {
    const asset = measuredBoard('asset', { role: 'asset' });
    const horizontalViewport = measuredBoard('outer', {
      role: 'node',
      'node-id': 'outer',
      'container-kind': 'scroll',
      'scroll-axis': 'horizontal',
    });
    const verticalViewport = measuredBoard('inner', {
      role: 'node',
      'node-id': 'inner',
      'container-kind': 'scroll',
      'scroll-axis': 'vertical',
    });
    const content = measuredBoard('content', {
      role: 'node',
      'node-id': 'content',
    });
    Object.assign(asset, { children: [horizontalViewport] });
    Object.assign(horizontalViewport, {
      parent: asset,
      children: [verticalViewport],
    });
    Object.assign(verticalViewport, {
      parent: horizontalViewport,
      children: [content],
    });
    Object.assign(content, {
      parent: verticalViewport,
      width: 120,
      height: 300,
    });

    expect(auditRenderedAssetBoard(asset).overflowNodeIds).toEqual(['content']);
  });

  it('preserves a ScrollBox axis when its child layout uses a VerticalBox', () => {
    expect(
      scrollAxisForNode({
        component: 'ScrollBox',
        props: {
          scroll_axis: 'vertical',
          scroll_x: false,
          scroll_y: true,
        },
        layout: { container: { kind: 'VerticalBox', gap: 10 } },
      }),
    ).toBe('vertical');
  });

  it('combines explicit horizontal and vertical scroll flags', () => {
    expect(
      scrollAxisForNode({
        component: 'ScrollableBox',
        props: { scroll_x: true, scroll_y: true },
        layout: { container: { kind: 'HorizontalBox' } },
      }),
    ).toBe('both');
  });

  it('keeps non-scroll layout containers out of scroll review', () => {
    expect(
      scrollAxisForNode({
        component: 'VerticalBox',
        layout: { container: { kind: 'VerticalBox' } },
      }),
    ).toBeNull();
  });
});
