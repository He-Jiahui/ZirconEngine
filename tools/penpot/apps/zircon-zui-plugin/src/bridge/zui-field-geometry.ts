import type { ControlGeometry } from './zui-control-geometry';
import type { ProjectionGeometry } from './penpot-projection-model';
import type { FieldProjection } from './zui-field-projection';

export function fieldGeometry(field: FieldProjection, width: number, height: number): ControlGeometry & {
  icons: Record<string, ProjectionGeometry>;
} {
  const result: ReturnType<typeof fieldGeometry> = { parts: {}, texts: {}, icons: {} };
  const paintHeight = field.search ? Math.min(height, field.maxHeight) : height;
  const top = (height - paintHeight) / 2;
  const labelWidth = Math.min(width, Math.max(0, field.labelWidth ?? 0));
  if (field.search || labelWidth > 0) result.parts['surface'] = {
    x: labelWidth, y: top, width: Math.max(0, width - labelWidth), height: paintHeight,
    fill: rgba(field.paint.fillColor, field.paint.fillOpacity) ?? '#00000000', radius: field.paint.borderRadius,
    stroke: rgba(field.paint.strokeColor, field.paint.strokeOpacity), strokeWidth: field.paint.strokeWidth,
  };
  const inside = (rect: ProjectionGeometry) => rect.width > 0 && rect.height > 0 &&
    rect.x >= 0 && rect.y >= top && rect.x + rect.width <= width && rect.y + rect.height <= top + paintHeight;
  const iconTop = top + Math.max(0, (paintHeight - field.iconSize) / 2);
  const padLeft = field.padLeft ?? field.pad, padRight = field.padRight ?? field.pad;
  const padTop = field.padTop ?? 0, padBottom = field.padBottom ?? 0;
  let right = padRight;
  if (field.stepper) {
    const left = width - field.stepperWidth;
    const divider = { x: left, y: top + field.gap, width: field.borderWidth, height: paintHeight - field.gap * 2 };
    const icon = { x: left + field.gap, y: iconTop, width: field.stepperGlyphWidth, height: field.iconSize };
    if (inside(divider) && inside(icon) && divider.x >= labelWidth && icon.x >= labelWidth) {
      result.parts['stepper-divider'] = { ...divider, fill: field.divider, radius: 0 };
      result.icons['stepper'] = icon;
      right += field.stepperWidth;
    }
  }
  if (field.search) {
    const icon = { x: padLeft, y: iconTop, width: field.iconSize, height: field.iconSize };
    if (inside(icon)) result.icons['search'] = icon;
    if (field.clear) {
      const size = Math.round(Math.min(field.iconSize, paintHeight));
      // Clear hit geometry is owned by native search_field_clear_action_frame;
      // authored text padding changes the text reserve, not the hit target.
      const action = { x: width - field.pad - size, y: top + Math.max(0, (paintHeight - size) / 2), width: size, height: size };
      if (inside(action)) {
        result.icons['clear'] = action;
        right = width - action.x + padRight;
      }
    }
  }
  const left = labelWidth + (field.search ? padLeft + field.iconSize + field.gap : padLeft);
  const text = { x: left, y: top + padTop + Math.max(0, (paintHeight - padTop - padBottom - field.lineHeight) / 2), width: Math.max(0, width - left - right), height: field.lineHeight };
  if (inside(text)) result.texts['primary'] = text;
  if (labelWidth > 0) {
    const label = { x: padLeft, y: text.y, width: Math.max(0, labelWidth - padLeft - padRight), height: field.lineHeight };
    if (inside(label) && label.x + label.width <= labelWidth) result.texts['field-label'] = label;
    if (width <= labelWidth) delete result.parts['surface'];
  }
  return result;
}

function rgba(color: string | null, opacity: number): string | undefined {
  return color ? `${color}${Math.round(opacity * 255).toString(16).padStart(2, '0')}` : undefined;
}
