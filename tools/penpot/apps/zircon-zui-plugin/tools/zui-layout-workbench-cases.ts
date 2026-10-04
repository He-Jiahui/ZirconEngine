import { createHash } from 'node:crypto';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';
import {
  assertWorkbenchManagedProjectPath,
  verifyWorkbenchManagedSceneFingerprint,
  type WorkbenchManagedSceneFingerprint,
} from './zui-layout-workbench-managed-inputs';
import type { ZuiDocument } from '../src/bridge/zui-document';
import {
  type LayoutReviewCase,
  type WorkbenchStateSelector,
} from './zui-layout-review-contract';
import {
  canonicalSha256,
  type WorkbenchPresentationSnapshot,
  validateWorkbenchPresentation,
} from './zui-layout-workbench-presentation';

export const WORKBENCH_WINDOW_SOURCE =
  'zircon_editor/assets/ui/editor/windows/workbench_window.zui';
const SCENE_TREE_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_scene_tree_panel.zui';
const INSPECTOR_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_inspector_panel.zui';
const TOOLBAR_SOURCE =
  'zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui';
const SNAPSHOTS_PATH = 'docs/_data/layout/workbench-product-state.json';
export const LONG_ENGLISH_NAME =
  'Cube — an intentionally long English scene object label used to check hierarchy row and inspector title overflow without losing entity identity';
export const LONG_CHINESE_NAME =
  '立方体节点的超长中文名称，用于验证层级行与检查器标题的溢出处理，同时保留实体身份与父子关系。';

export interface WorkbenchCaseSnapshots {
  schema: 'dev.zircon.editor.workbench-case-snapshots';
  version: 1;
  sourceFingerprint: { sourcePath: string; sha256: string };
  managedSceneFingerprint: WorkbenchManagedSceneFingerprint;
  states: {
    default: WorkbenchPresentationSnapshot;
    empty: WorkbenchPresentationSnapshot;
    selected: WorkbenchPresentationSnapshot;
    hierarchy: WorkbenchPresentationSnapshot;
    longEnglish: WorkbenchPresentationSnapshot;
    longChinese: WorkbenchPresentationSnapshot;
  };
}

