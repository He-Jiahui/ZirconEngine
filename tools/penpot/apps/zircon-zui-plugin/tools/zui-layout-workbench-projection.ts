import { sourceNode, sourceNodeByControl } from './zui-layout-source-control';
import {
  prepareInspectorPropertyRows,
  applyInspectorPropertyRows,
} from './zui-layout-inspector-properties';
import { validateWorkbenchPresentation } from './zui-layout-workbench-presentation';
import type { ZuiDocument, ZuiNode } from '../src/bridge/zui-document';

const WORKBENCH_SHELL_SOURCE =
  'zircon_editor/assets/ui/editor/host/workbench_shell.zui';
const SCENE_TREE_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui';
const INSPECTOR_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui';
const WORKBENCH_TAB_COMPONENT =
  'res://ui/editor/components/workbench/primitives/inputs/workbench_tab.zui#WorkbenchTab';

type JsonObject = Record<string, unknown>;
export type WorkbenchHierarchyRow = {
  id: string;
  parentId: string | null;
  depth: number;
  name: string;
  hasChildren: boolean;
};
type ProductRow = {
  id: string;
  title: string;
  selected?: boolean;
  [key: string]: unknown;
};

/** Apply the normal hierarchy search ancestry and per-entity disclosure state. */
export function projectWorkbenchHierarchyRows<
  Row extends WorkbenchHierarchyRow,
>(
  rows: readonly Row[],
  filterQuery: string,
  expandedIds: readonly string[],
): Row[] {
  const query = filterQuery.trim().toLowerCase();
  const parentIndexes: Array<number | undefined> = [];
  const ancestors: Array<{ index: number; depth: number }> = [];
  for (let index = 0; index < rows.length; index += 1) {
    const row = rows[index]!;
    while (
      ancestors.at(-1)?.depth !== undefined &&
      ancestors.at(-1)!.depth >= row.depth
    )
      ancestors.pop();
    parentIndexes.push(ancestors.at(-1)?.index);
    ancestors.push({ index, depth: row.depth });
  }

  const included = rows.map(
    (row) => query.length === 0 || row.name.toLowerCase().includes(query),
  );
  if (query.length > 0) {
    for (let index = rows.length - 1; index >= 0; index -= 1) {
      if (!included[index]) continue;
      const parentIndex = parentIndexes[index];
      if (parentIndex !== undefined) included[parentIndex] = true;
    }
  }

  const expanded = new Set(expandedIds);
  const visible = new Array<boolean>(rows.length).fill(false);
  const result: Row[] = [];
  for (let index = 0; index < rows.length; index += 1) {
    if (!included[index]) continue;
    const parentIndex = parentIndexes[index];
    const row = rows[index]!;
    visible[index] =
      query.length > 0 ||
      parentIndex === undefined ||
      (visible[parentIndex] && expanded.has(rows[parentIndex]!.id));
    if (visible[index]) result.push(row);
  }
  return result;
}

/** Add data-backed tab instances before import expansion. */
export function prepareWorkbenchPresentationStructure(
  document: ZuiDocument,
  value: Record<string, unknown>,
): void {
  validateWorkbenchPresentation(value);
  const pages = object(value['pages'], 'workbenchPresentation.pages');
  const pageItems = array(pages['items'], 'workbenchPresentation.pages.items');
  const pageStrip = sourceNode(
    document,
    WORKBENCH_SHELL_SOURCE,
    'host_page_strip',
  );
  replaceWithTabRows(
    document,
    pageStrip,
    'workbench_page',
    pageItems.map((item) => {
      const row = object(item, 'workbench page');
      return productRow(
        row,
        stringValue(row['id'], 'workbench page id') === pages['activeId'],
        'workbench page',
      );
    }),
    'mainPage',
  );

  const documents = object(
    value['documents'],
    'workbenchPresentation.documents',
  );
  const documentItems = array(
    documents['items'],
    'workbenchPresentation.documents.items',
  );
  const documentTabs = sourceNode(
    document,
    WORKBENCH_SHELL_SOURCE,
    'document_tabs',
  );
  replaceWithTabRows(
    document,
    documentTabs,
    'workbench_document',
    documentItems.map((item) => {
      const row = object(item, 'workbench document');
      return productRow(
        row,
        stringValue(row['id'], 'workbench document id') ===
          documents['activeId'],
        'workbench document',
      );
    }),
    'document',
  );

  const drawers = array(value['drawers'], 'workbenchPresentation.drawers');
  const grouped = new Map<string, JsonObject[]>();
  for (const drawerValue of drawers) {
    const drawer = object(drawerValue, 'workbench drawer');
    const target = drawerShell(String(drawer['slot']));
    grouped.set(target, [...(grouped.get(target) ?? []), drawer]);
  }
  for (const [targetId, entries] of grouped) {
    const visible = entries.filter(
      (entry) => entry['visible'] === true && entry['mode'] !== 'collapsed',
    );
    if (visible.length > 1)
      throw new Error(
        `WorkbenchShell cannot project multiple visible product drawers into ${targetId}`,
      );
    const drawer = visible[0] ?? entries[0]!;
    const shell = sourceNode(document, WORKBENCH_SHELL_SOURCE, targetId);
    const headerId = `${targetId.replace('_shell', '')}_header`;
    const header = sourceNode(document, WORKBENCH_SHELL_SOURCE, headerId);
    const tabs = array(
      drawer['tabs'],
      `workbench drawer ${String(drawer['slot'])}.tabs`,
    );
    replaceWithTabRows(
      document,
      header,
      `workbench_drawer_${String(drawer['slot'])}`,
      tabs.map((item) => {
        const row = object(item, 'workbench drawer tab');
        return productRow(
          row,
          stringValue(row['id'], 'workbench drawer tab id') ===
            drawer['activeTabId'],
          'workbench drawer tab',
        );
      }),
      `drawer:${String(drawer['slot'])}`,
    );
    shell['penpot_review_drawer'] = structuredClone(drawer);
  }
}

