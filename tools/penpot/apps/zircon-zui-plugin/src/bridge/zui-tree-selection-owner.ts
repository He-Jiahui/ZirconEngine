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

/** The actual nearest Tree needs its own uniquely scoped Native semantic receipt. */
export function treeOwnerFocusedFromPublication(document: ZuiDocument, nodeId: string, value: unknown, parents: Parents = parentsFor(document)): boolean | undefined {
  const ownerId = treeSelectionOwner(document, nodeId, parents);
  const owner = ownerId ? document.nodes?.[ownerId] : undefined;
  const identity = owner ? sourceOwner(owner) : undefined;
  const nodes = normalTreeOwnerNodes(value);
  if (!identity || !nodes) return undefined;
  if (Object.values(document.nodes ?? {}).filter((node) => {
    const candidate = sourceOwner(node);
    return candidate && sameSource(candidate, identity);
  }).length !== 1) return undefined;
  const matches = nodes.filter((node) => completeSourceOwner(node) && sameSource(node, identity));
  if (matches.length !== 1) return undefined;
  const semantic = matches[0]!;
  const publication = nativeTreeOwnerFocusPublication(semantic['treeOwnerFocusPublication']);
  if (!publication || !sameSource(publication.sourceOwner, identity) ||
    semantic['treeOwnerFocused'] !== publication.value) return undefined;
  return publication.value;
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
