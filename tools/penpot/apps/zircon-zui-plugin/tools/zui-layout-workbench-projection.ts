import { sourceNode, sourceNodeByControl } from './zui-layout-source-control';
import { readViewportSnapState, viewportSnapControlProps } from '../src/bridge/zui-viewport-snap-state';
import {
  prepareInspectorPropertyRows,
  applyInspectorPropertyRows,
} from './zui-layout-inspector-properties';
import { validateWorkbenchPresentation } from './zui-layout-workbench-presentation';
import { readViewportToolbarPublication } from '../src/bridge/zui-viewport-toolbar-publication';
import type { ZuiDocument, ZuiNode, ZuiTable, ZuiValue } from '../src/bridge/zui-document';

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

const MENU_RUN_SOURCES = new Set(['menus', 'toolbar', 'menu_popups'].map(
  (name) => `zircon_editor/assets/ui/editor/unreal/components/${name}.zui`,
));
const MENU_RUN_COMPONENTS = new Set(['Button', 'IconButton', 'PopupMenu']);
const MENU_RUN_PROPERTIES = ['menu_items', 'popup_open', 'selected', 'checked',
  'value', 'text', 'value_text', 'label'] as const;
const FOOTER_STATE_TARGETS = [
  ['ready', 'WorkbenchStatusReady', 'Label', 'text'],
  ['errors', 'WorkbenchStatusErrors', 'Label', 'text'],
  ['warnings', 'WorkbenchStatusWarnings', 'Label', 'text'],
  ['messages', 'WorkbenchStatusMessages', 'Label', 'text'],
  ['grid', 'WorkbenchStatusGrid', 'Button', 'text'],
  ['snap', 'WorkbenchStatusSnap', 'Button', 'text'],
  ['snap_toggle', 'WorkbenchStatusSnapToggle', 'IconButton', 'checked'],
] as const;
const PRESENTATION_CURSOR_FIELDS = ['structure', 'geometry', 'interaction', 'viewport',
  'hitTest', 'theme', 'diagnostics'] as const;
type SourceStep = { sourcePath: string; sourceNodeId: string };
type ObservedIdentity = SourceStep & { instancePath: SourceStep[]; controlId: string | null };

