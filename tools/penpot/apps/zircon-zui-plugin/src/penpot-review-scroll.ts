import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_LAYOUT_MODE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_METADATA_SCROLL_AXIS,
  ZUI_METADATA_VISUAL_DETACHED,
  ZUI_ROLE_NODE,
} from './metadata';
import type { LayoutReviewCaseMessage } from './model';

/** Resolve the scroll axis independently of the case's interactive state. */
export function reviewScrollPosition(
  reviewCase: Pick<LayoutReviewCaseMessage, 'state' | 'scrollPosition'>,
): 'start' | 'end' | undefined {
  if (reviewCase.scrollPosition) return reviewCase.scrollPosition;
  const state = reviewCase.state.toLowerCase();
  return state === 'scroll-before'
    ? 'start'
    : state === 'scroll-after'
      ? 'end'
      : undefined;
}

/** Minimal Penpot shape surface used by the design-only scroll reviewer. */
export interface ReviewScrollShape {
  type: string;
  x: number;
  y: number;
  width: number;
  height: number;
  children?: ReviewScrollShape[];
  clipContent?: boolean;
  layoutChild?: { absolute: boolean };
  flex?: unknown;
  grid?: unknown;
  getSharedPluginData(namespace: string, key: string): string;
  setSharedPluginData(namespace: string, key: string, value: string): void;
}

function isReviewScrollShape(value: unknown): value is ReviewScrollShape {
  if (!value || typeof value !== 'object') return false;
  const shape = value as Partial<ReviewScrollShape>;
  return (
    typeof shape.type === 'string' &&
    typeof shape.x === 'number' &&
    typeof shape.y === 'number' &&
    typeof shape.width === 'number' &&
    typeof shape.height === 'number' &&
    (shape.children === undefined || Array.isArray(shape.children)) &&
    typeof shape.getSharedPluginData === 'function' &&
    typeof shape.setSharedPluginData === 'function'
  );
}

function metadata(shape: ReviewScrollShape, key: string): string {
  return shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, key);
}

const REVIEW_SCROLL_POSITION = 'review-scroll-position';
const REVIEW_SCROLL_OFFSET_X = 'review-scroll-offset-x';
const REVIEW_SCROLL_OFFSET_Y = 'review-scroll-offset-y';
const REVIEW_SCROLL_ORIGINAL_CLIP = 'review-scroll-original-clip';
const REVIEW_SCROLL_ORIGINAL_X = 'review-scroll-original-x';
const REVIEW_SCROLL_ORIGINAL_Y = 'review-scroll-original-y';
const REVIEW_SCROLL_ORIGINAL_ABSOLUTE = 'review-scroll-original-absolute';
const REVIEW_SCROLL_TARGET_X = 'review-scroll-target-x';
const REVIEW_SCROLL_TARGET_Y = 'review-scroll-target-y';
const REVIEW_SCROLL_CAPTURED_X = 'review-scroll-captured-x';
const REVIEW_SCROLL_CAPTURED_Y = 'review-scroll-captured-y';
const REVIEW_SCROLL_CAPTURED_POSITIONS = 'review-scroll-captured-positions';

export interface CapturedReviewScrollPosition {
  x: number;
  y: number;
}

export interface ReviewScrollLayoutSnapshot {
  containers: readonly (readonly CapturedReviewScrollPosition[])[];
}

export interface ReviewScrollLayoutDiagnostic {
  axis: string;
  childCount: number;
  layoutMode: string;
  positioned: boolean;
  contentExtent: number;
  viewportExtent: number;
}

// Penpot can recreate a projected child after a metadata write. Keep the
// settled geometry by both object identity and stable semantic node id until
// the design-only capture is restored.
let capturedReviewScrollPositions = new WeakMap<
  ReviewScrollShape,
  CapturedReviewScrollPosition
>();
const capturedReviewScrollPositionsByNodeId = new Map<
  string,
  CapturedReviewScrollPosition
>();

function clearCapturedReviewScrollPositions(): void {
  capturedReviewScrollPositions = new WeakMap();
  capturedReviewScrollPositionsByNodeId.clear();
}

function cacheReviewScrollPosition(shape: ReviewScrollShape): void {
  const position = { x: shape.x, y: shape.y };
  capturedReviewScrollPositions.set(shape, position);
  const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
  if (nodeId) capturedReviewScrollPositionsByNodeId.set(nodeId, position);
}

function cachedReviewScrollPosition(
  shape: ReviewScrollShape,
): CapturedReviewScrollPosition | undefined {
  return (
    capturedReviewScrollPositions.get(shape) ??
    capturedReviewScrollPositionsByNodeId.get(
      metadata(shape, ZUI_METADATA_NODE_ID),
    )
  );
}

