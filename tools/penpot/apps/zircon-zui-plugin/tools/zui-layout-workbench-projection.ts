import { sourceNode, sourceNodeByControl } from './zui-layout-source-control';
import { readViewportSnapState, viewportSnapControlProps } from '../src/bridge/zui-viewport-snap-state';
import {
  prepareInspectorPropertyRows,
  applyInspectorPropertyRows,
} from './zui-layout-inspector-properties';
import { validateWorkbenchPresentation } from './zui-layout-workbench-presentation';
import type { ZuiDocument, ZuiNode } from '../src/bridge/zui-document';

const WORKBENCH_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
const WORKBENCH_SHELL_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/editor_shell.zui';
const VIEWPORT_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/components/viewport.zui';
const SCENE_TREE_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/components/outliner.zui';
const INSPECTOR_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/components/details.zui';
const STATUS_BAR_SOURCE =
  'zircon_editor/assets/ui/editor/unreal/components/status_bar.zui';
const WORKBENCH_TAB_COMPONENT =
  'res://ui/editor/components/workbench/primitives/inputs/tab.zui#WorkbenchTab';

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
  // The Unreal level editor has no legacy host page strip. Keep the complete
  // product page DTO source-owned for review/provenance without fabricating a
  // non-existent control or changing the authored menu/toolbar structure.
  const windowRoot = sourceNode(document, WORKBENCH_WINDOW_SOURCE, 'root');
  windowRoot['penpot_review_pages'] = structuredClone(
    pageItems.map((item) => {
      const row = object(item, 'workbench page');
      return productRow(
        row,
        stringValue(row['id'], 'workbench page id') === pages['activeId'],
        'mainPage',
      );
    }),
  );

  const documents = object(
    value['documents'],
    'workbenchPresentation.documents',
  );
  const documentItems = array(
    documents['items'],
    'workbenchPresentation.documents.items',
  );
  // UE HideTabWell owns no authored DocumentTabs node. Preserve the typed
  // document inventory for provenance and let the retained host own its tab
  // surface; do not synthesize an invisible control into the Penpot file.
  windowRoot['penpot_review_documents'] = structuredClone(
    documentItems.map((item) => {
      const row = object(item, 'workbench document');
      return productRow(
        row,
        stringValue(row['id'], 'workbench document id') ===
          documents['activeId'],
        'workbench document',
      );
    }),
  );

  const drawers = array(value['drawers'], 'workbenchPresentation.drawers');
  // Drawer slots are retained as typed product state. The Unreal authored view
  // owns one real content-drawer overlay; left/right legacy shell overlays do
  // not exist and must never be synthesized into the Penpot document.
  windowRoot['penpot_review_drawers'] = structuredClone(drawers);
}

/** Bind factual product rows and values to the expanded authored workbench. */
export function applyWorkbenchPresentationProjection(
  document: ZuiDocument,
  value: Record<string, unknown>,
): void {
  validateWorkbenchPresentation(value);
  document['penpot_review_workbench_presentation'] = structuredClone(value);

  const snapProps = viewportSnapControlProps(readViewportSnapState(value['viewportSnap']));
  for (const [controlId, stateProps] of Object.entries(snapProps)) {
    const control = sourceNodeByControl(document, VIEWPORT_SOURCE, controlId);
    control.props = { ...(control.props ?? {}), ...stateProps };
  }

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
  const windowStatus = sourceNodeByControl(
    document,
    STATUS_BAR_SOURCE,
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

  applyInspectorProjection(document, value['inspector']);
  applyDrawerProjection(document, value['drawers']);
}

function applyInspectorProjection(document: ZuiDocument, value: unknown): void {
  const panel = sourceNode(document, INSPECTOR_SOURCE, 'panel');
  panel.props = { ...(panel.props ?? {}), visibility: 'visible' };
  const hasSelection = value !== null;
  const filter = sourceNodeByControl(document, INSPECTOR_SOURCE, 'WorkbenchInspectorFilter');
  filter.props = { ...(filter.props ?? {}), disabled: !hasSelection };
  for (const controlId of [
    'WorkbenchInspectorMetadata', 'WorkbenchInspectorTransform', 'WorkbenchAddComponent',
  ])
    setVisibilityByControl(document, INSPECTOR_SOURCE, controlId, hasSelection ? 'visible' : 'collapsed');
  setVisibilityByControl(document, INSPECTOR_SOURCE, 'WorkbenchInspectorFilterEmpty', 'collapsed');
  const title = sourceNodeByControl(document, INSPECTOR_SOURCE, 'WorkbenchInspectorTitle');
  if (!hasSelection) {
    title.props = { ...(title.props ?? {}), text: 'No Selection' };
    for (const controlId of ['WorkbenchInspectorMesh', 'WorkbenchInspectorMeshProperties'])
      setVisibilityByControl(document, INSPECTOR_SOURCE, controlId, 'collapsed');
    return;
  }
  const inspector = object(value, 'workbenchPresentation.inspector');
  const name = stringValue(inspector['name'], 'inspector.name').trim();
  title.props = { ...(title.props ?? {}), text: name || 'Entity' };
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
  const windowRoot = sourceNode(document, WORKBENCH_WINDOW_SOURCE, 'root');
  windowRoot['penpot_review_drawers'] = structuredClone(drawers);
  const bottom = drawers
    .map((item) => object(item, 'workbench drawer'))
    .find((drawer) => String(drawer['slot']).toLowerCase() === 'bottom');
  if (!bottom) return;
  const node = sourceNode(document, WORKBENCH_WINDOW_SOURCE, 'content_drawer');
  const visible =
    bottom['visible'] === true && bottom['mode'] !== 'collapsed';
  node.props = { ...(node.props ?? {}), visibility: visible ? 'visible' : 'collapsed' };
  node['penpot_review_drawer'] = structuredClone(bottom);
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
