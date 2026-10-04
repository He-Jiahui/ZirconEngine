import {
  isTreeRowPainter,
  refreshTreeRowContent,
  layoutTreeRowText,
} from './penpot-tree-row-controls';
import { allocateNativeLayoutSize } from './penpot-layout-allocation';
import type { Board, CommonLayout, Shape, Text } from '@penpot/plugin-types';
import { createSemanticTexts, textInsets } from './penpot-semantic-text';
import { applyNativeScrollContentLayout } from './penpot-scroll-content-layout';
import { applyNativeWrapContentLayout } from './penpot-wrap-content-layout';
import { applySlotPadding } from './penpot-slot-padding';
import { createImportCheckpoint } from './penpot-import-scheduling';
import { renderedSourceIdentity } from './penpot-rendered-source-identity';
import {
  createSliderDecoration,
  refreshSliderDecoration,
  layoutSliderText,
} from './penpot-slider-controls';
import { createPrefabIcon, previewIconPlacement } from './penpot-prefab-icons';
import { refreshDividerDecoration } from './penpot-divider-controls';
import {
  createTableMeasurements,
  layoutTableText,
  TABLE_MEASUREMENT,
  TABLE_MEASUREMENT_REQUIRED,
  tableMeasurementShapes,
} from './penpot-table-controls';
import {
  createInputDecoration,
  refreshInputDecoration,
} from './penpot-input-controls';
import { inputControlGeometry } from './bridge/zui-input-projection';
import {
  refreshFieldDecoration,
  layoutFieldText,
} from './penpot-field-controls';
import { semanticTextCharacters } from './penpot-text-overflow';
import {
  refreshSegmentedDecoration,
  layoutSegmentedText,
} from './penpot-segmented-controls';
import {
  refreshPropertyRowDecoration,
  layoutPropertyRowText,
} from './penpot-property-row-controls';
import {
  createControlContent,
  refreshControlContent,
} from './penpot-control-content';
import {
  tokenOverviewSize,
  createTokenOverview,
} from './penpot-document-overview';

import {
  gridCellForChildIndex,
  resolvedAnchoredPosition,
} from './penpot-capture-validation.js';
import { PENPOT_BRIDGE_VERSION } from './bridge/penpot-asset.js';
import {
  encodeZuiMetadata,
  zuiNodes,
  type ZuiDocument,
  type ZuiNode,
} from './bridge/zui-document.js';
import type {
  ProjectedZuiNode,
  ProjectionContainer,
  ZuiAssetProjection,
} from './bridge/penpot-projection.js';
import { cloneProjectionSnapshot } from './bridge/penpot-projection';
import { applyWeightedFlexLayout } from './penpot-weighted-flex';
import {
  synchronizeLinearContentMeasurements,
  measuredLinearDesiredSizes,
  linearContentMeasurementSnapshot,
} from './penpot-linear-content';
import {
  createImageContent,
  refreshImageContent,
} from './penpot-image-content';
import { refreshPopupOverlays } from './penpot-popup-controls';
import {
  PENPOT_PALETTE,
  resolveDesignNumber,
  type ZuiPrefabRole,
} from './bridge/zui-prefab-system.js';
import {
  auditLayoutBounds,
  applyResolvedLayoutChildConstraints,
  arrangedFreeParentExtent,
  childDimensionSizing,
  ASSET_GAP,
  previewAnchoredContainerMinimum,
  previewAssetGeometry,
  renderedRelativePosition,
  type LayoutAuditParentKind,
  type LayoutBoundsCandidate,
  type LayoutScrollContext,
  type RenderedLayoutAudit,
} from './penpot-render-layout.js';
import {
  ZUI_METADATA_BRIDGE_VERSION,
  ZUI_METADATA_CHILD_INDEX,
  ZUI_METADATA_COMPONENT,
  ZUI_METADATA_CONTAINER_KIND,
  ZUI_METADATA_EXPLICIT_OVERLAY,
  ZUI_METADATA_DOCUMENT,
  ZUI_METADATA_EMPTY_TEXT,
  ZUI_METADATA_FILE_NAME,
  ZUI_METADATA_LAYOUT_MODE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_METADATA_SCROLL_AXIS,
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_VISUAL_DETACHED,
  ZUI_METADATA_VISUAL_DETACHED_PARENT,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_DETACHED,
  ZUI_ROLE_NODE,
  ZUI_ROLE_TEXT,
} from './metadata.js';