/** Bind factual product rows and values to the expanded authored workbench. */
export function applyWorkbenchPresentationProjection(
  document: ZuiDocument,
  value: Record<string, unknown>,
): void {
  validateWorkbenchPresentation(value);
  document['penpot_review_workbench_presentation'] = structuredClone(value);

  for (const node of Object.values(document.nodes ?? {})) {
    const generated = node['penpot_review_generated_product_row'];
    if (!isRecord(generated) || generated['kind'] === 'inspectorProperty')
      continue;
    const selected = generated['selected'] === true;
    node.props = {
      ...(node.props ?? {}),
      text: stringValue(generated['title'], 'generated product row title'),
      selected,
      checked: selected,
    };
  }

  const hierarchy = object(
    value['hierarchy'],
    'workbenchPresentation.hierarchy',
  );
  const rows = array(
    hierarchy['rows'],
    'workbenchPresentation.hierarchy.rows',
  ) as WorkbenchHierarchyRow[];
  const filterQuery = stringValue(
    hierarchy['filterQuery'],
    'workbenchPresentation.hierarchy.filterQuery',
  );
  const expandedIds = array(
    hierarchy['expandedIds'],
    'workbenchPresentation.hierarchy.expandedIds',
  ).map((id) =>
    stringValue(id, 'workbenchPresentation.hierarchy.expandedIds entry'),
  );
  const visibleRows = projectWorkbenchHierarchyRows(
    rows,
    filterQuery,
    expandedIds,
  );
  const visibleEntityIds = new Set(visibleRows.map((row) => row.id));
  const filterExpandedIds = new Set(
    filterQuery.trim().length === 0
      ? []
      : visibleRows
          .map((row) => row.parentId)
          .filter(
            (parentId): parentId is string =>
              parentId !== null && visibleEntityIds.has(parentId),
          ),
  );
  const selectedIds = new Set(
    array(
      hierarchy['selectedIds'],
      'workbenchPresentation.hierarchy.selectedIds',
    ).map((id) =>
      stringValue(id, 'workbenchPresentation.hierarchy.selectedIds entry'),
    ),
  );
  const tree = sourceNodeByControl(
    document,
    SCENE_TREE_SOURCE,
    'WorkbenchSceneTree',
  );
  const rowNodes = (tree.children ?? []).map(({ node }) => {
    const child = document.nodes?.[node];
    if (!child) throw new Error(`Workbench scene row ${node} is missing`);
    return child;
  });
  if (rows.length > rowNodes.length)
    throw new Error(
      `Workbench hierarchy has ${rows.length} rows but the authored tree exposes ${rowNodes.length} row slots`,
    );
  const searchField = sourceNodeByControl(
    document,
    SCENE_TREE_SOURCE,
    'WorkbenchSceneSearchField',
  );
  searchField.props = { ...(searchField.props ?? {}), query: filterQuery };
  for (const [index, node] of rowNodes.entries()) {
    const rowValue = visibleRows[index];
    const props = { ...(node.props ?? {}) };
    delete props['icon'];
    if (rowValue === undefined) {
      delete props['text'];
      delete props['selected'];
      delete props['active'];
      delete props['has_children'];
      node.props = { ...props, visibility: 'collapsed' };
      continue;
    }
    const row = object(rowValue, `workbench hierarchy row ${index}`);
    node.props = {
      ...props,
      text: stringValue(row['name'], `hierarchy.rows[${index}].name`),
      tree_depth: numberValue(row['depth'], `hierarchy.rows[${index}].depth`),
      selected: selectedIds.has(
        stringValue(row['id'], `hierarchy.rows[${index}].id`),
      ),
      active: booleanValue(row['active'], `hierarchy.rows[${index}].active`),
      has_children: booleanValue(
        row['hasChildren'],
        `hierarchy.rows[${index}].hasChildren`,
      ),
      expanded:
        expandedIds.includes(
          stringValue(row['id'], `hierarchy.rows[${index}].id`),
        ) ||
        filterExpandedIds.has(
          stringValue(row['id'], `hierarchy.rows[${index}].id`),
        ),
      entity_id: stringValue(row['id'], `hierarchy.rows[${index}].id`),
      ...(row['parentId'] === null
        ? {}
        : {
            parent_id: stringValue(
              row['parentId'],
              `hierarchy.rows[${index}].parentId`,
            ),
          }),
      entity_kind: stringValue(row['kind'], `hierarchy.rows[${index}].kind`),
      hierarchy_generation: stringValue(
        row['generation'],
        `hierarchy.rows[${index}].generation`,
      ),
      subtree_hash: stringValue(
        row['subtreeHash'],
        `hierarchy.rows[${index}].subtreeHash`,
      ),
      visibility: 'visible',
    };
    node['penpot_review_hierarchy_row'] = structuredClone(row);
  }

  const status = object(value['status'], 'workbenchPresentation.status');
  const shellStatus = sourceNode(
    document,
    WORKBENCH_SHELL_SOURCE,
    'status_text',
  );
  shellStatus.props = {
    ...(shellStatus.props ?? {}),
    text: stringValue(
      status['primary'],
      'workbenchPresentation.status.primary',
    ),
  };
  shellStatus['penpot_review_product_status'] = structuredClone(status);
  const windowStatus = sourceNodeByControl(
    document,
    'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui',
    'WorkbenchStatusReady',
  );
  windowStatus.props = {
    ...(windowStatus.props ?? {}),
    text: stringValue(
      status['primary'],
      'workbenchPresentation.status.primary',
    ),
  };
  windowStatus['penpot_review_product_status'] = structuredClone(status);
  for (const controlId of [
    'WorkbenchStatusErrors',
    'WorkbenchStatusWarnings',
    'WorkbenchStatusMessages',
    'WorkbenchStatusTaskProgress',
    'WorkbenchStatusGrid',
    'WorkbenchStatusSnap',
    'WorkbenchStatusSnapToggle',
    'WorkbenchStatusWorld',
    'WorkbenchStatusTarget',
    'WorkbenchStatusZoom',
  ])
    setVisibilityByControl(
      document,
      'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_status_bar.zui',
      controlId,
      'collapsed',
    );

  applyInspectorProjection(document, value['inspector']);
  applyDrawerProjection(document, value['drawers']);
}

