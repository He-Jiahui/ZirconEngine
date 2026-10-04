import {
  ZuiDocumentError,
  type ZuiDocument,
  type ZuiNode,
} from './zui-document';
import type { ProjectionGeometry } from './penpot-projection-model';
import { resolveZuiTextStyle } from './zui-text-style';

export const INPUT_CONTROL_STYLE_KEYS = [
  'label_color',
  'accent_color',
  'open_background_color',
  'layout_icon_size',
  'layout_spacing',
  'horizontal_inset',
  'caret_size',
  'caret_right_inset',
  'caret_gap',
  'dot_size',
  'track_width',
  'track_height',
  'thumb_size',
  'checked_background_color',
  'checked_border_color',
  'toggle_background_color',
  'thumb_color',
  'selected_thumb_color',
] as const;

export interface InputControlProjection {
  kind: 'checkbox' | 'radio' | 'toggle' | 'dropdown';
  markSize: number;
  inset: number;
  gap: number;
  rightInset: number;
  lineHeight: number;
  radius: number;
  borderWidth: number;
  fill: string;
  border: string;
  accent: string;
  active: boolean;
  secondary?: { size: number; color: string };
  track?: { width: number; height: number };
}

export function isSelectionControl(node: ZuiNode): boolean {
  return ['Checkbox', 'Radio', 'Toggle', 'Switch'].includes(node.component);
}

/** Defaults and state precedence follow surface/render/selection_controls and dropdowns. */
export function projectInputControl(
  document: ZuiDocument,
  node: ZuiNode,
): InputControlProjection | undefined {
  if (!isSelectionControl(node) && node.component !== 'Dropdown')
    return undefined;
  const props = { ...node.props };
  const flag = (key: string) => props[key] === true;
  const disabled =
    flag('disabled') || flag('loading') || props['enabled'] === false;
  const checkbox = node.component === 'Checkbox';
  const radio = node.component === 'Radio';
  const toggle = node.component === 'Toggle' || node.component === 'Switch';
  const selection = checkbox || radio || toggle;
  if (!selection && typeof props['label'] === 'string' && props['label'].trim())
    throw new ZuiDocumentError(
      'Labeled Dropdown requires separate editable label and value mapping.',
    );
  const active = selection
    ? flag('checked') || flag('selected') || flag('value')
    : flag('open') || flag('popup_open');
  const number = (key: string, fallback: number): number => {
    const value = resolvedProperty(document, props[key]);
    if (value === undefined) return fallback;
    if (typeof value !== 'number' || !Number.isFinite(value) || value < 0)
      throw new ZuiDocumentError(
        `Unsupported ${node.component}.${key}: expected a nonnegative number.`,
      );
    return value;
  };
  const color = (key: string, fallback: string): string => {
    const value = resolvedProperty(document, props[key]);
    if (value === undefined) return fallback;
    if (
      typeof value !== 'string' ||
      !/^(#[\da-f]{3}|#[\da-f]{4}|#[\da-f]{6}|#[\da-f]{8}|transparent)$/i.test(
        value,
      )
    )
      throw new ZuiDocumentError(
        `Unsupported ${node.component}.${key}: expected a resolved color.`,
      );
    return value;
  };
  const focus =
    flag('focused') ||
    flag('pressed') ||
    (flag('hovered') && (!selection || !active));
  const accent = color('accent_color', color('focus_border_color', '#60aeff'));
  const fill = disabled
    ? color('disabled_background_color', '#2b2b2b')
    : active
      ? color(
          radio
            ? 'checked_background_color'
            : selection
              ? 'selected_background_color'
              : 'open_background_color',
          radio ? '#2f2f2f' : '#243f5a',
        )
      : (toggle || !selection) && flag('pressed')
        ? color('pressed_background_color', '#383838')
        : (toggle || !selection) && flag('hovered')
          ? color('hover_background_color', '#454545')
          : toggle
            ? color(
                'toggle_background_color',
                color('background_color', '#2f2f2f'),
              )
            : color('background_color', '#0f0f0f');
  const style = resolveZuiTextStyle(document, node, {
    size: selection ? 14 : 12,
    weight: '400',
  });
  return {
    kind: checkbox
      ? 'checkbox'
      : radio
        ? 'radio'
        : toggle
          ? 'toggle'
          : 'dropdown',
    markSize: number(
      selection ? 'layout_icon_size' : 'caret_size',
      selection ? 16 : 12,
    ),
    inset: selection ? 10 : number('horizontal_inset', 8),
    gap: number(selection ? 'layout_spacing' : 'caret_gap', selection ? 9 : 4),
    rightInset: toggle ? 8 : selection ? 10 : number('caret_right_inset', 12),
    lineHeight: style.size * style.lineHeight,
    radius: number('corner_radius', 4),
    borderWidth: number('border_width', 1),
    fill,
    border: disabled
      ? color('disabled_border_color', '#363636')
      : focus || (active && !radio)
        ? accent
        : radio && active
          ? color('checked_border_color', '#484848')
          : color('border_color', '#484848'),
    accent: selection
      ? accent
      : disabled
        ? color('disabled_foreground_color', '#737373')
        : color('icon_color', '#b3b3b3'),
    active,
    ...(radio || toggle
      ? {
          secondary: {
            size: number(radio ? 'dot_size' : 'thumb_size', radio ? 7 : 12),
            color: disabled
              ? color('disabled_foreground_color', '#737373')
              : radio
                ? accent
                : active
                  ? color('selected_thumb_color', '#e8e8e8')
                  : color('thumb_color', color('foreground_color', '#b3b3b3')),
          },
        }
      : {}),
    ...(toggle
      ? {
          track: {
            width: number('track_width', 34),
            height: number('track_height', 18),
          },
        }
      : {}),
  };
}

