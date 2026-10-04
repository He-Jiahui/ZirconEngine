import type { ZuiDocument, ZuiNode, ZuiTable } from './zui-document';
import type { ProjectionText } from './penpot-projection-model';
import { controlProperties } from './zui-control-properties';
import { resolveZuiTextStyle } from './zui-text-style';

export const SEGMENTED_STYLE_KEYS = [
  'selected_text_color',
  'idle_text_color',
  'selected_border_width',
  'selected_underline_color',
  'selected_underline_height',
  'group_label_height',
  'group_label_gap',
  'segment_text_inset_x',
  'segment_text_inset_y',
  'selected_inset',
  'tab_text_inset_x',
  'tab_font_size',
  'tab_line_height',
  'tab_line_height_ratio',
] as const;

export interface SegmentedOption {
  key: string;
  index: number;
  field: string | null;
  value: string;
  selected: boolean;
}

export interface SegmentedProjection {
  kind: 'segments' | 'tab';
  options: Array<{ key: string; selected: boolean }>;
  hasLabel: boolean;
  active: boolean;
  offsetX: number;
  offsetY: number;
  labelHeight: number;
  labelGap: number;
  insetX: number;
  insetY: number;
  selectedInset: number;
  radius: number;
  borderWidth: number;
  selectedBorderWidth: number;
  underlineHeight: number;
  lineHeight: number;
  tabLineHeight: number;
  tabInsetX: number;
  colors: {
    background: string | null;
    border: string;
    selected: string;
    selectedBorder: string;
    underline: string;
  };
}

export function isSegmentedControl(node: ZuiNode): boolean {
  return ['SegmentedControl', 'Segmented', 'Tab', 'PanelTab'].includes(
    node.component,
  );
}

export function selectedSegmentProperty(props: ZuiTable): string | undefined {
  return ['value', 'value_text', 'selected', 'text'].find(
    (key) => typeof props[key] === 'string',
  );
}

function asciiLower(value: string): string {
  return value.replace(/[A-Z]/g, (char) => char.toLowerCase());
}

export function segmentLabel(value: string): string {
  return value.trim().replace(/^[a-z]/, (char) => char.toUpperCase());
}

/** Keep source array indices, including options the native renderer filters out. */
export function segmentedOptions(props: ZuiTable): SegmentedOption[] {
  const options = props['options'];
  if (!Array.isArray(options)) return [];
  const selectedProperty = selectedSegmentProperty(props);
  const selected = selectedProperty
    ? String(props[selectedProperty]).trim()
    : '';
  return options.flatMap((option, index) => {
    let field: string | null = null;
    let value: unknown = option;
    if (option && typeof option === 'object' && !Array.isArray(option)) {
      const table = option as ZuiTable;
      field =
        ['label', 'text', 'value', 'id', 'name'].find((key) =>
          Object.hasOwn(table, key),
        ) ?? null;
      value = field ? table[field] : undefined;
    }
    if (typeof value !== 'string' || !value.trim()) return [];
    return [
      {
        key: `option-${index}`,
        index,
        field,
        value,
        selected:
          Boolean(selected) &&
          asciiLower(value.trim()) === asciiLower(selected),
      },
    ];
  });
}

