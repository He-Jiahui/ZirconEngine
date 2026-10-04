import { nativePainterComponent } from './bridge/zui-native-painter-role';
import featherIcons from 'feather-icons/dist/icons.json';
import { recolorIconShape } from './penpot-prefab-icons';
import type { Board, Shape, Text } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import type { ProjectionText } from './bridge/penpot-projection-model';
import {
  applyPenpotTextStyle,
  resolvePenpotTextStyle,
  type PenpotTextStyle,
} from './penpot-text-style';
import { PENPOT_PALETTE } from './bridge/zui-prefab-system';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';
import {
  nativePainterContent,
  type NativePainterIcon,
  type NativePainterPanel,
  type NativePainterText,
  type NativePainterTone,
} from './penpot-native-painter-content';

type ContentPlacement = {
  x: number;
  y: number;
  width: number;
  height: number;
  exactBounds?: boolean;
};

const NATIVE_PAINTER_COLORS: Record<NativePainterTone, string> = {
  accent: PENPOT_PALETTE.accentStrong,
  danger: PENPOT_PALETTE.danger,
  info: PENPOT_PALETTE.info,
  muted: PENPOT_PALETTE.textMuted,
  primary: PENPOT_PALETTE.text,
  selected: PENPOT_PALETTE.surfaceSelected,
  success: PENPOT_PALETTE.accent,
  surface: PENPOT_PALETTE.surfaceRaised,
  warning: PENPOT_PALETTE.warning,
};

function addText(
  board: Board,
  value: string,
  placement: ContentPlacement,
  selected = false,
  style?: PenpotTextStyle,
  tone?: NativePainterTone,
  color?: string,
): Text {
  const text = penpot.createText(value);
  if (!text) throw new Error('Unable to create control content');
  applyPenpotTextStyle(
    text,
    style ?? {
      family: 'Fira Sans',
      id: 'firasans',
      size: 14,
      weight: selected ? '600' : '400',
      lineHeight: 1.4,
    },
  );
  text.growType = 'fixed';
  text.verticalAlign = 'center';
  text.fills = [
    {
      fillColor:
        color ??
        (tone
          ? NATIVE_PAINTER_COLORS[tone]
          : selected
            ? PENPOT_PALETTE.accent
            : PENPOT_PALETTE.text),
      fillOpacity: 1,
    },
  ];
  text.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  text.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-placement',
    JSON.stringify(placement),
  );
  board.appendChild(text);
  if (text.layoutChild) text.layoutChild.absolute = true;
  return text;
}

function addPanel(board: Board, panel: NativePainterPanel): Board {
  const child = penpot.createBoard();
  if (!child) throw new Error('Unable to create native painter content');
  child.name = `Penpot · ${panel.kind}`;
  child.resize(
    Math.max(1, board.width * panel.placement.width),
    Math.max(1, board.height * panel.placement.height),
  );
  child.x = board.x + board.width * panel.placement.x;
  child.y = board.y + board.height * panel.placement.y;
  child.fills = [
    {
      fillColor: panel.color?.slice(0, 7) ?? NATIVE_PAINTER_COLORS[panel.tone],
      fillOpacity: panel.opacity ?? (panel.tone === 'surface' ? 1 : 0.9),
    },
  ];
  child.borderRadius = panel.radius ?? 4;
  child.strokes = panel.borderColor
    ? [
        {
          strokeColor: panel.borderColor,
          strokeWidth: panel.borderWidth ?? 1,
          strokeOpacity: 1,
          strokeStyle: 'solid',
          strokeAlignment: 'inner',
        },
      ]
    : [];
  child.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  child.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-panel-placement',
    JSON.stringify(panel.placement),
  );
  board.appendChild(child);
  if (child.layoutChild) child.layoutChild.absolute = true;
  return child;
}

const NATIVE_ICON_EXTENTS = {
  s: 16,
  m: 20,
  l: 24,
  xl: 32,
} as const;

