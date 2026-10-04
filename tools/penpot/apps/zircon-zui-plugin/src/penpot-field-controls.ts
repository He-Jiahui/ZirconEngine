import type { Board, Shape, Text } from '@penpot/plugin-types';
import { fieldGeometry } from './bridge/zui-field-geometry';
import type { FieldProjection, FIELD_ICON_PATHS } from './bridge/zui-field-projection';
import { layoutControlText, refreshControlParts } from './penpot-control-parts';
import { ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY } from './metadata';

export function refreshFieldDecoration(board: Board, field?: FieldProjection): boolean {
  if (!field) return false;
  const geometry = fieldGeometry(field, board.width, board.height);
  if (field.search) {
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
      board.appendChild(icon);
      if (icon.layoutChild) icon.layoutChild.absolute = true;
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
    tint(icon, key === 'stepper' ? field.stepperColor : field.iconColor, strokeScale);
    icon.resize(width, height);
    icon.x = board.x + rect.x + (rect.width - width) / 2;
    icon.y = board.y + rect.y + (rect.height - height) / 2;
  }
  for (const icon of existing.values()) icon.remove();
  return true;
}

function hasUsableSize(shape: Shape): boolean {
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
  layoutControlText(board, text, fieldGeometry(field, board.width, board.height).texts);
}

function tint(shape: Shape, color: string, strokeScale: number): void {
  const opacity = color.length === 9 ? parseInt(color.slice(7), 16) / 255 : 1;
  if (Array.isArray(shape.fills))
    shape.fills = shape.fills.map((fill) => ({ ...fill, fillColor: color.slice(0, 7), fillOpacity: opacity }));
  shape.strokes = shape.strokes.map((stroke) => ({ ...stroke, strokeColor: color.slice(0, 7), strokeOpacity: opacity,
    strokeWidth: (stroke.strokeWidth ?? 0) * strokeScale }));
  if ('children' in shape) for (const child of shape.children) tint(child, color, strokeScale);
}
