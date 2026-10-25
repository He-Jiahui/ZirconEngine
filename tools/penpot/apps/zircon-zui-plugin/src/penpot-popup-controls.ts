import { popupLayoutGeometry } from './penpot-popup-layout-geometry';
import type { Board, Shape, Text } from '@penpot/plugin-types';
import type {
  ProjectedZuiNode,
  ZuiAssetProjection,
} from './bridge/penpot-projection-model';
import type { ZuiDocument, ZuiNode, ZuiTable } from './bridge/zui-document';
import {
  PENPOT_PALETTE,
  resolveDesignColor,
  resolveDesignNumber,
} from './bridge/zui-prefab-system';
import {
  applyPenpotTextStyle,
  resolvePenpotTextStyle,
} from './penpot-text-style';
import {
  ZUI_METADATA_EXPLICIT_OVERLAY,
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_NODE_ID,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
  ZUI_ROLE_NODE,
} from './metadata';

const TRIGGER_POPUP_COMPONENTS = new Set(['dropdown', 'select']);
const CONTENT_POPUP_COMPONENTS = new Set([
  'contextactionmenu',
  'contextmenu',
  'dropdownpopup',
  'modal',
  'popover',
  'popup',
  'popupmenu',
]);

const POPUP_OVERLAY_FOR = 'popup-overlay-for';
const POPUP_CONTENT_FOR = 'popup-content-for';
const POPUP_OPTION_ROLE = 'popup-option';
const POPUP_SOURCE_PROPERTY = 'popup-source-property';

export interface PopupOption {
  value: string;
  label: string;
  selected: boolean;
  disabled: boolean;
  focused: boolean;
  hovered: boolean;
  pressed?: boolean;
  loading?: boolean;
  separator: boolean;
  danger?: boolean;
}

const POPUP_MENU_PALETTE = {
  popup: '#383838',
  border: '#383838',
  selected: '#383838',
  hovered: '#0070e0',
  text: '#c0c0c0',
  textSelected: '#ffffff',
  disabled: '#808080',
  danger: '#ef3535',
  accent: '#0070e0',
  rowRadius: 3,
  panelRadius: 0,
  separatorInsetX: 8,
} as const;

export interface PopupMenuRowPaint {
  fillColor: string | null;
  strokeColor: string | null;
  textColor: string;
  borderRadius: number;
  separatorInsetX: number;
}

export function popupMenuPanelPaint() {
  return {
    fillColor: POPUP_MENU_PALETTE.popup,
    strokeColor: POPUP_MENU_PALETTE.border,
    strokeWidth: 1,
    borderRadius: POPUP_MENU_PALETTE.panelRadius,
  };
}

/** Match the ordinary Editor's retained PopupMenu row contract. */
export function popupMenuRowPaint(option: PopupOption): PopupMenuRowPaint {
  if (option.separator) {
    return {
      fillColor: POPUP_MENU_PALETTE.border,
      strokeColor: null,
      textColor: POPUP_MENU_PALETTE.text,
      borderRadius: 0,
      separatorInsetX: POPUP_MENU_PALETTE.separatorInsetX,
    };
  }

  const unavailable = option.disabled || option.loading === true;
  const selected = option.selected && !unavailable;
  const pressed = option.pressed === true && !unavailable;
  const hot = (option.hovered || pressed) && !unavailable;
  const focusedOnly =
    option.focused &&
    !unavailable &&
    !selected &&
    !pressed;
  return {
    fillColor: unavailable
      ? null
      : hot
        ? POPUP_MENU_PALETTE.hovered
        : selected
          ? POPUP_MENU_PALETTE.selected
          : null,
    strokeColor:
      selected || focusedOnly ? POPUP_MENU_PALETTE.accent : null,
    textColor: unavailable
      ? POPUP_MENU_PALETTE.disabled
      : option.danger
        ? POPUP_MENU_PALETTE.danger
        : hot
          ? POPUP_MENU_PALETTE.textSelected
          : POPUP_MENU_PALETTE.text,
    borderRadius: POPUP_MENU_PALETTE.rowRadius,
    separatorInsetX: 0,
  };
}

