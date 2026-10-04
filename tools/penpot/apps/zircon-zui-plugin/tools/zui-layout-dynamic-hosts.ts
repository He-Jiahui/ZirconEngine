import {
  zuiRootNodeIds,
  type ZuiDocument,
  type ZuiNode,
} from '../src/bridge/zui-document';

/**
 * Design-only state used while reviewing authored dynamic hosts in Penpot.
 * The runtime source is kept in `penpot_original_source`; this marker is
 * intentionally metadata on the projection and is never exported as a source
 * replacement.
 */
export const DYNAMIC_REVIEW_STATE_KEY = 'penpot_dynamic_review_state';

export interface DynamicReviewState {
  schema: 'dev.zircon.zui.dynamic-review-state';
  version: 1;
  sourcePath: string;
  selectedNodeIds: string[];
  selectionPolicy:
    | 'authored-first-visible-branch'
    | 'authored-visible-root';
}

export interface DynamicReviewPreparation {
  changes: string[];
  state?: DynamicReviewState;
}

export interface DynamicReviewContext {
  /**
   * A factual editor snapshot cannot select fixture-only content for a host
   * branch whose active view is absent from that snapshot. Keep those authored
   * branches in their source default state instead.
   */
  preserveUnselectedBranchDefaults?: boolean;
}

const ADDITIONAL_BRANCHES = [
  'ability_workspace_body',
  'tags_workspace_body',
  'perception_workspace_body',
  'render_workspace_body',
  'hud_workspace_body',
] as const;

const EXTENSION_BRANCHES = [
  'shader_editor_workspace_body',
  'lighting_bake_workspace_body',
  'post_process_workspace_body',
  'sequencer_workspace_body',
  'montage_editor_workspace_body',
  'blend_space_workspace_body',
  'pose_library_workspace_body',
  'retarget_workspace_body',
  'control_rig_workspace_body',
  'motion_matching_workspace_body',
  'animation_compression_workspace_body',
  'terrain_editor_workspace_body',
  'foliage_editor_workspace_body',
  'level_streaming_workspace_body',
  'level_variant_workspace_body',
  'prefab_editor_workspace_body',
  'scatter_editor_workspace_body',
  'volume_editor_workspace_body',
  'weather_editor_workspace_body',
  'spawn_rules_workspace_body',
  'world_state_workspace_body',
  'collision_proxy_workspace_body',
  'physics_collision_workspace_body',
  'navmesh_ai_workspace_body',
  'lobby_editor_workspace_body',
  'matchmaking_editor_workspace_body',
  'data_table_workspace_body',
  'source_control_workspace_body',
  'build_export_workspace_body',
  'automation_report_workspace_body',
  'project_overview_workspace_body',
  'plugin_manager_workspace_body',
  'save_data_workspace_body',
  'particle_library_workspace_body',
  'ui_asset_editor_workspace_body',
  'ui_binding_workspace_body',
  'icon_library_workspace_body',
  'accessibility_audit_workspace_body',
  'menu_flow_workspace_body',
  'font_atlas_workspace_body',
  'console_diagnostics_workspace_body',
  'runtime_diagnostics_workspace_body',
  'performance_workspace_body',
  'telemetry_dashboard_workspace_body',
] as const;

const MODULE_BRANCHES = [
  'effect_workspace_body',
  'material_workspace_body',
  'behavior_workspace_body',
  'vfx_workspace_body',
  'assets_workspace_body',
] as const;

/**
 * Select one real authored workspace branch for a static Penpot review.
 * Runtime hosts normally select these branches from editor state; the source
 * files intentionally leave them collapsed by default.  Selecting a branch
 * here keeps all authored nodes, events and component references intact while
 * making the review case deterministic and renderable.
 */
export function prepareDynamicReviewState(
  document: ZuiDocument,
  sourcePath: string,
  context: DynamicReviewContext = {},
): DynamicReviewPreparation {
  if (sourcePath.endsWith('/workbench_component_drawer.zui'))
    return context.preserveUnselectedBranchDefaults
      ? { changes: [] }
      : prepareComponentDrawerState(document, sourcePath);
  if (sourcePath.endsWith('/workbench_main_band.zui'))
    return prepareMainBandState(document, sourcePath);
  if (sourcePath.endsWith('/workbench_skeleton.zui'))
    return prepareSkeletonState(document, sourcePath);
  const branches = branchPolicy(sourcePath);
  if (!branches) return prepareDirectWorkspaceRoot(document, sourcePath);

  const nodes = document.nodes ?? {};
  const selected = branches.find((id) => Boolean(nodes[id]));
  if (!selected) return { changes: [] };

  const branchSet = new Set(branches);
  for (const id of branches) {
    const node = nodes[id];
    if (!node) continue;
    node.props = { ...(node.props ?? {}), visibility: id === selected ? 'visible' : 'collapsed' };
  }

  // The extension and additional wrappers are component roots and inherit a
  // collapsed default from their authored definition.  The module wrapper is
  // already visible, but setting it explicitly makes the projection stable
  // when a component default changes.
  for (const rootId of zuiRootNodeIds(document)) {
    const root = nodes[rootId];
    if (root && isDynamicWrapperRoot(root, sourcePath))
      root.props = { ...(root.props ?? {}), visibility: 'visible' };
  }

  // Nested copies are produced by LayoutDependencies.embed with a qualified
  // id.  Keep their authored branch state aligned with the selected branch so
  // a module wrapper cannot accidentally expose a second workspace instance.
  for (const [id, node] of Object.entries(nodes)) {
    const suffix = id.slice(id.lastIndexOf('__') + 2);
    if (!branchSet.has(suffix)) continue;
    const selectedNested = suffix === selected;
    node.props = {
      ...(node.props ?? {}),
      visibility: selectedNested ? 'visible' : 'collapsed',
    };
  }

  const state: DynamicReviewState = {
    schema: 'dev.zircon.zui.dynamic-review-state',
    version: 1,
    sourcePath,
    selectedNodeIds: [selected],
    selectionPolicy: 'authored-first-visible-branch',
  };
  document[DYNAMIC_REVIEW_STATE_KEY] = state;
  return {
    changes: ['inject-deterministic-authored-host-state'],
    state,
  };
}

