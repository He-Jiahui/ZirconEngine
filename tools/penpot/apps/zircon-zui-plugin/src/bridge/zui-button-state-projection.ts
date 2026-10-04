import type { ZuiNode, ZuiTable, ZuiValue } from './zui-document';

export const BUTTON_STATE_COLOR_KEYS = [
  'hover_background_color',
  'pressed_background_color',
  'selected_background_color',
  'disabled_background_color',
  'focus_border_color',
  'disabled_border_color',
  'disabled_foreground_color',
  'text_color',
  'icon_color',
  'selected_icon_color',
] as const;

/** Project authored button palettes using surface/render/buttons/{state,style}.rs. */
export function buttonStatePreviewNode(node: ZuiNode): ZuiNode {
  if (
    !['Button', 'ToggleButton', 'IconButton'].includes(node.component) ||
    !BUTTON_STATE_COLOR_KEYS.some((key) => node.props?.[key] !== undefined)
  )
    return node;
  const props: ZuiTable = { ...node.props };
  // V2 import overlays authored state onto props before reading metadata flags.
  const attributes = { ...node.props, ...node.state };
  const booleanAttribute = (...keys: string[]): boolean | undefined => {
    for (const key of keys) {
      const value = attributes[key];
      if (typeof value === 'boolean') return value;
    }
    return undefined;
  };
  const flag = (key: string) => booleanAttribute(key) ?? false;
  const unavailable =
    flag('disabled') ||
    flag('loading') ||
    booleanAttribute('enabled') === false;
  const marked = flag('selected') || flag('checked');
  const open = booleanAttribute('open', 'popup_open') ?? false;
  const dropHovered =
    booleanAttribute('drop_hovered', 'active_drag_target') ?? false;
  const hot = flag('hovered') || open || flag('dragging') || dropHovered;
  // Drag/drop outrank pressed in the runtime's scalar painter state.
  const pressed = flag('pressed') && !flag('dragging') && !dropHovered;
  const focusVisible =
    booleanAttribute('focus_visible', 'focusVisible') ?? flag('focused');
  const focused = focusVisible && !pressed && !flag('dragging') && !dropHovered;
  const icon = node.component === 'IconButton';
  const kind = ['button_color', 'button_variant', 'validation_level'].map(
    (key) => String(props[key] ?? '').toLowerCase(),
  );
  const danger = kind.some((value) => /danger|error/.test(value));
  const primary = !danger && kind.some((value) => value.includes('primary'));
  const tertiary =
    !danger && !primary && kind.some((value) => /tertiary|text/.test(value));
  const color = (key: string, fallback: ZuiValue): ZuiValue =>
    props[key] ?? fallback;
  // These are ButtonVisual's built-in workbench_dark defaults, not prefab guesses.
  const normal = color(
    'background_color',
    icon
      ? '#2f2f2f'
      : danger
        ? '#4c2427'
        : primary
          ? '#253f59'
          : tertiary
            ? '#151515'
            : '#242424',
  );
  const hover = color(
    'hover_background_color',
    !icon && primary ? '#243f5a' : '#454545',
  );
  const down = color('pressed_background_color', '#383838');
  const selected = color(
    'selected_background_color',
    icon ? '#243f5a' : danger ? normal : hover,
  );
  props['background_color'] = unavailable
    ? color('disabled_background_color', '#2b2b2b')
    : icon && marked
      ? selected
      : pressed
        ? danger && !icon
          ? normal
          : down
        : marked
          ? selected
          : hot
            ? danger && !icon
              ? normal
              : hover
            : normal;
  props['border_color'] = unavailable
    ? color('disabled_border_color', '#363636')
    : focused || pressed || marked
      ? color('focus_border_color', '#60aeff')
      : color(
          'border_color',
          !icon && primary
            ? '#60aeff'
            : !icon && danger
              ? '#eb605c'
              : '#484848',
        );
  const foreground = color(
    'foreground_color',
    color('text_color', danger ? '#eb605c' : tertiary ? '#b3b3b3' : '#e8e8e8'),
  );
  const iconNormal = color(
    'icon_color',
    color('foreground_color', color('text_color', '#b3b3b3')),
  );
  props['foreground_color'] = unavailable
    ? color('disabled_foreground_color', '#737373')
    : foreground;
  if (icon)
    props['icon_color'] = unavailable
      ? color('disabled_foreground_color', '#737373')
      : marked || pressed
        ? color('selected_icon_color', color('icon_color', '#60aeff'))
        : iconNormal;
  return { ...node, props };
}
