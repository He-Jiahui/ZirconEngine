import {
  ZuiDocumentError,
  cloneZuiDocument,
  validateZuiDocument,
  zuiNodes,
  zuiRootNodeIds,
  type ZuiDocument,
  type ZuiChildMount,
  type ZuiNode,
  type ZuiTable,
} from './zui-document';
import type {
  PenpotAssetSnapshot,
  PenpotShapeSnapshot,
  ProjectionContainer,
  ProjectionEditableState,
  ProjectionPadding,
  ProjectionPaint,
  ReconciledZuiDocument,
} from './penpot-projection-model';
import { writeAuthoredPaintProperty } from './zui-paint-properties';
import { applyNodeTextChanges, serializedColor } from './penpot-text-reconcile';
import { resolvedSelfStyle } from './zui-style-projection';
import { projectZuiDocument } from './penpot-projection';
import { reconcileSlotPadding } from './zui-slot-padding';
import { validateEditorButtonPaintEdits } from './penpot-editor-button-reconcile';
import { validateEditorFieldEdits } from './penpot-field-reconcile';
import { reconcileContainerGapEdits } from './penpot-container-gaps';

export function reconcileZuiDocument(
  source: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
): ReconciledZuiDocument {
  assertValidDocument(source);
  if (snapshot.assetId !== source.asset.id) {
    throw new ZuiDocumentError(
      `Penpot asset ${snapshot.assetId} does not match ZUI asset ${source.asset.id}.`,
    );
  }

  const sourceNodes = zuiNodes(source);
  const shapeByNode = new Map<string, PenpotShapeSnapshot>();
  for (const shape of snapshot.shapes) {
    if (shapeByNode.has(shape.nodeId)) {
      throw new ZuiDocumentError(
        `Duplicate semantic node ${shape.nodeId} in Penpot asset.`,
      );
    }
    shapeByNode.set(shape.nodeId, shape);
  }
  for (const nodeId of Object.keys(sourceNodes)) {
    if (!shapeByNode.has(nodeId)) {
      throw new ZuiDocumentError(
        `Missing semantic node ${nodeId} in Penpot asset.`,
      );
    }
  }
  for (const [nodeId, shape] of shapeByNode) {
    const sourceNode = sourceNodes[nodeId];
    if (!sourceNode) {
      throw new ZuiDocumentError(
        `Unknown semantic node ${nodeId} in Penpot asset.`,
      );
    }
    if (shape.component !== sourceNode.component) {
      throw new ZuiDocumentError(
        `Component metadata for node ${nodeId} changed from ${sourceNode.component} to ${shape.component}.`,
      );
    }
  }

  const desiredParent = validateSnapshotHierarchy(
    source,
    snapshot,
    shapeByNode,
  );
  const sourceParent = sourceParentIndex(sourceNodes);
  const document = cloneZuiDocument(source);
  const documentNodes = zuiNodes(document);
  const changes: string[] = [];
  applyHierarchy(document, snapshot, desiredParent, changes);
  const desiredMounts = childMountIndex(documentNodes);

  for (const shape of snapshot.shapes) {
    const node = documentNodes[shape.nodeId];
    const ancestors: ZuiNode[] = [];
    let parentId = desiredParent.get(shape.nodeId);
    while (parentId) {
      ancestors.push(documentNodes[parentId]);
      parentId = desiredParent.get(parentId);
    }
    applyEditableChanges(
      node,
      shape,
      changes,
      parentLayoutTransition(
        shape.nodeId,
        sourceParent,
        desiredParent,
        shapeByNode,
      ),
      desiredMounts.get(shape.nodeId),
      resolvedSelfStyle(document, node, ancestors),
    );
  }

  validateEditorButtonPaintEdits(document, snapshot, projectZuiDocument);
  validateEditorFieldEdits(document, snapshot, projectZuiDocument);
  const diagnostics = validateZuiDocument(document);
  const errors = diagnostics.filter(({ severity }) => severity === 'error');
  if (errors.length > 0) {
    throw new ZuiDocumentError(
      `Penpot edits produced an invalid .zui document: ${errors
        .map(({ message }) => message)
        .join('; ')}`,
      diagnostics,
    );
  }
  if (changes.length > 0) {
    diagnostics.push({
      severity: 'info',
      code: 'supported-edits-applied',
      message: `${changes.length} supported Penpot edits were applied to the ZUI document.`,
    });
  }
  return { document, diagnostics, changes };
}

