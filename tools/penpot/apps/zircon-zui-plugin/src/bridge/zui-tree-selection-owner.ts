import type { ZuiDocument, ZuiNode, ZuiTable } from './zui-document';
import { nativePainterComponent } from './zui-native-painter-role';
import { styledPreviewNode } from './zui-style-projection';
import { authoredPaintPreviewNode } from './zui-paint-properties';
import { controlProperties } from './zui-control-properties';

type Parents = ReadonlyMap<string, string>;
interface SourceOwner {
  sourcePath: string;
  sourceNodeId: string;
  instancePath: string;
  controlId: string;
}
export interface NativeTreeOwnerFocusPublication {
  authority: 'normal-template-tree-row-painter-input';
  sourceAuthority: 'UiSurface.focus/tree_selection_owner';
  observed: true;
  value: boolean;
  sourceOwner: SourceOwner;
  originComponent: SourceOwner;
  sourceOwnerMappingComplete: true;
  sourceFrame: { treeId: string; generation: number; ownerNodeId: number; rowPrototypeNodeId: number };
  paintedNativeControlId: string;
  submittedOwnerCommandIndices: number[];
}
interface CurrentInventoryTreeOwnerFocus {
  authority: 'current-inventory-projection';
  value: boolean;
}
const record = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object' && !Array.isArray(value);
const nonempty = (value: unknown): value is string =>
  typeof value === 'string' && value.trim().length > 0;

function parentsFor(document: ZuiDocument): Map<string, string> {
  const parents = new Map<string, string>();
  for (const [id, node] of Object.entries(document.nodes ?? {}))
    for (const child of node.children ?? []) parents.set(child.node, id);
  return parents;
}

/** Source ownership is independent of selection and authored focus flags. */
export function treeSelectionOwner(document: ZuiDocument, nodeId: string, parents: Parents = parentsFor(document)): string | undefined {
  const seen = new Set<string>();
  let current: string | undefined = nodeId;
  while (current && !seen.has(current)) {
    seen.add(current);
    const node = document.nodes?.[current];
    if (!node) return undefined;
    if (node.props?.['tree_selection_owner'] === true) return current;
    current = parents.get(current);
  }
  return undefined;
}

/** undefined is unknown authority; null is an explicit observed absence of focus. */
export function treeOwnerFocused(document: ZuiDocument, nodeId: string, focusedNodeId: string | null | undefined): boolean | undefined {
  if (focusedNodeId === undefined) return undefined;
  const parents = parentsFor(document);
  const owner = treeSelectionOwner(document, nodeId, parents);
  if (owner === undefined) return undefined;
  if (focusedNodeId !== null && !document.nodes?.[focusedNodeId])
    throw new Error(`Unknown live focus node ${focusedNodeId}`);
  return focusedNodeId !== null && treeSelectionOwner(document, focusedNodeId, parents) === owner;
}

function sourceOwner(node: ZuiNode): SourceOwner | undefined {
  const value = {
    sourcePath: node['penpot_review_source_path'],
    sourceNodeId: node['penpot_review_source_node_id'],
    instancePath: node['penpot_review_instance_path'],
    controlId: node.control_id,
  };
  return Object.values(value).every(nonempty) ? value as SourceOwner : undefined;
}
function sameSource(a: SourceOwner, b: SourceOwner): boolean {
  return a.sourcePath === b.sourcePath && a.sourceNodeId === b.sourceNodeId &&
    a.instancePath === b.instancePath && a.controlId === b.controlId;
}

