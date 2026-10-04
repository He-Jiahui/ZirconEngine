import type { Board, Shape } from '@penpot/plugin-types';
import icons from 'feather-icons/dist/icons.json';
import {
  inputControlGeometry,
  type InputControlProjection,
} from './bridge/zui-input-projection';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';

export function createInputDecoration(
  board: Board,
  control?: InputControlProjection,
): boolean {
  if (!control) return false;
  if (control.kind !== 'dropdown') {
    const mark = penpot.createBoard();
    mark.name = `${control.kind} mark`;
    mark.fills = [{ fillColor: control.fill, fillOpacity: 1 }];
    mark.strokes = [
      {
        strokeColor: control.border,
        strokeOpacity: 1,
        strokeWidth: control.borderWidth,
        strokeStyle: 'solid',
        strokeAlignment: 'inner',
      },
    ];
    mark.borderRadius =
      control.kind === 'checkbox'
        ? control.radius
        : (control.track?.height ?? control.markSize) / 2;
    append(board, mark, 'mark');
    if (control.kind === 'checkbox' && control.active) {
      // Match the runtime checkbox's three filled tick segments.
      for (let index = 0; index < 3; index++) {
        const segment = penpot.createBoard();
        segment.name = 'Checkbox tick';
        segment.fills = [{ fillColor: control.accent, fillOpacity: 1 }];
        segment.strokes = [];
        segment.borderRadius = control.borderWidth;
        append(board, segment, `tick-${index}`);
      }
    }
    if (control.secondary && (control.kind === 'toggle' || control.active)) {
      const secondary = penpot.createBoard();
      secondary.name = control.kind === 'radio' ? 'Radio dot' : 'Toggle thumb';
      secondary.fills = [
        { fillColor: control.secondary.color, fillOpacity: 1 },
      ];
      secondary.strokes = [];
      secondary.borderRadius = control.secondary.size / 2;
      append(board, secondary, 'secondary');
    }
  } else {
    const name = control.active ? 'chevron-up' : 'chevron-down';
    const caret = penpot.createShapeFromSvg(
      `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="${control.accent}" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${icons[name]}</svg>`,
    );
    if (!caret) throw new Error('Penpot could not create the dropdown caret.');
    caret.name = 'Dropdown caret';
    append(board, caret, 'mark');
  }
  refreshInputDecoration(board, control);
  return true;
}

export function refreshInputDecoration(
  board: Board,
  control?: InputControlProjection,
): void {
  if (!control) return;
  const { mark, secondary } = inputControlGeometry(
    control,
    board.width,
    board.height,
  );
  const segments = [
    [3, 7, 3, 3],
    [5, 9, 3, 3],
    [8, 4, 3, 8],
  ];
  for (const child of board.children) {
    const part = child.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      'input-part',
    );
    if (!part) continue;
    if (part === 'secondary' && secondary) {
      child.resize(secondary.width, secondary.height);
      child.x = board.x + secondary.x;
      child.y = board.y + secondary.y;
      continue;
    }
    const segment = part.startsWith('tick-')
      ? segments[Number(part.slice(5))]
      : undefined;
    const unit = mark.width / 16;
    child.resize(
      segment ? segment[2] * unit : mark.width,
      segment ? segment[3] * unit : mark.height,
    );
    child.x = board.x + mark.x + (segment ? segment[0] * unit : 0);
    child.y = board.y + mark.y + (segment ? segment[1] * unit : 0);
  }
}

function append(board: Board, shape: Shape, part: string): void {
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    ZUI_METADATA_ROLE,
    ZUI_ROLE_AUXILIARY,
  );
  shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, 'input-part', part);
  board.appendChild(shape);
  if (shape.layoutChild) shape.layoutChild.absolute = true;
}
