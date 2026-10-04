import type { ZuiDocument, ZuiNode, ZuiTable, ZuiValue } from './zui-document';
import { BUTTON_STATE_COLOR_KEYS } from './zui-button-state-projection';
import { INPUT_CONTROL_STYLE_KEYS } from './zui-input-projection';
import { SLIDER_STYLE_KEYS } from './zui-slider-projection';
import { SEGMENTED_STYLE_KEYS } from './zui-segmented-projection';

function table(value: unknown): ZuiTable {
  return value && typeof value === 'object' && !Array.isArray(value)
    ? (value as ZuiTable)
    : {};
}

/** Resolve the static self style; interaction selectors remain runtime metadata. */
export function styledPreviewNode(
  document: ZuiDocument,
  node: ZuiNode,
  ancestors: ZuiNode[] = [],
): ZuiNode {
  const props = resolvedSelfStyle(document, node, ancestors);
  return { ...node, props: { ...node.props, ...node.state, ...props } };
}

export function resolvedSelfStyle(
  document: ZuiDocument,
  node: ZuiNode,
  ancestors: ZuiNode[] = [],
): ZuiTable {
  const props: ZuiTable = {};
  const matched: Array<{ selector: string; style: ZuiTable; order: number }> =
    [];
  for (const sheet of document.stylesheets ?? []) {
    if (sheet['id'] === 'zircon_penpot_prefabs_v1') continue;
    for (const value of Array.isArray(sheet['rules']) ? sheet['rules'] : []) {
      const rule = table(value);
      const selector = String(rule['selector'] ?? '');
      for (const part of selector.split(',').map((part) => part.trim())) {
        if (matchesSelector(part, node, ancestors))
          matched.push({
            selector: part,
            style: table(table(rule['set'])['self']),
            order: matched.length,
          });
      }
    }
  }
  const specificity = (selector: string) =>
    (selector.match(/#/g)?.length ?? 0) * 100 +
    (selector.match(/[.:]/g)?.length ?? 0) * 10 +
    (selector.match(/(?:^|[>\s,])[A-Za-z_][\w-]*/g)?.length ?? 0);
  matched.sort(
    (a, b) =>
      specificity(a.selector) - specificity(b.selector) || a.order - b.order,
  );
  for (const rule of matched) applyStyle(props, rule.style);
  applyStyle(props, table(node.style?.['self']));
  return props;
}

function matchesSelector(
  selector: string,
  node: ZuiNode,
  ancestors: ZuiNode[],
): boolean {
  const parts = selector.split(/\s*(>)\s*|\s+/).filter(Boolean);
  let current = node;
  let parent = 0;
  if (!matchesCompound(parts.pop() ?? '', current)) return false;
  while (parts.length) {
    const direct = parts.at(-1) === '>';
    if (direct) parts.pop();
    const expected = parts.pop() ?? '';
    if (direct) {
      current = ancestors[parent++];
      if (!current || !matchesCompound(expected, current)) return false;
    } else {
      while (
        parent < ancestors.length &&
        !matchesCompound(expected, ancestors[parent])
      )
        parent++;
      current = ancestors[parent++];
      if (!current) return false;
    }
  }
  return true;
}

function matchesCompound(selector: string, node: ZuiNode): boolean {
  const states = new Set<string>();
  for (const [name, value] of [
    ...Object.entries(node.props ?? {}),
    ...Object.entries(node.state ?? {}),
  ]) {
    if (value !== true) continue;
    states.add(name);
    const alias: Record<string, string> = {
      hovered: 'hover',
      pressed: 'active',
      focused: 'focus',
      focus_visible: 'focus-visible',
      focusVisible: 'focus-visible',
      popup_open: 'open',
    };
    if (alias[name]) states.add(alias[name]);
    if (['focus_visible', 'focus-visible', 'focusVisible'].includes(name)) {
      states.add('focused');
      states.add('focus');
    }
  }
  const tokens = selector.match(/(?:[.#:]?[A-Za-z_][\w-]*|\*)/g) ?? [];
  if (tokens.join('') !== selector || tokens.length === 0) return false;
  return tokens.every((token) => {
    if (token.startsWith('.'))
      return (node.classes ?? []).includes(token.slice(1));
    if (token.startsWith('#')) return node.control_id === token.slice(1);
    if (token.startsWith(':')) return states.has(token.slice(1));
    return token === '*' || token === node.component;
  });
}

function applyStyle(props: ZuiTable, style: ZuiTable): void {
  for (const key of [
    'background_color',
    'foreground_color',
    'border_color',
    'border_width',
    'font_size',
    'font_weight',
    'opacity',
    'background',
    'foreground',
    'border',
    'font',
    'radius',
    'fg',
    'color',
    'outline',
    'text_font_weight',
    'text_align',
    'font_family',
    'line_height',
    'line_height_ratio',
    ...BUTTON_STATE_COLOR_KEYS,
    ...INPUT_CONTROL_STYLE_KEYS,
    ...SLIDER_STYLE_KEYS,
    ...SEGMENTED_STYLE_KEYS,
  ]) {
    if (style[key] !== undefined) props[key] = style[key];
  }
  const set = (name: string, value: ZuiValue | undefined) => {
    if (value !== undefined) props[name] = value;
  };
  set('background_color', table(style['background'])['color']);
  set('foreground_color', table(style['foreground'])['color']);
  const border = table(style['border']);
  set('border_color', border['color']);
  set('border_width', border['width']);
  set('corner_radius', border['radius']);
  const text = table(style['text']);
  set('font_size', text['font_size'] ?? text['size']);
  set('font_weight', text['font_weight'] ?? text['weight']);
  const font = table(style['font']);
  set('font_size', font['size']);
  set('font_weight', font['weight']);
}
