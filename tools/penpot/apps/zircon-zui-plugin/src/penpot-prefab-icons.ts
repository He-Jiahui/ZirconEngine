import icons from 'feather-icons/dist/icons.json';
import type { Board, Shape } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { buttonStatePreviewNode } from './bridge/zui-button-state-projection';
import {
  editorButtonPalette,
  editorButtonState,
} from './bridge/zui-editor-button-style';
import { usesWorkbenchLanguage } from './bridge/zui-editor-button-identity';
import { PENPOT_PALETTE, resolveDesignColor } from './bridge/zui-prefab-system';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';

export function previewIconName(node: ZuiNode): string | null {
  const value =
    node.props?.['icon'] ?? node.props?.['icon_name'] ?? node.props?.['source'];
  if (typeof value !== 'string' || value.length === 0) return null;
  const name = value
    .split('/')
    .pop()!
    .replace(/\.svg$/, '')
    .replace(/-outline$/, '')
    .replaceAll('_', '-');
  if (name in icons) return name;
  const aliases: Record<string, string> = {
    close: 'x',
    settings: 'settings',
    'chevron-down': 'chevron-down',
    cube: 'box',
    'folder-open': 'folder',
    'save-outline': 'save',
    'play-arrow': 'play',
    'more-horiz': 'more-horizontal',
    add: 'plus',
    'minus-circle': 'minus-circle',
    document: 'file',
    refresh: 'refresh-cw',
    undo: 'corner-up-left',
    redo: 'corner-up-right',
    translate: 'move',
    rotate: 'rotate-cw',
    scale: 'maximize-2',
    filter: 'filter',
    AddCircle: 'plus-circle',
    'add-circle': 'plus-circle',
    alert: 'alert-circle',
    button: 'square',
    cancel: 'x-circle',
    capsule: 'sliders',
    chart: 'bar-chart-2',
    component: 'layers',
    error: 'alert-octagon',
    gear: 'settings',
    history: 'clock',
    leaf: 'feather',
    light: 'sun',
    locate: 'crosshair',
    more: 'more-horizontal',
    navigate: 'navigation',
    options: 'sliders',
    'pulse-rifle': 'crosshair',
    puzzle: 'grid',
    renderer: 'monitor',
    resize: 'maximize-2',
    route: 'git-branch',
    sparkles: 'star',
    timeline: 'activity',
    tree: 'git-branch',
    warning: 'alert-triangle',
  };
  return aliases[name] ?? null;
}

export function previewIconPlacement(
  node: ZuiNode,
  iconOnly = false,
): 'none' | 'center' | 'leading' | 'trailing' {
  if (iconOnly) return 'center';
  const authored = node.props?.['icon_placement'];
  if (authored === undefined || authored === '') return 'leading';
  if (typeof authored !== 'string')
    throw new Error(`Unsupported icon placement: ${String(authored)}`);
  switch (authored.toLowerCase()) {
    case 'start':
    case 'before':
    case 'leading':
      return 'leading';
    case 'end':
    case 'after':
    case 'trailing':
      return 'trailing';
    case 'icon_only':
    case 'icon-only':
    case 'only':
      return 'center';
    case 'none':
      return 'none';
    default:
      throw new Error(`Unsupported icon placement: ${authored}`);
  }
}

/** The Editor retained painter tints IconButton geometry; its SVG paint is not authoritative. */
export function previewIconTint(
  document: ZuiDocument,
  node: ZuiNode,
): string | null {
  if (node.component !== 'IconButton') return null;
  if (
    typeof document['penpot_host_theme_source'] !== 'string' ||
    !usesWorkbenchLanguage(node)
  ) {
    return (
      resolveDesignColor(
        document,
        buttonStatePreviewNode(node).props?.['icon_color'],
      ) ?? null
    );
  }
  const palette = editorButtonPalette(document);
  const state = editorButtonState(node);
  const danger = [
    node.control_id,
    node.props?.['icon'],
    node.props?.['validation_level'],
  ]
    .map((value) => String(value ?? '').toLowerCase())
    .some((value) =>
      ['delete', 'trash', 'danger', 'error'].some((key) => value.includes(key)),
    );
  if (state.disabled || state.loading) return palette('text_disabled');
  if (danger) return palette('error');
  if (state.pressed || state.selected || state.open || state.dragging)
    return palette('accent');
  if (state.hovered) return palette('text_primary');
  return (
    resolveDesignColor(document, node.props?.['icon_color']) ??
    palette('text_primary')
  );
}

export function recolorIconShape(shape: Shape, tint: string | null): void {
  const painted = (color: string | undefined) =>
    !!color &&
    !['transparent', 'none', '#00000000'].includes(color.toLowerCase());
  const black = (color: string | undefined) =>
    !!color &&
    ['#000000', '#000', 'black', 'currentcolor'].includes(color.toLowerCase());
  if (Array.isArray(shape.fills))
    shape.fills = shape.fills.map((fill) =>
      tint
        ? painted(fill.fillColor)
          ? { ...fill, fillColor: tint }
          : fill
        : black(fill.fillColor)
          ? { ...fill, fillColor: PENPOT_PALETTE.text }
          : fill,
    );
  shape.strokes = shape.strokes.map((stroke) =>
    tint
      ? painted(stroke.strokeColor)
        ? { ...stroke, strokeColor: tint }
        : stroke
      : black(stroke.strokeColor)
        ? { ...stroke, strokeColor: PENPOT_PALETTE.text }
        : stroke,
  );
  if ('children' in shape)
    for (const child of shape.children) recolorIconShape(child, tint);
}

export function createPrefabIcon(
  board: Board,
  node: ZuiNode,
  document: ZuiDocument,
  iconOnly = true,
): void {
  const placement = previewIconPlacement(node, iconOnly);
  if (placement === 'none') return;
  const name = previewIconName(node);
  const authoredSvg = node['penpot_icon_svg'];
  if (
    (!name && typeof authoredSvg !== 'string') ||
    typeof penpot.createShapeFromSvg !== 'function'
  )
    return;
  const svg =
    typeof authoredSvg === 'string'
      ? authoredSvg
      : `<svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="${PENPOT_PALETTE.textMuted}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">${icons[name as keyof typeof icons]}</svg>`;
  const icon = penpot.createShapeFromSvg(svg);
  if (!icon) throw new Error(`Could not render icon ${name}`);
  recolorIconShape(icon, previewIconTint(document, node));
  icon.name = `Icon: ${String(node.props?.['label'] ?? name)}`;
  icon.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'icon-placement', placement);
  icon.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  board.appendChild(icon);
  if (icon.layoutChild) icon.layoutChild.absolute = true;
  const size = Math.max(8, Math.min(18, board.width - 8, board.height - 8));
  icon.resize(size, size);
  icon.x =
    board.x +
    (placement === 'center'
      ? (board.width - size) / 2
      : placement === 'trailing'
        ? board.width - size - 10
        : 10);
  icon.y = board.y + (board.height - size) / 2;
}