export interface PopupRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PopupLayout extends PopupRect {
  opensAbove: boolean;
  authority?: 'source-measured-fallback';
}

interface PopupLayoutOptions {
  rowHeight?: number;
  paddingTop?: number;
  paddingRight?: number;
  paddingBottom?: number;
  paddingLeft?: number;
  offsetX?: number;
  offsetY?: number;
  width?: number;
  height?: number;
  rowSpacing?: number;
  textInsetX?: number;
}

/**
 * Convert authored option data into a stable visual contract.  The returned
 * values are design-only; the source node remains the runtime authority.
 */
export function popupOptions(node: ZuiNode): PopupOption[] {
  const props = node.props ?? {};
  const source = popupOptionSource(props);
  const selectedValues = valueSet(
    props['selected_options'] ?? props['selectedOptions'],
  );
  const disabledValues = valueSet(props['disabled_options']);
  const focusedValues = valueSet(props['focused_options']);
  const hoveredValues = valueSet(props['hovered_options']);
  const pressedValues = valueSet(props['pressed_options']);
  const selectedIndex = finiteInteger(props['selected_index']);
  const currentValue = props['value'];
  const currentValueText =
    typeof props['value_text'] === 'string' ? props['value_text'] : null;

  return source.flatMap((raw, index) => {
    const parsed = parsePopupOption(raw);
    if (!parsed) return [];
    if (parsed.separator) return [parsed];
    const selected =
      parsed.selected ||
      (selectedIndex !== null && selectedIndex === index) ||
      selectedValues.has(parsed.value) ||
      (typeof currentValue === 'string' && currentValue === parsed.value) ||
      (Array.isArray(currentValue) && currentValue.includes(parsed.value));
    const label =
      parsed.label === parsed.value &&
      selected &&
      currentValueText !== null &&
      parsed.value !== ''
        ? currentValueText
        : parsed.label;
    return [
      {
        ...parsed,
        label,
        selected,
        disabled: parsed.disabled || disabledValues.has(parsed.value),
        focused: parsed.focused || focusedValues.has(parsed.value),
        hovered:
        parsed.hovered ||
        hoveredValues.has(parsed.value) ||
        pressedValues.has(parsed.value),
      ...(parsed.pressed || pressedValues.has(parsed.value)
        ? { pressed: true }
        : {}),
      },
    ];
  });
}

/** Only an authored state/property can open a design-time popup. */
export function popupIsOpen(node: ZuiNode): boolean {
  const state = node.state ?? {};
  const props = node.props ?? {};
  const candidates = [
    state['open'],
    state['popup_open'],
    props['open'],
    props['popup_open'],
  ];
  const authored = candidates.find((value) => typeof value === 'boolean');
  return authored === true;
}

export function computePopupLayout(
  trigger: PopupRect,
  viewport: PopupRect,
  options: readonly PopupOption[],
  layoutOptions: PopupLayoutOptions = {},
): PopupLayout {
  const rowHeight = clampPositive(layoutOptions.rowHeight ?? 28, 1);
  const paddingTop = clampPositive(layoutOptions.paddingTop ?? 8, 0);
  const paddingBottom = clampPositive(layoutOptions.paddingBottom ?? 8, 0);
  const contentHeight = options.reduce(
    (height, option) => height + (option.separator ? 1 : rowHeight),
    0,
  );
  const height = Math.max(1, paddingTop + contentHeight + paddingBottom);
  const requestedWidth = Math.max(trigger.width, layoutOptions.width ?? 0);
  const width = Math.min(
    Math.max(1, requestedWidth),
    Math.max(1, viewport.width),
  );
  const offsetX = layoutOptions.offsetX ?? 0;
  const offsetY = layoutOptions.offsetY ?? 4;
  const belowY = trigger.y + trigger.height + offsetY;
  const aboveY = trigger.y - height - offsetY;
  const bottom = viewport.y + viewport.height;
  const canOpenAbove = aboveY >= viewport.y;
  const opensAbove = belowY + height > bottom && canOpenAbove;
  const unclampedY = opensAbove ? aboveY : belowY;
  const y = clamp(
    unclampedY,
    viewport.y,
    Math.max(viewport.y, bottom - height),
  );
  const x = clamp(
    trigger.x + offsetX,
    viewport.x,
    Math.max(viewport.x, viewport.x + viewport.width - width),
  );
  return { x, y, width, height, opensAbove };
}