function applyInspectorProjection(document: ZuiDocument, value: unknown): void {
  const panel = sourceNode(document, INSPECTOR_SOURCE, 'inspector_panel');
  if (value === null) {
    panel.props = { ...(panel.props ?? {}), visibility: 'collapsed' };
    return;
  }
  const inspector = object(value, 'workbenchPresentation.inspector');
  panel['penpot_review_inspector'] = structuredClone(inspector);
  const [x, y, z] = array(inspector['translation'], 'inspector.translation');
  const [sx, sy, sz] = array(inspector['scale'], 'inspector.scale');
  for (const [controlId, fieldValue] of [
    ['WorkbenchTransformPositionX', displayValue(x)],
    ['WorkbenchTransformPositionY', displayValue(y)],
    ['WorkbenchTransformPositionZ', displayValue(z)],
    ['WorkbenchTransformScaleX', displayValue(sx)],
    ['WorkbenchTransformScaleY', displayValue(sy)],
    ['WorkbenchTransformScaleZ', displayValue(sz)],
    ['WorkbenchInspectorRenderLayerMask', String(inspector['renderLayerMask'])],
  ] as const) {
    const node = sourceNodeByControl(document, INSPECTOR_SOURCE, controlId);
    node.props = { ...(node.props ?? {}), value: fieldValue };
  }
  const rotation = inspector['rotationDegrees'];
  setVisibilityByControl(
    document,
    INSPECTOR_SOURCE,
    'WorkbenchTransformRotation',
    'visible',
  );
  for (const [index, axis] of ['X', 'Y', 'Z'].entries()) {
    const node = sourceNodeByControl(
      document,
      INSPECTOR_SOURCE,
      `WorkbenchTransformRotation${axis}`,
    );
    node.props = {
      ...(node.props ?? {}),
      value: Array.isArray(rotation)
        ? `${displayValue(rotation[index])} deg`
        : '—',
      read_only: true,
      editable_text: false,
      input_focusable: false,
      input_clickable: false,
    };
  }
  prepareInspectorPropertyRows(document, inspector);
  applyInspectorPropertyRows(document, inspector);
}

