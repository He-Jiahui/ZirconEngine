import type { Board, Text } from '@penpot/plugin-types';
import { segmentedGeometry } from './bridge/zui-segmented-geometry';
import type { SegmentedProjection } from './bridge/zui-segmented-projection';
import { refreshControlParts, layoutControlText } from './penpot-control-parts';

export function refreshSegmentedDecoration(
  board: Board,
  segmented?: SegmentedProjection,
): boolean {
  if (!segmented) return false;
  refreshControlParts(
    board,
    'segmented',
    segmentedGeometry(segmented, board.width, board.height).parts,
  );
  return true;
}

export function layoutSegmentedText(
  board: Board,
  text: Text,
  segmented: SegmentedProjection,
): void {
  layoutControlText(
    board,
    text,
    segmentedGeometry(segmented, board.width, board.height).texts,
  );
}
