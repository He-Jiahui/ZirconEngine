import type { Board, Shape, Text } from '@penpot/plugin-types';
import {
  tableGeometry,
  TABLE_COLUMN_SAMPLES,
} from './bridge/zui-table-geometry';
import type { TableProjection } from './bridge/zui-table-projection';
import { applyPenpotTextStyle } from './penpot-text-style';
import { updateControlTextGeometryGuard } from './penpot-control-guard';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_AUXILIARY,
} from './metadata';

export const TABLE_MEASUREMENT = 'table-measurement';
export const TABLE_MEASUREMENT_REQUIRED = 'table-measurement-required';

/** Return the labels that need native font measurements for a table. */
export function tableMeasurementLabels(table: TableProjection): string[] {
  return [
    ...new Set([
      ...TABLE_COLUMN_SAMPLES,
      ...table.cells.map((cell) => cell.value),
    ]),
  ];
}

/** Find the asset board that owns design-only measurement text. */
export function tableMeasurementOwner(board: Board): Board {
  let current: Shape | null = board;
  while (current) {
    if (
      current.type === 'board' &&
      current.getSharedPluginData(ZUI_METADATA_NAMESPACE, ZUI_METADATA_ROLE) ===
        ZUI_ROLE_ASSET
    )
      return current;
    current = current.parent;
  }
  // Unit-test doubles and detached component probes do not always provide an
  // ancestor chain; retain the local-owner behavior for those callers.
  return board;
}

/** Read the pooled auxiliary measurement shapes for an asset. */
export function tableMeasurementShapes(board: Board): Text[] {
  const owner = tableMeasurementOwner(board);
  return owner.children.filter(
    (shape): shape is Text =>
      (shape.type === 'text' || !('type' in shape)) &&
      Boolean(
        shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, TABLE_MEASUREMENT),
      ),
  );
}

export function createTableMeasurements(
  board: Board,
  table?: TableProjection,
): void {
  if (!table) return;
  const owner = tableMeasurementOwner(board);
  const labels = tableMeasurementLabels(table);
  const existing = new Set(
    tableMeasurementShapes(owner).map((shape) =>
      shape.getSharedPluginData(ZUI_METADATA_NAMESPACE, TABLE_MEASUREMENT),
    ),
  );
  for (const label of labels) {
    if (existing.has(label)) continue;
    const text = penpot.createText(label);
    if (!text) throw new Error('Unable to measure native table text');
    applyPenpotTextStyle(text, {
      family: table.fontFamily,
      id: table.fontFamily,
      weight: '400',
      size: table.fontSize,
      lineHeight: table.lineHeight,
    });
    text.growType = 'auto-width';
    text.name = `Table font measurement: ${label}`;
    // Nonvisual measurement shapes let Penpot shape the exact selected font.
    text.opacity = 0;
    text.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_ROLE,
      ZUI_ROLE_AUXILIARY,
    );
    text.setSharedPluginData(ZUI_METADATA_NAMESPACE, TABLE_MEASUREMENT, label);
    owner.appendChild(text);
    if (text.layoutChild) text.layoutChild.absolute = true;
    text.x = owner.x;
    text.y = owner.y;
    existing.add(label);
  }
}

export function layoutTableText(
  board: Board,
  text: Text,
  table: TableProjection,
): void {
  // Native table rows pin cell text to the row's top edge. Set this before
  // measurement lookup so the style guard is stable even on the first pass.
  text.verticalAlign = 'top';
  // Retained cells use allocated column slots; numeric cells use the measured
  // glyph width. Keep both fixed after shaping so later auto-width updates
  // cannot invalidate the no-edit geometry baseline.
  text.growType = 'fixed';
  const measurements = new Map(
    tableMeasurementShapes(board).map(
      (child) =>
        [
          child.getSharedPluginData(ZUI_METADATA_NAMESPACE, TABLE_MEASUREMENT),
          child.width,
        ] as const,
    ),
  );
  const geometry = tableGeometry(table, board.width, board.height, (label) => {
    const width = measurements.get(label);
    if (width === undefined || !Number.isFinite(width) || width <= 0)
      throw new Error(`Table font measurement unavailable: ${label}`);
    return width;
  });
  const key =
    text.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'text-fragment') ||
    'primary';
  const cell = table.cells.find((candidate) => candidate.key === key);
  const target = geometry.texts[key];
  if (!target) {
    // The import preview may be smaller than the authored editor shell. Keep
    // the first cell editable and visible until the review viewport supplies
    // a real column allocation; hide only explicitly dropped columns. The
    // previous visible geometry needs a new guard after a visibility change.
    text.hidden = cell ? cell.column !== 0 : true;
    text.growType = 'fixed';
    updateControlTextGeometryGuard(board, text);
    return;
  }
  if (cell && target) {
    text.hidden = false;
    text.resize(target.width, target.height);
    text.x = board.x + target.x;
    text.y = board.y + target.y;
    updateControlTextGeometryGuard(board, text);
  }
}