/** Consume normal menu/run and project-context state without importing arranged geometry. */
export function applyObservedWorkbenchMenuAndRunState(document: ZuiDocument, value: unknown): void {
  const evidence = object(value, 'same-frame menu/run evidence');
  // Reuse the existing full prepared-toolbar validator, including all seven
  // normal/business cursors, successful frame, source graph and snap values.
  if (!readViewportToolbarPublication(evidence))
    throw new Error('Same-frame menu/run requires the actual viewport toolbar publication');
  const business = object(evidence['workbenchStatePublication'], 'business publication');
  const presentation = object(business['workbenchPresentation'], 'business presentation');
  const fingerprint = object(presentation['sourceFingerprint'], 'business source fingerprint');
  if (fingerprint['sourcePath'] !== WORKBENCH_WINDOW_SOURCE)
    throw new Error('Same-frame menu/run has the wrong Level Editor owner');
  const inventory = object(evidence['arrangedInventoryPublication'], 'arranged inventory publication');
  if (inventory['schema'] !== 'dev.zircon.editor.normal-arranged-inventory' || inventory['version'] !== 1 ||
      inventory['authority'] !== 'normal-retained-runtime-surfaces' || inventory['observed'] !== true)
    throw new Error('Same-frame menu/run requires the normal arranged inventory authority');
  booleanValue(inventory['complete'], 'arranged inventory complete');
  const cursor = object(evidence['presentationCursor'], 'normal presentation cursor');
  const inventoryCursor = object(inventory['presentationCursor'], 'inventory presentation cursor');
  if (Object.keys(inventoryCursor).length !== PRESENTATION_CURSOR_FIELDS.length ||
      PRESENTATION_CURSOR_FIELDS.some((key) => inventoryCursor[key] !== cursor[key]) ||
      inventory['presentedFrameCount'] !== evidence['presentedFrameCount'])
    throw new Error('Same-frame menu/run inventory cursor or successful frame differs');
  const surfaces = array(inventory['surfaces'], 'arranged inventory surfaces').map(
    (item) => object(item, 'arranged inventory surface'),
  ).filter((surface) => surface['scope'] === 'workbench-window');
  if (surfaces.length !== 1) throw new Error('Same-frame menu/run requires one workbench-window surface');
  const surface = surfaces[0]!;
  if (surface['schema'] !== 'dev.zircon.editor.normal-surface-arranged-inventory' || surface['version'] !== 1 ||
      surface['documentId'] !== 'res://ui/editor/unreal/level_editor_window.zui' ||
      surface['coordinateSpace'] !== 'surface-logical' || surface['frameAuthority'] !== 'retained-host-projection')
    throw new Error('Same-frame menu/run surface has the wrong owner or frame authority');
  const generation = surface['sourceFrameGeneration'];
  if (typeof generation !== 'number' || !Number.isSafeInteger(generation) || generation < 0)
    throw new Error('Same-frame menu/run surface generation is missing');
  const rows = array(surface['nodes'], 'arranged surface nodes').map((row) => object(row, 'arranged node'));
  const nativeById = new Map<string, JsonObject>();
  const nativeSurfaceIds = new Set<number>();
  for (const row of rows) {
    const id = nonempty(row['nodeId'], 'arranged node ID');
    const surfaceId = row['surfaceNodeId'];
    if (nativeById.has(id)) throw new Error(`Duplicate arranged node ID ${id}`);
    if (surfaceId !== undefined && surfaceId !== null && (typeof surfaceId !== 'number' ||
        !Number.isSafeInteger(surfaceId) || surfaceId < 1 || nativeSurfaceIds.has(surfaceId)))
      throw new Error(`Missing or duplicate arranged node ID ${id}`);
    if (typeof surfaceId === 'number') nativeSurfaceIds.add(surfaceId);
    nativeById.set(id, row);
  }
  const nativeRoots = rows.filter((row) => row['controlId'] === 'WorkbenchWindowRoot');
  if (nativeRoots.length !== 1) throw new Error('Same-frame menu/run requires one actual WorkbenchWindowRoot');
  const nativeRoot = nativeRoots[0]!;
  const nativeRootIdentity = observedIdentity(nativeRoot);
  const rootIds = array(surface['rootIds'], 'arranged surface roots');
  if (nativeRootIdentity.sourcePath !== WORKBENCH_WINDOW_SOURCE || nativeRootIdentity.sourceNodeId !== 'root' ||
      nativeRoot['component'] !== 'Overlay' || nativeRoot['parent'] !== null ||
      rootIds.length !== 1 || rootIds[0] !== nativeRoot['surfaceNodeId'])
    throw new Error('Same-frame menu/run actual owner root differs');

  // A real review host adds its own mount callsites. Relativize only against
  // each side's observed owner root, retaining every complete descendant step.
  const reviewNodes = document.nodes ?? {};
  const reviewParents = new Map<string, string | null>();
  const visit = (id: string, parent: string | null): void => {
    if (!reviewNodes[id] || reviewParents.has(id))
      throw new Error('Same-frame menu/run review graph has a missing, cyclic or shared node');
    reviewParents.set(id, parent);
    for (const child of reviewNodes[id].children ?? []) visit(child.node, id);
  };
  visit(nonempty(document.root?.node, 'review host root'), null);
  const reviewRoots = [...reviewParents.keys()].filter((id) => reviewNodes[id].control_id === 'WorkbenchWindowRoot');
  if (reviewRoots.length !== 1) throw new Error('Same-frame menu/run requires one review WorkbenchWindowRoot');
  const reviewRootId = reviewRoots[0]!;
  const reviewRoot = reviewNodes[reviewRootId];
  const reviewRootIdentity = reviewIdentity(reviewRoot);
  if (reviewRootIdentity.sourcePath !== WORKBENCH_WINDOW_SOURCE || reviewRootIdentity.sourceNodeId !== 'root' ||
      reviewRoot.component !== 'Overlay')
    throw new Error('Same-frame menu/run review owner root differs');
  const nativeTargets = new Map<string, JsonObject>();
  for (const row of rows) {
    if (!MENU_RUN_COMPONENTS.has(String(row['component'])) ||
        !MENU_RUN_SOURCES.has(sourcePath(row['sourcePath']))) continue;
    assertNativeOwner(row, nativeRoot, nativeById);
    const identity = observedIdentity(row);
    const key = menuRunKey(identity, nativeRootIdentity.instancePath);
    if (nativeTargets.has(key)) throw new Error('Duplicate arranged menu/run source identity');
    nativeTargets.set(key, row);
  }
  // Validate all joins/properties before touching the review clone.
  const patches: Array<{ node: ZuiNode; props: ZuiTable; observation: JsonObject }> = [];
  const reviewKeys = new Set<string>();
  for (const [id] of reviewParents) {
    const node = reviewNodes[id];
    if (!MENU_RUN_COMPONENTS.has(node.component) ||
        !MENU_RUN_SOURCES.has(String(node['penpot_review_source_path']))) continue;
    let owner: string | null | undefined = id;
    while (owner !== undefined && owner !== null && owner !== reviewRootId) owner = reviewParents.get(owner);
    if (owner !== reviewRootId) throw new Error('Review menu/run node is outside its actual owner root');
    const identity = reviewIdentity(node), key = menuRunKey(identity, reviewRootIdentity.instancePath);
    if (reviewKeys.has(key)) throw new Error('Duplicate review menu/run source identity');
    reviewKeys.add(key);
    const row = nativeTargets.get(key);
    if (!row || row['component'] !== node.component)
      throw new Error(`Missing actual menu/run source target ${identity.sourceNodeId}/${identity.controlId}`);
    const projected = object(row['projected'], 'arranged node projection');
    const raw = object(object(row['rawMetadata'], 'arranged raw metadata')['attributes'], 'arranged attributes');
    const properties = object(projected['properties'], 'arranged projected properties');
    const selfVisible = booleanValue(row['selfVisible'], 'arranged selfVisible');
    const enabled = booleanValue(row['enabled'], 'arranged enabled');
    const projectedDisabled = Object.hasOwn(projected, 'disabled')
      ? booleanValue(projected['disabled'], 'projected.disabled') : undefined;
    const rawDisabled = Object.hasOwn(raw, 'disabled') ? booleanValue(raw['disabled'], 'attributes.disabled') : undefined;
    if (rawDisabled !== undefined && projectedDisabled !== undefined && rawDisabled !== projectedDisabled)
      throw new Error('Menu/run raw and projected disabled sources conflict');
    const disabled = !enabled || projectedDisabled === true;
    const props: ZuiTable = { visible: selfVisible, visibility: selfVisible ? 'visible' : 'collapsed',
      disabled, enabled };
    for (const property of MENU_RUN_PROPERTIES) {
      if (Object.hasOwn(raw, property) && Object.hasOwn(properties, property) &&
          !sameObservedValue(raw[property], properties[property]))
        throw new Error(`Menu/run raw and projected ${property} sources conflict`);
      if (Object.hasOwn(raw, property)) props[property] = menuRunProperty(property, raw[property]);
      else if (Object.hasOwn(properties, property)) props[property] = menuRunProperty(property, properties[property]);
    }
    // PopupMenu's generic preview alias must not retain a stale authored open.
    if (typeof props['popup_open'] === 'boolean') props['open'] = props['popup_open'];
    patches.push({ node, props, observation: { sourcePath: row['sourcePath'], sourceNodeId: row['sourceNodeId'],
      instancePath: structuredClone(row['instancePath']), controlId: row['controlId'], surfaceNodeId: row['surfaceNodeId'],
      selfVisible, enabled, projectedDisabled: projectedDisabled ?? null, rawDisabled: rawDisabled ?? null,
      disabledFromEnabled: !enabled, disabledFromProjected: projectedDisabled === true, effectiveDisabled: disabled } });
  }
  if (reviewKeys.size === 0 || reviewKeys.size !== nativeTargets.size)
    throw new Error('Arranged menu/run targets do not match the actual review owner subtree');
  const contextPatches: Array<{ node: ZuiNode; props: ZuiTable; observation: JsonObject }> = [];
  for (const [sourceNodeId, controlId, component, parentSourceNodeId] of [
    ['project_context', 'UnrealProjectDocumentContext', 'HorizontalGroup', 'editor_content'],
    ['project_context_text', 'UnrealProjectDocumentContextText', 'Label', 'project_context'],
  ]) {
    const matches = rows.filter((row) => sourcePath(row['sourcePath']) === WORKBENCH_WINDOW_SOURCE &&
      row['sourceNodeId'] === sourceNodeId);
    const reviewMatches = [...reviewParents.keys()].filter((id) =>
      reviewNodes[id]['penpot_review_source_path'] === WORKBENCH_WINDOW_SOURCE &&
      reviewNodes[id]['penpot_review_source_node_id'] === sourceNodeId);
    if (matches.length !== 1 || reviewMatches.length !== 1)
      throw new Error('Project context requires one actual and review Source target: ' + sourceNodeId);
    const row = matches[0]!, id = reviewMatches[0]!, node = reviewNodes[id];
    const expected: ObservedIdentity = { sourcePath: WORKBENCH_WINDOW_SOURCE, sourceNodeId,
      instancePath: [], controlId };
    if (row['component'] !== component || node.component !== component ||
        menuRunKey(observedIdentity(row), nativeRootIdentity.instancePath) !== menuRunKey(expected, []) ||
        menuRunKey(reviewIdentity(node), reviewRootIdentity.instancePath) !== menuRunKey(expected, []))
      throw new Error('Project context Source identity or complete owner invocation differs');
    assertNativeOwner(row, nativeRoot, nativeById);
    const nativeParent = object(row['parent'], 'project context actual parent');
    const reviewParent = reviewNodes[reviewParents.get(id) ?? ''];
    if (!reviewParent || sourcePath(nativeParent['sourcePath']) !== WORKBENCH_WINDOW_SOURCE ||
        nativeParent['sourceNodeId'] !== parentSourceNodeId ||
        !sameObservedValue(sourceSteps(nativeParent['instancePath']), nativeRootIdentity.instancePath) ||
        reviewParent['penpot_review_source_path'] !== WORKBENCH_WINDOW_SOURCE ||
        reviewParent['penpot_review_source_node_id'] !== parentSourceNodeId ||
        !sameObservedValue(reviewIdentity(reviewParent).instancePath, reviewRootIdentity.instancePath))
      throw new Error('Project context is outside its exact authored parent');
    let ancestor: string | null | undefined = id;
    while (ancestor !== null && ancestor !== undefined && ancestor !== reviewRootId) ancestor = reviewParents.get(ancestor);
    if (ancestor !== reviewRootId) throw new Error('Project context is outside its actual review owner root');
    const raw = object(object(row['rawMetadata'], 'project context metadata')['attributes'], 'project context attributes');
    const properties = object(object(row['projected'], 'project context projection')['properties'], 'project context properties');
    const selfVisible = booleanValue(row['selfVisible'], 'project context selfVisible');
    const props: ZuiTable = { visible: selfVisible, visibility: selfVisible ? 'visible' : 'collapsed' };
    for (const key of ['visible', 'visibility', 'text']) {
      if (Object.hasOwn(raw, key) && Object.hasOwn(properties, key) && !sameObservedValue(raw[key], properties[key]))
        throw new Error('Project context raw and projected property sources conflict: ' + key);
    }
    if (sourceNodeId === 'project_context') {
      // set_visible publishes both source properties. An ancestor's clipping or
      // bounds must never be converted into this node's authored visibility.
      if (booleanValue(raw['visible'], 'project context visible') !== selfVisible ||
          booleanValue(properties['visible'], 'project context projected visible') !== selfVisible ||
          raw['visibility'] !== props['visibility'] || properties['visibility'] !== props['visibility'])
        throw new Error('Project context visibility differs from its actual Source properties');
    } else {
      const text = stringValue(raw['text'], 'project context text');
      if (text.length > 4096 || properties['text'] !== text)
        throw new Error('Project context text differs from its actual Source properties');
      props['text'] = text;
    }
    contextPatches.push({ node, props, observation: { sourcePath: row['sourcePath'], sourceNodeId,
      instancePath: structuredClone(row['instancePath']), controlId, surfaceNodeId: row['surfaceNodeId'],
      selfVisible, properties: structuredClone(props) } });
  }
  const footerPatches: Array<{ node: ZuiNode; props: ZuiTable; observation: JsonObject }> = [];
  const footerInvocation = [{ sourcePath: WORKBENCH_WINDOW_SOURCE, sourceNodeId: 'status_bar' }];
  const assertFooterMetadata = (row: JsonObject): void => {
    const raw = object(row['rawMetadata'], 'footer raw metadata');
    if (!sameObservedValue(observedIdentity(row), observedIdentity({ sourcePath: raw['source_path'],
        sourceNodeId: raw['source_node_id'], instancePath: raw['instance_path'], controlId: raw['control_id'] })) ||
        raw['component'] !== row['component'])
      throw new Error('Footer raw and projected Source identities differ');
  };
  for (const [sourceNodeId, controlId, component, property] of FOOTER_STATE_TARGETS) {
    const matches = rows.filter((row) => sourcePath(row['sourcePath']) === STATUS_BAR_SOURCE &&
      row['sourceNodeId'] === sourceNodeId);
    const reviewMatches = [...reviewParents.keys()].filter((id) =>
      reviewNodes[id]['penpot_review_source_path'] === STATUS_BAR_SOURCE &&
      reviewNodes[id]['penpot_review_source_node_id'] === sourceNodeId);
    if (matches.length !== 1 || reviewMatches.length !== 1)
      throw new Error('Footer requires one actual and review Source target: ' + sourceNodeId);
    const row = matches[0]!, id = reviewMatches[0]!, node = reviewNodes[id];
    const expected: ObservedIdentity = { sourcePath: STATUS_BAR_SOURCE, sourceNodeId,
      instancePath: footerInvocation, controlId };
    if (row['component'] !== component || node.component !== component ||
        menuRunKey(observedIdentity(row), nativeRootIdentity.instancePath) !== menuRunKey(expected, []) ||
        menuRunKey(reviewIdentity(node), reviewRootIdentity.instancePath) !== menuRunKey(expected, []))
      throw new Error('Footer Source identity or complete owner invocation differs');
    assertNativeOwner(row, nativeRoot, nativeById);
    assertFooterMetadata(row);
    const projected = object(row['projected'], 'footer projected node');
    const parent = nativeById.get(nonempty(projected['parentId'], 'footer projected parent'));
    const reviewParent = reviewNodes[reviewParents.get(id) ?? ''];
    const parentIdentity: ObservedIdentity = { sourcePath: STATUS_BAR_SOURCE, sourceNodeId: 'root',
      instancePath: footerInvocation, controlId: 'WorkbenchWindowStatusBarRegion' };
    if (!parent || parent['component'] !== 'HorizontalGroup' || !reviewParent || reviewParent.component !== 'HorizontalGroup' ||
        menuRunKey(observedIdentity(parent), nativeRootIdentity.instancePath) !== menuRunKey(parentIdentity, []) ||
        menuRunKey(reviewIdentity(reviewParent), reviewRootIdentity.instancePath) !== menuRunKey(parentIdentity, []))
      throw new Error('Footer target is outside its exact authored status-bar parent');
    assertFooterMetadata(parent);
    let ancestor: string | null | undefined = id;
    while (ancestor !== null && ancestor !== undefined && ancestor !== reviewRootId) ancestor = reviewParents.get(ancestor);
    if (ancestor !== reviewRootId) throw new Error('Footer is outside its actual review owner root');
    const raw = object(object(row['rawMetadata'], 'footer metadata')['attributes'], 'footer attributes');
    const properties = object(projected['properties'], 'footer projected properties');
    // Both actual Source observations must agree. Do not derive counts from
    // Ready text, GPU-elided text, paint colors, or observed rectangles.
    if (!Object.hasOwn(raw, property) || !Object.hasOwn(properties, property) ||
        !sameObservedValue(raw[property], properties[property]))
      throw new Error('Footer raw and projected state differs: ' + property);
    const observed = property === 'checked' ? booleanValue(raw[property], 'footer checked') :
      stringValue(raw[property], 'footer text');
    if (typeof observed === 'string' && observed.length > 4096)
      throw new Error('Footer text exceeds the bounded Source value');
    const props: ZuiTable = { [property]: observed };
    footerPatches.push({ node, props, observation: { sourcePath: row['sourcePath'], sourceNodeId,
      instancePath: structuredClone(row['instancePath']), controlId, surfaceNodeId: row['surfaceNodeId'],
      parentId: projected['parentId'], properties: structuredClone(props) } });
  }
  for (const { node, props, observation } of patches) {
    node.props = { ...(node.props ?? {}), ...structuredClone(props) };
    node.state = { ...(node.state ?? {}), ...structuredClone(props) };
    node['penpot_review_same_frame_menu_run'] = observation;
  }
  for (const { node, props, observation } of contextPatches) {
    node.props = { ...(node.props ?? {}), ...structuredClone(props) };
    node.state = { ...(node.state ?? {}), ...structuredClone(props) };
    node['penpot_review_same_frame_project_context'] = observation;
  }
  for (const { node, props, observation } of footerPatches) {
    node.props = { ...(node.props ?? {}), ...structuredClone(props) };
    node.state = { ...(node.state ?? {}), ...structuredClone(props) };
    node['penpot_review_same_frame_footer'] = observation;
  }
  document['penpot_review_same_frame_menu_run'] = { authority: inventory['authority'], scope: surface['scope'],
    presentationCursor: structuredClone(cursor), presentedFrameCount: evidence['presentedFrameCount'],
    sourceFrameGeneration: generation, inventoryComplete: inventory['complete'],
    inventoryAudit: structuredClone(inventory['inventoryAudit']), pending: structuredClone(inventory['pending']),
    surfaceInventoryAudit: structuredClone(surface['inventoryAudit']), targetCount: patches.length };
  document['penpot_review_same_frame_project_context'] = {
    authority: inventory['authority'], scope: surface['scope'], presentationCursor: structuredClone(cursor),
    presentedFrameCount: evidence['presentedFrameCount'], sourceFrameGeneration: generation,
    targetCount: contextPatches.length,
  };
  document['penpot_review_same_frame_footer'] = {
    authority: inventory['authority'], scope: surface['scope'], presentationCursor: structuredClone(cursor),
    presentedFrameCount: evidence['presentedFrameCount'], sourceFrameGeneration: generation,
    inventoryComplete: inventory['complete'], targetCount: footerPatches.length,
  };
}