interface SourceStep { sourcePath: string; sourceNodeId: string }
const workbenchSource = 'zircon_editor/assets/ui/editor/unreal/level_editor_window.zui';
function identityNeed(value: unknown): asserts value {
  if (!value) throw new Error('Tree focus owner identity is incomplete or ambiguous');
}
function canonicalPath(value: unknown): string {
  identityNeed(nonempty(value));
  const path = value.startsWith('res://ui/editor/')
    ? 'zircon_editor/assets/' + value.slice('res://'.length) : value;
  if (path.startsWith('zircon_editor/assets/ui/editor/'))
    identityNeed(!path.includes('\\') && path.split('/').every((part) => part && part !== '.' && part !== '..'));
  return path;
}
function invocationSteps(value: unknown): SourceStep[] {
  const parsed: unknown = typeof value === 'string' ? JSON.parse(value) : value;
  identityNeed(Array.isArray(parsed) && parsed.length <= 64);
  return parsed.map((step: unknown) => {
    identityNeed(record(step) && Object.keys(step).length === 2 && nonempty(step['sourceNodeId']));
    return { sourcePath: canonicalPath(step['sourcePath']), sourceNodeId: step['sourceNodeId'] };
  });
}
function startsWithSteps(steps: SourceStep[], prefix: SourceStep[]): boolean {
  return prefix.length <= steps.length && prefix.every((step, index) =>
    step.sourcePath === steps[index]!.sourcePath && step.sourceNodeId === steps[index]!.sourceNodeId);
}
function scopedKey(value: Record<string, unknown>, prefix: SourceStep[], replacement: SourceStep[] = prefix): string {
  identityNeed(nonempty(value['sourceNodeId']) && (value['controlId'] === null || nonempty(value['controlId'])));
  const steps = invocationSteps(value['instancePath']);
  identityNeed(startsWithSteps(steps, prefix));
  return JSON.stringify([canonicalPath(value['sourcePath']), value['sourceNodeId'],
    [...replacement, ...steps.slice(prefix.length)], value['controlId']]);
}

