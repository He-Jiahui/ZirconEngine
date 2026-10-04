import type { Board } from '@penpot/plugin-types';
import {
  dividerGeometry,
  type DividerProjection,
} from './bridge/zui-divider-projection';
import { refreshControlParts } from './penpot-control-parts';

export function refreshDividerDecoration(
  board: Board,
  divider: DividerProjection | undefined,
): boolean {
  if (!divider) return false;
  const geometry = dividerGeometry(divider, board.width, board.height);
  refreshControlParts(
    board,
    'divider',
    geometry
      ? {
          line: { ...geometry, fill: divider.color, radius: 0 },
        }
      : {},
  );
  return true;
}