function addIcon(board: Board, icon: NativePainterIcon): void {
  if (typeof penpot.createShapeFromSvg !== 'function') return;
  const path = featherIcons[icon.name as keyof typeof featherIcons];
  if (!path && !icon.svg) return;
  const color = icon.color ?? NATIVE_PAINTER_COLORS[icon.tone];
  const shape = penpot.createShapeFromSvg(
    icon.svg ??
      `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${path}</svg>`,
  );
  if (!shape) return;
  if (icon.svg) recolorIconShape(shape, color);
  shape.name = `Penpot · icon · ${icon.name}`;
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-icon-placement',
    JSON.stringify(icon.placement),
  );
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'content-icon-size',
    icon.size,
  );
  board.appendChild(shape);
  if (shape.layoutChild) shape.layoutChild.absolute = true;
  resizeNativeIcon(board, shape, icon.placement, icon.size);
}

function resizeNativeIcon(
  board: Board,
  shape: Shape,
  placement: ContentPlacement,
  size: NativePainterIcon['size'],
): void {
  if (placement.exactBounds) {
    shape.resize(
      Math.max(1, board.width * placement.width),
      Math.max(1, board.height * placement.height),
    );
    shape.x = board.x + board.width * placement.x;
    shape.y = board.y + board.height * placement.y;
    return;
  }
  const availableWidth = Math.max(1, board.width * placement.width - 8);
  const availableHeight = Math.max(1, board.height * placement.height - 8);
  const extent = Math.max(
    8,
    Math.min(NATIVE_ICON_EXTENTS[size], availableWidth, availableHeight),
  );
  shape.resize(extent, extent);
  shape.x =
    board.x +
    board.width * placement.x +
    (board.width * placement.width - extent) / 2;
  shape.y =
    board.y +
    board.height * placement.y +
    (board.height * placement.height - extent) / 2;
}

function nativeTextStyle(
  style: PenpotTextStyle | undefined,
  text: NativePainterText,
): PenpotTextStyle {
  const base =
    style ??
    ({
      family: 'Fira Sans',
      id: 'firasans',
      size: 14,
      weight: '400',
      lineHeight: 1.4,
    } satisfies PenpotTextStyle);
  return {
    ...base,
    size: text.size ?? base.size,
    weight: text.weight ?? base.weight,
    lineHeight: text.lineHeight ?? base.lineHeight,
  };
}

function createNativePainterContent(
  board: Board,
  node: ZuiNode,
  style: PenpotTextStyle | undefined,
  document?: ZuiDocument,
): boolean {
  const content = nativePainterContent(
    node,
    document
      ? { document, width: board.width, height: board.height }
      : undefined,
  );
  if (!content) return false;
  for (const panel of content.panels) addPanel(board, panel);
  for (const icon of content.icons ?? []) addIcon(board, icon);
  const treeRow = nativePainterComponent(node) === 'TreeRow';
  // TreeRow uses the visible source-owned semantic text, not an auxiliary clone.
  for (const text of treeRow ? [] : content.texts)
    addText(
      board,
      text.value,
      text.placement,
      false,
      nativeTextStyle(style, text),
      text.tone,
      text.color,
    );
  return !treeRow;
}

export function createControlContent(
  board: Board,
  node: ZuiNode,
  document?: ZuiDocument,
  projected?: ProjectionText | null,
): boolean {
  const style = document
    ? resolvePenpotTextStyle(document, node, projected)
    : undefined;
  const component = node.component.toLowerCase();
  const options = node.props?.['options'];
  const items = node.props?.['items'];
  let replacesText = createNativePainterContent(board, node, style, document);
  // Empty slots have no product pixels. Their review hosts must supply a real
  // instance; drawing a label or dashed guide would turn missing content into
  // a misleading visual substitute and hide slot contract failures.
  if (
    !replacesText &&
    (component === 'table' || component === 'segmentedcontrol') &&
    Array.isArray(options) &&
    options.length
  ) {
    const count = options.length;
    options.forEach((value, index) => {
      const label = String(value).split('|')[0];
      addText(
        board,
        label,
        { x: index / count, y: 0, width: 1 / count, height: 1 },
        label === node.props?.['value'],
        style,
      );
    });
    replacesText = true;
  } else if (
    !replacesText &&
    ['virtuallist', 'pagedlist', 'list'].includes(component) &&
    Array.isArray(items)
  ) {
    const rowHeight = Number(node.props?.['item_extent'] ?? 24);
    const count = Math.min(
      items.length,
      Math.max(1, Math.floor(board.height / rowHeight)),
    );
    for (let index = 0; index < count; index++) {
      const value = items[index];
      if (typeof value === 'string')
        addText(
          board,
          value,
          {
            x: 0,
            y: index / count,
            width: 1,
            height: 1 / count,
          },
          false,
          style,
        );
    }
    replacesText = count > 0;
  } else if (
    !replacesText &&
    component === 'slider' &&
    typeof node.props?.['value'] === 'number'
  ) {
    addText(
      board,
      Number(node.props['value']).toFixed(2),
      { x: 0.78, y: 0, width: 0.22, height: 1 },
      false,
      style,
    );
    replacesText = true;
  } else if (
    !replacesText &&
    !projected &&
    (component.includes('number') ||
      component.includes('vector') ||
      component.includes('range'))
  ) {
    const value = node.props?.['value'] ?? node.props?.['value_number'];
    if (typeof value === 'number' || Array.isArray(value)) {
      addText(
        board,
        Array.isArray(value) ? value.join('    ') : String(value),
        { x: 0, y: 0, width: 1, height: 1 },
        false,
        style,
      );
      replacesText = true;
    }
  }
  refreshControlContent(board);
  return replacesText;
}