/** Rebase only complete invocation prefixes proved by both actual Workbench roots. */
function workbenchFocusMapping(document: ZuiDocument, ownerId: string, value: unknown, parents: Parents): {
  matches: (native: SourceOwner, review: SourceOwner) => boolean;
  sameReview: (a: SourceOwner, b: SourceOwner) => boolean;
  prove: (semantic: Record<string, unknown>, publication: NativeTreeOwnerFocusPublication) => void;
  projectedFocus: (originId: string) => CurrentInventoryTreeOwnerFocus;
} | undefined {
  const roots = Object.entries(document.nodes ?? {}).filter(([, node]) => node.control_id === 'WorkbenchWindowRoot');
  if (roots.length === 0) return undefined; // Existing non-Workbench component receipts keep their exact tuple contract.
  identityNeed(roots.length === 1 && record(value));
  const [rootId, root] = roots[0]!;
  const rootIdentity = sourceOwner(root);
  identityNeed(rootIdentity && root.component === 'Overlay' && rootIdentity.sourceNodeId === 'root' &&
    canonicalPath(rootIdentity.sourcePath) === workbenchSource);
  const reviewPrefix = invocationSteps(rootIdentity.instancePath);
  const reviewChain = (start: string): void => {
    const seen = new Set<string>();
    let current = start;
    while (true) {
      identityNeed(!seen.has(current)); seen.add(current);
      const node = document.nodes?.[current];
      identityNeed(node && nonempty(node['penpot_review_source_node_id']) &&
        canonicalPath(node['penpot_review_source_path']) &&
        startsWithSteps(invocationSteps(node['penpot_review_instance_path']), reviewPrefix));
      if (current === rootId) return;
      const owners = Object.entries(document.nodes ?? {}).filter(([, parent]) =>
        (parent.children ?? []).some((child) => child.node === current));
      identityNeed(owners.length === 1 && parents.get(current) === owners[0]![0]);
      current = owners[0]![0];
    }
  };
  reviewChain(ownerId);
  const business = value['workbenchStatePublication'];
  const presentation = record(business) ? business['workbenchPresentation'] : undefined;
  const fingerprint = record(presentation) ? presentation['sourceFingerprint'] : undefined;
  identityNeed(record(fingerprint) && canonicalPath(fingerprint['sourcePath']) === workbenchSource);
  const inventory = value['arrangedInventoryPublication'];
  identityNeed(record(inventory) && inventory['schema'] === 'dev.zircon.editor.normal-arranged-inventory' &&
    inventory['version'] === 1 && inventory['authority'] === 'normal-retained-runtime-surfaces' &&
    inventory['observed'] === true && typeof inventory['complete'] === 'boolean' &&
    actualCursor(inventory['presentationCursor']) && actualCursor(value['presentationCursor']) &&
    inventory['presentedFrameCount'] === value['presentedFrameCount']);
  const inventoryCursor = inventory['presentationCursor'], cursor = value['presentationCursor'];
  identityNeed(cursorFields.every((key) => inventoryCursor[key] === cursor[key]));
  const surfaces = inventory['surfaces'];
  identityNeed(Array.isArray(surfaces) && surfaces.every(record));
  const owners = surfaces.filter((surface) => surface['scope'] === 'workbench-window');
  identityNeed(owners.length === 1);
  const surface = owners[0]!;
  identityNeed(surface['schema'] === 'dev.zircon.editor.normal-surface-arranged-inventory' && surface['version'] === 1 &&
    surface['documentId'] === 'res://ui/editor/unreal/level_editor_window.zui' &&
    surface['coordinateSpace'] === 'surface-logical' && surface['frameAuthority'] === 'retained-host-projection' &&
    uint(surface['sourceFrameGeneration']));
  const rows = surface['nodes'];
  identityNeed(Array.isArray(rows) && rows.every(record));
  const nativeRoots = rows.filter((row) => row['controlId'] === 'WorkbenchWindowRoot');
  identityNeed(nativeRoots.length === 1);
  const nativeRoot = nativeRoots[0]!, nativePrefix = invocationSteps(nativeRoot['instancePath']);
  identityNeed(nativeRoot['component'] === 'Overlay' && nativeRoot['sourceNodeId'] === 'root' &&
    canonicalPath(nativeRoot['sourcePath']) === workbenchSource && nativeRoot['parent'] === null &&
    uint(nativeRoot['surfaceNodeId']) && nativeRoot['surfaceNodeId'] > 0 &&
    Array.isArray(surface['rootIds']) && surface['rootIds'].length === 1 && surface['rootIds'][0] === nativeRoot['surfaceNodeId']);
  const byId = new Map<string, Record<string, unknown>>(), surfaceIds = new Set<number>();
  for (const row of rows) {
    identityNeed(nonempty(row['nodeId']) && !byId.has(row['nodeId']));
    byId.set(row['nodeId'], row);
    if (row['surfaceNodeId'] !== null && row['surfaceNodeId'] !== undefined) {
      identityNeed(uint(row['surfaceNodeId']) && row['surfaceNodeId'] > 0 && !surfaceIds.has(row['surfaceNodeId']));
      surfaceIds.add(row['surfaceNodeId']);
    }
  }
  const nativeKey = (raw: Record<string, unknown>): string => scopedKey(raw, nativePrefix, reviewPrefix);
  const reviewKey = (raw: SourceOwner): string => scopedKey({ ...raw }, reviewPrefix);
  const sameHead = (a: Record<string, unknown>, b: Record<string, unknown>): boolean =>
    a['sourceNodeId'] === b['sourceNodeId'] && a['controlId'] === b['controlId'] &&
    nonempty(a['sourcePath']) && canonicalPath(a['sourcePath']) === canonicalPath(b['sourcePath']);
  const nativeMatches = (a: Record<string, unknown>, b: Record<string, unknown>): boolean =>
    sameHead(a, b) && nativeKey(a) === nativeKey(b);
  const matchesReview = (native: SourceOwner, review: SourceOwner): boolean =>
    sameHead({ ...native }, { ...review }) && nativeKey({ ...native }) === reviewKey(review);
  const nativeChain = (start: Record<string, unknown>): void => {
    const seen = new Set<string>();
    let row = start;
    while (true) {
      identityNeed(nonempty(row['nodeId']) && !seen.has(row['nodeId'])); seen.add(row['nodeId']);
      const projected = row['projected'];
      identityNeed(uint(row['surfaceNodeId']) && row['surfaceNodeId'] > 0 && record(projected) &&
        projected['nodeId'] === row['nodeId'] && projected['surfaceNodeId'] === row['surfaceNodeId'] &&
        projected['component'] === row['component'] && nativeKey(projected) === nativeKey(row));
      if (row === nativeRoot) { identityNeed(projected['parentId'] === null); return; }
      const parent = row['parent'];
      identityNeed(record(parent) && nonempty(parent['nodeId']) && projected['parentId'] === parent['nodeId']);
      const actualParent = byId.get(parent['nodeId']);
      identityNeed(actualParent && nativeKey({ ...parent, controlId: actualParent['controlId'] }) === nativeKey(actualParent));
      row = actualParent;
    }
  };
  nativeChain(nativeRoot);
  return {
    matches: matchesReview,
    sameReview: (a, b) => sameHead({ ...a }, { ...b }) && reviewKey(a) === reviewKey(b),
    projectedFocus: (originId) => {
      // A layout-only owner has no submitted painter command. Its independent
      // authority is the current Runtime projection, not a synthesized receipt.
      const inventoryAudit = inventory['inventoryAudit'], audit = surface['inventoryAudit'];
      identityNeed(record(inventoryAudit) && inventoryAudit['complete'] === true &&
        inventoryAudit['surfaceCount'] === surfaces.length && record(audit) && audit['complete'] === true &&
        Array.isArray(audit['issues']) && audit['issues'].length === 0 &&
        surface['rawNodeCount'] === rows.length && surface['arrangedNodeCount'] === rows.length &&
        surface['projectedNodeCount'] === rows.length && Number(surface['sourceFrameGeneration']) > 0 &&
        Array.isArray(surface['unarrangedNodes']) && surface['unarrangedNodes'].length === 0 &&
        Array.isArray(surface['unmatchedProjectionNodes']) && surface['unmatchedProjectionNodes'].length === 0);
      const reviewOwner = document.nodes?.[ownerId], reviewOrigin = document.nodes?.[originId];
      const ownerIdentity = reviewOwner ? sourceOwner(reviewOwner) : undefined;
      const originIdentity = reviewOrigin ? sourceOwner(reviewOrigin) : undefined;
      identityNeed(ownerIdentity && originIdentity && nativePainterComponent(reviewOrigin) === 'TreeRow' &&
        treeSelectionOwner(document, originId, parents) === ownerId);
      reviewChain(originId);
      const reviewOrigins = Object.values(document.nodes ?? {}).filter((node) => {
        const identity = sourceOwner(node);
        return identity && sameHead({ ...identity }, { ...originIdentity }) && reviewKey(identity) === reviewKey(originIdentity);
      });
      identityNeed(reviewOrigins.length === 1);
      const currentRows = (identity: SourceOwner): Record<string, unknown>[] => rows.filter((row) =>
        sameHead(row, { ...identity }) && nativeKey(row) === reviewKey(identity));
      const ownerRows = currentRows(ownerIdentity), originRows = currentRows(originIdentity);
      identityNeed(ownerRows.length === 1 && originRows.length === 1);
      const owner = ownerRows[0]!, origin = originRows[0]!;
      identityNeed(owner !== origin && origin['component'] === 'TreeRow');
      nativeChain(owner); nativeChain(origin);
      const currentProjection = (row: Record<string, unknown>) => {
        const raw = row['rawMetadata'], projected = row['projected'];
        identityNeed(record(raw) && record(raw['attributes']) && record(projected) && record(projected['properties']) &&
          raw['component'] === row['component'] && nativeMatches({ sourcePath: raw['source_path'],
            sourceNodeId: raw['source_node_id'], instancePath: raw['instance_path'], controlId: raw['control_id'] }, row));
        const attributes = raw['attributes'], properties = projected['properties'];
        const declared = attributes['tree_selection_owner'], projectedDeclaration = properties['tree_selection_owner'];
        identityNeed((declared === undefined || typeof declared === 'boolean') &&
          (projectedDeclaration === undefined || typeof projectedDeclaration === 'boolean') &&
          (declared === true) === (projectedDeclaration === true));
        return { attributes, properties };
      };
      let row = origin;
      let nearestOwner: Record<string, unknown> | undefined;
      const seen = new Set<string>();
      while (true) {
        identityNeed(nonempty(row['nodeId']) && !seen.has(row['nodeId'])); seen.add(row['nodeId']);
        const projection = currentProjection(row);
        if (!nearestOwner && projection.attributes['tree_selection_owner'] === true) nearestOwner = row;
        if (row === nativeRoot) break;
        const parent = row['parent'];
        identityNeed(record(parent) && nonempty(parent['nodeId']));
        const actualParent = byId.get(parent['nodeId']);
        identityNeed(actualParent && Array.isArray(actualParent['children']) &&
          actualParent['children'].filter((child) => child === row['surfaceNodeId']).length === 1);
        row = actualParent;
      }
      identityNeed(nearestOwner === owner);
      const ownerFocus = currentProjection(owner).properties['tree_owner_focused'];
      const originFocus = currentProjection(origin).properties['tree_owner_focused'];
      identityNeed(typeof ownerFocus === 'boolean' && typeof originFocus === 'boolean' && ownerFocus === originFocus);
      // rawMetadata may contain authored focus flags; only Runtime-derived
      // projected properties supply this value after the current binding proof.
      return { authority: 'current-inventory-projection', value: ownerFocus };
    },
    prove: (semantic, publication) => {
      const matches = rows.filter((row) => nativeMatches(row, semantic));
      identityNeed(matches.length === 1 && matches[0]!['surfaceNodeId'] === publication.sourceFrame.ownerNodeId);
      nativeChain(matches[0]!);
      const origins = rows.filter((row) => nativeMatches(row, { ...publication.originComponent }));
      identityNeed(origins.length === 1 && origins[0]!['surfaceNodeId'] === publication.sourceFrame.rowPrototypeNodeId);
      nativeChain(origins[0]!);
      const originParent = origins[0]!['parent'];
      identityNeed(record(originParent) && originParent['nodeId'] === matches[0]!['nodeId']);
      const reviewOrigins = Object.entries(document.nodes ?? {}).filter(([, node]) => {
        const identity = sourceOwner(node);
        return identity && matchesReview(publication.originComponent, identity);
      });
      identityNeed(reviewOrigins.length === 1 && parents.get(reviewOrigins[0]![0]) === ownerId);
      reviewChain(reviewOrigins[0]![0]);
      // Cached submitted nodes retain their immutable source generation while
      // unrelated arrangements may advance the surface. Join the actual commands.
      const commands = value['commands'];
      identityNeed(Array.isArray(commands) && commands.every(record));
      const byIndex = new Map<number, Record<string, unknown>>();
      for (const command of commands) {
        identityNeed(uint(command['commandIndex']) && !byIndex.has(command['commandIndex']));
        byIndex.set(command['commandIndex'], command);
      }
      const ownerCommands = commands.filter((command) => record(command['source']) && nativeMatches(command['source'], semantic));
      identityNeed(ownerCommands.length === publication.submittedOwnerCommandIndices.length &&
        ownerCommands.every((command) => publication.submittedOwnerCommandIndices.includes(command['commandIndex'] as number)));
      const owner = matches[0]!, parent = owner['parent'];
      identityNeed(record(parent) && nonempty(parent['nodeId']));
      const actualParent = byId.get(parent['nodeId']);
      identityNeed(actualParent && publication.sourceFrame.treeId === `template.v2.${surface['documentId']}` &&
        publication.sourceFrame.generation > 0 && publication.sourceFrame.generation <= Number(surface['sourceFrameGeneration']));
      for (const index of publication.submittedOwnerCommandIndices) {
        const command = byIndex.get(index), source = command?.['source'];
        identityNeed(record(source) && nativeMatches(source, semantic) && semantic['nodeId'] === owner['nodeId']);
        const reference = source['sourceRef'];
        identityNeed(record(reference) && Object.keys(reference).length === 7 && reference['authority'] === 'surface-node-owner' &&
          reference['primitiveIdentity'] === false && reference['treeId'] === publication.sourceFrame.treeId &&
          reference['generation'] === publication.sourceFrame.generation && reference['nodeId'] === publication.sourceFrame.ownerNodeId &&
          uint(reference['nodeCommandIndex']) && uint(reference['fragmentIndex']) && source['parentNodeId'] === actualParent['nodeId'] &&
          nativeKey({ sourcePath: source['parentSourcePath'], sourceNodeId: source['parentSourceNodeId'],
            instancePath: source['parentInstancePath'], controlId: actualParent['controlId'] }) === nativeKey(actualParent));
      }
    },
  };
}