/**
 * The standalone authored review selects a real Components tab and exposes
 * its authored body. A composed product host skips that choice because its
 * active-tab state does not include this independent showcase drawer.
 */
function prepareComponentDrawerState(
  document: ZuiDocument,
  sourcePath: string,
): DynamicReviewPreparation {
  const nodes = document.nodes ?? {};
  const find = (controlId: string | undefined, suffix: string): string[] =>
    Object.entries(nodes)
      .filter(
        ([id, node]) =>
          (controlId !== undefined && node.control_id === controlId) ||
          id === suffix ||
          id.endsWith(`__${suffix}`),
      )
      .map(([id]) => id);
  const tabs = find(undefined, 'drawer_tabs');
  const content = find(undefined, 'component_drawer_content');
  const showcaseTabs = find(undefined, 'retired_showcase_tabs');
  const showcaseContent = find(undefined, 'retired_showcase_content');
  const componentBodies = find('WorkbenchComponentDrawerBody', 'component_body');
  const consoleBodies = find(
    'WorkbenchComponentDrawerConsoleBody',
    'console_body',
  );
  const componentTabs = find('WorkbenchDrawerTabComponents', 'drawer_tab_components');
  const consoleTabs = find('WorkbenchDrawerTabConsole', 'drawer_tab_console');
  const selectedNodeIds: string[] = [];
  const setVisibility = (ids: string[], visibility: string): void => {
    for (const id of ids) {
      const node = nodes[id];
      if (!node) continue;
      node.props = { ...(node.props ?? {}), visibility };
      if (visibility === 'visible') selectedNodeIds.push(id);
    }
  };
  // Set both the shell wrappers and their authored branch.  Matching by
  // control id covers materialized prefab roots; matching by qualified suffix
  // covers copies nested under another expanded component.
  setVisibility(showcaseTabs, 'visible');
  setVisibility(showcaseContent, 'visible');
  setVisibility(componentBodies, 'visible');
  setVisibility(consoleBodies, 'collapsed');
  for (const id of componentTabs) {
    const node = nodes[id];
    if (node) node.props = { ...(node.props ?? {}), selected: true, checked: true };
  }
  for (const id of consoleTabs) {
    const node = nodes[id];
    if (node) node.props = { ...(node.props ?? {}), selected: false, checked: false };
  }
  // Keep the immediate shell mounts visible if an authored wrapper carried a
  // stale hidden value.  Do not invent children or labels for empty slots.
  setVisibility(tabs, 'visible');
  setVisibility(content, 'visible');
  if (!selectedNodeIds.length) return { changes: [] };
  const state: DynamicReviewState = {
    schema: 'dev.zircon.zui.dynamic-review-state',
    version: 1,
    sourcePath,
    selectedNodeIds: [...new Set(selectedNodeIds)],
    selectionPolicy: 'authored-visible-root',
  };
  document[DYNAMIC_REVIEW_STATE_KEY] = state;
  return {
    changes: ['inject-deterministic-workbench-host-state'],
    state,
  };
}

/** Select the authored default scene branch while retaining inactive branches. */
function prepareMainBandState(
  document: ZuiDocument,
  sourcePath: string,
): DynamicReviewPreparation {
  return selectSceneAndRetainModule(document, sourcePath);
}

/**
 * WorkbenchSkeleton expands WorkbenchMainBand as a nested prefab. Apply the
 * authored default scene selection while keeping its inactive module branch
 * and source metadata available to the complete host projection.
 */
function prepareSkeletonState(
  document: ZuiDocument,
  sourcePath: string,
): DynamicReviewPreparation {
  return selectSceneAndRetainModule(document, sourcePath);
}