export function inputControlGeometry(
  control: InputControlProjection,
  width: number,
  height: number,
): {
  mark: ProjectionGeometry;
  text: ProjectionGeometry;
  secondary?: ProjectionGeometry;
} {
  if (control.kind === 'toggle' && control.track && control.secondary) {
    const trackWidth = Math.min(control.track.width, Math.max(1, width - 20));
    const trackHeight = Math.min(control.track.height, Math.max(1, height));
    const mark = {
      x: Math.max(0, width - control.rightInset - trackWidth),
      y: Math.max(0, (height - trackHeight) / 2),
      width: trackWidth,
      height: trackHeight,
    };
    const thumb = Math.max(
      1,
      Math.min(control.secondary.size, trackWidth, trackHeight),
    );
    return {
      mark,
      text: {
        x: 10,
        y: 5,
        width: Math.max(1, mark.x - 10 - control.gap),
        height: Math.max(height - 10, control.lineHeight),
      },
      secondary: {
        x:
          mark.x +
          2 +
          (control.active ? Math.max(0, trackWidth - thumb - 4) : 0),
        y: mark.y + (trackHeight - thumb) / 2,
        width: thumb,
        height: thumb,
      },
    };
  }
  const leading = control.kind === 'checkbox' || control.kind === 'radio';
  const mark = {
    x: leading
      ? control.inset
      : Math.max(0, width - control.rightInset - control.markSize),
    y: Math.max(0, (height - control.markSize) / 2),
    width: control.markSize,
    height: control.markSize,
  };
  const left = leading ? mark.x + mark.width + control.gap : control.inset;
  return {
    mark,
    ...(control.kind === 'radio' && control.secondary
      ? {
          secondary: {
            x:
              mark.x +
              (mark.width - Math.min(control.secondary.size, mark.width)) / 2,
            y:
              mark.y +
              (mark.height - Math.min(control.secondary.size, mark.height)) / 2,
            width: Math.min(control.secondary.size, mark.width),
            height: Math.min(control.secondary.size, mark.height),
          },
        }
      : {}),
    text: {
      x: left,
      y: leading ? 5 : Math.max(0, (height - control.lineHeight) / 2),
      width: Math.max(
        1,
        (leading ? width - control.rightInset : mark.x - control.gap) - left,
      ),
      height: leading
        ? Math.max(height - 10, control.lineHeight)
        : Math.min(height, control.lineHeight),
    },
  };
}

export function inputStatePreviewNode(node: ZuiNode): ZuiNode {
  if (!isSelectionControl(node) && node.component !== 'Dropdown') return node;
  const props = { ...node.props };
  const disabled =
    props['disabled'] === true ||
    props['loading'] === true ||
    props['enabled'] === false;
  if (disabled)
    props['foreground_color'] = props['disabled_foreground_color'] ?? '#737373';
  return { ...node, props };
}

function resolvedProperty(document: ZuiDocument, value: unknown): unknown {
  const visited = new Set<string>();
  while (typeof value === 'string' && value.startsWith('$')) {
    if (
      visited.has(value) ||
      !Object.hasOwn(document.tokens ?? {}, value.slice(1))
    )
      throw new ZuiDocumentError(`Unresolved input control token: ${value}`);
    visited.add(value);
    value = document.tokens![value.slice(1)];
  }
  return value;
}