const cursorFields = ['structure', 'geometry', 'hitTest', 'interaction', 'viewport', 'theme', 'diagnostics'] as const;
const uint = (value: unknown): value is number =>
  typeof value === 'number' && Number.isSafeInteger(value) && value >= 0;

function completeSourceOwner(value: unknown): value is SourceOwner {
  return record(value) && ['sourcePath', 'sourceNodeId', 'instancePath', 'controlId']
    .every((key) => nonempty(value[key]));
}

function actualCursor(value: unknown): value is Record<string, number> {
  return record(value) && Object.keys(value).length === cursorFields.length &&
    cursorFields.every((key) => Object.hasOwn(value, key) && uint(value[key]));
}

/** Keep the source publication in its successful normal present's business packet. */
function normalTreeOwnerNodes(value: unknown): Record<string, unknown>[] | undefined {
  if (!record(value) || !Array.isArray(value['semanticNodes']) ||
    !actualCursor(value['presentationCursor']) || !uint(value['presentedFrameCount']) ||
    value['presentedFrameCount'] === 0) return undefined;
  const business = value['workbenchStatePublication'];
  if (!record(business) || business['authority'] !== 'normal-retained-workbench-projection' ||
    business['observed'] !== true || !actualCursor(business['presentationCursor']) ||
    business['presentedFrameCount'] !== value['presentedFrameCount']) return undefined;
  const cursor = value['presentationCursor'];
  const businessCursor = business['presentationCursor'];
  if (!cursorFields.every((key) => cursor[key] === businessCursor[key])) return undefined;
  if (!value['semanticNodes'].every(record)) return undefined;
  return value['semanticNodes'];
}

