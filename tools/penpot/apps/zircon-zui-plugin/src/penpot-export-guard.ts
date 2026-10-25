import { hasCurrentPopupLayoutGeometry } from './penpot-popup-layout-geometry';
import { currentContentMeasurementScale, isCurrentContentMeasurement } from './penpot-content-measurement-guard';
import { hasCurrentNativeSemanticLayoutGeometry } from './penpot-native-semantic-layout-geometry';
import { decodeZuiMetadata } from './bridge/zui-document';
import { reviewHost } from './bridge/zui-review-host';
import { localizedText } from './bridge/penpot-projection';
import { nativeSemanticGlyph, nativeSemanticGlyphGeometry } from './penpot-prefab-icons';
import type { Board, Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_BASELINE,
  ZUI_METADATA_DOCUMENT,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_NODE,
  ZUI_ROLE_TEXT,
} from './metadata';
import {
  guardedPathContent,
  normalizedIconPathContent,
  parseIconPathContent,
  PATH_NUMBER,
} from './penpot-icon-path-content';

const EXPORT_GUARD = 'export-shapes-guard';
const EXPORT_GUARD_VERSION = 7;
// Real Penpot layout/save/reopen can move an internal icon vector by two
// Float32 steps (1/8192 DIP). Keep this bound below a 0.0002 path edit.
const ICON_PATH_NUMERIC_TOLERANCE = 2 ** -13;
type ShapeGuard = Record<string, unknown>;

function guardedState(shape: Shape): ShapeGuard {
  const role = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
  );
  const semantic = role === ZUI_ROLE_NODE;
  const text = role === ZUI_ROLE_TEXT && shape.type === 'text';
  const layout = shape.layoutChild;
  const baselineText = semantic && shape.type === 'board'
    ? shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_BASELINE) : '';
  const baseline: unknown = baselineText ? JSON.parse(baselineText) : null;
  const mapsHeightMin = baseline !== null && typeof baseline === 'object' &&
    !Array.isArray(baseline) && 'heightMin' in baseline &&
    typeof baseline.heightMin === 'number' && Number.isFinite(baseline.heightMin) &&
    baseline.heightMin >= 0;
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
    ...(mapsHeightMin ? { mappedHeightMin: true } : { minHeight: layout?.minHeight }),
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

/** Validate already-reflowed legacy Add content without changing its guard. */
function sourceOwnedLegacyButtonGeometry(asset: Board, shape: Shape | undefined, actual: unknown): boolean {
  const parent = shape?.parent;
  if (!shape || parent?.type !== 'board' ||
    parent.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !== ZUI_ROLE_NODE) return false;
  const textPart = shape.type === 'text' &&
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) === ZUI_ROLE_TEXT;
  if (!textPart && shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph') !== 'add-component')
    return false;
  try {
    const original = decodeZuiMetadata(asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_DOCUMENT));
    const document = reviewHost(original) ?? original;
    const id = parent.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_NODE_ID);
    const node = document.nodes?.[id];
    if (!node) return false;
    const glyph = nativeSemanticGlyph(document, node);
    if (glyph?.family !== 'add-component' || (!textPart &&
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'native-semantic-glyph-asset') !== glyph.assetPath)) return false;
    for (const [key, value] of Object.entries({ sourcePath: node['penpot_review_source_path'],
      sourceNodeId: node['penpot_review_source_node_id'], instancePath: node['penpot_review_instance_path'],
      controlId: node.control_id })) {
      if (typeof value !== 'string' || !value || parent.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) !== value)
        return false;
    }
    const probes = asset.children.filter(child => child.type === 'text' &&
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'linear-content-measurement') === id);
    if (probes.length !== 1 || probes[0].type !== 'text') return false;
    const probe = probes[0], scale = currentContentMeasurementScale(probe);
    const reference = node.state?.['text'] ?? node.props?.['text'];
    const caption = typeof reference === 'string' ? reference
      : reference !== null && typeof reference === 'object' && !Array.isArray(reference) &&
        !(reference instanceof Date) && typeof reference['text_key'] === 'string'
        ? localizedText(document, reference) : undefined;
    if (scale === undefined || caption === undefined || probe.characters !== caption)
      return false;
    const lineHeight = shape.type === 'text' ? Math.max(Number(shape.fontSize),
      Number(shape.fontSize) * Number(shape.lineHeight)) : 0;
    const geometry = nativeSemanticGlyphGeometry(document, node, parent.width, parent.height, lineHeight, probe.width / scale);
    if (!geometry?.icon || geometry.pendingMeasurement) return false;
    const target = textPart ? { ...geometry.text } : geometry.icon;
    if (textPart && shape.type === 'text') {
      const ink = Math.min(geometry.text.width, probe.width / scale);
      target.x -= Math.max(0, geometry.text.width - ink) *
        (shape.align === 'center' ? 0.5 : shape.align === 'right' ? 1 : 0);
      target.width = Math.max(0.01, target.width);
      target.height = Math.max(0.01, target.height);
    }
    const round = (value: number) => Math.round(value * 100) / 100;
    return JSON.stringify({ x: round(target.x), y: round(target.y),
      width: round(target.width), height: round(target.height) }) === JSON.stringify(actual);
  } catch { return false; }
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
  if (normalizedIconPathContent(shape) === undefined) return false;
  const left = parseIconPathContent(before);
  const right = parseIconPathContent(after);
  return (
    left !== undefined &&
    right !== undefined &&
    left.skeleton === right.skeleton &&
    left.coordinates.length === right.coordinates.length &&
    left.coordinates.every(
      (value, index) =>
        Math.abs(value - right.coordinates[index]!) <=
        ICON_PATH_NUMERIC_TOLERANCE,
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
  assertShapeIndex(asset, guard.shapes, shapeIndex(asset));
}

function assertShapeIndex(
  asset: Board, expected: Record<string, ShapeGuard>, current: Record<string, ShapeGuard>,
): void {
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
          key === 'geometry' &&
          (hasCurrentNativeSemanticLayoutGeometry(findShape(asset, id)) ||
            sourceOwnedLegacyButtonGeometry(asset, findShape(asset, id), current[id][key]))
        ) &&
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
          `Penpot shape ${String(current[id]['nodeId'] || id)} (${shapePath(asset, id)}) changed ${key} without a ZUI source mapping${key === 'content' ? contentDifference(expected[id][key], current[id][key]) : ['geometry', 'strokes', 'fieldHostStrokes'].includes(key) ? `; before ${JSON.stringify(expected[id][key])}; after ${JSON.stringify(current[id][key])}` : ''}.`,
        );
  }
}


