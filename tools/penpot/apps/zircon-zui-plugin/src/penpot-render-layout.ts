import type { LayoutChildProperties } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { resolveDesignNumber } from './bridge/zui-prefab-system';

export const ASSET_GAP = 48;
const DETACHED_GAP = 12;
const DETACHED_MIN_WIDTH = 280;
const DETACHED_PADDING = 16;
const LAYOUT_TOLERANCE = 0.5;

export function childDimensionSizing(
  document: ZuiDocument,
  dimension: Record<string, unknown> | undefined,
  contentContainer: boolean,
  boundary: unknown,
): 'auto' | 'fill' | 'fix' {
  if (dimension?.['stretch'] === 'Stretch') return 'fill';
  if (
    contentContainer &&
    (boundary === undefined || boundary === 'ContentDriven') &&
    resolveDesignNumber(document, dimension?.['preferred']) === null
  )
    return 'auto';
  return 'fix';
}

type LayoutChildConstraintKey =
  'minWidth' | 'maxWidth' | 'minHeight' | 'maxHeight';

type LayoutChildConstraints = Pick<
  LayoutChildProperties,
  LayoutChildConstraintKey
>;

/**
 * Applies only concrete bounds. Penpot's current plugin runtime rejects null
 * through these setters despite the nullable values in its public type.
 */
export function applyResolvedLayoutChildConstraints(
  target: LayoutChildConstraints,
  constraints: LayoutChildConstraints,
): void {
  const keys: readonly LayoutChildConstraintKey[] = [
    'minWidth',
    'maxWidth',
    'minHeight',
    'maxHeight',
  ];
  for (const key of keys) {
    const value = constraints[key];
    if (typeof value === 'number' && Number.isFinite(value)) {
      target[key] = value;
    }
  }
}

export interface PreviewSize {
  width: number;
  height: number;
}

interface PreviewPoint {
  x: number;
  y: number;
}

export interface PreviewAssetGeometry {
  primaryWidth: number;
  primaryHeight: number;
  detachedLaneX: number | null;
  detachedLaneWidth: number;
  detachedLaneHeight: number;
  width: number;
  height: number;
}

export type LayoutAuditParentKind =
  'asset' | 'detached' | 'flex' | 'grid' | 'scroll' | 'free';

/** Scroll semantics inherited from an ancestor, not a synthetic clipping rule. */
export interface LayoutScrollContext {
  horizontal: boolean;
  vertical: boolean;
}