/** This validates Native R46 evidence; no pointer id or focus-visible supplies a bool. */
export function nativeTreeOwnerFocusPublication(value: unknown): NativeTreeOwnerFocusPublication | undefined {
  if (!record(value) || value['authority'] !== 'normal-template-tree-row-painter-input' ||
    value['sourceAuthority'] !== 'UiSurface.focus/tree_selection_owner' ||
    value['observed'] !== true || value['sourceOwnerMappingComplete'] !== true ||
    typeof value['value'] !== 'boolean' || !completeSourceOwner(value['sourceOwner']) ||
    !completeSourceOwner(value['originComponent'])) return undefined;
  const frame = value['sourceFrame'];
  if (!record(frame) || Object.keys(frame).length !== 4 || !nonempty(frame['treeId']) ||
    !uint(frame['generation']) || !uint(frame['ownerNodeId']) || !uint(frame['rowPrototypeNodeId']) ||
    frame['ownerNodeId'] === frame['rowPrototypeNodeId']) return undefined;
  const control = value['paintedNativeControlId'];
  if (!nonempty(control) || !control.startsWith('NativeHierarchyEntity:') ||
    !nonempty(control.slice('NativeHierarchyEntity:'.length))) return undefined;
  const indices = value['submittedOwnerCommandIndices'];
  if (!Array.isArray(indices) || indices.length === 0 || !indices.every(uint) ||
    new Set(indices).size !== indices.length) return undefined;
  return value as unknown as NativeTreeOwnerFocusPublication;
}

