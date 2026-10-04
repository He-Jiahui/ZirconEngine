import type { ControlGeometry } from './zui-control-geometry';
import type { ProjectionGeometry } from './penpot-projection-model';
import type { FieldProjection } from './zui-field-projection';

export function fieldGeometry(field: FieldProjection, width: number, height: number): ControlGeometry & {
  icons: Record<string, ProjectionGeometry>;
} {
  const result: ReturnType<typeof fieldGeometry> = { parts: {}, texts: {}, icons: {} };
  const paintHeight = field.search ? Math.min(height, field.maxHeight) : height;
  const top = (height - paintHeight) / 2;
  if (field.search) result.parts['surface'] = {
    x: 0, y: top, width, height: paintHeight,
    fill: rgba(field.paint.fillColor, field.paint.fillOpacity) ?? '#00000000', radius: field.paint.borderRadius,
    stroke: rgba(field.paint.strokeColor, field.paint.strokeOpacity), strokeWidth: field.paint.strokeWidth,
  };
  const inside = (rect: ProjectionGeometry) => rect.width > 0 && rect.height > 0 &&
    rect.x >= 0 && rect.y >= top && rect.x + rect.width <= width && rect.y + rect.height <= top + paintHeight;
  const iconTop = top + Math.max(0, (paintHeight - field.iconSize) / 2);
  let right = field.pad;
  if (field.stepper) {
    const left = width - field.stepperWidth;
    const divider = { x: left, y: top + field.gap, width: field.borderWidth, height: paintHeight - field.gap * 2 };
    const icon = { x: left + field.gap, y: iconTop, width: field.stepperGlyphWidth, height: field.iconSize };
    if (inside(divider) && inside(icon)) {
      result.parts['stepper-divider'] = { ...divider, fill: field.divider, radius: 0 };
      result.icons['stepper'] = icon;
      right += field.stepperWidth;
    }
  }
  if (field.search) {
    const icon = { x: field.pad, y: iconTop, width: field.iconSize, height: field.iconSize };
    if (inside(icon)) result.icons['search'] = icon;
    if (field.clear) {
      const size = Math.round(Math.min(field.iconSize, paintHeight));
      const action = { x: width - field.pad - size, y: top + Math.max(0, (paintHeight - size) / 2), width: size, height: size };
      if (inside(action)) {
        result.icons['clear'] = action;
        right = width - action.x + field.pad;
      }
    }
  }
  const left = field.search ? field.pad + field.iconSize + field.gap : field.pad;
  const text = { x: left, y: top + Math.max(0, (paintHeight - field.lineHeight) / 2), width: Math.max(0, width - left - right), height: field.lineHeight };
  if (inside(text)) result.texts['primary'] = text;
  return result;
}

function rgba(color: string | null, opacity: number): string | undefined {
  return color ? `${color}${Math.round(opacity * 255).toString(16).padStart(2, '0')}` : undefined;
}