function validateSnapshotHierarchy(
  source: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
  shapeByNode: Map<string, PenpotShapeSnapshot>,
): Map<string, string> {
  const desiredParent = new Map<string, string>();
  for (const shape of snapshot.shapes) {
    const uniqueChildren = new Set(shape.childNodeIds);
    if (uniqueChildren.size !== shape.childNodeIds.length) {
      throw new ZuiDocumentError(
        `Node ${shape.nodeId} contains duplicate semantic children.`,
      );
    }
    for (const childId of shape.childNodeIds) {
      if (!shapeByNode.has(childId)) {
        throw new ZuiDocumentError(
          `Node ${shape.nodeId} references missing semantic node ${childId} in Penpot asset.`,
        );
      }
      const previous = desiredParent.get(childId);
      if (previous) {
        throw new ZuiDocumentError(
          `Semantic node ${childId} is nested under both ${previous} and ${shape.nodeId}.`,
        );
      }
      desiredParent.set(childId, shape.nodeId);
    }
  }

  const rootNodeIds = zuiRootNodeIds(source);
  if (!stringArraysEqual(snapshot.rootNodeIds, rootNodeIds)) {
    throw new ZuiDocumentError(
      `Penpot root node metadata does not match ZUI roots; expected ${rootNodeIds.join(', ') || 'none'}.`,
    );
  }
  const rootSet = new Set(rootNodeIds);
  const derivedDetached = snapshot.shapes
    .filter(({ nodeId }) => !rootSet.has(nodeId) && !desiredParent.has(nodeId))
    .map(({ nodeId }) => nodeId);
  if (!sameStringSet(snapshot.detachedNodeIds, derivedDetached)) {
    throw new ZuiDocumentError(
      `Penpot detached node metadata disagrees with the semantic hierarchy; expected ${derivedDetached.join(', ') || 'none'}.`,
    );
  }

  for (const rootId of rootNodeIds) {
    if (desiredParent.has(rootId)) {
      throw new ZuiDocumentError(
        `ZUI root node ${rootId} must remain at the asset top level.`,
      );
    }
  }
  for (const shape of snapshot.shapes) {
    const declaredParent = desiredParent.get(shape.nodeId) ?? null;
    if (shape.parentNodeId !== declaredParent) {
      throw new ZuiDocumentError(
        `Penpot hierarchy metadata disagrees for node ${shape.nodeId}; expected parent ${String(
          declaredParent,
        )}, found ${String(shape.parentNodeId)}.`,
      );
    }
  }
  assertNoSnapshotCycle(snapshot, shapeByNode);
  return desiredParent;
}

function assertNoSnapshotCycle(
  snapshot: PenpotAssetSnapshot,
  shapeByNode: Map<string, PenpotShapeSnapshot>,
): void {
  const visiting = new Set<string>();
  const visited = new Set<string>();
  const visit = (nodeId: string): void => {
    if (visiting.has(nodeId)) {
      throw new ZuiDocumentError(
        `Penpot semantic hierarchy contains a cycle at ${nodeId}.`,
      );
    }
    if (visited.has(nodeId)) {
      return;
    }
    visiting.add(nodeId);
    for (const childId of shapeByNode.get(nodeId)?.childNodeIds ?? []) {
      visit(childId);
    }
    visiting.delete(nodeId);
    visited.add(nodeId);
  };
  for (const shape of snapshot.shapes) {
    visit(shape.nodeId);
  }
}

function applyHierarchy(
  document: ZuiDocument,
  snapshot: PenpotAssetSnapshot,
  desiredParent: Map<string, string>,
  changes: string[],
): void {
  const nodes = zuiNodes(document);
  const mountByNode = new Map(
    Object.values(nodes).flatMap((node) =>
      (node.children ?? []).map((mount) => [mount.node, mount] as const),
    ),
  );

  for (const shape of snapshot.shapes) {
    const node = nodes[shape.nodeId];
    const currentChildren = (node.children ?? []).map(
      ({ node: childId }) => childId,
    );
    if (stringArraysEqual(currentChildren, shape.childNodeIds)) {
      continue;
    }
    node.children = shape.childNodeIds.map(
      (childId) => mountByNode.get(childId) ?? { node: childId },
    );
    changes.push(`nodes.${shape.nodeId}.children`);
  }

  for (const [childId, parentId] of desiredParent) {
    if (!nodes[parentId].children?.some(({ node }) => node === childId)) {
      throw new ZuiDocumentError(
        `Failed to mount semantic node ${childId} under ${parentId}.`,
      );
    }
  }
}

