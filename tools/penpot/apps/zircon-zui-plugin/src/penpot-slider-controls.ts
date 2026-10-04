import type { Board, Text } from '@penpot/plugin-types';
import { sliderGeometry } from './bridge/zui-slider-geometry';
import type { SliderProjection } from './bridge/zui-slider-projection';
import { refreshControlParts, layoutControlText } from './penpot-control-parts';

export function createSliderDecoration(
  board: Board,
  slider?: SliderProjection,
): boolean {
  if (!slider) return false;
  refreshSliderDecoration(board, slider);
  return true;
}

export function refreshSliderDecoration(
  board: Board,
  slider?: SliderProjection,
): void {
  if (!slider) return;
  refreshControlParts(
    board,
    'slider',
    sliderGeometry(slider, board.width, board.height).parts,
  );
}

export function layoutSliderText(
  board: Board,
  text: Text,
  slider: SliderProjection,
): void {
  layoutControlText(
    board,
    text,
    sliderGeometry(slider, board.width, board.height).texts,
  );
}