function sourcePath(value: unknown): string {
  if (typeof value !== 'string') return '';
  const path = value.startsWith('res://ui/editor/')
    ? 'zircon_editor/assets/' + value.slice('res://'.length) : value;
  return path.startsWith('zircon_editor/assets/ui/editor/') && !path.includes('\\') &&
    path.split('/').every((part) => part !== '.' && part !== '..' && part.length > 0) ? path : '';
}

function nonempty(value: unknown, label: string): string {
  if (typeof value !== 'string' || value.trim().length === 0) throw new Error(`${label} is missing`);
  return value;
}

function sourceSteps(value: unknown): SourceStep[] {
  let decoded = value;
  if (typeof decoded === 'string') {
    try { decoded = JSON.parse(decoded); } catch { throw new Error('Menu/run instancePath is not a callsite list'); }
  }
  return array(decoded, 'menu/run instancePath').map((item) => {
    const step = object(item, 'menu/run callsite');
    const path = sourcePath(step['sourcePath']);
    if (Object.keys(step).length !== 2 || !path) throw new Error('Menu/run callsite requires its complete source step');
    return { sourcePath: path, sourceNodeId: nonempty(step['sourceNodeId'], 'callsite source node') };
  });
}

function observedIdentity(row: JsonObject): ObservedIdentity {
  const path = sourcePath(row['sourcePath']);
  if (!path) throw new Error('Menu/run source path is missing or outside the editor owner');
  return { sourcePath: path, sourceNodeId: nonempty(row['sourceNodeId'], 'menu/run source node'),
    instancePath: sourceSteps(row['instancePath']), controlId: row['controlId'] === null ? null : nonempty(row['controlId'], 'menu/run control') };
}