function applyEditableChanges(
  node: ZuiNode,
  shape: PenpotShapeSnapshot,
  changes: string[],
  parentTransition: ParentLayoutTransition,
  desiredMount: DesiredMount | undefined,
  selfStyle: ZuiTable,
): void {
  const { baseline, current } = shape;
  validateEditableState(shape.nodeId, baseline);
  validateEditableState(shape.nodeId, current);
  reconcileSlotPadding(
    shape.nodeId,
    baseline.slotPadding,
    current.slotPadding,
    desiredMount,
    parentTransition === 'auto-to-free',
    changes,
  );

  const materializeDimensions = parentTransition !== 'none';
  const materializePosition = parentTransition === 'auto-to-free';
  if (
    materializeDimensions ||
    !nearlyEqual(current.geometry.width, baseline.geometry.width)
  ) {
    setFixedDimension(node, 'width', current.geometry.width);
    changes.push(`nodes.${shape.nodeId}.layout.width`);
    setMountFixedDimension(
      desiredMount,
      'width',
      current.geometry.width,
      changes,
    );
  }
  if (
    materializeDimensions ||
    !nearlyEqual(current.geometry.height, baseline.geometry.height)
  ) {
    setFixedDimension(node, 'height', current.geometry.height);
    changes.push(`nodes.${shape.nodeId}.layout.height`);
    setMountFixedDimension(
      desiredMount,
      'height',
      current.geometry.height,
      changes,
    );
  }
  const xChanged = !nearlyEqual(current.geometry.x, baseline.geometry.x);
  const yChanged = !nearlyEqual(current.geometry.y, baseline.geometry.y);
  if (materializePosition || xChanged || yChanged) {
    const layout = ensureNodeTable(node, 'layout');
    const position = ensureNestedTable(layout, 'position');
    if (materializePosition || xChanged) {
      position['x'] = round(current.geometry.x);
      changes.push(`nodes.${shape.nodeId}.layout.position.x`);
      setMountPosition(desiredMount, 'x', current.geometry.x, changes);
    }
    if (materializePosition || yChanged) {
      position['y'] = round(current.geometry.y);
      changes.push(`nodes.${shape.nodeId}.layout.position.y`);
      setMountPosition(desiredMount, 'y', current.geometry.y, changes);
    }
  }

  applyPaintChanges(
    node,
    shape.nodeId,
    baseline.paint,
    current.paint,
    changes,
    selfStyle,
  );
  applyNodeTextChanges(
    node,
    shape.nodeId,
    baseline,
    current,
    changes,
    selfStyle,
  );
  applyContainerChanges(
    node,
    shape.nodeId,
    baseline.container,
    current.container,
    changes,
  );
}

type ParentLayoutTransition = 'none' | 'free-to-auto' | 'auto-to-free';

interface DesiredMount {
  parentId: string;
  mount: ZuiChildMount;
}

function sourceParentIndex(
  nodes: Record<string, ZuiNode>,
): Map<string, string> {
  const parentByNode = new Map<string, string>();
  for (const [parentId, node] of Object.entries(nodes)) {
    for (const child of node.children ?? []) {
      parentByNode.set(child.node, parentId);
    }
  }
  return parentByNode;
}

function childMountIndex(
  nodes: Record<string, ZuiNode>,
): Map<string, DesiredMount> {
  const mounts = new Map<string, DesiredMount>();
  for (const [parentId, node] of Object.entries(nodes)) {
    for (const mount of node.children ?? []) {
      mounts.set(mount.node, { parentId, mount });
    }
  }
  return mounts;
}

function parentLayoutTransition(
  nodeId: string,
  sourceParent: Map<string, string>,
  desiredParent: Map<string, string>,
  shapeByNode: Map<string, PenpotShapeSnapshot>,
): ParentLayoutTransition {
  const baselineKind = parentContainerKind(
    sourceParent.get(nodeId),
    shapeByNode,
    'baseline',
  );
  const currentKind = parentContainerKind(
    desiredParent.get(nodeId),
    shapeByNode,
    'current',
  );
  const baselineAuto = baselineKind !== 'free';
  const currentAuto = currentKind !== 'free';
  if (baselineAuto === currentAuto) return 'none';
  return baselineAuto ? 'auto-to-free' : 'free-to-auto';
}

