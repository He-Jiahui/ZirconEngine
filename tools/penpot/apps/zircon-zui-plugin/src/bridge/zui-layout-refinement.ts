import type { ZuiDocument, ZuiTable } from './zui-document';
import { prefabRoleForNode, resolveDesignNumber } from './zui-prefab-system';

export function refineLayoutPresentation(
  document: ZuiDocument,
  changes: Set<string>,
): void {
  const rootId =
    document.root?.node ?? Object.values(document.components ?? {})[0]?.root;
  const root = rootId && document.nodes?.[rootId];
  const workspace =
    document.asset.kind === 'component' &&
    /workspace\.zui$/.test(document.asset.id);
  if (
    root &&
    document.asset.kind === 'component' &&
    !workspace &&
    (root.children?.length ?? 0) >= 5 &&
    !/horizontal/i.test(
      String(record(root.layout?.['container'])['kind'] ?? root.component),
    )
  ) {
    root.layout ??= {};
    const width = record(root.layout['width']);
    if (width['preferred'] === undefined && width['max'] === undefined) {
      root.layout['width'] = { ...width, preferred: 480 };
      changes.add('establish-component-preview-width');
    }
  }
  if (
    root &&
    (document.asset.kind === 'view' || workspace) &&
    Object.keys(document.nodes ?? {}).length >= 20
  ) {
    root.layout ??= {};
    for (const [axis, minimum] of [
      ['width', 1280],
      ['height', 800],
    ] as const) {
      const dimension = record(root.layout[axis]);
      if (
        dimension['preferred'] === undefined &&
        dimension['max'] === undefined
      ) {
        root.layout[axis] = { ...dimension, preferred: minimum };
        changes.add('establish-editor-viewport');
      }
    }
    if (document.asset.id.includes('/ui/editor/'))
      root.layout['padding'] ??= { left: 16, right: 16, top: 16, bottom: 16 };
  }
  for (const [id, node] of Object.entries(document.nodes ?? {})) {
    const role = prefabRoleForNode(node, id, false);
    if (
      node.props &&
      [
        'label',
        'caption',
        'title',
        'chip',
        'badge',
        'button',
        'tab',
        'field',
        'row',
      ].includes(role)
    ) {
      const fontSize = resolveDesignNumber(document, node.props['font_size']);
      if (fontSize !== null && fontSize > 0 && fontSize < 11) {
        node.props['font_size'] = 11;
        const height = record(node.layout?.['height']);
        for (const key of ['min', 'preferred', 'max']) {
          const value = resolveDesignNumber(document, height[key]);
          if (value !== null && value < 18) height[key] = 18;
        }
        const width = record(node.layout?.['width']);
        const text = node.props['text'] ?? node.props['label'];
        if (typeof text === 'string' && width['stretch'] === 'Fixed') {
          const required = Math.ceil(text.length * 11 * 0.54 + 12);
          for (const key of ['min', 'preferred', 'max']) {
            const value = resolveDesignNumber(document, width[key]);
            if (value !== null && value < required) width[key] = required;
          }
        }
        changes.add('readable-control-typography');
      }
    }
  }
  const nodes = document.nodes ?? {};
  if (/\/(asset_browser|assets_activity)\.zui$/.test(document.asset.id)) {
    const rows: Record<string, string[]> = {
      content_asset_table_header: ['Name', 'Type', 'Size', 'Rev'],
      content_asset_row_01: ['Host', 'UI', '12K', 'r42'],
      content_asset_row_02: ['Base', 'Style', '8K', 'r41'],
      content_asset_row_03: ['Folder', 'Tex', '4K', 'r40'],
      content_asset_row_04: ['A11y', 'Widget', '16K', 'r39'],
    };
    for (const [id, options] of Object.entries(rows)) {
      if (nodes[id]) nodes[id].props = { ...nodes[id].props, options };
    }
    changes.add('align-authored-asset-table-cells');
  }
  // The utility region displays one selected page. These are runtime-switched peers.
  const utility = nodes['utility_content_panel'];
  if (
    utility &&
    /\/(asset_browser|assets_activity)\.zui$/.test(document.asset.id)
  ) {
    for (const [index, child] of (utility.children ?? []).entries()) {
      const node = nodes[child.node];
      if (node)
        node.props = {
          ...node.props,
          visibility: index === 0 ? 'visible' : 'collapsed',
        };
    }
    changes.add('isolate-selected-utility-tab');
  }
}

function record(value: unknown): ZuiTable {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : {};
}
