import type { Board, Shape, Text } from '@penpot/plugin-types';
import { fieldGeometry } from './bridge/zui-field-geometry';
import type { FieldProjection, FIELD_ICON_PATHS } from './bridge/zui-field-projection';
import { layoutControlText, refreshControlParts } from './penpot-control-parts';
import { assertControlParts, assertControlTextGeometry } from './penpot-control-guard';
import { ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY } from './metadata';

const FIELD_ICON_PAINT = 'field-icon-paint';
type IconPaint = { fills: Shape['fills']; strokes: Shape['strokes'] };
interface IconPaintState {
  version: 1;
  sourceSize: { width: number; height: number };
  sourceWidths: number[];
  originalWidths?: Record<string, number[]>;
  desired?: string;
  generated?: Record<string, IconPaint>;
}

export function refreshFieldDecoration(board: Board, field?: FieldProjection): boolean {
  if (!field) return false;
  const geometry = fieldGeometry(field, board.width, board.height);
  if (field.search || (field.labelWidth ?? 0) > 0) {
    board.fills = [];
    board.strokes = [];
  }
  refreshControlParts(board, 'field', geometry.parts);
  const existing = new Map<string, Shape>();
  for (const shape of board.children) {
    const key = shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-icon');
    if (key) existing.set(key, shape);
  }
  for (const [key, rect] of Object.entries(geometry.icons)) {
    let icon = existing.get(key);
    const svg = field.icons[key as keyof typeof FIELD_ICON_PATHS];
    if (!icon) {
      if (!svg) throw new Error(`Missing native field icon resource: ${key}`);
      icon = penpot.createShapeFromSvg(svg) ?? undefined;
      if (!icon) throw new Error(`Could not render native field icon: ${key}`);
      icon.name = `Field ${key}`;
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-icon', key);
      icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, FIELD_ICON_PAINT, JSON.stringify(newIconPaintState(svg)));
      board.appendChild(icon);
      if (icon.layoutChild) icon.layoutChild.absolute = true;
      icon.constraintsHorizontal = key === 'search' ? 'left' : 'right';
    }
    existing.delete(key);
    // Penpot scales path geometry but leaves stroke widths in document pixels.
    // createShapeFromSvg dispatches the SVG import asynchronously. Its proxy
    // can still expose empty bounds here, so use the source viewport until the
    // imported shape has usable dimensions.
    const sourceSize = hasUsableSize(icon)
      ? { width: icon.width, height: icon.height }
      : svg
        ? intrinsicSvgSize(svg)
        : undefined;
    if (!sourceSize)
      throw new Error(`Field icon ${key} has no settled or intrinsic size.`);
    const strokeScale = Math.min(
      rect.width / sourceSize.width,
      rect.height / sourceSize.height,
    );
    const width = sourceSize.width * strokeScale;
    const height = sourceSize.height * strokeScale;
    tint(icon, key === 'stepper' ? field.stepperColor : field.iconColor, rect);
    icon.resize(width, height);
    icon.x = board.x + rect.x + (rect.width - width) / 2;
    icon.y = board.y + rect.y + (rect.height - height) / 2;
  }
  for (const icon of existing.values()) icon.remove();
  return true;
}

function hasUsableSize(shape: Pick<Shape, 'width' | 'height'>): boolean {
  return (
    Number.isFinite(shape.width) &&
    shape.width > 0 &&
    Number.isFinite(shape.height) &&
    shape.height > 0
  );
}

function intrinsicSvgSize(svg: string): { width: number; height: number } {
  const rootAttributes = /<svg\b([^>]*)>/i.exec(svg)?.[1];
  if (rootAttributes === undefined)
    throw new Error('Field icon resource has no SVG root element.');
  const attribute = (name: string): string | undefined =>
    new RegExp(`(?:^|\\s)${name}\\s*=\\s*(["'])(.*?)\\1`, 'i').exec(
      rootAttributes,
    )?.[2];
  const viewBox = attribute('viewBox');
  if (viewBox !== undefined) {
    const values = viewBox.trim().split(/[\s,]+/).map(Number);
    if (
      values.length !== 4 ||
      !Number.isFinite(values[2]) ||
      !Number.isFinite(values[3]) ||
      values[2]! < 0 ||
      values[3]! < 0
    )
      throw new Error(`Field icon has an invalid SVG viewBox: ${viewBox}`);
    return { width: Math.max(1, values[2]!), height: Math.max(1, values[3]!) };
  }
  const dimension = (name: string): number => {
    const raw = attribute(name);
    const value = raw === undefined ? Number.NaN : Number.parseFloat(raw);
    return Number.isFinite(value) ? Math.max(1, value) : 100;
  };
  return { width: dimension('width'), height: dimension('height') };
}