/**
 * Rebuild only the design-time content painted by a native Runtime control.
 *
 * A retained host review can change a popup/painter state without changing
 * the authored component tree.  Keeping this operation separate from the
 * generic control-content path lets the review host update a cloned board
 * without rematerializing every native component in a large Workbench.
 */
export function refreshNativePainterContent(
  board: Board,
  node: ZuiNode,
  document?: ZuiDocument,
  projected?: ProjectionText | null,
): boolean {
  const painterChildren = board.children.filter(
    (child) =>
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'content-placement') ||
      child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'content-panel-placement',
      ) ||
      child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'content-icon-placement',
      ),
  );
  for (const child of painterChildren) child.remove();
  const style = document
    ? resolvePenpotTextStyle(document, node, projected)
    : undefined;
  const painted = createNativePainterContent(board, node, style, document);
  if (painted) refreshControlContent(board);
  return painted;
}

export function refreshControlContent(board: Board): void {
  for (const child of board.children) {
    const decoration = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'control-decoration',
    );
    if (decoration) {
      const sliderValueLane =
        decoration === 'slider' && board.width >= 96 ? 56 : 0;
      const railWidth = Math.max(16, board.width - 16 - sliderValueLane);
      if (child.name.endsWith(' rail')) {
        child.resize(railWidth, 4);
        child.x = board.x + 8;
        child.y = board.y + (board.height - child.height) / 2;
      } else {
        const fraction = Number(
          child.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'control-fraction'),
        );
        const thumbWidth = decoration === 'splitter' ? 8 : 12;
        const thumbHeight =
          decoration === 'splitter' ? Math.max(16, board.height - 10) : 12;
        child.resize(thumbWidth, thumbHeight);
        child.x =
          board.x +
          8 +
          (railWidth - thumbWidth) * (Number.isFinite(fraction) ? fraction : 0);
        child.y = board.y + (board.height - child.height) / 2;
      }
      continue;
    }
    const panelStored = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'content-panel-placement',
    );
    if (panelStored) {
      const placement = JSON.parse(panelStored) as ContentPlacement;
      child.resize(
        Math.max(1, board.width * placement.width),
        Math.max(1, board.height * placement.height),
      );
      child.x = board.x + board.width * placement.x;
      child.y = board.y + board.height * placement.y;
      continue;
    }
    const iconStored = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'content-icon-placement',
    );
    if (iconStored) {
      const placement = JSON.parse(iconStored) as ContentPlacement;
      const size = child.getSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        'content-icon-size',
      );
      if (size === 's' || size === 'm' || size === 'l' || size === 'xl') {
        resizeNativeIcon(board, child as Shape, placement, size);
      }
      continue;
    }
    const stored = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'content-placement',
    );
    if (!stored) continue;
    const placement = JSON.parse(stored) as {
      x: number;
      y: number;
      width: number;
      height: number;
      exactBounds?: boolean;
    };
    child.resize(
      Math.max(
        1,
        board.width * placement.width - (placement.exactBounds ? 0 : 16),
      ),
      Math.max(1, board.height * placement.height),
    );
    child.x =
      board.x + board.width * placement.x + (placement.exactBounds ? 0 : 8);
    child.y = board.y + board.height * placement.y;
  }
}
