import type { ProjectionGeometry } from './penpot-projection-model';
import type { SliderProjection } from './zui-slider-projection';
import type { ControlPart } from './zui-control-geometry';

export type SliderPart = ControlPart;

export function sliderGeometry(
  s: SliderProjection,
  width: number,
  height: number,
): {
  parts: Record<string, SliderPart>;
  texts: Record<string, ProjectionGeometry>;
} {
  const parts: Record<string, SliderPart> = {};
  const texts: Record<string, ProjectionGeometry> = {};
  const minimum = Math.max(s.borderWidth, 1.1920929e-7);
  const valueHeight = Math.min(
    Math.max(height - s.valueInset, s.valueMinHeight),
    s.valueMinHeight + s.inset - s.borderWidth * 2,
  );
  if (valueHeight < s.valueMinHeight)
    throw new Error('Slider value height bounds are inverted.');
  const value =
    width >= s.valueWidth + s.inset * 2 + s.valueGap
      ? {
          x: width - s.inset - s.valueWidth,
          y: Math.max(0, height - valueHeight) / 2,
          width: s.valueWidth,
          height: valueHeight,
        }
      : null;
  const left =
    (s.hasLabel ? s.labelWidth + s.labelGap : 0) + s.inset + s.contentOffset;
  const right = Math.max(
    left,
    (value ? value.x - s.valueGap : width - s.inset) + s.firstCellOffset,
  );
  const track = {
    x: left,
    y: Math.max(0, height - s.trackHeight) / 2,
    width: right - left,
    height: s.trackHeight,
  };
  if (width <= minimum || height <= minimum || track.width <= minimum)
    return { parts, texts };
  if (s.hasLabel)
    texts['label'] = {
      x: s.inset,
      y: Math.max(0, height - s.lineHeight) / 2,
      width: s.labelWidth,
      height: s.lineHeight,
    };
  parts['track'] = {
    ...track,
    fill: s.colors.track,
    radius: s.trackHeight / 2,
  };
  const start = Math.min(s.rangeMin ?? 0, s.percent);
  const end = Math.max(s.rangeMin ?? 0, s.percent);
  if (end > start)
    parts['fill'] = {
      ...track,
      x: track.x + track.width * start,
      width: Math.max(minimum, track.width * (end - start)),
      fill: s.colors.fill,
      radius: s.trackHeight / 2,
    };
  const ticks = Math.min(s.tickCount, Math.floor(track.width), 256);
  for (let index = 0; index < ticks && ticks >= 2; index++)
    parts[`tick-${index}`] = {
      x: track.x + (track.width * index) / (ticks - 1) - s.tickWidth / 2,
      y: track.y + s.tickOffset,
      width: s.tickWidth,
      height: s.tickHeight,
      fill: s.colors.tick,
      radius: 0,
    };
  const thumb = (key: string, fraction: number) => {
    const x = track.x + track.width * fraction;
    const y = track.y + track.height / 2;
    if (s.halo)
      parts[`${key}-halo`] = {
        x: x - s.haloSize / 2,
        y: y - s.haloSize / 2,
        width: s.haloSize,
        height: s.haloSize,
        fill: s.colors.halo,
        radius: s.haloSize / 2,
      };
    parts[key] = {
      x: x - s.thumbSize / 2,
      y: y - s.thumbSize / 2,
      width: s.thumbSize,
      height: s.thumbSize,
      fill: s.colors.thumb,
      radius: s.thumbSize / 2,
      stroke: s.colors.outline,
      strokeWidth: s.borderWidth,
    };
  };
  if (s.rangeMin !== null) thumb('range-thumb', s.rangeMin);
  thumb('thumb', s.percent);
  const valueBox = (key: string, rect: ProjectionGeometry, border: string) => {
    parts[key] = {
      ...rect,
      fill: s.colors.value,
      radius: s.valueRadius,
      stroke: border,
      strokeWidth: s.borderWidth,
    };
    texts[key] = {
      x: rect.x + s.valueInset,
      y: rect.y + Math.max(0, rect.height - s.lineHeight) / 2,
      width: Math.max(minimum, rect.width - s.valueInset * 2),
      height: s.lineHeight,
    };
  };
  if (
    s.rangeMin !== null &&
    height >= s.rangeMinHeight &&
    track.width >= s.valueWidth
  )
    valueBox(
      'range-min',
      {
        x: track.x,
        y: track.y + s.rangeTop,
        width: s.valueWidth,
        height: s.valueMinHeight,
      },
      s.colors.rangeBorder,
    );
  if (value) valueBox('primary', value, s.colors.valueBorder);
  return { parts, texts };
}