/** Mirrors surface/render/segmented_controls; labels and values remain separate. */
export function projectSegmentedControl(
  document: ZuiDocument,
  node: ZuiNode,
):
  | {
      segmented: SegmentedProjection;
      text: ProjectionText | null;
      textFragments?: Record<string, ProjectionText>;
    }
  | undefined {
  if (!isSegmentedControl(node)) return undefined;
  const props = node.props ?? {};
  const { resolve, number, metric, color } = controlProperties(
    document,
    node,
    'segmented control',
  );
  const tab = ['Tab', 'PanelTab'].includes(node.component);
  const unavailable =
    props['disabled'] === true ||
    props['loading'] === true ||
    props['enabled'] === false;
  const active = props['checked'] === true || props['selected'] === true;
  const pressed = !unavailable && props['pressed'] === true;
  const focused = !unavailable && props['focused'] === true;
  const hot = [
    'hovered',
    'open',
    'popup_open',
    'dragging',
    'drop_hovered',
  ].some((key) => props[key] === true);
  const disabledColor = color(['disabled_foreground_color'], '#737373');
  const primaryColor = color(
    ['selected_foreground_color', 'selected_text_color'],
    '#e8e8e8',
  );
  const idleColor = color(['foreground_color', 'idle_text_color'], '#b3b3b3');
  const fontSize = metric(['font_size'], 14, true);
  const tabFontSize = metric(['tab_font_size'], 12, true);
  const lineHeight = (
    absolute: string,
    ratio: string,
    size: number,
    fallback: number,
  ) =>
    props[absolute] !== undefined
      ? metric([absolute], fallback, true)
      : props[ratio] !== undefined
        ? size * metric([ratio], 1.4, true)
        : fallback;
  const mainLineHeight = lineHeight(
    'line_height',
    'line_height_ratio',
    fontSize,
    19.6,
  );
  const tabLineHeight = lineHeight(
    'tab_line_height',
    'tab_line_height_ratio',
    tabFontSize,
    16.8,
  );
  const family = resolveZuiTextStyle(document, node, {
    size: fontSize,
    weight: '400',
  }).family;
  const text = (
    characters: string,
    property: ProjectionText['property'],
    rawColor: string,
    size: number,
    height: number,
  ): ProjectionText => ({
    characters,
    property,
    color: rawColor.slice(0, 7),
    colorOpacity:
      rawColor.length === 9 ? parseInt(rawColor.slice(7), 16) / 255 : 1,
    fontFamily: family,
    fontSize: size,
    fontWeight: '400',
    lineHeight: height / size,
    align: 'left',
  });
  const options = tab ? [] : segmentedOptions(props);
  const labelProperty = (
    tab
      ? ['text', 'label', 'value_text']
      : ['label', 'label_text', 'group_label']
  ).find((key) => typeof props[key] === 'string');
  const label = labelProperty ? String(props[labelProperty]).trim() : '';
  const fragments = Object.fromEntries(
    options.map((option) => [
      option.key,
      text(
        segmentLabel(option.value),
        'options',
        unavailable
          ? disabledColor
          : option.selected
            ? primaryColor
            : idleColor,
        fontSize,
        mainLineHeight,
      ),
    ]),
  );
  const background = unavailable
    ? color(['disabled_background_color'], '#2b2b2b')
    : pressed
      ? color(['pressed_background_color'], '#383838')
      : hot
        ? color(['hover_background_color'], '#454545')
        : tab &&
            (props['background_color'] === undefined ||
              resolve('background_color') === 'transparent')
          ? null
          : color(['background_color'], '#2f2f2f');
  return {
    text:
      label && labelProperty
        ? text(
            label,
            labelProperty as ProjectionText['property'],
            unavailable
              ? disabledColor
              : tab
                ? active
                  ? primaryColor
                  : idleColor
                : color(['label_color'], '#b3b3b3'),
            tab ? tabFontSize : 12,
            tab ? tabLineHeight : 16.8,
          )
        : null,
    ...(options.length ? { textFragments: fragments } : {}),
    segmented: {
      kind: tab ? 'tab' : 'segments',
      options: options.map(({ key, selected }) => ({ key, selected })),
      hasLabel: Boolean(label),
      active,
      offsetX: number(['layout_offset_x'], 0),
      offsetY: number(['layout_offset_y'], 0),
      labelHeight: metric(['group_label_height'], 14, true),
      labelGap: metric(['group_label_gap'], 4),
      insetX: metric(['segment_text_inset_x'], 8),
      insetY: metric(['segment_text_inset_y'], 5),
      selectedInset: metric(['selected_inset'], 2),
      radius: metric(['corner_radius', 'radius'], 4),
      borderWidth: metric(['border_width'], 1),
      selectedBorderWidth: metric(['selected_border_width'], 0),
      underlineHeight: metric(['selected_underline_height'], 2),
      lineHeight: mainLineHeight,
      tabLineHeight,
      tabInsetX: metric(['tab_text_inset_x'], 12),
      colors: {
        background,
        border: tab
          ? '#484848'
          : unavailable
            ? color(['disabled_border_color'], '#363636')
            : pressed || focused || hot
              ? color(['focus_border_color'], '#60aeff')
              : color(['border_color'], '#484848'),
        selected: tab
          ? '#243f5a'
          : unavailable
            ? color(['disabled_background_color'], '#2b2b2b')
            : color(['selected_background_color'], '#243f5a'),
        selectedBorder: tab
          ? '#60aeff'
          : color(['selected_border_color'], '#60aeff'),
        underline: unavailable
          ? disabledColor
          : color(['selected_underline_color', 'accent_color'], '#60aeff'),
      },
    },
  };
}
