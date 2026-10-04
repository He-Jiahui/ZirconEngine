import type { Fill, Stroke } from '@penpot/plugin-types';

export interface CapturedGeometry {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface LayoutPoint {
  x: number;
  y: number;
}

export interface LayoutSize {
  width: number;
  height: number;
}

export interface CapturedLayoutSizing {
  horizontal: string | null;
  vertical: string | null;
}

export type CapturedContainerKind = 'free' | 'flex' | 'grid';

export function resolvedAnchoredPosition(
  parent: LayoutSize,
  child: LayoutSize,
  anchor: LayoutPoint,
  pivot: LayoutPoint,
  position: LayoutPoint,
): LayoutPoint {
  return {
    x: parent.width * anchor.x - child.width * pivot.x + position.x,
    y: parent.height * anchor.y - child.height * pivot.y + position.y,
  };
}

export function authoredAnchoredPosition(
  parent: LayoutSize,
  child: LayoutSize,
  anchor: LayoutPoint,
  pivot: LayoutPoint,
  resolved: LayoutPoint,
): LayoutPoint {
  return {
    x: resolved.x - parent.width * anchor.x + child.width * pivot.x,
    y: resolved.y - parent.height * anchor.y + child.height * pivot.y,
  };
}

export function gridCellForChildIndex(
  index: number,
  columns = 1,
): {
  row: number;
  column: number;
} {
  if (!Number.isInteger(index) || index < 0) {
    throw new Error('Grid child index must be a non-negative integer.');
  }
  if (!Number.isInteger(columns) || columns <= 0) {
    throw new Error('Grid column count must be a positive integer.');
  }
  // Penpot grid cell coordinates are one-based, unlike track mutation indexes.
  return {
    row: Math.floor(index / columns) + 1,
    column: (index % columns) + 1,
  };
}

export function stableSemanticChildOrder(
  childNodeIds: readonly string[],
  siblingIndexByNode: ReadonlyMap<string, number>,
): string[] {
  const traversalIndex = new Map(
    childNodeIds.map((nodeId, index) => [nodeId, index]),
  );
  return [...childNodeIds].sort((left, right) => {
    const leftIndex = siblingIndexByNode.get(left);
    const rightIndex = siblingIndexByNode.get(right);
    if (leftIndex !== undefined && rightIndex !== undefined) {
      return leftIndex - rightIndex;
    }
    if (leftIndex !== undefined) return -1;
    if (rightIndex !== undefined) return 1;
    return (traversalIndex.get(left) ?? 0) - (traversalIndex.get(right) ?? 0);
  });
}

export function resolvedParentContainerKinds(
  sourceParentId: string | null,
  currentParentId: string | null,
  baselineKinds: ReadonlyMap<string, CapturedContainerKind>,
  currentKinds: ReadonlyMap<string, CapturedContainerKind>,
): {
  baseline: CapturedContainerKind | null;
  current: CapturedContainerKind | null;
} {
  return {
    baseline:
      sourceParentId === null
        ? 'free'
        : (baselineKinds.get(sourceParentId) ?? null),
    current:
      currentParentId === null
        ? 'free'
        : (currentKinds.get(currentParentId) ?? null),
  };
}

export const EMPTY_TEXT_SENTINEL = '\u200b';

export function capturedSemanticText(
  characters: string,
  originatedEmpty: boolean,
): string {
  if (!originatedEmpty) return characters;
  return characters.startsWith(EMPTY_TEXT_SENTINEL)
    ? characters.slice(EMPTY_TEXT_SENTINEL.length)
    : characters;
}

export function capturedMappedTextStyle(
  fontSize: string | 'mixed',
  fontWeight: string | 'mixed',
  align: 'left' | 'center' | 'right' | 'justify' | 'mixed' | null,
  owner: string,
): {
  fontSize: number;
  fontWeight: string;
  align: 'left' | 'center' | 'right' | 'justify';
} {
  const numericFontSize = Number(fontSize);
  if (
    fontSize === 'mixed' ||
    !Number.isFinite(numericFontSize) ||
    numericFontSize <= 0
  ) {
    throw new Error(`${owner} has a mixed or invalid font size.`);
  }
  if (fontWeight === 'mixed' || fontWeight.trim() === '') {
    throw new Error(`${owner} has a mixed or invalid font weight.`);
  }
  if (align === 'mixed' || align === null) {
    throw new Error(`${owner} has mixed or missing text alignment.`);
  }
  return { fontSize: numericFontSize, fontWeight, align };
}

export function capturedFontTypography(
  family: string,
  lineHeight: string,
  owner: string,
): { fontFamily: string; lineHeight: number } {
  if (family === 'mixed' || !family.trim())
    throw new Error(`${owner} has a mixed or missing font family.`);
  const ratio = Number(lineHeight);
  if (!Number.isFinite(ratio) || ratio <= 0)
    throw new Error(`${owner} has a mixed or invalid line height.`);
  return { fontFamily: family, lineHeight: ratio };
}

export function symmetricLayoutGap(
  rowGap: number,
  columnGap: number,
  owner: string,
): number {
  if (
    !Number.isFinite(rowGap) ||
    !Number.isFinite(columnGap) ||
    Math.abs(rowGap - columnGap) >= 0.01
  ) {
    throw new Error(
      `${owner} has asymmetric row and column gaps; .zui exposes one container gap.`,
    );
  }
  return rowGap;
}

export function authoredGeometry(
  current: CapturedGeometry,
  baseline: CapturedGeometry | undefined,
  autoLayoutChild: boolean,
  horizontalSizing: string | undefined,
  verticalSizing: string | undefined,
): CapturedGeometry {
  if (!baseline || !autoLayoutChild) return current;
  return {
    x: baseline.x,
    y: baseline.y,
    width: horizontalSizing === 'fill' ? baseline.width : current.width,
    height: verticalSizing === 'fill' ? baseline.height : current.height,
  };
}

export function layoutSizingIsCompatible(
  baseline: CapturedLayoutSizing,
  current: CapturedLayoutSizing,
  baselineParentKind: 'free' | 'flex' | 'grid' | null,
  currentParentKind: 'free' | 'flex' | 'grid' | null,
): boolean {
  if (
    baseline.horizontal === current.horizontal &&
    baseline.vertical === current.vertical
  ) {
    return true;
  }
  if (baselineParentKind === null || currentParentKind === null) return false;
  const baselineAuto = baselineParentKind !== 'free';
  const currentAuto = currentParentKind !== 'free';
  if (baselineAuto === currentAuto) return false;
  if (!baselineAuto && currentAuto) {
    return (
      baseline.horizontal === null &&
      baseline.vertical === null &&
      current.horizontal === 'fix' &&
      current.vertical === 'fix'
    );
  }
  return current.horizontal === null && current.vertical === null;
}

export function singleSolidFill(
  fills: readonly Fill[] | 'mixed',
  owner: string,
): Fill | undefined {
  if (fills === 'mixed') {
    throw unsupportedPaint(owner, 'mixed fills');
  }
  if (fills.length > 1) {
    throw unsupportedPaint(owner, 'multiple fills');
  }
  const fill = fills[0];
  if (!fill) return undefined;
  if (
    !fill.fillColor ||
    fill.fillColorGradient ||
    fill.fillImage ||
    fill.fillColorRefFile ||
    fill.fillColorRefId
  ) {
    throw unsupportedPaint(owner, 'a non-solid or referenced fill');
  }
  return fill;
}

export function singleSolidStroke(
  strokes: readonly Stroke[],
  owner: string,
): Stroke | undefined {
  if (strokes.length > 1) {
    throw unsupportedPaint(owner, 'multiple strokes');
  }
  const stroke = strokes[0];
  if (!stroke) return undefined;
  if (
    !stroke.strokeColor ||
    stroke.strokeColorGradient ||
    stroke.strokeImage ||
    stroke.strokeColorRefFile ||
    stroke.strokeColorRefId ||
    (stroke.strokeStyle !== undefined && stroke.strokeStyle !== 'solid') ||
    (stroke.strokeAlignment !== undefined &&
      stroke.strokeAlignment !== 'inner') ||
    stroke.strokeCapStart != null ||
    stroke.strokeCapEnd != null
  ) {
    throw unsupportedPaint(owner, 'an unsupported stroke');
  }
  return stroke;
}

function unsupportedPaint(owner: string, detail: string): Error {
  return new Error(
    `${owner} uses ${detail}; the .zui bridge only exports one solid fill and one inner solid stroke.`,
  );
}

/** Native Wrap derives parent alignment; unsupported authoring must be explicit. */
export function validateCapturedWrapAlignment(
  nodeId: string,
  wrap: boolean,
  alignItems: string | undefined,
  alignContent: string | undefined,
): void {
  if (wrap && (alignItems !== 'start' || alignContent !== 'start'))
    throw new Error(
      `Semantic node ${nodeId} changed native Wrap parent alignment outside the ZUI bridge profile; use supported child slot alignment or restore the imported parent alignment.`,
    );
}