const ROOT: WorkbenchStateSelector = {
  sourcePath: WORKBENCH_WINDOW_SOURCE,
  sourceNodeId: 'root',
  controlId: 'WorkbenchWindowRoot',
};
const SEARCH: WorkbenchStateSelector = {
  sourcePath: SCENE_TREE_SOURCE,
  sourceNodeId: 'scene_search_field',
  controlId: 'WorkbenchSceneSearchField',
};
const SCENE_SCROLL: WorkbenchStateSelector = {
  sourcePath: SCENE_TREE_SOURCE,
  sourceNodeId: 'scene_tree',
  controlId: 'WorkbenchSceneTree',
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function exactFields(
  value: Record<string, unknown>,
  fields: readonly string[],
  context: string,
): void {
  const expected = new Set(fields);
  if (
    Object.keys(value).length !== fields.length ||
    Object.keys(value).some((key) => !expected.has(key))
  )
    throw new Error(`${context} has missing or unsupported fields`);
}

/** Read product-owned state only after checking its authored source fingerprint. */
export async function readWorkbenchCaseSnapshots(
  repoRoot: string,
): Promise<WorkbenchCaseSnapshots> {
  const bytes = await readFile(resolve(repoRoot, SNAPSHOTS_PATH));
  const parsed: unknown = JSON.parse(bytes.toString('utf8'));
  if (!isRecord(parsed))
    throw new Error('Workbench snapshots must be an object');
  exactFields(
    parsed,
    [
      'schema',
      'version',
      'sourceFingerprint',
      'managedSceneFingerprint',
      'states',
    ],
    'Workbench snapshots',
  );
  if (
    parsed['schema'] !== 'dev.zircon.editor.workbench-case-snapshots' ||
    parsed['version'] !== 1 ||
    !isRecord(parsed['sourceFingerprint']) ||
    !isRecord(parsed['states'])
  )
    throw new Error('Workbench snapshot schema is unsupported');
  exactFields(
    parsed['sourceFingerprint'],
    ['sourcePath', 'sha256'],
    'Source fingerprint',
  );
  exactFields(
    parsed['states'],
    [
      'default',
      'empty',
      'selected',
      'hierarchy',
      'longEnglish',
      'longChinese',
    ],
    'Snapshot states',
  );
  if (parsed['sourceFingerprint']['sourcePath'] !== WORKBENCH_WINDOW_SOURCE)
    throw new Error(
      'Workbench snapshot source path is not the product WorkbenchWindow',
    );
  const currentSourceSha = createHash('sha256')
    .update(await readFile(resolve(repoRoot, WORKBENCH_WINDOW_SOURCE)))
    .digest('hex');
  if (parsed['sourceFingerprint']['sha256'] !== currentSourceSha)
    throw new Error(
      'Workbench product snapshot is stale against its authored ZUI source',
    );
  const managedSceneFingerprint = await verifyWorkbenchManagedSceneFingerprint(
    parsed['managedSceneFingerprint'],
  );
  for (const state of [
    'default',
    'empty',
    'selected',
    'hierarchy',
    'longEnglish',
    'longChinese',
  ] as const) {
    const value = parsed['states'][state];
    validateWorkbenchPresentation(value);
    const status = isRecord(value) ? value['status'] : undefined;
    assertWorkbenchManagedProjectPath(
      managedSceneFingerprint,
      isRecord(status) ? status['projectPath'] : undefined,
    );
    const expectedLocale = state === 'longChinese' ? 'zh-CN' : 'en';
    if (
      !isRecord(value) ||
      value['activeLocale'] !== expectedLocale ||
      !isRecord(value['sourceFingerprint']) ||
      value['sourceFingerprint']['sourcePath'] !== WORKBENCH_WINDOW_SOURCE ||
      value['sourceFingerprint']['sha256'] !== currentSourceSha
    )
      throw new Error(
        `Workbench ${state} snapshot has a stale source fingerprint`,
      );
  }
  validateLongTextProductState(
    parsed['states']['longEnglish'],
    'en',
    LONG_ENGLISH_NAME,
  );
  validateLongTextProductState(
    parsed['states']['longChinese'],
    'zh-CN',
    LONG_CHINESE_NAME,
  );
  return parsed as unknown as WorkbenchCaseSnapshots;
}

function validateLongTextProductState(
  value: unknown,
  expectedLocale: 'en' | 'zh-CN',
  expectedName: string,
): void {
  if (!isRecord(value) || value['activeLocale'] !== expectedLocale)
    throw new Error(
      `Workbench long-text snapshot must use active locale ${expectedLocale}`,
    );
  const inspector = value['inspector'];
  const hierarchy = value['hierarchy'];
  if (
    !isRecord(inspector) ||
    typeof inspector['entityId'] !== 'string' ||
    typeof inspector['name'] !== 'string' ||
    !isRecord(hierarchy) ||
    !Array.isArray(hierarchy['selectedIds']) ||
    !Array.isArray(hierarchy['rows'])
  )
    throw new Error('Workbench long-text snapshot lacks actual selection data');
  const entityId = inspector['entityId'];
  if (!hierarchy['selectedIds'].includes(entityId))
    throw new Error('Workbench long-text inspector entity is not selected');
  const rows = hierarchy['rows'].filter(
    (row): row is Record<string, unknown> => isRecord(row),
  );
  if (rows.length !== hierarchy['rows'].length)
    throw new Error('Workbench long-text hierarchy contains an invalid row');
  const selectedRow = rows.find((row) => row['id'] === entityId);
  if (
    !selectedRow ||
    selectedRow['name'] !== inspector['name'] ||
    inspector['name'] !== expectedName ||
    Buffer.byteLength(inspector['name'], 'utf8') < 48
  )
    throw new Error(
      'Workbench long-text state must preserve the selected entity name in both projections',
    );
}

function popupSelector(
  document: ZuiDocument,
  nodeId: string,
): WorkbenchStateSelector {
  const node = document.nodes?.[nodeId];
  if (!node || typeof node.control_id !== 'string' || !node.control_id)
    throw new Error(
      `Workbench popup ${nodeId} is missing from the authored root`,
    );
  if (
    !Object.hasOwn(node.props ?? {}, 'popup_open') &&
    !Object.hasOwn(node.props ?? {}, 'open')
  )
    throw new Error(`Workbench popup ${nodeId} has no authored open state`);
  return {
    sourcePath: WORKBENCH_WINDOW_SOURCE,
    sourceNodeId: nodeId,
    controlId: node.control_id,
  };
}

function dataFor(
  presentation: WorkbenchPresentationSnapshot,
  selector: WorkbenchStateSelector,
  managedSceneFingerprint: WorkbenchManagedSceneFingerprint,
): LayoutReviewCase['data'] {
  return {
    workbenchPresentation: presentation,
    workbenchState: selector,
    managedSceneFingerprint,
  };
}

function rowsForHierarchySnapshot(
  presentation: WorkbenchPresentationSnapshot,
): Record<string, unknown>[] {
  const hierarchy = presentation['hierarchy'];
  if (!isRecord(hierarchy) || !Array.isArray(hierarchy['rows']))
    throw new Error('Workbench hierarchy snapshot is missing its product rows');
  const rows = hierarchy['rows'].filter(
    (row): row is Record<string, unknown> => isRecord(row),
  );
  if (rows.length !== hierarchy['rows'].length)
    throw new Error('Workbench hierarchy snapshot contains an invalid row');
  return rows;
}

function cubeParentEntityId(
  presentation: WorkbenchPresentationSnapshot,
): string {
  const rows = rowsForHierarchySnapshot(presentation);
  const cubes = rows.filter((row) => row['name'] === 'Cube');
  if (cubes.length !== 1)
    throw new Error('Workbench hierarchy snapshot must identify one actual Cube row');
  const cube = cubes[0]!;
  const parentId = cube['parentId'];
  if (typeof parentId !== 'string')
    throw new Error('Workbench hierarchy snapshot Cube must have an actual parent');
  const parent = rows.find((row) => row['id'] === parentId);
  if (
    !parent ||
    parent['hasChildren'] !== true ||
    typeof parent['depth'] !== 'number' ||
    typeof cube['depth'] !== 'number' ||
    cube['depth'] !== parent['depth'] + 1
  )
    throw new Error('Workbench hierarchy snapshot Cube parent relationship is inconsistent');
  return parentId;
}

function withHierarchyViewState(
  presentation: WorkbenchPresentationSnapshot,
  filterQuery: string,
  expandedIds: readonly string[],
): WorkbenchPresentationSnapshot {
  const result = structuredClone(presentation);
  const hierarchy = result['hierarchy'];
  if (!isRecord(hierarchy))
    throw new Error('Workbench hierarchy snapshot must contain a hierarchy object');
  hierarchy['filterQuery'] = filterQuery;
  hierarchy['expandedIds'] = [...expandedIds];
  const state = Object.fromEntries(
    Object.entries(result).filter(([key]) => key !== 'stateFingerprint'),
  );
  return {
    ...state,
    stateFingerprint: canonicalSha256(state),
  } as WorkbenchPresentationSnapshot;
}

/**
 * Give every real WorkbenchWindow case a product snapshot and one exact authored
 * control. Extra popup and text cases use the same product state and source IDs.
 */
export function buildWorkbenchReviewCases(
  cases: LayoutReviewCase[],
  originalDocument: ZuiDocument,
  snapshots: WorkbenchCaseSnapshots,
): LayoutReviewCase[] {
  if (
    !cases.length ||
    cases.some((item) => item.sourcePath !== WORKBENCH_WINDOW_SOURCE)
  )
    throw new Error(
      'Workbench cases must contain only the authored WorkbenchWindow',
    );
  const mainMenu = popupSelector(originalDocument, 'toolbar_main_menu');
  const selectedSnapshot = snapshots.states.selected;
  const emptySnapshot = snapshots.states.empty;
  const defaultSnapshot = snapshots.states.default;
  const hierarchySnapshot = snapshots.states.hierarchy;
  const completeCases = [...cases];
  const sizes = cases.filter((item) => item.state === 'default');
  if (sizes.length !== 4 || new Set(sizes.map((item) => item.id)).size !== 4)
    throw new Error(
      'Workbench matrix requires its four authored viewport and DPI sizes',
    );
  for (const state of [
    'open',
    'closed',
    'focused',
    'empty',
    'focus-return',
    'selected',
  ]) {
    for (const baseline of sizes) {
      const id = `${state}-${baseline.viewport.width}x${baseline.viewport.height}-dpi${baseline.dpi}`;
      if (!completeCases.some((item) => item.id === id))
        completeCases.push({ ...baseline, id, state });
    }
  }
  const compact = sizes.find(
    (item) =>
      item.viewport.width === 640 &&
      item.viewport.height === 520 &&
      item.dpi === 1,
  );
  if (!compact) throw new Error('Workbench matrix lacks its compact viewport');
  for (const state of ['scroll-before', 'scroll-after'] as const) {
    const id = `${state}-640x520-dpi1`;
    if (!completeCases.some((item) => item.id === id))
      completeCases.push({ ...compact, id, state });
  }
  if (
    !completeCases.some((item) => item.id === 'open-scroll-after-640x520-dpi1')
  )
    completeCases.push({
      ...compact,
      id: 'open-scroll-after-640x520-dpi1',
      state: 'open',
      scrollPosition: 'end',
    });
  const result = completeCases.map((item): LayoutReviewCase => {
    const presentation =
      item.state === 'selected'
        ? selectedSnapshot
        : item.state === 'empty'
          ? emptySnapshot
          : defaultSnapshot;
    const selector =
      item.state === 'open' ||
      item.state === 'closed' ||
      item.state === 'focus-return'
        ? mainMenu
        : item.state === 'focused'
          ? SEARCH
          : item.state === 'scroll-before' || item.state === 'scroll-after'
            ? SCENE_SCROLL
            : ROOT;
    return {
      ...item,
      data: dataFor(
        presentation,
        item.scrollPosition && item.state === 'open'
          ? { ...selector, scrollTarget: SCENE_SCROLL }
          : selector,
        snapshots.managedSceneFingerprint,
      ),
    };
  });
  const baseline = result.find((item) => item.id === 'default-900x620-dpi1');
  if (!baseline)
    throw new Error('Workbench review lacks the 900x620 product baseline');

  const cubeParentId = cubeParentEntityId(hierarchySnapshot);
  const unmatchedQuery = '__zircon_no_matching_scene_entity__';
  if (
    rowsForHierarchySnapshot(hierarchySnapshot).some((row) =>
      String(row['name']).toLowerCase().includes(unmatchedQuery),
    )
  )
    throw new Error('Workbench hierarchy no-match query unexpectedly matches a product row');
  const hierarchySizes = [
    ['1280x800-dpi1', 1280, 800, 1],
    ['640x520-dpi1', 640, 520, 1],
    ['900x620-dpi1', 900, 620, 1],
    ['1280x800-dpi1.5', 1280, 800, 1.5],
  ] as const;
  const hierarchySize = (key: (typeof hierarchySizes)[number][0]) => {
    const sizeSpec = hierarchySizes.find(([id]) => id === key);
    if (!sizeSpec) throw new Error(`Unknown hierarchy viewport ${key}`);
    const [, width, height, dpi] = sizeSpec;
    const found = sizes.find(
      (item) =>
        item.viewport.width === width &&
        item.viewport.height === height &&
        item.dpi === dpi,
    );
    if (!found) throw new Error(`Workbench matrix lacks hierarchy viewport ${key}`);
    return found;
  };
  const hierarchyCases = [
    {
      kind: 'collapsed',
      size: hierarchySize('1280x800-dpi1'),
      filterQuery: '',
      expandedIds: [],
    },
    {
      kind: 'expanded',
      size: hierarchySize('640x520-dpi1'),
      filterQuery: '',
      expandedIds: [cubeParentId],
    },
    {
      kind: 'search-match',
      size: hierarchySize('900x620-dpi1'),
      filterQuery: 'Cube',
      expandedIds: [],
    },
    {
      kind: 'search-no-match',
      size: hierarchySize('1280x800-dpi1.5'),
      filterQuery: unmatchedQuery,
      expandedIds: [],
    },
    {
      kind: 'search-cleared',
      size: hierarchySize('1280x800-dpi1'),
      filterQuery: '',
      expandedIds: [cubeParentId],
    },
  ] as const;
  for (const item of hierarchyCases) {
    const sizeId = `${item.size.viewport.width}x${item.size.viewport.height}-dpi${item.size.dpi}`;
    result.push({
      ...item.size,
      id: `hierarchy-${item.kind}-${sizeId}`,
      // The actual hierarchy view state is fully represented by the product
      // snapshot. Keep the generic transient UI state at its normal baseline.
      state: 'default',
      data: dataFor(
        withHierarchyViewState(
          hierarchySnapshot,
          item.filterQuery,
          item.expandedIds,
        ),
        ROOT,
        snapshots.managedSceneFingerprint,
      ),
    });
  }

  // Each popup is authored in the window root and stays distinct from the
  // selected main-menu case already present at all four requested sizes.
  for (const nodeId of [
    'toolbar_run_mode_menu',
    'toolbar_layout_menu',
    'toolbar_module_overflow_menu',
    'assets_world_tools_menu',
    'assets_gameplay_tools_menu',
    'assets_production_tools_menu',
    'ability_animation_tools_menu',
    'render_tools_menu',
    'hud_tools_menu',
    'command_palette',
    'settings_window',
    'notification_center',
  ]) {
    const selector = popupSelector(originalDocument, nodeId);
    const states = [
      'command_palette',
      'settings_window',
      'notification_center',
    ].includes(nodeId)
      ? (['open', 'closed'] as const)
      : (['open', 'closed', 'focus-return'] as const);
    for (const state of states)
      result.push({
        ...baseline,
        id: `${nodeId}-${state}-900x620-dpi1`,
        state,
        data: dataFor(
          defaultSnapshot,
          selector,
          snapshots.managedSceneFingerprint,
        ),
      });
  }

  for (const [region, selector, presentation] of [
    [
      'inspector',
      {
        sourcePath: INSPECTOR_SOURCE,
        sourceNodeId: 'inspector_content',
        controlId: 'RightDrawerContentRoot',
      },
      selectedSnapshot,
    ],
    [
      'toolbar',
      {
        sourcePath: TOOLBAR_SOURCE,
        sourceNodeId: 'module_tab_scroll',
        controlId: 'WorkbenchModuleTabScroll',
      },
      defaultSnapshot,
    ],
  ] as const) {
    for (const state of ['scroll-before', 'scroll-after'] as const)
      result.push({
        ...compact,
        id: `${region}-${state}-640x520-dpi1`,
        state,
        data: dataFor(
          presentation,
          selector,
          snapshots.managedSceneFingerprint,
        ),
      });
  }

  const textBaselines = result.filter(
    (value) => value.state === 'default' && !value.id.startsWith('hierarchy-'),
  );
  for (const [id, locale, presentation] of [
    [
      'long-en',
      'en-US',
      snapshots.states.longEnglish,
    ],
    [
      'long-zh',
      'zh-CN',
      snapshots.states.longChinese,
    ],
  ] as const) {
    for (const item of textBaselines)
      result.push({
        ...item,
        id: `${id}-${item.viewport.width}x${item.viewport.height}-dpi${item.dpi}`,
        locale,
        data: dataFor(presentation, ROOT, snapshots.managedSceneFingerprint),
      });
  }
  return result;
}