/** PopupMenu uses a control anchor and a separately sized source preview. */
export function computePopupMenuLayout(
  anchor: PopupRect,
  viewport: PopupRect,
  options: readonly PopupOption[],
  layoutOptions: PopupLayoutOptions = {},
): PopupLayout {
  const rowHeight = clampPositive(layoutOptions.rowHeight ?? 24, 1);
  const rowSpacing = clampPositive(layoutOptions.rowSpacing ?? 0, 0);
  const paddingTop = clampPositive(layoutOptions.paddingTop ?? 0, 0);
  const paddingBottom = clampPositive(layoutOptions.paddingBottom ?? 0, 0);
  const contentHeight =
    options.length * rowHeight + Math.max(0, options.length - 1) * rowSpacing;
  const height = Math.max(
    1,
    layoutOptions.height ?? 0,
    paddingTop + contentHeight + paddingBottom,
  );
  const width = Math.min(
    clampPositive(layoutOptions.width ?? anchor.width, 1),
    Math.max(1, viewport.width),
  );
  const offsetX = layoutOptions.offsetX ?? 0;
  const offsetY = layoutOptions.offsetY ?? 4;
  const belowY = anchor.y + anchor.height + offsetY;
  const aboveY = anchor.y - height - offsetY;
  const bottom = viewport.y + viewport.height;
  const canOpenAbove = aboveY >= viewport.y;
  const opensAbove = belowY + height > bottom && canOpenAbove;
  const unclampedY = opensAbove ? aboveY : belowY;
  return {
    x: clamp(
      anchor.x + offsetX,
      viewport.x,
      Math.max(viewport.x, viewport.x + viewport.width - width),
    ),
    y: clamp(unclampedY, viewport.y, Math.max(viewport.y, bottom - height)),
    width,
    height,
    opensAbove,
    authority: 'source-measured-fallback',
  };
}

/**
 * Reconcile projection-only popup surfaces.  Popup boards and rows are marked
 * auxiliary/explicit-overlay so source-node export and semantic node audits do
 * not mistake the preview layer for a replacement runtime tree.
 */