/** The nearest Tree needs a scoped painter receipt or its current Runtime projection. */
export function treeOwnerFocusedFromPublication(document: ZuiDocument, nodeId: string, value: unknown, parents: Parents = parentsFor(document)): boolean | undefined {
  const ownerId = treeSelectionOwner(document, nodeId, parents);
  const owner = ownerId ? document.nodes?.[ownerId] : undefined;
  const identity = owner ? sourceOwner(owner) : undefined;
  const nodes = normalTreeOwnerNodes(value);
  if (!identity || !nodes) return undefined;
  try {
    const mapping = workbenchFocusMapping(document, ownerId!, value, parents);
    const matchesSource = mapping?.matches ?? sameSource;
    const sameReview = mapping?.sameReview ?? sameSource;
    if (Object.values(document.nodes ?? {}).filter((node) => {
      const candidate = sourceOwner(node);
      return candidate && sameReview(candidate, identity);
    }).length !== 1) return undefined;
    const matches = nodes.filter((node) => completeSourceOwner(node) && matchesSource(node, identity));
    if (matches.length === 0) {
      // A malformed semantic tuple must not downgrade a receipt that still
      // explicitly claims this complete owner to the independent authority.
      if (nodes.some((node) => {
        const publication = node['treeOwnerFocusPublication'];
        return record(publication) && completeSourceOwner(publication['sourceOwner']) &&
          matchesSource(publication['sourceOwner'], identity);
      })) return undefined;
      return mapping?.projectedFocus(nodeId).value;
    }
    if (matches.length !== 1) return undefined;
    const semantic = matches[0]!;
    const publication = nativeTreeOwnerFocusPublication(semantic['treeOwnerFocusPublication']);
    if (!publication || !matchesSource(publication.sourceOwner, identity) ||
      semantic['treeOwnerFocused'] !== publication.value) return undefined;
    mapping?.prove(semantic, publication);
    return publication.value;
  } catch { return undefined; }
}