export interface LayoutBoundsCandidate {
  nodeId: string;
  hidden?: boolean;
  /** Authored Overlay semantics, retained only for visual audit policy. */
  explicitOverlay?: boolean;
  /** Scroll axes supplied by an ancestor between this node and its asset root. */
  scrollContext?: LayoutScrollContext;
  parentKind: LayoutAuditParentKind;
  parentClips: boolean;
  parentWidth: number;
  parentHeight: number;
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface RenderedLayoutAudit {
  /** Current geometry is diagnostic only when supported source reflow failed. */
  refreshError?: string;
  assetBounds?: { x: number; y: number; width: number; height: number };
  /** Intrinsic native content measurements independent of arranged frames. */
  contentMeasurements?: {
    styleSignature?: string;
    nodeId: string;
    text: string;
    width: number;
    height: number;
  }[];
  /** Asset-level pooled native table measurements used by layout settlement. */
  tableMeasurements?: {
    shapeId: string;
    text: string;
    width: number;
    height: number;
  }[];
  semanticNodes?: {
    nodeId: string;
    shapeId: string;
    sourcePath?: string | null;
    sourceNodeId?: string | null;
    controlId?: string | null;
    instancePath?: string;
    parentSourcePath?: string | null;
    parentSourceNodeId?: string | null;
    parentInstancePath?: string;
    parentNodeId: string | null;
    component: string;
    visible: boolean;
    detached: boolean;
    /** Native Penpot clip frame used by text-structure visibility audits. */
    clip?: boolean;
    /** Effective clip frame in asset coordinates, when available. */
    clipBounds?: { x: number; y: number; width: number; height: number };
    nativeComponent?: { id: string; source: string; copy: boolean };
    bounds: { x: number; y: number; width: number; height: number };
    text: string;
    textParts?: {
      shapeId: string;
      text: string;
      bounds: { x: number; y: number; width: number; height: number };
    }[];
    /** Capture-only coordinates recorded for an explicit scroll-end review. */
    reviewScroll?: {
      originalX?: number;
      originalY?: number;
      targetX?: number;
      targetY?: number;
      capturedX?: number;
      capturedY?: number;
    };
    tableMeasurements?: {
      shapeId: string;
      text: string;
      width: number;
      height: number;
    }[];
  }[];
  totalNodes: number;
  checkedNodes: number;
  overflowCount: number;
  overflowNodeIds: string[];
  overflowDetails: LayoutBoundsCandidate[];
  invalidGeometryCount: number;
  invalidGeometryNodeIds: string[];
  invalidGeometryDetails: LayoutBoundsCandidate[];
}

export function previewBoardMinimum(assetKind: string): {
  width: number;
  height: number;
} {
  return assetKind === 'component'
    ? { width: 240, height: 48 }
    : { width: 320, height: 240 };
}

export function previewAssetGeometry(
  assetKind: string,
  rootSizes: PreviewSize[],
  detachedSizes: PreviewSize[],
  tokenOverview: PreviewSize | null,
): PreviewAssetGeometry {
  const minimum = previewBoardMinimum(assetKind);
  const primaryWidth = Math.max(
    minimum.width,
    tokenOverview?.width ?? 0,
    ...rootSizes.map(({ width }) => width),
  );
  const primaryHeight = Math.max(
    minimum.height,
    tokenOverview?.height ?? 0,
    ...rootSizes.map(({ height }) => height),
  );
  const detachedLaneWidth =
    detachedSizes.length === 0
      ? 0
      : Math.max(
          DETACHED_MIN_WIDTH,
          DETACHED_PADDING * 2,
          ...detachedSizes.map(({ width }) => width + DETACHED_PADDING * 2),
        );
  const detachedContentHeight =
    detachedSizes.length === 0
      ? 0
      : DETACHED_PADDING * 2 +
        detachedSizes.reduce((total, { height }) => total + height, 0) +
        DETACHED_GAP * (detachedSizes.length - 1);
  const detachedLaneHeight =
    detachedSizes.length === 0
      ? 0
      : Math.max(primaryHeight, detachedContentHeight);
  const detachedLaneX =
    detachedSizes.length === 0 ? null : primaryWidth + ASSET_GAP;

  return {
    primaryWidth,
    primaryHeight,
    detachedLaneX,
    detachedLaneWidth,
    detachedLaneHeight,
    width:
      primaryWidth +
      (detachedLaneWidth === 0 ? 0 : ASSET_GAP + detachedLaneWidth),
    height: Math.max(primaryHeight, detachedLaneHeight),
  };
}

export function previewAnchoredContainerMinimum(
  child: PreviewSize,
  anchor: PreviewPoint,
  pivot: PreviewPoint,
  position: PreviewPoint,
): PreviewSize {
  return {
    width: anchoredContainerDimension(
      child.width,
      anchor.x,
      pivot.x,
      position.x,
    ),
    height: anchoredContainerDimension(
      child.height,
      anchor.y,
      pivot.y,
      position.y,
    ),
  };
}

export function renderedRelativePosition(
  child: PreviewPoint,
  parent: PreviewPoint,
): PreviewPoint {
  return { x: child.x - parent.x, y: child.y - parent.y };
}

export function auditLayoutBounds(
  candidates: LayoutBoundsCandidate[],
): RenderedLayoutAudit {
  const overflowNodeIds: string[] = [];
  const overflowDetails: LayoutBoundsCandidate[] = [];
  const invalidGeometryNodeIds: string[] = [];
  const invalidGeometryDetails: LayoutBoundsCandidate[] = [];
  let checkedNodes = 0;

  for (const candidate of candidates) {
    if (candidate.hidden) continue;
    if (candidate.parentKind === 'free' && !candidate.parentClips) {
      recordInvalidGeometry(
        candidate,
        invalidGeometryNodeIds,
        invalidGeometryDetails,
      );
      continue;
    }
    checkedNodes += 1;
    if (
      recordInvalidGeometry(
        candidate,
        invalidGeometryNodeIds,
        invalidGeometryDetails,
      )
    )
      continue;
    // An authored Overlay may intentionally paint beyond a clipped free
    // parent (for example a popup shadow or anchored decoration). Keep the
    // candidate in the audit and still reject invalid numbers, but do not
    // treat the clipped visual extent as an accidental layout overflow.
    if (
      candidate.explicitOverlay === true &&
      candidate.parentKind === 'free' &&
      candidate.parentClips
    ) {
      continue;
    }
    const horizontalOverflow =
      candidate.x < -LAYOUT_TOLERANCE ||
      candidate.x + candidate.width > candidate.parentWidth + LAYOUT_TOLERANCE;
    const verticalOverflow =
      candidate.y < -LAYOUT_TOLERANCE ||
      candidate.y + candidate.height >
        candidate.parentHeight + LAYOUT_TOLERANCE;
    // Scrolling permits content beyond its viewport. A descendant must still
    // fit its own flex/grid parent, even when that parent lives in the scroll.
    const scrollContext =
      candidate.parentKind === 'scroll' ? candidate.scrollContext : undefined;
    if (
      (horizontalOverflow && !scrollContext?.horizontal) ||
      (verticalOverflow && !scrollContext?.vertical)
    ) {
      overflowNodeIds.push(candidate.nodeId);
      overflowDetails.push(candidate);
    }
  }

  return {
    totalNodes: candidates.length,
    checkedNodes,
    overflowCount: overflowNodeIds.length,
    overflowNodeIds,
    overflowDetails,
    invalidGeometryCount: invalidGeometryNodeIds.length,
    invalidGeometryNodeIds,
    invalidGeometryDetails,
  };
}

function anchoredContainerDimension(
  childSize: number,
  anchor: number,
  pivot: number,
  position: number,
): number {
  const minimums = [Math.max(1, childSize)];
  if (anchor > 0) minimums.push((childSize * pivot - position) / anchor);
  if (anchor < 1) {
    minimums.push((childSize * (1 - pivot) + position) / (1 - anchor));
  }
  return Math.max(...minimums.filter(Number.isFinite));
}

function recordInvalidGeometry(
  candidate: LayoutBoundsCandidate,
  nodeIds: string[],
  details: LayoutBoundsCandidate[],
): boolean {
  const values = [
    candidate.parentWidth,
    candidate.parentHeight,
    candidate.x,
    candidate.y,
    candidate.width,
    candidate.height,
  ];
  const invalid =
    values.some((value) => !Number.isFinite(value)) ||
    candidate.parentWidth < 0 ||
    candidate.parentHeight < 0 ||
    candidate.width < 0 ||
    candidate.height < 0;
  if (invalid) {
    nodeIds.push(candidate.nodeId);
    details.push(candidate);
  }
  return invalid;
}

export function fillsFreeParentDimension(
  _document: Pick<ZuiDocument, 'asset' | 'tokens'>,
  dimension: Record<string, unknown> | undefined,
): boolean {
  return dimension?.['stretch'] === 'Stretch';
}

/** Matches Runtime arranged_axis_extent: stretch uses available, then min/max. */
export function arrangedFreeParentExtent(
  document: Pick<ZuiDocument, 'asset' | 'tokens'>,
  dimension: Record<string, unknown> | undefined,
  preferred: number,
  available: number,
): number {
  const minimum =
    resolveDesignNumber(document as ZuiDocument, dimension?.['min']) ?? 0;
  const maximum =
    resolveDesignNumber(document as ZuiDocument, dimension?.['max']) ??
    Infinity;
  const base = fillsFreeParentDimension(document, dimension)
    ? available
    : preferred;
  return Math.max(1, minimum, Math.min(base, Math.max(minimum, maximum)));
}
