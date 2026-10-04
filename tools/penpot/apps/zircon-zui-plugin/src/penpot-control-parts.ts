import type { Board, Text } from '@penpot/plugin-types';
import type { ControlGeometry } from './bridge/zui-control-geometry';
import { TEXT_FRAGMENT_KEY } from './penpot-text-capture';
import {
  guardControlParts,
  guardControlTextGeometry,
} from './penpot-control-guard';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_AUXILIARY,
} from './metadata';

export function refreshControlParts(
  board: Board,
  kind: string,
  parts: ControlGeometry['parts'],
): void {
  const metadataKey = `${kind}-part`;
  const existing = new Map(
    board.children
      .map(
        (child) =>
          [
            child.getSharedPluginData(ZUI_METADATA_NAMESPACE, metadataKey),
            child,
          ] as const,
      )
      .filter(([key]) => Boolean(key)),
  );
  for (const [key, part] of Object.entries(parts)) {
    let shape = existing.get(key);
    if (!shape) {
      shape = penpot.createBoard();
      shape.name = `${kind} ${key}`;
      shape.setSharedPluginData(
        ZUI_METADATA_NAMESPACE,
        ZUI_METADATA_ROLE,
        ZUI_ROLE_AUXILIARY,
      );
      shape.setSharedPluginData(ZUI_METADATA_NAMESPACE, metadataKey, key);
      board.appendChild(shape);
      if (shape.layoutChild) shape.layoutChild.absolute = true;
    }
    existing.delete(key);
    shape.hidden = false;
    shape.resize(part.width, part.height);
    shape.x = board.x + part.x;
    shape.y = board.y + part.y;
    const fill = splitColor(part.fill);
    shape.fills = [{ fillColor: fill.color, fillOpacity: fill.opacity }];
    shape.strokes = part.stroke
      ? [
          {
            strokeColor: splitColor(part.stroke).color,
            strokeOpacity: splitColor(part.stroke).opacity,
            strokeWidth: part.strokeWidth ?? 0,
            strokeStyle: 'solid',
            strokeAlignment: 'inner',
          },
        ]
      : [];
    if (shape.type === 'board') shape.borderRadius = part.radius;
  }
  // Only native frame-size, option and state rules may remove these projection parts.
  for (const shape of existing.values()) shape.remove();
  guardControlParts(
    board,
    board.children.filter((child) =>
      child.getSharedPluginData(ZUI_METADATA_NAMESPACE, metadataKey),
    ),
  );
}

export function layoutControlText(
  board: Board,
  text: Text,
  texts: ControlGeometry['texts'],
): void {
  const key =
    text.getSharedPluginData(ZUI_METADATA_NAMESPACE, TEXT_FRAGMENT_KEY) ||
    'primary';
  const geometry = texts[key];
  text.hidden = !geometry;
  if (geometry) {
    text.resize(geometry.width, geometry.height);
    text.x = board.x + geometry.x;
    text.y = board.y + geometry.y;
    text.verticalAlign = 'top';
  }
  guardControlTextGeometry(board, text);
}

function splitColor(value: string): { color: string; opacity: number } {
  return {
    color: value.slice(0, 7),
    opacity: value.length === 9 ? parseInt(value.slice(7), 16) / 255 : 1,
  };
}
