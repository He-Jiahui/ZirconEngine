import type { ProjectionGeometry } from './penpot-projection-model';
import type { ControlGeometry } from './zui-control-geometry';
import type { PropertyRowProjection } from './zui-property-row-projection';

export function propertyRowGeometry(
  row: PropertyRowProjection,
  width: number,
  height: number,
): ControlGeometry {
  const result: ControlGeometry = { parts: {}, texts: {} };
  if (width <= 0 || height <= 0) return result;
  const text = (key: string, rect: ProjectionGeometry) => {
    if (rect.width > 0 && rect.height > 0) result.texts[key] = rect;
  };
  const labelWidth = Math.max(
    0,
    Math.min(Math.max(row.labelWidth, row.labelMinWidth), width * 0.45),
  );
  const labelInset = Math.min(row.textInsetX, labelWidth * 0.5);
  const labelRight = Math.min(
    row.textInsetX * 0.5,
    Math.max(0, labelWidth - labelInset),
  );
  const labelY = Math.min(row.textInsetY, height * 0.5);
  if (row.hasLabel)
    text('property-label', {
      x: labelInset,
      y: labelY,
      width: Math.max(0, labelWidth - labelInset - labelRight),
      height: Math.max(0, height - labelY * 2),
    });
  if (!row.hasValue) return result;
  const valueWidth = Math.max(0, width - labelWidth - row.textInsetX);
  const fieldY = Math.min(row.fieldInsetY, height * 0.5);
  const field = (key: string, x: number, available: number) => {
    const rect = {
      x,
      y: fieldY,
      width: available,
      height: Math.max(0, height - fieldY * 2),
    };
    if (rect.width > 0 && rect.height > 0)
      result.parts[`field-${key}`] = {
        ...rect,
        fill: row.fieldFill,
        stroke: row.fieldBorder,
        strokeWidth: row.fieldBorderWidth,
        radius: row.fieldRadius,
      };
    const insetX = Math.min(row.textInsetX, available * 0.5);
    const insetY = Math.min(row.textInsetY, rect.height * 0.5);
    text(key, {
      x: x + insetX,
      y: fieldY + insetY,
      width: Math.max(0, available - insetX * 2),
      height: Math.max(0, rect.height - insetY * 2),
    });
  };
  if (!row.axisKeys.length) {
    field('property-value', labelWidth, valueWidth);
    return result;
  }
  const count = row.axisKeys.length;
  const gap = Math.min(row.groupGap, valueWidth / count);
  const groupWidth = Math.max(0, (valueWidth - gap * (count - 1)) / count);
  row.axisKeys.forEach((key, index) => {
    const x = labelWidth + (groupWidth + gap) * index;
    const axisWidth = Math.min(row.axisWidth, groupWidth * 0.35);
    const remaining = Math.max(0, groupWidth - axisWidth);
    const axisGap = Math.min(row.axisGap, remaining * 0.2);
    text(`${key}-label`, {
      x,
      y: labelY,
      width: axisWidth,
      height: Math.max(0, height - labelY * 2),
    });
    field(key, x + axisWidth + axisGap, Math.max(0, remaining - axisGap));
  });
  return result;
}
