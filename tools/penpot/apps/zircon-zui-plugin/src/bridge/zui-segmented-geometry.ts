import type { ControlGeometry } from './zui-control-geometry';
import type { SegmentedProjection } from './zui-segmented-projection';

export function segmentedGeometry(
  s: SegmentedProjection,
  width: number,
  height: number,
): ControlGeometry {
  const result: ControlGeometry = { parts: {}, texts: {} };
  const { parts, texts } = result;
  const minimum = Math.max(s.borderWidth, 1.1920929e-7);
  if (width <= minimum || height <= minimum) return result;
  if (s.kind === 'tab') {
    if (s.colors.background)
      parts['surface'] = {
        x: 0,
        y: 0,
        width,
        height,
        fill: s.colors.background,
        radius: 0,
      };
    if (s.active && s.underlineHeight > 0)
      parts['underline'] = {
        x: 0,
        y: Math.max(0, height - s.underlineHeight),
        width,
        height: s.underlineHeight,
        fill: s.colors.underline,
        radius: 0,
      };
    if (s.hasLabel)
      texts['primary'] = {
        x: s.tabInsetX,
        y: Math.max(0, height - s.tabLineHeight) / 2,
        width: Math.max(minimum, width - s.tabInsetX * 2),
        height: s.tabLineHeight,
      };
    return result;
  }
  if (!s.options.length) return result;
  if (s.hasLabel)
    texts['primary'] = { x: 0, y: 0, width, height: s.labelHeight };
  const labelBlock = s.hasLabel ? s.labelHeight + s.labelGap : 0;
  const body = {
    x: s.offsetX,
    y: s.offsetY + labelBlock,
    width,
    height: Math.max(minimum, height - labelBlock),
  };
  parts['surface'] = {
    ...body,
    fill: s.colors.background!,
    stroke: s.colors.border,
    strokeWidth: s.borderWidth,
    radius: s.radius,
  };
  const segmentWidth = width / s.options.length;
  s.options.forEach((option, index) => {
    const segment = {
      ...body,
      x: body.x + segmentWidth * index,
      width: segmentWidth,
    };
    if (index > 0 && s.borderWidth > 0)
      parts[`divider-${index}`] = {
        x: segment.x,
        y: segment.y + s.insetY - s.borderWidth,
        width: s.borderWidth,
        height: Math.max(
          minimum,
          segment.height - (s.insetY - s.borderWidth) * 2,
        ),
        fill: s.colors.border,
        radius: 0,
      };
    if (option.selected) {
      const selected = {
        x: segment.x + s.selectedInset,
        y: segment.y + s.selectedInset,
        width: Math.max(minimum, segment.width - s.selectedInset * 2),
        height: Math.max(minimum, segment.height - s.selectedInset * 2),
      };
      parts[`selected-${option.key}`] = {
        ...selected,
        fill: s.colors.selected,
        ...(s.selectedBorderWidth > 0
          ? {
              stroke: s.colors.selectedBorder,
              strokeWidth: s.selectedBorderWidth,
            }
          : {}),
        radius: Math.max(0, s.radius - s.borderWidth),
      };
      if (s.underlineHeight > 0)
        parts[`underline-${option.key}`] = {
          x: selected.x,
          y: selected.y + Math.max(0, selected.height - s.underlineHeight),
          width: selected.width,
          height: Math.max(
            minimum,
            Math.min(s.underlineHeight, selected.height),
          ),
          fill: s.colors.underline,
          radius: 0,
        };
    }
    texts[option.key] = {
      x: segment.x + s.insetX,
      y: segment.y + s.insetY,
      width: Math.max(minimum, segment.width - s.insetX * 2),
      height: Math.max(s.lineHeight, segment.height - s.insetY * 2),
    };
  });
  return result;
}
