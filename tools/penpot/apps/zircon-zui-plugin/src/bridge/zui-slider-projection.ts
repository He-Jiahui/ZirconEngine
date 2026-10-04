import { type ZuiDocument, type ZuiNode } from './zui-document';
import type { ProjectionText } from './penpot-projection-model';
import { resolveZuiTextStyle } from './zui-text-style';
import { controlProperties } from './zui-control-properties';

export const SLIDER_STYLE_KEYS = [
  'track_color',
  'disabled_track_color',
  'value_background_color',
  'value_border_color',
  'thumb_outline_color',
  'state_layer_color',
  'tick_color',
  'value_color',
  'warning_color',
  'error_color',
  'thumb_halo_size',
  'label_width',
  'label_gap',
  'value_width',
  'value_gap',
  'value_text_inset',
  'value_corner_radius',
  'value_min_height',
  'range_value_min_frame_height',
  'range_value_top',
  'tick_width',
  'tick_height',
  'tick_offset_y',
] as const;

export interface SliderProjection {
  percent: number;
  rangeMin: number | null;
  tickCount: number;
  hasLabel: boolean;
  halo: boolean;
  trackHeight: number;
  thumbSize: number;
  haloSize: number;
  inset: number;
  labelWidth: number;
  labelGap: number;
  valueWidth: number;
  valueGap: number;
  valueInset: number;
  valueRadius: number;
  valueMinHeight: number;
  rangeMinHeight: number;
  rangeTop: number;
  tickWidth: number;
  tickHeight: number;
  tickOffset: number;
  lineHeight: number;
  borderWidth: number;
  contentOffset: number;
  firstCellOffset: number;
  colors: {
    track: string;
    fill: string;
    thumb: string;
    outline: string;
    halo: string;
    tick: string;
    value: string;
    valueBorder: string;
    rangeBorder: string;
  };
}

export function isSlider(node: ZuiNode): boolean {
  return ['RangeField', 'Slider', 'RangeSlider'].includes(node.component);
}