export function refreshPopupOverlays(
  assetBoard: Board,
  document: ZuiDocument,
  projection: ZuiAssetProjection,
): void {
  const semanticBoards = semanticBoardIndex(assetBoard);
  const projected = new Map(
    projection.shapes.map((shape) => [shape.nodeId, shape]),
  );
  const existing = new Map<string, Board>();
  for (const child of assetBoard.children) {
    if (child.type !== 'board') continue;
    const nodeId = metadata(child, POPUP_OVERLAY_FOR);
    if (nodeId) existing.set(nodeId, child);
  }

  const desired = new Set<string>();
  for (const shape of projection.shapes) {
    const node = document.nodes?.[shape.nodeId];
    const component = node?.component.toLowerCase();
    if (!node || !component) continue;
    const options = popupOptions(node);
    if (CONTENT_POPUP_COMPONENTS.has(component)) {
      const board = semanticBoards.get(shape.nodeId);
      const isPopupMenu = component === 'popupmenu';
      removePopupContent(board, shape.nodeId);
      const anchorBoard = isPopupMenu
        ? popupMenuAnchorBoard(node, document, semanticBoards)
        : board;
      if (
        board &&
        anchorBoard &&
        popupIsOpen(node) &&
        options.length > 0 &&
        shape.children.length === 0 &&
        (isPopupMenu || !board.hidden)
      ) {
        const overlay = existing.get(shape.nodeId) ?? penpot.createBoard();
        if (!overlay) throw new Error('Unable to create Penpot popup overlay');
        if (!existing.has(shape.nodeId)) assetBoard.appendChild(overlay);
        const projectedNode = projected.get(shape.nodeId);
        const layoutOptions = isPopupMenu
          ? popupMenuLayoutOptions(node, document)
          : popupLayoutOptions(node, document);
        const layout: PopupLayout = isPopupMenu
          ? computePopupMenuLayout(
              popupRect(anchorBoard),
              popupRect(assetBoard),
              options,
              {
                ...layoutOptions,
                width: measurePopupMenuWidth(
                  node,
                  document,
                  projectedNode,
                  options,
                  layoutOptions,
                ),
              },
            )
          : {
              x: board.x,
              y: board.y,
              width: board.width,
              height: board.height,
              opensAbove: false,
            };
        configurePopupBoard(
          overlay,
          `${shape.nodeId} · popup preview`,
          shape.nodeId,
          node,
          document,
          layout,
          isPopupMenu,
        );
        renderPopupRows(
          overlay,
          shape.nodeId,
          node,
          document,
          projectedNode,
          options,
          layoutOptions,
          true,
          isPopupMenu,
        );
        desired.add(shape.nodeId);
      }
      continue;
    }
    if (!TRIGGER_POPUP_COMPONENTS.has(component)) continue;
    const overlay = existing.get(shape.nodeId);
    const trigger = semanticBoards.get(shape.nodeId);
    if (!trigger || !popupIsOpen(node) || options.length === 0) continue;
    const layout = computePopupLayout(
      {
        x: trigger.x,
        y: trigger.y,
        width: trigger.width,
        height: trigger.height,
      },
      {
        x: assetBoard.x,
        y: assetBoard.y,
        width: assetBoard.width,
        height: assetBoard.height,
      },
      options,
      popupLayoutOptions(node, document),
    );
    const popup = overlay ?? penpot.createBoard();
    if (!popup) throw new Error('Unable to create Penpot popup overlay');
    if (!overlay) assetBoard.appendChild(popup);
    configurePopupBoard(
      popup,
      `${shape.nodeId} · popup preview`,
      shape.nodeId,
      node,
      document,
      layout,
    );
    renderPopupRows(
      popup,
      shape.nodeId,
      node,
      document,
      projected.get(shape.nodeId),
      options,
      popupLayoutOptions(node, document),
      true,
    );
    desired.add(shape.nodeId);
  }

  for (const [nodeId, popup] of existing) {
    if (!desired.has(nodeId)) popup.remove();
  }
}

