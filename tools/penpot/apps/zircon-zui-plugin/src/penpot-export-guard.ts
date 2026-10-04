import { hasCurrentPopupLayoutGeometry } from './penpot-popup-layout-geometry';
import { isCurrentContentMeasurement } from './penpot-content-measurement-guard';
import type { Board, Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_NODE,
  ZUI_ROLE_TEXT,
} from './metadata';
import {
  guardedPathContent,
  isGeneratedPath,
  PATH_NUMBER,
} from './penpot-icon-path-content';

const EXPORT_GUARD = 'export-shapes-guard';
const EXPORT_GUARD_VERSION = 5;
const ICON_PATH_NUMERIC_TOLERANCE = 0.0001;
type ShapeGuard = Record<string, unknown>;

function guardedState(shape: Shape): ShapeGuard {
  const role = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
  );
  const semantic = role === ZUI_ROLE_NODE;
  const text = role === ZUI_ROLE_TEXT && shape.type === 'text';
  const layout = shape.layoutChild;
  const state: ShapeGuard = {
    type: shape.type,
    role,
    nodeId: shape.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_NODE_ID,
    ),
    parent: role === ZUI_ROLE_ASSET ? null : shape.parent?.id,
    children:
      'children' in shape ? shape.children.map((child) => child.id) : [],
    hidden: shape.hidden,
    rotation: shape.rotation,
    flipX: shape.flipX,
    flipY: shape.flipY,
    blendMode: shape.blendMode,
    shadows: shape.shadows,
    blur: shape.blur,
    backgroundBlur: shape.backgroundBlur,
    constraintsHorizontal: shape.constraintsHorizontal,
    constraintsVertical: shape.constraintsVertical,
    fixedWhenScrolling: shape.fixedWhenScrolling,
    absolute: layout?.absolute,
    alignSelf: layout?.alignSelf,
    minWidth: layout?.minWidth,
    maxWidth: layout?.maxWidth,
    minHeight: layout?.minHeight,
    maxHeight: layout?.maxHeight,
  };
  if (semantic) {
    if (
      'children' in shape &&
      shape.children.some(
        (child) =>
          child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') ===
          'surface',
      )
    ) {
      state['fieldHostFills'] = shape.fills;
      state['fieldHostStrokes'] = shape.strokes;
      state['fieldHostRadius'] = shape.borderRadius;
    }
    state['cornerOffsets'] = [
      shape.borderRadiusTopLeft,
      shape.borderRadiusTopRight,
      shape.borderRadiusBottomRight,
      shape.borderRadiusBottomLeft,
    ].map((radius) =>
      radius === undefined ? null : radius - shape.borderRadius,
    );
  } else {
    const round = (value: number) => Math.round(value * 100) / 100;
    state['geometry'] = {
      x: role === ZUI_ROLE_ASSET ? 0 : round(shape.x - (shape.parent?.x ?? 0)),
      y: role === ZUI_ROLE_ASSET ? 0 : round(shape.y - (shape.parent?.y ?? 0)),
      width: round(shape.width),
      height: round(shape.height),
    };
    state['opacity'] = shape.opacity;
    if (shape.type === 'text') state['growType'] = shape.growType;
    if (!text) {
      state['fills'] = shape.fills;
      state['strokes'] = shape.strokes;
      state['radius'] = shape.borderRadius;
      if (shape.type === 'text') state['characters'] = shape.characters;
      if (shape.type === 'path') state['content'] = guardedPathContent(shape);
    }
  }
  if (isCurrentContentMeasurement(shape)) {
    // Source signatures validate the probe. Its measured size and characters
    // follow supported text edits and never become authored shape geometry.
    delete state['geometry'];
    delete state['characters'];
  }
  if (
    shape.type === 'text' &&
    role === ZUI_ROLE_AUXILIARY &&
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'popup-option') ===
      'label'
  ) {
    // Preview option labels have no live typography source mapping. Keep these
    // values immutable so reflow cannot discard a direct label style edit.
    state['popupTypography'] = {
      family: shape.fontFamily,
      size: shape.fontSize,
      weight: shape.fontWeight,
      variant: shape.fontVariantId,
      style: shape.fontStyle,
      lineHeight: shape.lineHeight,
      letterSpacing: shape.letterSpacing,
      align: shape.align,
      verticalAlign: shape.verticalAlign,
    };
  }
  if (hasCurrentPopupLayoutGeometry(shape)) delete state['geometry'];
  return state;
}

function shapeIndex(asset: Board): Record<string, ShapeGuard> {
  const result = Object.create(null) as Record<string, ShapeGuard>;
  const visit = (shape: Shape): void => {
    result[shape.id] = guardedState(shape);
    if ('children' in shape) for (const child of shape.children) visit(child);
  };
  visit(asset);
  return result;
}

