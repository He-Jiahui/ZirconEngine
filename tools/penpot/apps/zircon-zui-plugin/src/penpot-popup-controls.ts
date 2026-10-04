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
  separator: boolean;
}

export interface PopupRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface PopupLayout extends PopupRect {
  opensAbove: boolean;
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
        hovered: parsed.hovered || hoveredValues.has(parsed.value),
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
      removePopupContent(board, shape.nodeId);
      if (
        board &&
        popupIsOpen(node) &&
        options.length > 0 &&
        shape.children.length === 0 &&
        !board.hidden
      ) {
        const overlay = existing.get(shape.nodeId) ?? penpot.createBoard();
        if (!overlay) throw new Error('Unable to create Penpot popup overlay');
        if (!existing.has(shape.nodeId)) assetBoard.appendChild(overlay);
        const layout: PopupLayout = {
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
        );
        renderPopupRows(
          overlay,
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
): void {
  board.name = name;
  board.resize(layout.width, layout.height);
  board.x = layout.x;
  board.y = layout.y;
  board.hidden = false;
  board.clipContent = true;
  board.fills = [
    {
      fillColor:
        resolveDesignColor(document, node.props?.['popup_background_color']) ??
        PENPOT_PALETTE.surfaceRaised,
      fillOpacity: 1,
    },
  ];
  board.strokes = [
    {
      strokeColor:
        resolveDesignColor(document, node.props?.['popup_border_color']) ??
        PENPOT_PALETTE.borderStrong,
      strokeOpacity: 1,
      strokeWidth: 1,
      strokeStyle: 'solid',
      strokeAlignment: 'inner',
    },
  ];
  board.borderRadius = Math.min(
    8,
    Math.max(
      0,
      resolveDesignNumber(document, node.props?.['popup_corner_radius']) ?? 4,
    ),
  );
  setMetadata(board, ZUI_METADATA_ROLE, ZUI_ROLE_AUXILIARY);
  setMetadata(board, ZUI_METADATA_EXPLICIT_OVERLAY, 'true');
  setMetadata(board, POPUP_OVERLAY_FOR, sourceNodeId);
  setMetadata(board, POPUP_SOURCE_PROPERTY, popupSourceProperty(node));
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
    const height = option.separator ? 1 : rowHeight;
    const existingRow = existingRows[index];
    const row = existingRow ?? penpot.createBoard();
    if (!row) throw new Error('Unable to create Penpot popup option');
    row.name = option.separator
      ? `Penpot · ${sourceNodeId} separator`
      : `Penpot · ${sourceNodeId} option · ${option.label}`;
    row.resize(Math.max(1, parent.width - paddingLeft - paddingRight), height);
    row.x = parent.x + paddingLeft;
    row.y = cursor;
    row.borderRadius = option.separator ? 0 : 2;
    row.fills = option.separator
      ? [{ fillColor: PENPOT_PALETTE.border, fillOpacity: 1 }]
      : option.selected
        ? [{ fillColor: selectedColor, fillOpacity: 1 }]
        : option.hovered || option.focused
          ? [{ fillColor: hoveredColor, fillOpacity: 1 }]
          : [];
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
        weight: option.selected ? '600' : style.weight,
      });
      text.align = 'left';
      text.verticalAlign = 'center';
      text.fills = [
        {
          fillColor: option.disabled
            ? disabledColor
            : option.selected
              ? accentColor
              : textColor,
          fillOpacity: 1,
        },
      ];
      text.resize(Math.max(1, row.width - 16), height);
      text.x = row.x + 8;
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
    cursor += height;
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
    const optionValue = parts.shift() ?? '';
    let label = optionValue;
    const flags = new Set<string>();
    for (const part of parts.join('|').split(',')) {
      const token = part.trim();
      if (!token) continue;
      const equals = token.indexOf('=');
      if (equals > 0) {
        const key = token.slice(0, equals).trim().toLowerCase();
        const item = token.slice(equals + 1).trim();
        if (key === 'label' || key === 'text' || key === 'title') label = item;
        else if (key === 'value') flags.add(`value:${item}`);
        continue;
      }
      flags.add(token.toLowerCase());
    }
    const explicitValue = [...flags].find((flag) => flag.startsWith('value:'));
    return {
      value: explicitValue ? explicitValue.slice('value:'.length) : optionValue,
      label,
      selected: flags.has('selected') || flags.has('checked'),
      disabled: flags.has('disabled'),
      focused: flags.has('focused'),
      hovered: flags.has('hovered') || flags.has('pressed'),
      separator: false,
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