function capturedReviewScrollPositionFromContainer(
  container: ReviewScrollShape,
  child: ReviewScrollShape,
  childIndex: number,
): CapturedReviewScrollPosition | undefined {
  const nodeId = metadata(child, ZUI_METADATA_NODE_ID);
  const serialized = metadata(container, REVIEW_SCROLL_CAPTURED_POSITIONS);
  if (!serialized) return undefined;
  try {
    const parsed: unknown = JSON.parse(serialized);
    if (!parsed || typeof parsed !== 'object') return undefined;
    const values = parsed as Record<string, unknown>;
    const byNodeId = values['byNodeId'] as Record<string, unknown> | undefined;
    const byIndex = values['byIndex'];
    const value =
      (nodeId ? byNodeId?.[nodeId] : undefined) ??
      (Array.isArray(byIndex) ? byIndex[childIndex] : undefined);
    if (!value || typeof value !== 'object') return undefined;
    const { x, y } = value as Record<string, unknown>;
    if (typeof x !== 'number' || typeof y !== 'number') return undefined;
    return Number.isFinite(x) && Number.isFinite(y) ? { x, y } : undefined;
  } catch {
    return undefined;
  }
}

function remember(shape: ReviewScrollShape, key: string, value: string): void {
  if (!metadata(shape, key))
    shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

function originalClip(shape: ReviewScrollShape): string {
  if (shape.clipContent === undefined) return 'unset';
  return shape.clipContent ? 'true' : 'false';
}

function nodeChildren(shape: ReviewScrollShape): ReviewScrollShape[] {
  return (shape.children ?? []).filter(
    (child) =>
      child.type === 'board' &&
      metadata(child, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE &&
      metadata(child, ZUI_METADATA_VISUAL_DETACHED) !== 'true',
  );
}

/**
 * A scroll-end capture must not detach Flex/Grid children until Penpot has
 * assigned their authored positions. A quiet all-at-origin frame is an
 * unarranged intermediate state, not a stable layout.
 */
export function reviewScrollLayoutReady(root: unknown): boolean {
  const diagnostics = reviewScrollLayoutDiagnostics(root);
  return (
    diagnostics.length > 0 &&
    diagnostics.every(
      ({ childCount, positioned, layoutMode }) =>
        childCount <= 1 || !['flex', 'grid'].includes(layoutMode) || positioned,
    )
  );
}

/**
 * Readiness used by a capture transaction. An all-at-origin auto-layout frame
 * is safe when the measured content fits the viewport: the scroll-end offset
 * is zero and no child will be detached. Overflowing content still requires
 * real Penpot positions before it can be moved.
 */
export function reviewScrollLayoutReadyForCapture(root: unknown): boolean {
  const diagnostics = reviewScrollLayoutDiagnostics(root);
  return (
    diagnostics.length > 0 &&
    diagnostics.every(
      ({ childCount, positioned, contentExtent, viewportExtent, layoutMode }) =>
        childCount <= 1 ||
        !['flex', 'grid'].includes(layoutMode) ||
        positioned ||
        contentExtent <= viewportExtent + 0.5,
    )
  );
}

/**
 * Explain the readiness decision without exposing Penpot objects. This is
 * also useful when a real frontend keeps an auto-layout scroll container at
 * its transient all-at-origin frame.
 */
export function reviewScrollLayoutDiagnostics(
  root: unknown,
): ReviewScrollLayoutDiagnostic[] {
  if (!isReviewScrollShape(root)) return [];
  const diagnostics: ReviewScrollLayoutDiagnostic[] = [];
  const visit = (shape: ReviewScrollShape): void => {
    const axis = metadata(shape, ZUI_METADATA_SCROLL_AXIS);
    if (
      shape.type === 'board' &&
      ['horizontal', 'vertical', 'both'].includes(axis)
    ) {
      const children = nodeChildren(shape);
      const layoutMode = metadata(shape, ZUI_METADATA_LAYOUT_MODE);
      const usesAutoLayout =
        layoutMode === 'flex' ||
        layoutMode === 'grid' ||
        Boolean(shape.flex || shape.grid);
      const xPositions = new Set(children.map((child) => child.x));
      const yPositions = new Set(children.map((child) => child.y));
      const horizontal = axis === 'horizontal';
      const vertical = axis === 'vertical';
      const positioned = horizontal
        ? xPositions.size > 1
        : vertical
          ? yPositions.size > 1
          : xPositions.size > 1 || yPositions.size > 1;
      const flex = isRecord(shape.flex) ? shape.flex : undefined;
      const grid = isRecord(shape.grid) ? shape.grid : undefined;
      const spacing = horizontal
        ? (numberProperty(flex, 'columnGap') ??
          numberProperty(grid, 'columnGap') ??
          0)
        : (numberProperty(flex, 'rowGap') ??
          numberProperty(grid, 'rowGap') ??
          0);
      const padding = horizontal
        ? (numberProperty(flex, 'leftPadding') ?? 0) +
          (numberProperty(flex, 'rightPadding') ?? 0)
        : (numberProperty(flex, 'topPadding') ?? 0) +
          (numberProperty(flex, 'bottomPadding') ?? 0);
      const contentExtent =
        padding +
        children.reduce(
          (total, child) => total + (horizontal ? child.width : child.height),
          0,
        ) +
        spacing * Math.max(0, children.length - 1);
      diagnostics.push({
        axis,
        childCount: children.length,
        layoutMode: usesAutoLayout
          ? layoutMode || (shape.flex ? 'flex' : 'grid')
          : 'free',
        positioned,
        contentExtent,
        viewportExtent: horizontal ? shape.width : shape.height,
      });
    }
    for (const child of shape.children ?? []) {
      if (isReviewScrollShape(child)) visit(child);
    }
  };
  visit(root);
  return diagnostics;
}

function numberProperty(
  value: Record<string, unknown> | undefined,
  key: string,
): number | undefined {
  const candidate = value?.[key];
  return typeof candidate === 'number' && Number.isFinite(candidate)
    ? candidate
    : undefined;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * Freeze settled global coordinates before a scroll-end capture changes
 * Flex/Grid children to absolute positioning. Penpot can briefly expose
 * origin coordinates during that transition, so the capture must not sample
 * live positions after the reviewer has declared the layout ready.
 */
export function captureReviewScrollLayoutSnapshot(
  root: unknown,
): ReviewScrollLayoutSnapshot {
  clearCapturedReviewScrollPositions();
  const containers: CapturedReviewScrollPosition[][] = [];
  if (!isReviewScrollShape(root)) return { containers };
  const visit = (shape: ReviewScrollShape): void => {
    const axis = metadata(shape, ZUI_METADATA_SCROLL_AXIS);
    if (
      shape.type === 'board' &&
      ['horizontal', 'vertical', 'both'].includes(axis)
    ) {
      const capturedByNodeId: Record<string, CapturedReviewScrollPosition> = {};
      const capturedByIndex: CapturedReviewScrollPosition[] = [];
      for (const child of nodeChildren(shape)) {
        cacheReviewScrollPosition(child);
        capturedByIndex.push({ x: child.x, y: child.y });
        const nodeId = metadata(child, ZUI_METADATA_NODE_ID);
        if (nodeId) capturedByNodeId[nodeId] = { x: child.x, y: child.y };
        child.setSharedPluginData(
          ZUI_METADATA_NAMESPACE,
          REVIEW_SCROLL_CAPTURED_X,
          String(child.x),
        );
        child.setSharedPluginData(
          ZUI_METADATA_NAMESPACE,
          REVIEW_SCROLL_CAPTURED_Y,
          String(child.y),
        );
      }
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_CAPTURED_POSITIONS,
        JSON.stringify({
          byNodeId: capturedByNodeId,
          byIndex: capturedByIndex,
        }),
      );
      containers.push(capturedByIndex);
    }
    for (const child of shape.children ?? []) {
      if (isReviewScrollShape(child)) visit(child);
    }
  };
  visit(root);
  return { containers };
}

export function captureReviewScrollLayoutPositions(root: unknown): number {
  return captureReviewScrollLayoutSnapshot(root).containers.length;
}

function capturedCoordinate(
  shape: ReviewScrollShape,
  key: string,
  fallback: number,
): number {
  const value = metadata(shape, key);
  if (!value) return fallback;
  const captured = Number(value);
  return Number.isFinite(captured) ? captured : fallback;
}

function moveToScrollEnd(
  shape: ReviewScrollShape,
  axis: string,
  snapshotPositions?: readonly CapturedReviewScrollPosition[],
): void {
  const children = nodeChildren(shape);
  if (!children.length) return;
  const horizontal = axis === 'horizontal' || axis === 'both';
  const vertical = axis === 'vertical' || axis === 'both';
  const positions = children.map((child, childIndex) => {
    const captured =
      snapshotPositions?.[childIndex] ??
      capturedReviewScrollPositionFromContainer(shape, child, childIndex) ??
      cachedReviewScrollPosition(child);
    return {
      child,
      x:
        captured?.x ??
        capturedCoordinate(child, REVIEW_SCROLL_CAPTURED_X, child.x),
      y:
        captured?.y ??
        capturedCoordinate(child, REVIEW_SCROLL_CAPTURED_Y, child.y),
    };
  });
  const offsetX = horizontal
    ? Math.max(
        0,
        Math.max(...positions.map(({ child, x }) => x + child.width)) -
          (shape.x + shape.width),
      )
    : 0;
  const offsetY = vertical
    ? Math.max(
        0,
        Math.max(...positions.map(({ child, y }) => y + child.height)) -
          (shape.y + shape.height),
      )
    : 0;
  if (!offsetX && !offsetY) return;
  for (const { child, x: originalX, y: originalY } of positions) {
    // Detaching a Penpot auto-layout child can reset its live coordinates.
    // Preserve the measured position before changing layout participation.
    remember(child, REVIEW_SCROLL_ORIGINAL_X, String(originalX));
    remember(child, REVIEW_SCROLL_ORIGINAL_Y, String(originalY));
    if (child.layoutChild)
      remember(
        child,
        REVIEW_SCROLL_ORIGINAL_ABSOLUTE,
        child.layoutChild.absolute ? 'true' : 'false',
      );
    if (child.layoutChild) child.layoutChild.absolute = true;
    const targetX = originalX - offsetX;
    const targetY = originalY - offsetY;
    child.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      REVIEW_SCROLL_TARGET_X,
      String(targetX),
    );
    child.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      REVIEW_SCROLL_TARGET_Y,
      String(targetY),
    );
    child.x = targetX;
    child.y = targetY;
  }
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    REVIEW_SCROLL_OFFSET_X,
    String(offsetX),
  );
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    REVIEW_SCROLL_OFFSET_Y,
    String(offsetY),
  );
}