export function hasTreeSelectionRows(document: ZuiDocument): boolean {
  return Object.values(document.nodes ?? {}).some((node) => nativePainterComponent(node) === 'TreeRow');
}

/** Mutates only the review clone; author-provided derived flags never supply authority. */
export function deriveTreeSelectionOwnerReview(document: ZuiDocument, publication: unknown): void {
  const parents = parentsFor(document);
  for (const [id, node] of Object.entries(document.nodes ?? {})) {
    if (nativePainterComponent(node) !== 'TreeRow') continue;
    const props: ZuiTable = { ...node.props };
    const state = node.state ? { ...node.state } : undefined;
    delete props['tree_owner_focused'];
    if (state) delete state['tree_owner_focused'];
    node.props = props;
    node.state = state;
    const ancestors: ZuiNode[] = [];
    const seen = new Set<string>();
    let parent = parents.get(id);
    while (parent && !seen.has(parent)) {
      seen.add(parent);
      const ancestor = document.nodes?.[parent];
      if (!ancestor) break;
      ancestors.push(ancestor);
      parent = parents.get(parent);
    }
    const preview = authoredPaintPreviewNode(document, styledPreviewNode(document, node, ancestors), id);
    const values = { ...preview.props, ...preview.state };
    const marked = values['selected'] === true || values['checked'] === true;
    const focused = treeOwnerFocusedFromPublication(document, id, publication, parents);
    if (focused !== undefined) props['tree_owner_focused'] = focused;
    if (!marked) continue;
    const unavailable = values['disabled'] === true || values['enabled'] === false || values['loading'] === true;
    const property = focused === false ? 'inactive_selected_background_color' :
      values['hovered'] === true ? 'selected_hover_background_color' : 'selected_background_color';
    const authored = controlProperties(document, preview, `tree row ${id}`);
    const color = unavailable ? 'transparent' : authored.color([property],
      focused === false ? '#40576f' : authored.color(['selected_background_color'], '#0070e0'));
    // Inline review style wins over static stylesheet selectors and remains derived state.
    const self = node.style?.['self'];
    if (self !== undefined && !record(self)) throw new Error(`Invalid self style for tree row ${id}`);
    node.style = { ...node.style, self: { ...self, background_color: color } };
  }
}