function selectSceneAndRetainModule(
  document: ZuiDocument,
  sourcePath: string,
): DynamicReviewPreparation {
  const nodes = document.nodes ?? {};
  const mainBandIds = Object.entries(nodes)
    .filter(
      ([, node]) =>
        node.control_id === 'WorkbenchMainBand' ||
        node.control_id === 'WorkbenchSkeletonMainBand',
    )
    .map(([id]) => id);
  // A standalone main-band source uses the authored root id; a skeleton has a
  // qualified instance root after dependency expansion.
  if (!mainBandIds.length) {
    for (const [id, node] of Object.entries(nodes)) {
      if (
        node.component === 'Overlay' &&
        (node.children ?? []).some((child) =>
          child.node === 'scene_workspace' ||
          child.node.endsWith('__scene_workspace'),
        ) &&
        (node.children ?? []).some((child) =>
          child.node === 'module_workspace' ||
          child.node.endsWith('__module_workspace'),
        )
      )
        mainBandIds.push(id);
    }
  }
  const mainBandId = mainBandIds[0];
  if (!mainBandId) return { changes: [] };
  const mainBand = nodes[mainBandId];
  const sceneId = (mainBand.children ?? []).map((child) => child.node).find((id) => {
    const node = nodes[id];
    return (
      node?.control_id === 'WorkbenchSceneWorkspace' ||
      id === 'scene_workspace' ||
      id.endsWith('__scene_workspace')
    );
  });
  const moduleId = (mainBand.children ?? []).map((child) => child.node).find((id) => {
    const node = nodes[id];
    return (
      node?.control_id === 'WorkbenchMainBandModuleWorkspace' ||
      id === 'module_workspace' ||
      id.endsWith('__module_workspace')
    );
  });
  if (!sceneId) return { changes: [] };
  const scene = nodes[sceneId]!;
  scene.props = { ...(scene.props ?? {}), visibility: 'visible' };
  const selectedNodeIds = [sceneId];
  const changes = ['select-retained-scene-workspace'];
  if (moduleId) {
    const module = nodes[moduleId];
    if (module) module.props = { ...(module.props ?? {}), visibility: 'collapsed' };
    changes.push('retain-inactive-workspace-branch');
  }
  const state: DynamicReviewState = {
    schema: 'dev.zircon.zui.dynamic-review-state',
    version: 1,
    sourcePath,
    selectedNodeIds,
    selectionPolicy: 'authored-visible-root',
  };
  document[DYNAMIC_REVIEW_STATE_KEY] = state;
  return { changes, state };
}

/**
 * A standalone module workspace is an authored component, but Runtime only
 * reveals it after the module router activates the corresponding extension.
 * The design projection must show that exact component tree instead of a
 * blank collapsed root.  This remains projection metadata: export restores
 * the original collapsed source byte-for-byte.
 */
function prepareDirectWorkspaceRoot(
  document: ZuiDocument,
  sourcePath: string,
): DynamicReviewPreparation {
  const nodes = document.nodes ?? {};
  // Component assets can contain more than one exported root. Find the
  // authored module body rather than assuming the first export is reviewable.
  const rootId = zuiRootNodeIds(document).find((id) => {
    const root = nodes[id];
    return root !== undefined && isDirectDynamicWorkspaceRoot(root, sourcePath);
  });
  if (!rootId) return { changes: [] };

  const root = nodes[rootId]!;
  root.props = { ...(root.props ?? {}), visibility: 'visible' };
  const state: DynamicReviewState = {
    schema: 'dev.zircon.zui.dynamic-review-state',
    version: 1,
    sourcePath,
    selectedNodeIds: [rootId],
    selectionPolicy: 'authored-visible-root',
  };
  document[DYNAMIC_REVIEW_STATE_KEY] = state;
  return {
    changes: ['inject-deterministic-authored-host-state'],
    state,
  };
}

function branchPolicy(sourcePath: string): readonly string[] | undefined {
  if (sourcePath.endsWith('/workbench_additional_module_workspaces.zui'))
    return ADDITIONAL_BRANCHES;
  if (sourcePath.endsWith('/workbench_extension_module_workspaces.zui'))
    return EXTENSION_BRANCHES;
  if (sourcePath.endsWith('/workbench_module_workspace.zui'))
    return MODULE_BRANCHES;
  return undefined;
}

function isDynamicWrapperRoot(node: ZuiNode, sourcePath: string): boolean {
  return (
    node.component === 'Overlay' &&
    (sourcePath.endsWith('/workbench_additional_module_workspaces.zui') ||
      sourcePath.endsWith('/workbench_extension_module_workspaces.zui') ||
      sourcePath.endsWith('/workbench_module_workspace.zui'))
  );
}

function isDirectDynamicWorkspaceRoot(
  node: ZuiNode,
  sourcePath: string,
): boolean {
  if (
    !sourcePath.includes('/components/workbench/modules/') ||
    !sourcePath.endsWith('_workspace.zui') ||
    node.component !== 'Overlay'
  )
    return false;
  const classes = new Set(node.classes ?? []);
  return (
    classes.has('workbench-module-body') ||
    classes.has('workbench-extension-module-body')
  );
}