export async function createAssetBoard(
  fileName: string,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
): Promise<Board> {
  const checkpoint = createImportCheckpoint();
  const detachedNodes = renderedDetachedNodes(projection);
  const tokenOverview =
    projection.shapes.length === 0 ? tokenOverviewSize(document) : null;
  const geometry = previewAssetGeometry(
    document.asset.kind,
    projection.rootNodes.map((projectionNode) => {
      const node = zuiNodes(document)[projectionNode.nodeId];
      return previewAnchoredContainerMinimum(
        projectionNode.geometry,
        layoutPoint(document, node, 'anchor'),
        layoutPoint(document, node, 'pivot'),
        layoutPoint(document, node, 'position', projectionNode.geometry),
      );
    }),
    detachedNodes.map(({ geometry: size }) => size),
    tokenOverview,
  );
  const assetBoard = penpot.createBoard();
  try {
    assetBoard.name = `ZUI · ${projection.displayName}`;
    assetBoard.resize(geometry.width, geometry.height);
    assetBoard.x = penpot.viewport.center.x - assetBoard.width / 2;
    assetBoard.y = penpot.viewport.center.y - assetBoard.height / 2;
    assetBoard.clipContent = false;
    assetBoard.fills = [{ fillColor: PENPOT_PALETTE.canvas, fillOpacity: 1 }];
    setMetadata(assetBoard, ZUI_METADATA_ROLE, ZUI_ROLE_ASSET);
    setMetadata(
      assetBoard,
      ZUI_METADATA_BRIDGE_VERSION,
      String(PENPOT_BRIDGE_VERSION),
    );
    setMetadata(assetBoard, ZUI_METADATA_DOCUMENT, encodeZuiMetadata(document));
    setMetadata(assetBoard, ZUI_METADATA_FILE_NAME, sanitizeFileName(fileName));
    const projected = new Map(
      projection.shapes.map((shape) => [shape.nodeId, shape]),
    );
    const hiddenDescendants = cloneProjectionSnapshot(projection).shapes.filter(
      (shape) =>
        shape.parentNodeId && projected.get(shape.parentNodeId)?.previewHidden,
    );
    setMetadata(assetBoard, 'virtual-nodes', JSON.stringify(hiddenDescendants));
    for (const [index, node] of projection.rootNodes.entries())
      await createSemanticBoard(
        node,
        document,
        assetBoard,
        index,
        null,
        checkpoint,
      );
    if (projection.shapes.length === 0) {
      createTokenOverview(assetBoard, document);
    }
    if (detachedNodes.length > 0) {
      await createDetachedLane(
        assetBoard,
        document,
        detachedNodes,
        geometry.detachedLaneX ?? geometry.primaryWidth,
        geometry.detachedLaneHeight,
        geometry.detachedLaneWidth,
        checkpoint,
      );
    }
    synchronizeLinearContentMeasurements(assetBoard, document, projection);
    refreshPopupOverlays(assetBoard, document, projection);
    return assetBoard;
  } catch (error) {
    assetBoard.remove();
    throw error;
  }
}

function renderedDetachedNodes(
  projection: ZuiAssetProjection,
): ProjectedZuiNode[] {
  const detachedIds = new Set([
    ...projection.detachedNodes.map(({ nodeId }) => nodeId),
    ...projection.shapes
      .filter((node) => node.visualDetached)
      .map(({ nodeId }) => nodeId),
  ]);
  return projection.shapes.filter(({ nodeId }) => detachedIds.has(nodeId));
}