/**
 * Apply a scroll position to explicit runtime scroll containers only. The
 * document remains authored; this moves the projected Penpot content inside
 * its clipped viewport so a capture can inspect both ends of a long list.
 */
export function applyReviewScrollPresentation(
  root: unknown,
  position: 'start' | 'end',
  snapshot?: ReviewScrollLayoutSnapshot,
): number {
  if (!isReviewScrollShape(root)) return 0;
  let containers = 0;
  const visit = (shape: ReviewScrollShape): void => {
    const axis = metadata(shape, ZUI_METADATA_SCROLL_AXIS);
    if (
      shape.type === 'board' &&
      ['horizontal', 'vertical', 'both'].includes(axis)
    ) {
      const snapshotPositions = snapshot?.containers[containers];
      containers += 1;
      remember(shape, REVIEW_SCROLL_ORIGINAL_CLIP, originalClip(shape));
      shape.clipContent = true;
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_POSITION,
        position,
      );
      if (position === 'end') moveToScrollEnd(shape, axis, snapshotPositions);
    }
    for (const child of shape.children ?? []) {
      if (isReviewScrollShape(child)) visit(child);
    }
  };
  visit(root);
  return containers;
}

/**
 * Restore the real Penpot geometry after a design-only scroll capture. This
 * runs before export so the source reconciler only sees authored edits.
 */
