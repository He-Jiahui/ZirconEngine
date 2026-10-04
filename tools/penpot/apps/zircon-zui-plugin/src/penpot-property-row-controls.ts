import type { Board, Text } from '@penpot/plugin-types';
import { propertyRowGeometry } from './bridge/zui-property-row-geometry';
import type { PropertyRowProjection } from './bridge/zui-property-row-projection';
import { refreshControlParts, layoutControlText } from './penpot-control-parts';

export function refreshPropertyRowDecoration(
  board: Board,
  row?: PropertyRowProjection,
): boolean {
  if (!row) return false;
  const geometry = propertyRowGeometry(row, board.width, board.height);
  refreshControlParts(board, 'property-row', geometry.parts);
  return true;
}

export function layoutPropertyRowText(
  board: Board,
  text: Text,
  row: PropertyRowProjection,
): void {
  layoutControlText(
    board,
    text,
    propertyRowGeometry(row, board.width, board.height).texts,
  );
}