function applyDrawerProjection(document: ZuiDocument, value: unknown): void {
  const drawers = array(value, 'workbenchPresentation.drawers');
  const bySide = new Map<string, JsonObject[]>();
  for (const item of drawers) {
    const drawer = object(item, 'workbench drawer');
    const side = drawerShell(String(drawer['slot']));
    bySide.set(side, [...(bySide.get(side) ?? []), drawer]);
  }
  for (const [sourceNodeId, entries] of bySide) {
    const node = sourceNode(document, WORKBENCH_SHELL_SOURCE, sourceNodeId);
    const visibleEntries = entries.filter(
      (entry) => entry['visible'] === true && entry['mode'] !== 'collapsed',
    );
    if (visibleEntries.length > 1)
      throw new Error(
        `WorkbenchShell cannot project multiple visible product drawers into ${sourceNodeId}`,
      );
    const drawer = visibleEntries[0] ?? entries[0]!;
    const visible =
      drawer['visible'] === true && drawer['mode'] !== 'collapsed';
    node.props = {
      ...(node.props ?? {}),
      visibility: visible ? 'visible' : 'collapsed',
    };
    node['penpot_review_drawers'] = structuredClone(entries);
    const layout = { ...(node.layout ?? {}) };
    const extent = numberValue(drawer['extent'], 'workbench drawer extent');
    const slot = String(drawer['slot']);
    const axis = slot === 'bottom' ? 'height' : 'width';
    const size = visible ? extent : 0;
    layout[axis] = {
      min: size,
      preferred: size,
      max: size,
      stretch: 'Fixed',
    };
    node.layout = layout;
  }
}

function replaceWithTabRows(
  document: ZuiDocument,
  target: ZuiNode,
  idPrefix: string,
  rows: readonly ProductRow[],
  productKind: string,
): void {
  const nodes = (document.nodes ??= {});
  target.component = 'HorizontalGroup';
  target.layout = {
    ...(target.layout ?? {}),
    container: { kind: 'HorizontalBox', gap: '$editor.density.gap.xsmall' },
  };
  target.children = rows.map((row, index) => {
    const nodeId = `${idPrefix}_${index}`;
    if (nodes[nodeId]) throw new Error(`Duplicate product tab node ${nodeId}`);
    nodes[nodeId] = {
      component: WORKBENCH_TAB_COMPONENT,
      props: {
        text: stringValue(row['title'], `${productKind} tab title`),
        selected: row['selected'] === true,
        checked: row['selected'] === true,
      },
      penpot_review_generated_product_row: {
        kind: productKind,
        ...structuredClone(row),
      },
    };
    return { node: nodeId };
  });
}

function setVisibilityByControl(
  document: ZuiDocument,
  sourcePath: string,
  controlId: string,
  visibility: string,
): void {
  const node = sourceNodeByControl(document, sourcePath, controlId);
  node.props = { ...(node.props ?? {}), visibility };
}

function drawerShell(slot: string): string {
  switch (slot) {
    case 'leftTop':
    case 'leftBottom':
      return 'left_drawer_shell';
    case 'rightTop':
    case 'rightBottom':
      return 'right_drawer_shell';
    case 'bottom':
      return 'bottom_drawer_shell';
    default:
      throw new Error(`Unsupported workbench drawer slot: ${slot}`);
  }
}

function object(value: unknown, context: string): JsonObject {
  if (!isRecord(value)) throw new Error(`${context} must be an object`);
  return value;
}

function array(value: unknown, context: string): unknown[] {
  if (!Array.isArray(value)) throw new Error(`${context} must be an array`);
  return value;
}

function stringValue(value: unknown, context: string): string {
  if (typeof value !== 'string') throw new Error(`${context} must be a string`);
  return value;
}

function productRow(
  row: JsonObject,
  selected: boolean,
  context: string,
): ProductRow {
  return {
    ...row,
    id: stringValue(row['id'], `${context}.id`),
    title: stringValue(row['title'], `${context}.title`),
    selected,
  };
}

function numberValue(value: unknown, context: string): number {
  if (typeof value !== 'number' || !Number.isFinite(value))
    throw new Error(`${context} must be a finite number`);
  return value;
}

function booleanValue(value: unknown, context: string): boolean {
  if (typeof value !== 'boolean')
    throw new Error(`${context} must be a boolean`);
  return value;
}

function displayValue(value: unknown): string {
  if (typeof value === 'string') return value;
  if (value === null || typeof value === 'number' || typeof value === 'boolean')
    return String(value);
  return JSON.stringify(value);
}

function isRecord(value: unknown): value is JsonObject {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}
