import { describe, expect, it } from 'vitest';
import type { Board } from '@penpot/plugin-types';
import {
  auditRenderedAssetBoard,
  refreshAssetTextLayout,
  scrollAxisForNode,
} from './penpot-asset-renderer';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import { projectZuiDocument } from './bridge/penpot-projection';
import type { ZuiDocument } from './bridge/zui-document';

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
    setSharedPluginData: (namespace: string, key: string, value: string) => {
      if (namespace === ZUI_METADATA_NAMESPACE) metadata[key] = value;
    },
    resize(width: number, height: number) {
      this.width = width;
      this.height = height;
    },
  } as unknown as Board;
}

describe('initial TreeRow layout refresh', () => {
  function initialRow(visibility: 'collapsed' | 'visible') {
    const asset = measuredBoard('asset', { role: 'asset', 'node-id': 'root' });
    const row = measuredBoard('row', { role: 'node', 'node-id': 'row' });
    Object.assign(asset, { children: [row] });
    Object.assign(row, { parent: asset, hidden: true });
    const document: ZuiDocument = {
      asset: { kind: 'view', id: 'initial-tree-row', version: 2 },
      root: { node: 'root' },
      penpot_host_theme_source: 'editor_tokens.zui',
      nodes: {
        root: { component: 'Overlay', children: [{ node: 'row' }] },
        row: { component: 'TreeRow', props: { text: '', visibility } },
      },
    };
    const projection = projectZuiDocument(document);
    return { row, refresh: () => refreshAssetTextLayout(asset, document, projection) };
  }

  it('keeps collapsed source rows unpainted during initial reflow', () => {
    const { row, refresh } = initialRow('collapsed');
    expect(refresh).not.toThrow();
    expect(row.children).toHaveLength(0);
    expect(row.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'tree-row-authoring-painter')).toBe('');
  });

  it('rejects missing painter metadata for visible source rows even if the board is hidden', () => {
    const { refresh } = initialRow('visible');
    expect(refresh).toThrow('TreeRow has no original painter metadata; reimport it.');
  });
});

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