/** Geometry, value formatting and state precedence follow surface/render/sliders.rs. */
export function projectSlider(
  document: ZuiDocument,
  node: ZuiNode,
):
  | {
      slider: SliderProjection;
      text: ProjectionText;
      textFragments?: Record<string, ProjectionText>;
    }
  | undefined {
  if (!isSlider(node)) return undefined;
  const props = node.props ?? {};
  const { number, metric, color } = controlProperties(document, node, 'slider');
  const flag = (key: string) => props[key] === true;
  const unavailable =
    flag('disabled') || flag('loading') || props['enabled'] === false;
  const min = number(['min'], 0);
  const max = number(['max'], 1);
  const percent =
    props['value_percent'] !== undefined
      ? declaredPercent(number(['value_percent'], 0))
      : Math.abs(max - min) <= 1.1920929e-7
        ? 0
        : clamp((number(['value'], 0) - min) / (max - min));
  const rangeKeys = [
    'range_min_percent',
    'layout_second_cell_offset_x',
    'range_min',
  ];
  const rangeMin = rangeKeys.some((key) => props[key] !== undefined)
    ? declaredPercent(number(rangeKeys, 0))
    : null;
  // Native label lookup chooses the first string before trimming it.
  const labelProperty = (['label', 'label_text', 'text'] as const).find(
    (key) => typeof props[key] === 'string',
  );
  const label = labelProperty ? String(props[labelProperty]).trim() : '';
  const disabledText = color(['disabled_foreground_color'], '#737373');
  const disabledBorder = color(['disabled_border_color'], '#363636');
  const valueBorder = unavailable
    ? disabledBorder
    : color(['value_border_color', 'border_color'], '#484848');
  const fill = unavailable
    ? disabledText
    : props['validation_level'] === 'warning'
      ? color(['warning_color'], '#dcac50')
      : ['error', 'danger'].includes(String(props['validation_level']))
        ? color(['error_color'], '#eb605c')
        : color(['value_color', 'accent_color'], '#565656');
  const font = resolveZuiTextStyle(document, node, { size: 14, weight: '400' });
  const fontSize = metric(['font_size'], 14, true);
  const lineHeight =
    props['line_height'] !== undefined
      ? metric(['line_height'], 19.6, true)
      : props['line_height_ratio'] !== undefined
        ? fontSize * metric(['line_height_ratio'], 1.4, true)
        : 19.6;
  const baseText = (
    characters: string,
    property: ProjectionText['property'],
    rawColor: string,
  ): ProjectionText => ({
    characters,
    property,
    color: rawColor.slice(0, 7),
    colorOpacity:
      rawColor.length === 9 ? parseInt(rawColor.slice(7), 16) / 255 : 1,
    fontFamily: font.family,
    fontSize,
    fontWeight: '400',
    lineHeight: lineHeight / fontSize,
    align: 'left',
  });
  const valueText =
    typeof props['value_text'] === 'string' && props['value_text'].trim()
      ? props['value_text'].trim()
      : percent.toFixed(2);
  const textColor = unavailable
    ? disabledText
    : color(['foreground_color'], '#e8e8e8');
  const fragments: Record<string, ProjectionText> = {};
  if (label && labelProperty)
    fragments['label'] = baseText(
      label,
      labelProperty,
      unavailable ? disabledText : color(['label_color'], '#b3b3b3'),
    );
  if (rangeMin !== null)
    fragments['range-min'] = baseText(rangeMin.toFixed(2), null, textColor);
  const declaredTicks = Math.round(
    number(['tick_count', 'steps', 'layout_third_cell_offset_x'], 0),
  );
  return {
    text: baseText(valueText, 'value_text', textColor),
    ...(Object.keys(fragments).length ? { textFragments: fragments } : {}),
    slider: {
      percent,
      rangeMin,
      hasLabel: Boolean(label),
      tickCount: declaredTicks < 2 ? 0 : Math.min(256, declaredTicks),
      halo:
        (!unavailable && (flag('pressed') || flag('focused'))) ||
        flag('hovered') ||
        flag('dragging') ||
        flag('drop_hovered'),
      trackHeight: metric(['track_height'], 4, true),
      thumbSize: metric(['thumb_size', 'layout_icon_size'], 8, true),
      haloSize: metric(['thumb_halo_size'], 16, true),
      inset: metric(['horizontal_inset'], 8),
      labelWidth: metric(['label_width'], 50),
      labelGap: metric(['label_gap'], 12),
      valueWidth: metric(['value_width'], 44, true),
      valueGap: metric(['value_gap'], 10),
      valueInset: metric(['value_text_inset'], 6),
      valueRadius: metric(['value_corner_radius', 'corner_radius'], 4),
      valueMinHeight: metric(['value_min_height'], 22, true),
      rangeMinHeight: metric(['range_value_min_frame_height'], 42, true),
      rangeTop: metric(['range_value_top'], 10),
      tickWidth: metric(['tick_width'], 1, true),
      tickHeight: metric(['tick_height'], 4, true),
      tickOffset: metric(['tick_offset_y'], 8),
      lineHeight,
      borderWidth: metric(['border_width'], 1),
      contentOffset: number(['layout_content_offset_x'], 0),
      firstCellOffset: number(['layout_first_cell_offset_x'], 0),
      colors: {
        track: unavailable
          ? color(['disabled_track_color'], '#2b2b2b')
          : color(['track_color', 'background_color'], '#151515'),
        fill,
        thumb: unavailable
          ? disabledText
          : color(['thumb_color', 'icon_color'], '#e8e8e8'),
        outline: unavailable
          ? disabledBorder
          : color(['thumb_outline_color', 'border_color'], '#484848'),
        halo: unavailable
          ? disabledText
          : color(['state_layer_color'], '#e8e8e820'),
        tick: unavailable ? disabledBorder : color(['tick_color'], '#333333'),
        value: unavailable
          ? color(['disabled_background_color'], '#2b2b2b')
          : color(['value_background_color'], '#151515'),
        valueBorder: !unavailable && flag('pressed') ? fill : valueBorder,
        rangeBorder: valueBorder,
      },
    },
  };
}

function clamp(value: number): number {
  return Math.min(1, Math.max(0, value));
}
function declaredPercent(value: number): number {
  return clamp(value > 1 ? value / 100 : value);
}