function parentContainerKind(
  parentId: string | undefined,
  shapeByNode: Map<string, PenpotShapeSnapshot>,
  phase: 'baseline' | 'current',
): ProjectionContainer['kind'] {
  if (parentId === undefined) return 'free';
  const parent = shapeByNode.get(parentId);
  if (!parent) {
    throw new ZuiDocumentError(
      `Missing semantic parent ${parentId} while reconciling layout context.`,
    );
  }
  return parent[phase].container.kind;
}

function applyPaintChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionPaint,
  current: ProjectionPaint,
  changes: string[],
  selfStyle: ZuiTable,
): void {
  const fillChanged = current.fillColor !== baseline.fillColor;
  const fillOpacityChanged = !nearlyEqual(
    current.fillOpacity,
    baseline.fillOpacity,
  );
  const strokeChanged = current.strokeColor !== baseline.strokeColor;
  const strokeOpacityChanged = !nearlyEqual(
    current.strokeOpacity,
    baseline.strokeOpacity,
  );
  const strokeWidthChanged = !nearlyEqual(
    current.strokeWidth,
    baseline.strokeWidth,
  );
  const radiusChanged = !nearlyEqual(
    current.borderRadius,
    baseline.borderRadius,
  );
  const opacityChanged = !nearlyEqual(current.opacity, baseline.opacity);
  if (
    !fillChanged &&
    !fillOpacityChanged &&
    !strokeChanged &&
    !strokeOpacityChanged &&
    !strokeWidthChanged &&
    !radiusChanged &&
    !opacityChanged
  ) {
    return;
  }

  if (fillChanged || fillOpacityChanged) {
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'background_color',
        serializedColor(current.fillColor, current.fillOpacity),
        selfStyle,
      ),
    );
  }
  if (strokeChanged || strokeOpacityChanged) {
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'border_color',
        serializedColor(current.strokeColor, current.strokeOpacity),
        selfStyle,
      ),
    );
  }
  if (strokeWidthChanged) {
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'border_width',
        round(current.strokeWidth),
        selfStyle,
      ),
    );
  }
  if (radiusChanged) {
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'corner_radius',
        round(current.borderRadius),
        selfStyle,
      ),
    );
  }
  if (opacityChanged) {
    changes.push(
      writeAuthoredPaintProperty(
        node,
        nodeId,
        'opacity',
        round(current.opacity),
        selfStyle,
      ),
    );
  }
}

function applyContainerChanges(
  node: ZuiNode,
  nodeId: string,
  baseline: ProjectionContainer,
  current: ProjectionContainer,
  changes: string[],
): void {
  const kindChanged =
    current.kind !== baseline.kind ||
    current.direction !== baseline.direction ||
    current.wrap !== baseline.wrap;
  const gapChanged = !nearlyEqual(current.gap, baseline.gap);
  const rowGapChanged = !nearlyEqual(current.rowGap, baseline.rowGap);
  const columnGapChanged = !nearlyEqual(current.columnGap, baseline.columnGap);
  const columnsChanged = current.columns !== baseline.columns;
  const rowsChanged = current.rows !== baseline.rows;
  const alignChanged = current.alignItems !== baseline.alignItems;
  const justifyChanged = current.justifyContent !== baseline.justifyContent;
  const paddingChanged = !paddingEqual(current.padding, baseline.padding);
  const clipChanged = current.clip !== baseline.clip;
  if (
    !kindChanged &&
    !gapChanged &&
    !rowGapChanged &&
    !columnGapChanged &&
    !columnsChanged &&
    !rowsChanged &&
    !alignChanged &&
    !justifyChanged &&
    !paddingChanged &&
    !clipChanged
  ) {
    return;
  }

  let changedLayout: ZuiTable | undefined;
  const layout = (): ZuiTable =>
    (changedLayout ??= ensureNodeTable(node, 'layout'));
  if (kindChanged) {
    const container = ensureNestedTable(layout(), 'container');
    container['kind'] = containerKind(current);
    changes.push(`nodes.${nodeId}.layout.container.kind`);
  }
  const authoredContainer = node.layout?.['container'];
  const gapEdits = reconcileContainerGapEdits(
    nodeId,
    baseline,
    current,
    isTable(authoredContainer) ? authoredContainer : undefined,
    isTable(authoredContainer)
      ? (authoredContainer['kind'] ?? node.component)
      : node.component,
  );
  for (const [key, value] of Object.entries(gapEdits)) {
    ensureNestedTable(layout(), 'container')[key] = round(value);
    changes.push(`nodes.${nodeId}.layout.container.${key}`);
  }
  if (columnsChanged) {
    ensureNestedTable(layout(), 'container')['columns'] = current.columns;
    changes.push(`nodes.${nodeId}.layout.container.columns`);
  }
  if (rowsChanged) {
    ensureNestedTable(layout(), 'container')['rows'] = current.rows;
    changes.push(`nodes.${nodeId}.layout.container.rows`);
  }
  if (alignChanged) {
    ensureNestedTable(layout(), 'container')['align_items'] = alignmentToZui(
      current.alignItems,
    );
    changes.push(`nodes.${nodeId}.layout.container.align_items`);
  }
  if (justifyChanged) {
    ensureNestedTable(layout(), 'container')['justify_content'] =
      justificationToZui(current.justifyContent);
    changes.push(`nodes.${nodeId}.layout.container.justify_content`);
  }
  if (paddingChanged) {
    const padding = ensureNestedTable(layout(), 'padding');
    for (const side of ['top', 'right', 'bottom', 'left'] as const) {
      if (!nearlyEqual(current.padding[side], baseline.padding[side])) {
        padding[side] = round(current.padding[side]);
        changes.push(`nodes.${nodeId}.layout.padding.${side}`);
      }
    }
  }
  if (clipChanged) {
    const hasLayoutClip = node.layout
      ? Object.hasOwn(node.layout, 'clip')
      : false;
    const hasPropsClip = node.props
      ? Object.hasOwn(node.props, 'clip_content')
      : false;
    if (hasLayoutClip || !hasPropsClip) {
      layout()['clip'] = current.clip;
      changes.push(`nodes.${nodeId}.layout.clip`);
    }
    if (hasPropsClip) {
      ensureNodeTable(node, 'props')['clip_content'] = current.clip;
      changes.push(`nodes.${nodeId}.props.clip_content`);
    }
  }
}