function reviewIdentity(node: ZuiNode): ObservedIdentity {
  return observedIdentity({ sourcePath: node['penpot_review_source_path'], sourceNodeId: node['penpot_review_source_node_id'],
    instancePath: node['penpot_review_instance_path'], controlId: node.control_id ?? null });
}

function menuRunKey(identity: ObservedIdentity, prefix: SourceStep[]): string {
  if (prefix.length > identity.instancePath.length || prefix.some((step, index) =>
    step.sourcePath !== identity.instancePath[index]?.sourcePath || step.sourceNodeId !== identity.instancePath[index]?.sourceNodeId))
    throw new Error('Menu/run instancePath is outside its complete owner prefix');
  return JSON.stringify([identity.sourcePath, identity.sourceNodeId, identity.instancePath.slice(prefix.length), identity.controlId]);
}

function assertNativeOwner(row: JsonObject, root: JsonObject, byId: Map<string, JsonObject>): void {
  const seen = new Set<JsonObject>();
  let current = row;
  while (true) {
    if (seen.has(current)) throw new Error('Cyclic menu/run arranged owner ancestry');
    seen.add(current);
    const surfaceId = current['surfaceNodeId'];
    if (typeof surfaceId !== 'number' || !Number.isSafeInteger(surfaceId) || surfaceId < 1)
      throw new Error('Menu/run target or ancestor lacks an actual runtime surface node ID');
    const projected = object(current['projected'], 'actual arranged projection');
    if (!sameObservedValue(observedIdentity(current), observedIdentity(projected)) ||
        projected['nodeId'] !== current['nodeId'] || projected['surfaceNodeId'] !== current['surfaceNodeId'] ||
        projected['component'] !== current['component'])
      throw new Error('Menu/run arranged and projected source identities differ');
    if (current === root) {
      if (projected['parentId'] !== null) throw new Error('Menu/run projected owner is not a root');
      return;
    }
    const parent = object(current['parent'], 'menu/run actual parent');
    const parentRow = byId.get(nonempty(parent['nodeId'], 'menu/run parent node'));
    if (!parentRow || projected['parentId'] !== parentRow['nodeId'] ||
        sourcePath(parent['sourcePath']) !== sourcePath(parentRow['sourcePath']) ||
        parent['sourceNodeId'] !== parentRow['sourceNodeId'] ||
        !sameObservedValue(sourceSteps(parent['instancePath']), sourceSteps(parentRow['instancePath'])) ||
        array(parentRow['children'], 'actual parent children').filter((id) => id === current['surfaceNodeId']).length !== 1)
      throw new Error('Menu/run node is outside its actual owner parent chain');
    current = parentRow;
  }
}