export function fieldPaintShape(board: Board): Shape {
  return board.children.find((shape) =>
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') === 'surface') ?? board;
}

export function layoutFieldText(board: Board, text: Text, field: FieldProjection): void {
  const texts = fieldGeometry(field, board.width, board.height).texts;
  const fragment = text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-fragment') || 'primary';
  if (texts[fragment]) {
    // Native width changes must preserve the projected text lane's insets.
    // A separate field label keeps its source width; it is not the value lane.
    text.constraintsHorizontal = fragment === 'field-label' ? 'left' : 'leftright';
  }
  layoutControlText(board, text, texts);
}

function newIconPaintState(svg: string): IconPaintState {
  // The field resources author each stroke width, either as an XML attribute
  // or inline style. Keep these values independent of native resize results.
  const sourceWidths = [...svg.matchAll(
    /\bstroke-width\s*(?:=\s*(["'])(.*?)\1|:\s*([^;"']+))/gi,
  )].map((match) => {
    const raw = (match[2] ?? match[3] ?? '').trim();
    const width = Number.parseFloat(raw);
    if (!/^(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?(?:px)?$/i.test(raw) ||
        !Number.isFinite(width) || width < 0)
      throw new Error(`Field icon has an unsupported source stroke width: ${raw}`);
    return width;
  });
  if (sourceWidths.length === 0)
    throw new Error('Field icon resource has no explicit source stroke widths.');
  return { version: 1, sourceSize: intrinsicSvgSize(svg), sourceWidths };
}

function samePaintValue(left: unknown, right: unknown): boolean {
  if (left === right) return true;
  if (left === null || right === null || typeof left !== 'object' || typeof right !== 'object')
    return false;
  if (Array.isArray(left) || Array.isArray(right))
    return Array.isArray(left) && Array.isArray(right) && left.length === right.length &&
      left.every((value, index) => samePaintValue(value, right[index]));
  const leftRecord = left as Record<string, unknown>;
  const rightRecord = right as Record<string, unknown>;
  const keys = Object.keys(leftRecord);
  return keys.length === Object.keys(rightRecord).length && keys.every((key) =>
    Object.prototype.hasOwnProperty.call(rightRecord, key) &&
    samePaintValue(leftRecord[key], rightRecord[key]));
}

function tint(shape: Shape, color: string, rect: { width: number; height: number }): void {
  const raw = shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, FIELD_ICON_PAINT);
  if (!raw) throw new Error('Field icon has no original paint metadata; reimport it.');
  const state = JSON.parse(raw) as IconPaintState;
  if (state.version !== 1 || !Array.isArray(state.sourceWidths) || state.sourceWidths.length === 0 ||
      !state.sourceSize || !hasUsableSize(state.sourceSize))
    throw new Error('Field icon has invalid original paint metadata.');
  const scale = Math.min(rect.width / state.sourceSize.width, rect.height / state.sourceSize.height);
  const desired = JSON.stringify([color, scale]);
  if (state.desired === desired) return;

  const shapes: Shape[] = [];
  const visit = (current: Shape): void => {
    shapes.push(current);
    if ('children' in current) for (const child of current.children) visit(child);
  };
  visit(shape);
  const current: Record<string, IconPaint> = Object.fromEntries(shapes.map((child) => [
    child.id, JSON.parse(JSON.stringify({ fills: child.fills, strokes: child.strokes })) as IconPaint,
  ]));
  if (!state.originalWidths) {
    const widths = shapes.flatMap((child) => child.strokes.map((stroke) => stroke.strokeWidth));
    // createShapeFromSvg may still have no children, or only part of its tree.
    // The imported viewport rectangle has no strokes and is not a source stroke.
    if (widths.length < state.sourceWidths.length || widths.some((width) => width == null)) return;
    if (JSON.stringify([...widths].sort((a, b) => a! - b!)) !==
        JSON.stringify([...state.sourceWidths].sort((a, b) => a - b)))
      throw new Error('Field icon original strokes do not match its SVG source.');
    state.originalWidths = Object.fromEntries(shapes.map((child) => [
      child.id, child.strokes.map((stroke) => stroke.strokeWidth!),
    ]));
  } else if (!state.generated || !samePaintValue(current, state.generated)) {
    // Check the entire tree before any paint write, including later siblings.
    throw new Error('Field icon paint changed without a ZUI source mapping.');
  }

  const opacity = color.length === 9 ? parseInt(color.slice(7), 16) / 255 : 1;
  const generated: Record<string, IconPaint> = Object.fromEntries(shapes.map((child) => {
    const paint = current[child.id];
    const widths = state.originalWidths![child.id];
    if (!widths || widths.length !== paint.strokes.length)
      throw new Error('Field icon original paint metadata does not cover its tree.');
    return [child.id, {
      fills: Array.isArray(paint.fills)
        ? paint.fills.map((fill) => ({ ...fill, fillColor: color.slice(0, 7), fillOpacity: opacity }))
        : paint.fills,
      strokes: paint.strokes.map((stroke, index) => ({
        ...stroke, strokeColor: color.slice(0, 7), strokeOpacity: opacity,
        strokeWidth: widths[index]! * scale,
      })),
    }];
  }));
  for (const child of shapes) {
    const paint = generated[child.id];
    if (Array.isArray(paint.fills)) child.fills = paint.fills;
    child.strokes = paint.strokes;
  }
  state.desired = desired;
  state.generated = generated;
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, FIELD_ICON_PAINT, JSON.stringify(state));
}


export interface SearchFieldWidthConstraint {
  shape: Shape;
  before: Shape['constraintsHorizontal'];
  after: 'leftright';
}

/** Validate the complete default SearchField surface before any native write. */
export function planSearchFieldWidthConstraints(
  board: Board, field: FieldProjection,
): SearchFieldWidthConstraint[] {
  if (!field.search || field.clear || field.stepper || (field.labelWidth ?? 0) !== 0)
    throw new Error('Search width migration requires the canonical default field.');
  if (!hasUsableSize(board) || board.hidden)
    throw new Error('Canonical SearchField has no visible, finite allocation.');
  assertControlParts(board);
  const geometry = fieldGeometry(field, board.width, board.height);
  const surface = board.children.filter((shape) =>
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-part') === 'surface');
  const primary = board.children.filter((shape) => shape.type === 'text' &&
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) === 'text' &&
    (shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-fragment') || 'primary') === 'primary');
  const search = board.children.filter((shape) =>
    shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'field-icon') === 'search');
  if (surface.length !== 1 || surface[0].type !== 'board' || primary.length !== 1 ||
      search.length !== 1 || board.children.length !== 3)
    throw new Error('Canonical SearchField parts or text topology changed.');
  if (board.children.some((shape) => shape.parent?.id !== board.id))
    throw new Error('Canonical SearchField part belongs to a different native parent.');
  if (search[0].constraintsHorizontal !== 'left')
    throw new Error('Canonical SearchField search icon must retain its fixed left constraint.');
  assertControlTextGeometry(board, primary[0]);
  const svg = field.icons.search;
  if (!svg || !geometry.parts.surface || !geometry.texts.primary || !geometry.icons.search)
    throw new Error('Canonical SearchField source geometry is incomplete.');
  const source = intrinsicSvgSize(svg), icon = geometry.icons.search;
  const scale = Math.min(icon.width / source.width, icon.height / source.height);
  const expectedIcon = { x: icon.x + (icon.width - source.width * scale) / 2,
    y: icon.y + (icon.height - source.height * scale) / 2,
    width: source.width * scale, height: source.height * scale };
  for (const [shape, expected] of [
    [surface[0], geometry.parts.surface], [primary[0], geometry.texts.primary],
    [search[0], expectedIcon],
  ] as const) {
    const actual = { x: shape.x - board.x, y: shape.y - board.y,
      width: shape.width, height: shape.height };
    if (shape.hidden || Object.entries(expected).some(([key, value]) =>
      ['x', 'y', 'width', 'height'].includes(key) &&
      (typeof value !== 'number' || !Number.isFinite(value) ||
        !Number.isFinite(actual[key as keyof typeof actual]) ||
        Math.abs(actual[key as keyof typeof actual] - value) > 0.01)))
      throw new Error('Canonical SearchField geometry differs from its source.');
  }
  return [surface[0], primary[0]].map((shape) => {
    if (!['left', 'leftright'].includes(shape.constraintsHorizontal))
      throw new Error('Canonical SearchField has an authored horizontal constraint override.');
    return { shape, before: shape.constraintsHorizontal, after: 'leftright' };
  });
}