function setFixedDimension(
  node: ZuiNode,
  axis: 'width' | 'height',
  value: number,
): void {
  setFixedLayoutDimension(ensureNodeTable(node, 'layout'), axis, value);
}

function setFixedLayoutDimension(
  layout: ZuiTable,
  axis: 'width' | 'height',
  value: number,
): void {
  const dimension = ensureNestedTable(layout, axis);
  dimension['min'] = round(value);
  dimension['preferred'] = round(value);
  dimension['max'] = round(value);
  dimension['stretch'] = 'Fixed';
}

function setMountFixedDimension(
  desiredMount: DesiredMount | undefined,
  axis: 'width' | 'height',
  value: number,
  changes: string[],
): void {
  if (!desiredMount) return;
  const layout = mountSlotLayout(desiredMount);
  if (!layout || !Object.hasOwn(layout, axis)) return;
  setFixedLayoutDimension(layout, axis, value);
  changes.push(
    `nodes.${desiredMount.parentId}.children.${desiredMount.mount.node}.slot.layout.${axis}`,
  );
}

function setMountPosition(
  desiredMount: DesiredMount | undefined,
  coordinate: 'x' | 'y',
  value: number,
  changes: string[],
): void {
  if (!desiredMount) return;
  const layout = mountSlotLayout(desiredMount);
  if (!layout || !Object.hasOwn(layout, 'position')) return;
  ensureNestedTable(layout, 'position')[coordinate] = round(value);
  changes.push(
    `nodes.${desiredMount.parentId}.children.${desiredMount.mount.node}.slot.layout.position.${coordinate}`,
  );
}

function mountSlotLayout(desiredMount: DesiredMount): ZuiTable | undefined {
  const layout = desiredMount.mount.slot?.['layout'];
  return isTable(layout) ? layout : undefined;
}