export function restoreReviewScrollPresentation(root: unknown): number {
  clearCapturedReviewScrollPositions();
  if (!isReviewScrollShape(root)) return 0;
  let containers = 0;
  const visit = (shape: ReviewScrollShape): void => {
    const originalClipValue = metadata(shape, REVIEW_SCROLL_ORIGINAL_CLIP);
    if (originalClipValue) {
      containers += 1;
      shape.clipContent =
        originalClipValue === 'unset'
          ? undefined
          : originalClipValue === 'true';
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_ORIGINAL_CLIP,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_POSITION,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_OFFSET_X,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_OFFSET_Y,
        '',
      );
    }
    const originalX = metadata(shape, REVIEW_SCROLL_ORIGINAL_X);
    const originalY = metadata(shape, REVIEW_SCROLL_ORIGINAL_Y);
    if (originalX) shape.x = Number(originalX);
    if (originalY) shape.y = Number(originalY);
    if (originalX || originalY) {
      const originalAbsolute = metadata(shape, REVIEW_SCROLL_ORIGINAL_ABSOLUTE);
      if (shape.layoutChild && originalAbsolute)
        shape.layoutChild.absolute = originalAbsolute === 'true';
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_ORIGINAL_X,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_ORIGINAL_Y,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_ORIGINAL_ABSOLUTE,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_TARGET_X,
        '',
      );
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        REVIEW_SCROLL_TARGET_Y,
        '',
      );
    }
    shape.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      REVIEW_SCROLL_CAPTURED_X,
      '',
    );
    shape.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      REVIEW_SCROLL_CAPTURED_Y,
      '',
    );
    shape.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      REVIEW_SCROLL_CAPTURED_POSITIONS,
      '',
    );
    for (const child of shape.children ?? []) {
      if (isReviewScrollShape(child)) visit(child);
    }
  };
  visit(root);
  return containers;
}