export function auditRenderedAssetBoard(
  assetBoard: Board,
): RenderedLayoutAudit {
  const candidates: LayoutBoundsCandidate[] = [];
  const semanticNodes: NonNullable<RenderedLayoutAudit['semanticNodes']> = [];
  const tableMeasurements = tableMeasurementShapes(assetBoard).map((shape) => ({
    shapeId: shape.id,
    text: shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, TABLE_MEASUREMENT),
    width: shape.width,
    height: shape.height,
  }));
  const visit = (
    parent: Board,
    parentHidden = false,
    detached = false,
  ): void => {
    const hidden = parentHidden || parent.hidden;
    const inDetachedLane =
      detached || metadata(parent, ZUI_METADATA_ROLE) === ZUI_ROLE_DETACHED;
    const scrollContext = scrollContextFor(parent);
    for (const child of parent.children) {
      if (child.type !== 'board') continue;
      if (metadata(child, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE) {
        const reviewScroll = reviewScrollTrace(child);
        const relativePosition = renderedRelativePosition(child, parent);
        candidates.push({
          nodeId: metadata(child, ZUI_METADATA_NODE_ID) || child.name,
          explicitOverlay:
            metadata(parent, ZUI_METADATA_EXPLICIT_OVERLAY) === 'true',
          ...(scrollContext.horizontal || scrollContext.vertical
            ? { scrollContext }
            : {}),
          hidden: hidden || child.hidden,
          parentKind: auditParentKind(parent),
          parentClips: parent.clipContent,
          parentWidth: parent.width,
          parentHeight: parent.height,
          x: relativePosition.x,
          y: relativePosition.y,
          width: child.width,
          height: child.height,
        });
        const textParts = child.children.filter(
          (shape): shape is Text =>
            shape.type === 'text' &&
            !shape.hidden &&
            metadata(shape, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT,
        );
        const tableMeasurements = child.children.flatMap((shape) => {
          const label = metadata(shape, TABLE_MEASUREMENT);
          return label
            ? [
                {
                  shapeId: shape.id,
                  text: label,
                  width: shape.width,
                  height: shape.height,
                },
              ]
            : [];
        });
        semanticNodes.push({
          nodeId: metadata(child, ZUI_METADATA_NODE_ID),
          shapeId: child.id,
          ...renderedSourceIdentity(child, parent),
          parentNodeId:
            (metadata(child, ZUI_METADATA_VISUAL_DETACHED) === 'true' &&
              metadata(child, ZUI_METADATA_VISUAL_DETACHED_PARENT)) ||
            metadata(parent, ZUI_METADATA_NODE_ID) ||
            null,
          component: metadata(child, ZUI_METADATA_COMPONENT),
          visible: !(hidden || child.hidden),
          detached: inDetachedLane,
          clip: child.clipContent,
          ...(child.clipContent
            ? {
                clipBounds: {
                  x: child.x - assetBoard.x,
                  y: child.y - assetBoard.y,
                  width: child.width,
                  height: child.height,
                },
              }
            : {}),
          ...(metadata(child, 'native-component-id')
            ? {
                nativeComponent: {
                  id: metadata(child, 'native-component-id'),
                  source: metadata(child, 'native-component-source'),
                  copy: child.isComponentCopyInstance(),
                },
              }
            : {}),
          bounds: {
            x: child.x - assetBoard.x,
            y: child.y - assetBoard.y,
            width: child.width,
            height: child.height,
          },
          text: textParts
            .map((shape) =>
              metadata(shape, ZUI_METADATA_EMPTY_TEXT) === 'true'
                ? ''
                : semanticTextCharacters(shape),
            )
            .join(''),
          textParts: textParts.map((shape) => ({
            shapeId: shape.id,
            text:
              metadata(shape, ZUI_METADATA_EMPTY_TEXT) === 'true'
                ? ''
                : semanticTextCharacters(shape),
            displayText: shape.characters,
            bounds: {
              x: shape.x - assetBoard.x,
              y: shape.y - assetBoard.y,
              width: shape.width,
              height: shape.height,
            },
          })),
          ...(reviewScroll ? { reviewScroll } : {}),
          ...(tableMeasurements.length ? { tableMeasurements } : {}),
        });
      }
      visit(child, hidden, inDetachedLane);
    }
  };
  visit(assetBoard);
  return {
    ...auditLayoutBounds(candidates),
    contentMeasurements: linearContentMeasurementSnapshot(assetBoard),
    semanticNodes,
    ...(tableMeasurements.length ? { tableMeasurements } : {}),
    assetBounds: {
      x: assetBoard.x,
      y: assetBoard.y,
      width: assetBoard.width,
      height: assetBoard.height,
    },
  };
}

function reviewScrollTrace(shape: Shape):
  | {
      originalX?: number;
      originalY?: number;
      targetX?: number;
      targetY?: number;
      capturedX?: number;
      capturedY?: number;
    }
  | undefined {
  const read = (key: string): number | undefined => {
    const value = metadata(shape, key);
    if (!value) return undefined;
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : undefined;
  };
  const trace = {
    originalX: read('review-scroll-original-x'),
    originalY: read('review-scroll-original-y'),
    targetX: read('review-scroll-target-x'),
    targetY: read('review-scroll-target-y'),
    capturedX: read('review-scroll-captured-x'),
    capturedY: read('review-scroll-captured-y'),
  };
  return Object.values(trace).some((value) => value !== undefined)
    ? trace
    : undefined;
}

export function resizeAssetReviewViewport(
  assetBoard: Board,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
  viewport: { width: number; height: number },
): void {
  if (
    !Number.isFinite(viewport.width) ||
    !Number.isFinite(viewport.height) ||
    viewport.width <= 0 ||
    viewport.height <= 0
  )
    throw new Error('Review viewport must have positive finite dimensions');
  assetBoard.resize(viewport.width, viewport.height);
  assetBoard.clipContent = true;
  for (const child of assetBoard.children) {
    if (metadata(child, ZUI_METADATA_ROLE) !== ZUI_ROLE_DETACHED) continue;
    child.x = assetBoard.x + viewport.width + ASSET_GAP;
    child.y = assetBoard.y;
  }
  for (const root of projection.rootNodes) {
    const board = assetBoard.children.find(
      (child) =>
        child.type === 'board' &&
        metadata(child, ZUI_METADATA_NODE_ID) === root.nodeId,
    );
    if (!board || board.type !== 'board') continue;
    const node = zuiNodes(document)[root.nodeId];
    const size = semanticBoardSize(assetBoard, null, root, node, document);
    board.resize(size.width, size.height);
    const position = resolvedAnchoredPosition(
      viewport,
      size,
      layoutPoint(document, node, 'anchor'),
      layoutPoint(document, node, 'pivot'),
      layoutPoint(document, node, 'position', root.geometry),
    );
    board.x = assetBoard.x + position.x;
    board.y = assetBoard.y + position.y;
  }
  refreshAssetTextLayout(assetBoard, document, projection);
}

function auditParentKind(parent: Board): LayoutAuditParentKind {
  const role = metadata(parent, ZUI_METADATA_ROLE);
  if (role === ZUI_ROLE_ASSET) return 'asset';
  if (role === ZUI_ROLE_DETACHED) return 'detached';
  if (metadata(parent, ZUI_METADATA_CONTAINER_KIND) === 'scroll')
    return 'scroll';
  if (
    metadata(parent, ZUI_METADATA_COMPONENT).toLowerCase().includes('scroll')
  ) {
    return 'scroll';
  }
  if (parent.flex) return 'flex';
  if (parent.grid) return 'grid';
  return 'free';
}

async function createDetachedLane(
  assetBoard: Board,
  document: ZuiDocument,
  detachedNodes: ProjectedZuiNode[],
  laneX: number,
  boardHeight: number,
  detachedWidth: number,
  checkpoint: () => Promise<void>,
): Promise<void> {
  const lane = penpot.createBoard();
  lane.name = 'ZUI · Detached nodes';
  lane.resize(detachedWidth, Math.max(boardHeight, detachedNodes.length * 72));
  lane.x = assetBoard.x + laneX;
  lane.y = assetBoard.y;
  lane.fills = [{ fillColor: PENPOT_PALETTE.surfaceInset, fillOpacity: 1 }];
  lane.strokes = [
    {
      strokeColor: PENPOT_PALETTE.borderStrong,
      strokeWidth: 1,
      strokeStyle: 'dashed',
      strokeAlignment: 'inner',
    },
  ];
  lane.borderRadius = 8;
  setMetadata(lane, ZUI_METADATA_ROLE, ZUI_ROLE_DETACHED);
  assetBoard.appendChild(lane);
  const flex = lane.addFlexLayout();
  flex.dir = 'column';
  flex.rowGap = 12;
  flex.paddingType = 'multiple';
  flex.topPadding = 16;
  flex.rightPadding = 16;
  flex.bottomPadding = 16;
  flex.leftPadding = 16;
  for (const [index, node] of detachedNodes.entries())
    await createSemanticBoard(
      node,
      document,
      lane,
      index,
      detachedLaneContainer(),
      checkpoint,
    );
}

function detachedLaneContainer(): ProjectionContainer {
  return {
    kind: 'flex',
    direction: 'column',
    wrap: false,
    gap: 12,
    rowGap: 12,
    columnGap: 12,
    columns: 1,
    rows: 1,
    padding: { top: 16, right: 16, bottom: 16, left: 16 },
    alignItems: 'stretch',
    justifyContent: 'start',
    clip: false,
  };
}

async function createSemanticBoard(
  projection: ProjectedZuiNode,
  document: ZuiDocument,
  parent: Board,
  childIndex: number,
  parentContainer: ProjectionContainer | null,
  checkpoint: () => Promise<void>,
): Promise<Board> {
  await checkpoint();
  const node = zuiNodes(document)[projection.nodeId];
  const board = penpot.createBoard();
  const size = semanticBoardSize(
    parent,
    parentContainer,
    projection,
    node,
    document,
  );
  board.name = projection.name;
  board.resize(size.width, size.height);
  board.opacity = projection.paint.opacity;
  board.hidden = projection.previewHidden;
  board.borderRadius = projection.paint.borderRadius;
  // Workbench table rows declare native single-line overflow semantics. Keep
  // their editable text intact while clipping the auto-width projection at
  // the row boundary, matching the retained host's ellipsis frame.
  board.clipContent =
    projection.container.clip ||
    Boolean(projection.table) ||
    Boolean(projection.propertyRow);
  board.fills = projection.paint.fillColor
    ? [
        {
          fillColor: projection.paint.fillColor,
          fillOpacity: projection.paint.fillOpacity,
        },
      ]
    : [];
  board.strokes = projection.paint.strokeColor
    ? [
        {
          strokeColor: projection.paint.strokeColor,
          strokeOpacity: projection.paint.strokeOpacity,
          strokeWidth: projection.paint.strokeWidth,
          strokeStyle: 'solid',
          strokeAlignment: 'inner',
        },
      ]
    : [];
  setMetadata(board, ZUI_METADATA_ROLE, ZUI_ROLE_NODE);
  setMetadata(board, ZUI_METADATA_NODE_ID, projection.nodeId);
  setMetadata(board, ZUI_METADATA_COMPONENT, projection.component);
  if (projection.sourcePath)
    setMetadata(board, 'sourcePath', projection.sourcePath);
  if (projection.sourceNodeId)
    setMetadata(board, 'sourceNodeId', projection.sourceNodeId);
  setMetadata(board, 'controlId', projection.controlId ?? '');
  if (projection.instancePath)
    setMetadata(board, 'instancePath', projection.instancePath);
  setMetadata(board, ZUI_METADATA_LAYOUT_MODE, projection.container.kind);
  const scrollAxis = scrollAxisForNode(node);
  if (scrollAxis) {
    setMetadata(board, ZUI_METADATA_CONTAINER_KIND, 'scroll');
    setMetadata(board, ZUI_METADATA_SCROLL_AXIS, scrollAxis);
  }
  if (isExplicitOverlayContainer(node))
    setMetadata(board, ZUI_METADATA_EXPLICIT_OVERLAY, 'true');
  setMetadata(board, 'prefab-role', projection.prefabRole);
  setMetadata(board, ZUI_METADATA_CHILD_INDEX, String(childIndex));
  if (projection.table) setMetadata(board, TABLE_MEASUREMENT_REQUIRED, 'true');
  if (projection.visualDetached && projection.visualDetachedParentId) {
    setMetadata(board, ZUI_METADATA_VISUAL_DETACHED, 'true');
    setMetadata(
      board,
      ZUI_METADATA_VISUAL_DETACHED_PARENT,
      projection.visualDetachedParentId,
    );
  }

  appendToParent(parent, board, childIndex, parentContainer);
  if (projection.previewHidden) return board;
  if (!parentContainer || parentContainer.kind === 'free') {
    const relativePosition = resolvedAnchoredPosition(
      { width: parent.width, height: parent.height },
      { width: board.width, height: board.height },
      layoutPoint(document, node, 'anchor'),
      layoutPoint(document, node, 'pivot'),
      layoutPoint(document, node, 'position', projection.geometry),
    );
    board.x = parent.x + relativePosition.x;
    board.y = parent.y + relativePosition.y;
  }
  applyContainer(board, projection.container, projection.children.length);
  applyChildSizing(board, node, document);
  applySlotPadding(board, projection.slotPadding);
  createImageContent(board, node);
  createPrefabDecoration(board, projection, node);
  if (
    !isTreeRowPainter(node) &&
    (node.props?.['icon'] ||
      node.props?.['icon_name'] ||
      projection.prefabRole === 'icon-button' ||
      node.component.toLowerCase().includes('icon'))
  ) {
    const iconOnly =
      projection.prefabRole === 'icon-button' ||
      node.component.toLowerCase().includes('icon') ||
      node.props?.['icon_placement'] === 'icon_only';
    const placement = previewIconPlacement(node, iconOnly);
    createPrefabIcon(board, node, document, iconOnly);
    if (placement === 'leading') setMetadata(board, 'has-leading-icon', 'true');
    if (placement === 'trailing')
      setMetadata(board, 'has-trailing-icon', 'true');
  }
  const replacesText =
    !projection.slider &&
    !projection.segmented &&
    !projection.table &&
    !projection.propertyRow &&
    createControlContent(board, node, document, projection.text);
  createTableMeasurements(board, projection.table);
  for (const [index, child] of projection.children.entries()) {
    if (child.visualDetached) continue;
    await createSemanticBoard(
      child,
      document,
      board,
      index,
      projection.container,
      checkpoint,
    );
  }
  createSemanticTexts(board, projection, document, node);
  if (isTreeRowPainter(node))
    for (const child of board.children)
      if (
        child.type === 'text' &&
        metadata(child, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT
      )
        layoutTreeRowText(board, child, node, document);
  if (replacesText)
    for (const child of board.children) {
      if (metadata(child, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT)
        child.hidden = true;
    }
  return board;
}

function semanticBoardSize(
  parent: Board,
  parentContainer: ProjectionContainer | null,
  projection: ProjectedZuiNode,
  node: ZuiNode,
  document: ZuiDocument,
): { width: number; height: number } {
  let width = Math.max(1, projection.geometry.width);
  let height = Math.max(1, projection.geometry.height);
  if (!parentContainer || parentContainer.kind === 'free') {
    const widthTable = nestedTable(node.layout, 'width');
    const heightTable = nestedTable(node.layout, 'height');
    width = arrangedFreeParentExtent(
      document,
      widthTable ?? undefined,
      width,
      parent.width -
        projection.geometry.x -
        (parentContainer?.padding.right ?? 0),
    );
    height = arrangedFreeParentExtent(
      document,
      heightTable ?? undefined,
      height,
      parent.height -
        projection.geometry.y -
        (parentContainer?.padding.bottom ?? 0),
    );
  }
  return { width, height };
}

export function scrollAxisForNode(
  node: ZuiNode,
): 'horizontal' | 'vertical' | 'both' | null {
  const container = nestedTable(node.layout, 'container');
  const scroll = nestedTable(node.layout, 'scroll');
  const component = node.component.toLowerCase();
  const containerKind = String(container?.['kind'] ?? '').toLowerCase();
  const componentScroll = ['scrollablebox', 'scrollbox'].includes(component);
  const containerScroll = ['scrollablebox', 'scrollbox'].includes(
    containerKind,
  );
  const props = node.props ?? {};
  const axis = String(
    container?.['axis'] ?? props['scroll_axis'] ?? '',
  ).toLowerCase();
  const horizontal =
    scroll?.['horizontal'] === true ||
    props['scroll_x'] === true ||
    axis === 'horizontal' ||
    axis === 'both';
  const vertical =
    scroll?.['vertical'] === true ||
    props['scroll_y'] === true ||
    axis === 'vertical' ||
    axis === 'both';
  if (!componentScroll && !containerScroll && !horizontal && !vertical)
    return null;
  if (horizontal && vertical) return 'both';
  if (horizontal) return 'horizontal';
  if (vertical) return 'vertical';
  if (componentScroll && !containerScroll) return 'vertical';
  // The Runtime permits an omitted/both axis. Preserve the established direct
  // container-kind behaviour while making component/props semantics precise.
  return 'both';
}

function scrollContextFor(parent: Board): LayoutScrollContext {
  const axis = metadata(parent, ZUI_METADATA_SCROLL_AXIS);
  return {
    horizontal: axis === 'horizontal' || axis === 'both',
    vertical: axis === 'vertical' || axis === 'both',
  };
}

function isExplicitOverlayContainer(node: ZuiNode): boolean {
  const container = nestedTable(node.layout, 'container');
  if (!container) return false;
  const kind = container['kind'];
  return typeof kind === 'string' && kind.toLowerCase() === 'overlay';
}

function appendToParent(
  parent: Board,
  child: Board,
  childIndex: number,
  parentContainer: ProjectionContainer | null,
): void {
  if (parent.grid) {
    const cell = gridCellForChildIndex(
      childIndex,
      parentContainer?.columns ?? 1,
    );
    parent.grid.appendChild(child, cell.row, cell.column);
  } else {
    parent.appendChild(child);
  }
}

function applyContainer(
  board: Board,
  container: ProjectionContainer,
  semanticChildCount: number,
): void {
  if (container.kind === 'flex') {
    const flex = board.addFlexLayout();
    flex.dir = container.direction;
    flex.wrap = container.wrap ? 'wrap' : 'nowrap';
    flex.rowGap = container.rowGap;
    flex.columnGap = container.columnGap;
    applyCommonLayout(flex, container);
  } else if (container.kind === 'grid') {
    const grid = board.addGridLayout();
    grid.dir = 'row';
    const columns = Math.max(1, container.columns);
    const rows = Math.max(
      container.rows,
      Math.ceil(Math.max(1, semanticChildCount) / columns),
    );
    for (let index = 0; index < columns; index += 1) {
      grid.addColumn('flex', 1);
    }
    for (let index = 0; index < rows; index += 1) {
      grid.addRow('auto');
    }
    grid.rowGap = container.rowGap;
    grid.columnGap = container.columnGap;
    applyCommonLayout(grid, container);
  }
}

function applyCommonLayout(
  layout: CommonLayout,
  container: ProjectionContainer,
): void {
  // Flex stretch uses the child fill sizing. Penpot's stretch alignment omits
  // the cross-axis start margin, so position these children from the start.
  layout.alignItems =
    container.kind === 'flex' && container.alignItems === 'stretch'
      ? 'start'
      : container.alignItems;
  layout.justifyContent = container.justifyContent;
  layout.paddingType = 'multiple';
  layout.topPadding = container.padding.top;
  layout.rightPadding = container.padding.right;
  layout.bottomPadding = container.padding.bottom;
  layout.leftPadding = container.padding.left;
}

function applyChildSizing(
  board: Board,
  node: ZuiNode,
  document: ZuiDocument,
): void {
  const child = board.layoutChild;
  if (!child) return;
  const width = nestedTable(node.layout, 'width');
  const height = nestedTable(node.layout, 'height');
  const contentContainer = Boolean(
    (board.flex || board.grid) && node.children?.length,
  );
  child.horizontalSizing = childDimensionSizing(
    document,
    width ?? undefined,
    contentContainer,
    node.layout?.['boundary'],
  );
  child.verticalSizing = childDimensionSizing(
    document,
    height ?? undefined,
    contentContainer,
    node.layout?.['boundary'],
  );
  applyResolvedLayoutChildConstraints(child, {
    minWidth: resolveDesignNumber(document, width?.['min']),
    maxWidth: resolveDesignNumber(document, width?.['max']),
    minHeight: resolveDesignNumber(document, height?.['min']),
    maxHeight: resolveDesignNumber(document, height?.['max']),
  });
}

function createPrefabDecoration(
  board: Board,
  projection: ProjectedZuiNode,
  node: ZuiNode,
): void {
  if (refreshDividerDecoration(board, projection.divider)) return;
  if (refreshPropertyRowDecoration(board, projection.propertyRow)) return;
  if (createSliderDecoration(board, projection.slider)) return;
  if (refreshSegmentedDecoration(board, projection.segmented)) return;
  if (createInputDecoration(board, projection.inputControl)) return;
  if (refreshFieldDecoration(board, projection.field)) return;
  const component = node.component.toLowerCase();
  if (['slider', 'scrollbar', 'splitter'].includes(component)) {
    const raw = Number(node.props?.['value'] ?? node.props?.['position'] ?? 0);
    const fraction = Math.max(0, Math.min(1, Number.isFinite(raw) ? raw : 0));
    const valueLane = component === 'slider' && board.width >= 96 ? 56 : 0;
    const rail = auxiliaryBoard(
      board,
      `Penpot · ${component} rail`,
      Math.max(16, board.width - 16 - valueLane),
      4,
      PENPOT_PALETTE.borderStrong,
      2,
    );
    rail.x = board.x + 8;
    rail.y = board.y + (board.height - rail.height) / 2;
    const thumb = auxiliaryBoard(
      board,
      `Penpot · ${component} thumb`,
      component === 'splitter' ? 8 : 12,
      component === 'splitter' ? Math.max(16, board.height - 10) : 12,
      PENPOT_PALETTE.accent,
      3,
    );
    thumb.x = rail.x + (rail.width - thumb.width) * fraction;
    thumb.y = board.y + (board.height - thumb.height) / 2;
    setMetadata(rail, 'control-decoration', component);
    setMetadata(thumb, 'control-decoration', component);
    setMetadata(thumb, 'control-fraction', String(fraction));
  } else if (
    projection.prefabRole === 'progress' &&
    component.includes('progress')
  ) {
    const value = Number(node.props?.['value'] ?? 0);
    const minimum = Number(node.props?.['min'] ?? 0);
    const maximum = Number(node.props?.['max'] ?? 1);
    const fraction = Math.max(
      0,
      Math.min(
        1,
        Number(
          node.props?.['value_percent'] ??
            (maximum > minimum ? (value - minimum) / (maximum - minimum) : 0),
        ),
      ),
    );
    const progress = auxiliaryBoard(
      board,
      'Penpot · progress value',
      Math.max(0.01, (board.width - 4) * fraction),
      Math.max(3, Math.min(6, board.height - 4)),
      PENPOT_PALETTE.accent,
      3,
    );
    progress.x = board.x + 2;
    progress.y = board.y + (board.height - progress.height) / 2;
    progress.hidden = fraction === 0;
    setMetadata(progress, 'progress-fraction', String(fraction));
  } else if (projection.prefabRole === 'toggle') {
    const indicator = auxiliaryBoard(
      board,
      'Penpot · toggle indicator',
      12,
      12,
      projection.paint.strokeColor === PENPOT_PALETTE.accent
        ? PENPOT_PALETTE.accent
        : PENPOT_PALETTE.surfaceInset,
      3,
      PENPOT_PALETTE.borderStrong,
    );
    indicator.x = board.x + 8;
    indicator.y = board.y + (board.height - indicator.height) / 2;
  }
}

function auxiliaryBoard(
  parent: Board,
  name: string,
  width: number,
  height: number,
  fill: string,
  radius: number,
  stroke: string | null = null,
): Board {
  const board = penpot.createBoard();
  board.name = name;
  board.resize(width, height);
  board.fills = [{ fillColor: fill, fillOpacity: 1 }];
  board.borderRadius = radius;
  board.strokes = stroke
    ? [
        {
          strokeColor: stroke,
          strokeOpacity: 1,
          strokeWidth: 1,
          strokeStyle: 'solid',
          strokeAlignment: 'inner',
        },
      ]
    : [];
  setMetadata(board, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
  parent.appendChild(board);
  if (board.layoutChild) board.layoutChild.absolute = true;
  return board;
}

export function refreshAssetTextLayout(
  asset: Board,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
): void {
  const projected = new Map(
    projection.shapes.map((node) => [node.nodeId, node]),
  );
  synchronizeLinearContentMeasurements(asset, document, projection);
  const desiredSizes = measuredLinearDesiredSizes(asset, document, projection);
  const visit = (board: Board) => {
    refreshImageContent(board);
    const currentProjection = projected.get(
      metadata(board, ZUI_METADATA_NODE_ID),
    );
    const currentNode = document.nodes?.[currentProjection?.nodeId ?? ''];
    if (!refreshTreeRowContent(board, currentNode, document))
      refreshControlContent(board);
    applyNativeScrollContentLayout(
      board,
      document,
      currentProjection?.container,
      desiredSizes,
    );
    applyNativeWrapContentLayout(
      board,
      document,
      currentProjection?.container,
      desiredSizes,
    );
    applyWeightedFlexLayout(
      board,
      document,
      currentProjection?.container,
      desiredSizes,
    );
    const control = currentProjection?.inputControl;
    refreshDividerDecoration(board, currentProjection?.divider);
    refreshInputDecoration(board, control);
    refreshFieldDecoration(board, currentProjection?.field);
    refreshSliderDecoration(board, currentProjection?.slider);
    refreshSegmentedDecoration(board, currentProjection?.segmented);
    refreshPropertyRowDecoration(board, currentProjection?.propertyRow);
    for (const child of board.children) {
      const progress = metadata(child, 'progress-fraction');
      if (progress) {
        child.resize(
          Math.max(0.01, (board.width - 4) * Number(progress)),
          Math.max(3, Math.min(6, board.height - 4)),
        );
        child.x = board.x + 2;
        child.y = board.y + (board.height - child.height) / 2;
      }
      const iconPlacement = metadata(child, 'icon-placement');
      if (iconPlacement) {
        child.x =
          board.x +
          (iconPlacement === 'leading'
            ? 10
            : iconPlacement === 'trailing'
              ? board.width - child.width - 10
              : (board.width - child.width) / 2);
        child.y = board.y + (board.height - child.height) / 2;
      }
      if (child.type === 'board') {
        const id = metadata(child, ZUI_METADATA_NODE_ID);
        const shape = projected.get(id);
        const parentId = metadata(board, ZUI_METADATA_NODE_ID);
        const parentContainer =
          metadata(board, ZUI_METADATA_ROLE) === ZUI_ROLE_DETACHED
            ? detachedLaneContainer()
            : (projected.get(parentId)?.container ?? null);
        const node = document.nodes?.[id];
        if (
          node &&
          shape &&
          (!parentContainer || parentContainer.kind === 'free')
        ) {
          const size = semanticBoardSize(
            board,
            parentContainer,
            shape,
            node,
            document,
          );
          allocateNativeLayoutSize(child, size.width, size.height, 'anchored');
          const position = resolvedAnchoredPosition(
            board,
            size,
            layoutPoint(document, node, 'anchor'),
            layoutPoint(document, node, 'pivot'),
            layoutPoint(document, node, 'position', shape.geometry),
          );
          child.x = board.x + position.x;
          child.y = board.y + position.y;
        }
        visit(child);
      }
      if (
        child.type !== 'text' ||
        metadata(child, ZUI_METADATA_ROLE) !== ZUI_ROLE_TEXT
      )
        continue;
      if (layoutTreeRowText(board, child, currentNode, document)) continue;
      if (control) {
        const geometry = inputControlGeometry(
          control,
          board.width,
          board.height,
        ).text;
        child.resize(geometry.width, geometry.height);
        child.x = board.x + geometry.x;
        child.y = board.y + geometry.y;
        continue;
      }
      if (currentProjection?.slider) {
        layoutSliderText(board, child, currentProjection.slider);
        continue;
      }
      if (currentProjection?.field) {
        layoutFieldText(board, child, currentProjection.field);
        continue;
      }
      if (currentProjection?.segmented) {
        layoutSegmentedText(board, child, currentProjection.segmented);
        continue;
      }
      if (currentProjection?.table) {
        layoutTableText(board, child, currentProjection.table);
        continue;
      }
      if (currentProjection?.propertyRow) {
        layoutPropertyRowText(board, child, currentProjection.propertyRow);
        continue;
      }
      const role = board.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'prefab-role',
      ) as ZuiPrefabRole;
      const inset = textInsets(role, board.width);
      if (metadata(board, 'has-leading-icon') === 'true')
        inset.left = Math.max(inset.left, 36);
      if (metadata(board, 'has-trailing-icon') === 'true')
        inset.right = Math.max(inset.right, 36);
      const width = Math.max(1, board.width - inset.left - inset.right);
      const height = Math.max(1, board.height);
      const labelColumn =
        metadata(board, ZUI_METADATA_COMPONENT) === 'PropertyRow'
          ? board.children.find(
              (item) =>
                item.type === 'board' &&
                metadata(item, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE,
            )
          : undefined;
      child.resize(
        labelColumn ? Math.max(1, labelColumn.width - inset.left - 4) : width,
        height,
      );
      child.x = board.x + inset.left;
      child.y = board.y + Math.max(0, (board.height - height) / 2);
    }
  };
  visit(asset);
  refreshPopupOverlays(asset, document, projection);
}

function nestedTable(
  value: unknown,
  key: string,
): Record<string, unknown> | null {
  if (!value || typeof value !== 'object' || Array.isArray(value)) return null;
  const nested = (value as Record<string, unknown>)[key];
  return nested && typeof nested === 'object' && !Array.isArray(nested)
    ? (nested as Record<string, unknown>)
    : null;
}

function layoutPoint(
  document: ZuiDocument,
  node: ZuiNode,
  key: 'anchor' | 'pivot' | 'position',
  fallback: { x: number; y: number } = { x: 0, y: 0 },
): { x: number; y: number } {
  const table = nestedTable(node.layout, key);
  return {
    x: resolveDesignNumber(document, table?.['x']) ?? fallback.x,
    y: resolveDesignNumber(document, table?.['y']) ?? fallback.y,
  };
}

function setMetadata(shape: Shape, key: string, value: string): void {
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

function metadata(shape: Shape, key: string): string {
  return shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, key);
}

function sanitizeFileName(value: string): string {
  const tail = value.replaceAll('\\', '/').split('/').pop() || 'zircon-ui.zui';
  const safe = tail.replace(/[^a-zA-Z0-9._-]+/g, '-').replace(/^-+|-+$/g, '');
  return safe.toLowerCase().endsWith('.zui')
    ? safe
    : `${safe || 'zircon-ui'}.zui`;
}