function validateEditableState(
  nodeId: string,
  state: ProjectionEditableState,
): void {
  const values = [
    state.geometry.x,
    state.geometry.y,
    state.geometry.width,
    state.geometry.height,
    state.paint.strokeWidth,
    state.paint.borderRadius,
    state.paint.opacity,
    state.paint.fillOpacity,
    state.paint.strokeOpacity,
    state.container.gap,
    state.container.rowGap,
    state.container.columnGap,
    state.container.columns,
    state.container.rows,
    ...Object.values(state.container.padding),
    ...Object.values(state.slotPadding ?? {}),
  ];
  const texts = [
    ...(state.text ? [state.text] : []),
    ...Object.values(state.textFragments ?? {}),
  ];
  for (const text of texts)
    values.push(text.colorOpacity, text.fontSize ?? 14, text.lineHeight);
  if (values.some((value) => !Number.isFinite(value))) {
    throw new ZuiDocumentError(
      `Node ${nodeId} contains non-finite Penpot geometry or style values.`,
    );
  }
  if (state.geometry.width < 0 || state.geometry.height < 0) {
    throw new ZuiDocumentError(
      `Node ${nodeId} width and height must not be negative.`,
    );
  }
  if (
    state.paint.strokeWidth < 0 ||
    state.paint.borderRadius < 0 ||
    state.container.gap < 0 ||
    state.container.rowGap < 0 ||
    state.container.columnGap < 0
  ) {
    throw new ZuiDocumentError(
      `Node ${nodeId} contains negative style or layout values.`,
    );
  }
  if (
    !Number.isInteger(state.container.columns) ||
    state.container.columns <= 0 ||
    !Number.isInteger(state.container.rows) ||
    state.container.rows <= 0
  ) {
    throw new ZuiDocumentError(
      `Node ${nodeId} grid track counts must be positive integers.`,
    );
  }
  const opacities = [
    state.paint.opacity,
    state.paint.fillOpacity,
    state.paint.strokeOpacity,
    ...texts.map((text) => text.colorOpacity),
  ];
  if (opacities.some((value) => value < 0 || value > 1)) {
    throw new ZuiDocumentError(
      `Node ${nodeId} opacity values must be between zero and one.`,
    );
  }
  const colors = [
    state.paint.fillColor,
    state.paint.strokeColor,
    ...texts.map((text) => text.color),
  ];
  if (
    colors.some((value) => value !== null && !/^#[0-9a-f]{6}$/i.test(value))
  ) {
    throw new ZuiDocumentError(
      `Node ${nodeId} contains a Penpot color outside the #RRGGBB bridge profile.`,
    );
  }
}

function alignmentToZui(value: ProjectionContainer['alignItems']): string {
  return value[0].toUpperCase() + value.slice(1);
}

function justificationToZui(
  value: ProjectionContainer['justifyContent'],
): string {
  return value
    .split('-')
    .map((part) => part[0].toUpperCase() + part.slice(1))
    .join('');
}

function containerKind(container: ProjectionContainer): string {
  if (container.kind === 'grid') return 'GridBox';
  if (container.kind === 'free') return 'Overlay';
  if (container.wrap && container.direction === 'row') return 'FlowBox';
  return container.direction === 'row' ? 'HorizontalBox' : 'VerticalBox';
}

function ensureNodeTable(node: ZuiNode, key: 'layout' | 'props'): ZuiTable {
  const current = node[key];
  if (current && isTable(current)) return current;
  const next: ZuiTable = {};
  node[key] = next;
  return next;
}

function ensureNestedTable(table: ZuiTable, key: string): ZuiTable {
  const current = table[key];
  if (isTable(current)) return current;
  const next: ZuiTable = {};
  table[key] = next;
  return next;
}

function assertValidDocument(document: ZuiDocument): void {
  const diagnostics = validateZuiDocument(document);
  const errors = diagnostics.filter(({ severity }) => severity === 'error');
  if (errors.length > 0) {
    throw new ZuiDocumentError(
      `Invalid .zui document: ${errors.map(({ message }) => message).join('; ')}`,
      diagnostics,
    );
  }
}

function isTable(value: unknown): value is ZuiTable {
  return (
    typeof value === 'object' &&
    value !== null &&
    !Array.isArray(value) &&
    !(value instanceof Date)
  );
}

function nearlyEqual(left: number, right: number): boolean {
  return Math.abs(left - right) < 0.01;
}

function round(value: number): number {
  return Math.round(value * 1000) / 1000;
}

function paddingEqual(
  left: ProjectionPadding,
  right: ProjectionPadding,
): boolean {
  return (
    nearlyEqual(left.top, right.top) &&
    nearlyEqual(left.right, right.right) &&
    nearlyEqual(left.bottom, right.bottom) &&
    nearlyEqual(left.left, right.left)
  );
}

function stringArraysEqual(left: string[], right: string[]): boolean {
  return (
    left.length === right.length &&
    left.every((value, index) => value === right[index])
  );
}

function sameStringSet(left: string[], right: string[]): boolean {
  return (
    new Set(left).size === left.length &&
    new Set(right).size === right.length &&
    left.length === right.length &&
    left.every((value) => right.includes(value))
  );
}