function sameObservedValue(a: unknown, b: unknown): boolean {
  if (Array.isArray(a)) return Array.isArray(b) && a.length === b.length && a.every((item, index) => sameObservedValue(item, b[index]));
  if (isRecord(a)) return isRecord(b) && Object.keys(a).length === Object.keys(b).length &&
    Object.keys(a).every((key) => Object.hasOwn(b, key) && sameObservedValue(a[key], b[key]));
  return a === b;
}

function menuRunProperty(key: typeof MENU_RUN_PROPERTIES[number], value: unknown): ZuiValue {
  if (['popup_open', 'selected', 'checked'].includes(key)) return booleanValue(value, `menu/run ${key}`);
  if (['text', 'value_text', 'label'].includes(key)) return stringValue(value, `menu/run ${key}`);
  if (key === 'value') {
    if (typeof value === 'string' || typeof value === 'boolean') return value;
    return numberValue(value, 'menu/run value');
  }
  const jsonValue = (item: unknown): item is ZuiValue => typeof item === 'string' || typeof item === 'boolean' ||
    (typeof item === 'number' && Number.isFinite(item)) || (Array.isArray(item) && item.every(jsonValue)) ||
    (isRecord(item) && Object.values(item).every(jsonValue));
  if (!Array.isArray(value) || !value.every((item) => (typeof item === 'string' || isRecord(item)) && jsonValue(item)))
    throw new Error('Menu/run menu_items requires observed menu strings or records');
  return structuredClone(value) as ZuiValue[];
}

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
  setVisibilityByControl(
    document,
    INSPECTOR_SOURCE,
    'UnrealDetailsSelectionHint',
    hasSelection ? 'collapsed' : 'visible',
  );
  for (const controlId of [
    'UnrealDetailsNameArea', 'WorkbenchInspectorFilter', 'RightDrawerContentRoot',
  ])
    setVisibilityByControl(document, INSPECTOR_SOURCE, controlId, hasSelection ? 'visible' : 'collapsed');
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