/** A source-admitted canonical migration may change these two constraints only.
 * No geometry, paint, text, topology or other baseline is captured again.
 */
export async function migrateSearchFieldWidthGuard(
  asset: Board,
  changes: ReadonlyArray<{ shape: Shape; before: Shape['constraintsHorizontal']; after: 'leftright' }>,
  settle: () => Promise<void>,
): Promise<boolean> {
  assertExportShapeGuard(asset);
  if (changes.length !== 2 || new Set(changes.map(({ shape }) => shape.id)).size !== 2)
    throw new Error('Search width migration requires exactly the surface and primary text.');
  for (const { shape, before, after } of changes) {
    // SDK children getters return fresh proxies for the same native shape.
    const current = findShape(asset, shape.id);
    if (!current || current.type !== shape.type || current.parent?.id !== shape.parent?.id ||
        current.constraintsHorizontal !== before || shape.constraintsHorizontal !== before ||
        !['left', 'leftright'].includes(before) || after !== 'leftright')
      throw new Error('Search width migration input changed.');
  }
  const [surface, text] = changes.map(({ shape }) => shape);
  if (surface.type !== 'board' || surface.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') !== 'surface' ||
      text.type !== 'text' || text.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) !== ZUI_ROLE_TEXT ||
      (text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-fragment') || 'primary') !== 'primary' ||
      !surface.parent || surface.parent.id !== text.parent?.id)
    throw new Error('Search width migration is not mapped to the field surface and primary text.');
  if (changes.every(({ before, after }) => before === after)) return false;
  const encoded = asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, EXPORT_GUARD);
  const next = JSON.parse(encoded) as { version: number; shapes: Record<string, ShapeGuard> };
  for (const { shape, after } of changes) next.shapes[shape.id]['constraintsHorizontal'] = after;
  try {
    for (const { shape, after } of changes) shape.constraintsHorizontal = after;
    await settle();
    if (asset.getSharedPluginData(ZUI_METADATA_NAMESPACE, EXPORT_GUARD) !== encoded)
      throw new Error('Search width migration guard changed concurrently.');
    for (const { shape, after } of changes)
      if (shape.constraintsHorizontal !== after)
        throw new Error('Penpot did not preserve SearchField width constraints.');
    assertShapeIndex(asset, next.shapes, shapeIndex(asset));
    asset.setSharedPluginData(ZUI_METADATA_NAMESPACE, EXPORT_GUARD, JSON.stringify(next));
    return true;
  } catch (error) {
    // Restore only values still owned by this migration; retain a concurrent edit.
    for (const { shape, before, after } of changes)
      if (shape.constraintsHorizontal === after) shape.constraintsHorizontal = before;
    await settle();
    if (changes.some(({ shape, before }) => shape.constraintsHorizontal !== before))
      throw new Error('Search width migration rollback failed; retain the original guard and inspect this source asset.');
    throw error;
  }
}