function configurePopupBoard(
  board: Board,
  name: string,
  sourceNodeId: string,
  node: ZuiNode,
  document: ZuiDocument,
  layout: PopupLayout,
  popupMenu = false,
): void {
  const popupMenuPaint = popupMenu ? popupMenuPanelPaint() : null;
  board.name = name;
  board.resize(layout.width, layout.height);
  board.x = layout.x;
  board.y = layout.y;
  board.hidden = false;
  board.clipContent = true;
  board.fills = [
    {
      fillColor:
        popupMenuPaint?.fillColor ??
        resolveDesignColor(
          document,
          node.props?.['popup_background_color'] ??
            node.props?.['background_color'],
        ) ??
        PENPOT_PALETTE.surfaceRaised,
      fillOpacity: 1,
    },
  ];
  board.strokes = [
    {
      strokeColor:
        popupMenuPaint?.strokeColor ??
        resolveDesignColor(
          document,
          node.props?.['popup_border_color'] ?? node.props?.['border_color'],
        ) ??
        PENPOT_PALETTE.borderStrong,
      strokeOpacity: 1,
      strokeWidth:
        popupMenuPaint?.strokeWidth ??
        resolveDesignNumber(
          document,
          node.props?.['popup_border_width'] ?? node.props?.['border_width'],
        ) ??
        1,
      strokeStyle: 'solid',
      strokeAlignment: 'inner',
    },
  ];
  board.borderRadius =
    popupMenuPaint?.borderRadius ??
    Math.max(
      0,
      resolveDesignNumber(
        document,
        node.props?.['popup_corner_radius'] ?? node.props?.['corner_radius'],
      ) ?? 4,
    );
  setMetadata(board, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
  setMetadata(board, ZUI_METADATA_EXPLICIT_OVERLAY, 'true');
  setMetadata(board, POPUP_OVERLAY_FOR, sourceNodeId);
  setMetadata(board, POPUP_SOURCE_PROPERTY, popupSourceProperty(node));
  setMetadata(
    board,
    'popup-layout-authority',
    layout.authority ?? 'source-projected-preview',
  );
  if (board.layoutChild) board.layoutChild.absolute = true;
  recordPopupLayoutGeometry(board);
}

function renderPopupRows(
  parent: Board,
  sourceNodeId: string,
  node: ZuiNode,
  document: ZuiDocument,
  projection: ProjectedZuiNode | undefined,
  options: readonly PopupOption[],
  layoutOptions: PopupLayoutOptions,
  overlay: boolean,
  popupMenu = false,
): void {
  // Source option order owns row identity. Reflow must not replace the
  // imported children that the immutable export guard already records.
  const existingRows = parent.children.filter(
    (child): child is Board =>
      child.type === 'board' &&
      metadata(child, POPUP_CONTENT_FOR) === sourceNodeId,
  );
  const paddingTop = clampPositive(layoutOptions.paddingTop ?? 8, 0);
  const paddingLeft = clampPositive(layoutOptions.paddingLeft ?? 8, 0);
  const paddingRight = clampPositive(layoutOptions.paddingRight ?? 8, 0);
  const rowHeight = clampPositive(layoutOptions.rowHeight ?? 28, 1);
  const rowSpacing = popupMenu
    ? clampPositive(layoutOptions.rowSpacing ?? 0, 0)
    : 0;
  const textInsetX = clampPositive(layoutOptions.textInsetX ?? 8, 0);
  const style = resolvePenpotTextStyle(
    document,
    node,
    projection?.text ?? null,
  );
  const textColor =
    resolveDesignColor(document, node.props?.['foreground_color']) ??
    PENPOT_PALETTE.text;
  const selectedColor =
    resolveDesignColor(document, node.props?.['selected_background_color']) ??
    PENPOT_PALETTE.surfaceSelected;
  const hoveredColor =
    resolveDesignColor(document, node.props?.['hover_background_color']) ??
    PENPOT_PALETTE.surface;
  const disabledColor =
    resolveDesignColor(document, node.props?.['disabled_foreground_color']) ??
    PENPOT_PALETTE.textSubtle;
  const accentColor =
    resolveDesignColor(document, node.props?.['accent_color']) ??
    PENPOT_PALETTE.accent;
  let cursor = parent.y + paddingTop;
  for (const [index, option] of options.entries()) {
    const slotHeight = popupMenu
      ? rowHeight
      : option.separator
        ? 1
        : rowHeight;
    const height = option.separator ? 1 : rowHeight;
    const rowOffsetY =
      popupMenu && option.separator ? (rowHeight - height) / 2 : 0;
    const menuPaint = popupMenu ? popupMenuRowPaint(option) : null;
    const rowPaddingLeft = paddingLeft + (menuPaint?.separatorInsetX ?? 0);
    const rowPaddingRight = paddingRight + (menuPaint?.separatorInsetX ?? 0);
    const existingRow = existingRows[index];
    const row = existingRow ?? penpot.createBoard();
    if (!row) throw new Error('Unable to create Penpot popup option');
    row.name = option.separator
      ? `Penpot · ${sourceNodeId} separator`
      : `Penpot · ${sourceNodeId} option · ${option.label}`;
    row.resize(Math.max(1, parent.width - rowPaddingLeft - rowPaddingRight), height);
    row.x = parent.x + rowPaddingLeft;
    row.y = cursor + rowOffsetY;
    row.borderRadius =
      menuPaint?.borderRadius ?? (option.separator ? 0 : 2);
    row.fills = menuPaint
      ? menuPaint.fillColor
        ? [{ fillColor: menuPaint.fillColor, fillOpacity: 1 }]
        : []
      : option.separator
        ? [{ fillColor: PENPOT_PALETTE.border, fillOpacity: 1 }]
        : option.selected
          ? [{ fillColor: selectedColor, fillOpacity: 1 }]
          : option.hovered || option.focused
            ? [{ fillColor: hoveredColor, fillOpacity: 1 }]
            : [];
    if (menuPaint) {
      row.strokes = menuPaint.strokeColor
        ? [
            {
              strokeColor: menuPaint.strokeColor,
              strokeOpacity: 1,
              strokeWidth: 1,
              strokeStyle: 'solid',
              strokeAlignment: 'inner',
            },
          ]
        : [];
    }
    setMetadata(row, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
    setMetadata(row, POPUP_CONTENT_FOR, sourceNodeId);
    setMetadata(
      row,
      POPUP_OPTION_ROLE,
      option.separator ? 'separator' : 'item',
    );
    setMetadata(row, 'popup-option-value', option.value);
    if (!existingRow) parent.appendChild(row);
    if (row.layoutChild) row.layoutChild.absolute = true;
    recordPopupLayoutGeometry(row);

    const labels = row.children.filter(
      (child): child is Text =>
        child.type === 'text' &&
        metadata(child, POPUP_CONTENT_FOR) === sourceNodeId &&
        metadata(child, POPUP_OPTION_ROLE) === 'label',
    );
    if (!option.separator && option.label !== '') {
      const existingText = labels[0];
      const text = existingText ?? penpot.createText(option.label);
      if (!text) throw new Error('Unable to create Penpot popup option text');
      text.characters = option.label;
      text.name = `Penpot · ${sourceNodeId} option text · ${option.label}`;
      text.growType = 'fixed';
      applyPenpotTextStyle(text, {
        ...style,
        weight: !popupMenu && option.selected ? '600' : style.weight,
      });
      text.align = 'left';
      text.verticalAlign = 'center';
      text.fills = [
        {
          fillColor:
            menuPaint?.textColor ??
            (option.disabled
              ? disabledColor
              : option.selected
                ? accentColor
                : textColor),
          fillOpacity: 1,
        },
      ];
      text.resize(Math.max(1, row.width - textInsetX * 2), height);
      text.x = row.x + textInsetX;
      text.y = row.y;
      setMetadata(text, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
      setMetadata(text, POPUP_CONTENT_FOR, sourceNodeId);
      setMetadata(text, POPUP_OPTION_ROLE, 'label');
      setMetadata(text, 'popup-option-value', option.value);
      if (text.layoutChild) text.layoutChild.absolute = true;
      if (!existingText) row.appendChild(text);
      recordPopupLayoutGeometry(text);
      for (const extra of labels.slice(1)) extra.remove();
    } else {
      for (const label of labels) label.remove();
    }
    cursor +=
      slotHeight + (popupMenu && index < options.length - 1 ? rowSpacing : 0);
  }
  for (const extra of existingRows.slice(options.length)) extra.remove();
  // Direct popup boards use their authored geometry. Trigger overlays use the
  // computed box and are the only boards that need an explicit overlay marker.
  if (overlay) setMetadata(parent, ZUI_METADATA_EXPLICIT_OVERLAY, 'true');
}

function popupLayoutOptions(
  node: ZuiNode,
  document: ZuiDocument,
): PopupLayoutOptions {
  const props = node.props ?? {};
  return {
    rowHeight:
      resolveDesignNumber(
        document,
        props['item_extent'] ?? props['row_height'],
      ) ?? 28,
    paddingTop: resolveDesignNumber(document, props['layout_padding_top']) ?? 8,
    paddingRight:
      resolveDesignNumber(document, props['layout_padding_right']) ?? 8,
    paddingBottom:
      resolveDesignNumber(document, props['layout_padding_bottom']) ?? 8,
    paddingLeft:
      resolveDesignNumber(document, props['layout_padding_left']) ?? 8,
    offsetX: resolveDesignNumber(document, props['popup_offset_x']) ?? 0,
    offsetY: resolveDesignNumber(document, props['popup_offset_y']) ?? 4,
    width:
      resolveDesignNumber(document, props['popup_anchor_width']) ?? undefined,
  };
}

function popupMenuLayoutOptions(
  node: ZuiNode,
  document: ZuiDocument,
): PopupLayoutOptions {
  const props = node.props ?? {};
  const denseHeight =
    resolveDesignNumber(document, '$editor.controls.dense_height') ?? 28;
  const smallGap =
    resolveDesignNumber(document, '$editor.density.gap.small') ?? 4;
  const borderWidth =
    resolveDesignNumber(document, '$editor.controls.border_width') ?? 1;
  return {
    rowHeight: Math.max(denseHeight - smallGap, borderWidth),
    rowSpacing: resolveDesignNumber(document, props['layout_spacing']) ?? 0,
    paddingTop:
      resolveDesignNumber(document, props['layout_padding_top']) ?? 0,
    paddingRight:
      resolveDesignNumber(document, props['layout_padding_right']) ?? 0,
    paddingBottom:
      resolveDesignNumber(document, props['layout_padding_bottom']) ?? 0,
    paddingLeft:
      resolveDesignNumber(document, props['layout_padding_left']) ?? 0,
    offsetX: resolveDesignNumber(document, props['popup_offset_x']) ?? 0,
    offsetY: resolveDesignNumber(document, props['popup_offset_y']) ?? 4,
    textInsetX:
      resolveDesignNumber(document, '$editor.density.gap.medium') ?? 8,
  };
}

function measurePopupMenuWidth(
  node: ZuiNode,
  document: ZuiDocument,
  projection: ProjectedZuiNode | undefined,
  options: readonly PopupOption[],
  layout: PopupLayoutOptions,
): number {
  const style = resolvePenpotTextStyle(document, node, projection?.text ?? null);
  let textWidth = 0;
  for (const option of options) {
    if (option.separator || !option.label) continue;
    const text = penpot.createText(option.label);
    if (!text) throw new Error('Unable to measure Penpot popup menu text');
    try {
      text.characters = option.label;
      text.growType = 'auto-width';
      applyPenpotTextStyle(text, style);
      const measured = text.textBounds.width;
      if (!Number.isFinite(measured) || measured < 0)
        throw new Error('Penpot popup menu text measurement is unavailable');
      textWidth = Math.max(textWidth, measured);
    } finally {
      text.remove();
    }
  }
  return (
    textWidth +
    clampPositive(layout.textInsetX ?? 8, 0) * 2 +
    clampPositive(layout.paddingLeft ?? 0, 0) +
    clampPositive(layout.paddingRight ?? 0, 0)
  );
}

export function popupMenuAnchorNodeId(
  node: ZuiNode,
  document: ZuiDocument,
): string | null {
  const widget = node['widget'];
  const anchor = isRecord(widget) ? widget['popup_anchor'] : undefined;
  if (!isRecord(anchor) || anchor['kind'] !== 'control') return null;
  const controlId = anchor['control_id'];
  if (typeof controlId !== 'string' || !controlId) return null;
  const matches = Object.entries(document.nodes ?? {}).filter(
    ([, candidate]) => candidate.control_id === controlId,
  );
  if (matches.length !== 1) return null;
  const [nodeId, control] = matches[0]!;
  if (
    control.state?.['disabled'] === true ||
    control.props?.['disabled'] === true ||
    control.props?.['enabled'] === false ||
    control.props?.['visible'] === false
  )
    return null;
  return nodeId;
}

function popupMenuAnchorBoard(
  node: ZuiNode,
  document: ZuiDocument,
  semanticBoards: ReadonlyMap<string, Board>,
): Board | undefined {
  const nodeId = popupMenuAnchorNodeId(node, document);
  if (!nodeId) return undefined;
  const board = semanticBoards.get(nodeId);
  return board && !board.hidden ? board : undefined;
}

function popupRect(board: Board): PopupRect {
  return { x: board.x, y: board.y, width: board.width, height: board.height };
}

function popupOptionSource(props: ZuiTable): unknown[] {
  for (const key of ['menu_items', 'options', 'items']) {
    const value = props[key];
    if (Array.isArray(value)) return value;
  }
  return [];
}

function parsePopupOption(value: unknown): PopupOption | null {
  if (typeof value === 'string') {
    const raw = value.trim();
    if (!raw) return null;
    if (raw === '---' || raw.toLowerCase() === 'separator')
      return emptyPopupOption(true);
    const parts = raw.split('|').map((part) => part.trim());
    const labelPart = parts[0] ?? '';
    let optionValue = labelPart;
    let label = labelPart;
    const flags = new Set<string>();
    for (const part of (parts[1] ?? '').split(',')) {
      const token = part.trim();
      if (!token) continue;
      const equals = token.indexOf('=');
      if (equals > 0) {
        const key = token.slice(0, equals).trim().toLowerCase();
        const item = token.slice(equals + 1).trim();
        if (key === 'label' || key === 'text' || key === 'title') label = item;
        else if (key === 'value' || key === 'action') optionValue = item;
        continue;
      }
      flags.add(token.toLowerCase());
    }
    return {
      value: optionValue,
      label,
      selected: flags.has('selected') || flags.has('checked'),
      disabled: flags.has('disabled'),
      focused: flags.has('focused'),
      hovered: flags.has('hovered') || flags.has('pressed'),
      separator: false,
      ...(flags.has('pressed') ? { pressed: true } : {}),
      ...(flags.has('loading') ? { loading: true } : {}),
      ...(flags.has('danger') ? { danger: true } : {}),
    };
  }
  if (!isRecord(value)) return null;
  const optionValue = stringValue(
    value['value'] ?? value['id'] ?? value['key'],
  );
  const label = stringValue(
    value['label'] ?? value['text'] ?? value['name'] ?? optionValue,
  );
  if (!optionValue && !label) return null;
  return {
    value: optionValue ?? label ?? '',
    label: label ?? optionValue ?? '',
    selected: value['selected'] === true || value['checked'] === true,
    disabled: value['disabled'] === true,
    focused: value['focused'] === true,
    hovered: value['hovered'] === true || value['pressed'] === true,
    separator: value['separator'] === true,
    ...(value['pressed'] === true ? { pressed: true } : {}),
    ...(value['loading'] === true ? { loading: true } : {}),
    ...(value['danger'] === true ? { danger: true } : {}),
  };
}

function emptyPopupOption(separator: boolean): PopupOption {
  return {
    value: '',
    label: '',
    selected: false,
    disabled: false,
    focused: false,
    hovered: false,
    separator,
  };
}

function semanticBoardIndex(assetBoard: Board): Map<string, Board> {
  const result = new Map<string, Board>();
  const visit = (shape: Shape): void => {
    if (shape.type === 'board') {
      if (metadata(shape, ZUI_METADATA_ROLE) === ZUI_ROLE_NODE) {
        const nodeId = metadata(shape, ZUI_METADATA_NODE_ID);
        if (nodeId) result.set(nodeId, shape);
      }
      for (const child of shape.children) visit(child);
    }
  };
  visit(assetBoard);
  return result;
}

function removePopupContent(parent: Board | undefined, nodeId: string): void {
  if (!parent) return;
  for (const child of [...parent.children]) {
    if (metadata(child, POPUP_CONTENT_FOR) === nodeId) child.remove();
  }
}

function valueSet(value: unknown): Set<string> {
  const values = Array.isArray(value) ? value : [value];
  return new Set(
    values.filter((item): item is string => typeof item === 'string'),
  );
}

function finiteInteger(value: unknown): number | null {
  return typeof value === 'number' && Number.isInteger(value) ? value : null;
}

function stringValue(value: unknown): string | null {
  return typeof value === 'string' && value.trim() !== '' ? value.trim() : null;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function popupSourceProperty(node: ZuiNode): string {
  const props = node.props ?? {};
  if (Array.isArray(props['menu_items'])) return 'menu_items';
  if (Array.isArray(props['options'])) return 'options';
  if (Array.isArray(props['items'])) return 'items';
  return 'none';
}

function metadata(shape: Shape, key: string): string {
  return shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, key) || '';
}

function setMetadata(shape: Shape, key: string, value: string): void {
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, key, value);
}

function clampPositive(value: number, minimum: number): number {
  return Number.isFinite(value) ? Math.max(minimum, value) : minimum;
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value));
}

/** This writer is private to the source-controlled popup renderer. */
function recordPopupLayoutGeometry(shape: Shape): void {
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    'popup-layout-geometry',
    JSON.stringify(popupLayoutGeometry(shape)),
  );
}