function findShape(asset: Board, id: string): Shape | undefined {
  const visit = (shape: Shape): Shape | undefined => {
    if (shape.id === id) return shape;
    if ('children' in shape)
      for (const child of shape.children) {
        const found = visit(child);
        if (found) return found;
      }
    return undefined;
  };
  return visit(asset);
}

function shapePath(asset: Board, id: string): string {
  const names: string[] = [];
  let shape = findShape(asset, id);
  while (shape) {
    names.unshift(shape.name || shape.type);
    shape = shape.parent ?? undefined;
  }
  return names.join('/') || id;
}

/** Ignore only sub-pixel number serialization on bridge-owned icon paths. */
function equivalentGeneratedIconPathContent(
  asset: Board,
  id: string,
  before: unknown,
  after: unknown,
): boolean {
  const shape = findShape(asset, id);
  if (
    shape?.type !== 'path' ||
    typeof before !== 'string' ||
    typeof after !== 'string'
  )
    return false;
  if (
    !isGeneratedPath(shape) ||
    before.replace(PATH_NUMBER, '#') !== after.replace(PATH_NUMBER, '#')
  )
    return false;
  const left = [...before.matchAll(PATH_NUMBER)].map(([value]) =>
    Number(value),
  );
  const right = [...after.matchAll(PATH_NUMBER)].map(([value]) =>
    Number(value),
  );
  return (
    left.length > 0 &&
    left.length === right.length &&
    left.every(
      (value, index) =>
        Number.isFinite(value) &&
        Number.isFinite(right[index]) &&
        Math.abs(value - right[index]) <= ICON_PATH_NUMERIC_TOLERANCE,
    )
  );
}

function contentDifference(before: unknown, after: unknown): string {
  const left = String(before ?? '');
  const right = String(after ?? '');
  let index = 0;
  while (
    index < Math.min(left.length, right.length) &&
    left[index] === right[index]
  )
    index += 1;
  const leftNumbers = [...left.matchAll(PATH_NUMBER)].map(([value]) =>
    Number(value),
  );
  const rightNumbers = [...right.matchAll(PATH_NUMBER)].map(([value]) =>
    Number(value),
  );
  const maxDelta =
    leftNumbers.length === rightNumbers.length
      ? Math.max(
          0,
          ...leftNumbers.map((value, offset) =>
            Math.abs(value - rightNumbers[offset]),
          ),
        )
      : 'unavailable';
  return `; content lengths ${left.length}/${right.length}, first difference at ${index}; numeric tokens ${leftNumbers.length}/${rightNumbers.length}, max numeric delta ${maxDelta}, same command skeleton ${left.replace(PATH_NUMBER, '#') === right.replace(PATH_NUMBER, '#')}`;
}

export function writeExportShapeGuard(asset: Board): void {
  asset.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    EXPORT_GUARD,
    JSON.stringify({
      version: EXPORT_GUARD_VERSION,
      shapes: shapeIndex(asset),
    }),
  );
}

// These visible attributes and structural edits have no source mapping yet.
// Keep their settled baseline so an export cannot silently discard an edit.
export function assertExportShapeGuard(asset: Board): void {
  const encoded = asset.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    EXPORT_GUARD,
  );
  if (!encoded)
    throw new Error(
      'Missing ZUI export shape guard; re-import the source asset.',
    );
  const guard = JSON.parse(encoded) as {
    version?: number;
    shapes?: Record<string, ShapeGuard>;
  };
  if (
    guard.version !== EXPORT_GUARD_VERSION ||
    !guard.shapes ||
    typeof guard.shapes !== 'object' ||
    Array.isArray(guard.shapes)
  )
    throw new Error(
      'Unsupported ZUI export shape guard coordinate version; re-import the source asset.',
    );
  const expected = guard.shapes;
  const current = shapeIndex(asset);
  for (const id of new Set([
    ...Object.keys(expected),
    ...Object.keys(current),
  ])) {
    if (!expected[id] || !current[id])
      throw new Error(`Unmapped added or removed Penpot shape: ${id}`);
    for (const key of new Set([
      ...Object.keys(expected[id]),
      ...Object.keys(current[id]),
    ]))
      if (
        JSON.stringify(expected[id][key]) !==
          JSON.stringify(current[id][key]) &&
        !(
          key === 'content' &&
          equivalentGeneratedIconPathContent(
            asset,
            id,
            expected[id][key],
            current[id][key],
          )
        )
      )
        throw new Error(
          `Penpot shape ${String(current[id]['nodeId'] || id)} (${shapePath(asset, id)}) changed ${key} without a ZUI source mapping${key === 'content' ? contentDifference(expected[id][key], current[id][key]) : key === 'geometry' ? `; before ${JSON.stringify(expected[id][key])}; after ${JSON.stringify(current[id][key])}` : ''}.`,
        );
  }
}
