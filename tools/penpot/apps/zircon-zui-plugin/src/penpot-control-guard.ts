import type { Board, Shape } from '@penpot/plugin-types';
import { ZUI_METADATA_NAMESPACE } from './metadata';

const GEOMETRY_GUARD = 'control-text-geometry';
const PART_GUARD = 'control-part-paint';
const PARTS_GUARD = 'control-part-list';

function geometry(board: Board, shape: Shape): unknown {
  const round = (value: number) => Math.round(value * 100) / 100;
  return {
    x: round(shape.x - board.x),
    y: round(shape.y - board.y),
    width: round(shape.width),
    height: round(shape.height),
    hidden: shape.hidden,
  };
}

function partPaint(board: Board, shape: Shape): unknown {
  return {
    geometry: geometry(board, shape),
    fills: shape.fills,
    strokes: shape.strokes,
    radius: shape.type === 'board' ? shape.borderRadius : null,
    opacity: shape.opacity,
  };
}

export function guardControlTextGeometry(board: Board, shape: Shape): void {
  shape.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    GEOMETRY_GUARD,
    JSON.stringify(geometry(board, shape)),
  );
}

export function updateControlTextGeometryGuard(
  board: Board,
  shape: Shape,
): void {
  guardControlTextGeometry(board, shape);
}

export function assertControlTextGeometry(board: Board, shape: Shape): void {
  const expected = shape.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    GEOMETRY_GUARD,
  );
  if (expected && expected !== JSON.stringify(geometry(board, shape)))
    throw new Error(
      `Control text ${shape.name} changed its native geometry or visibility; edit the source layout metrics. Expected ${expected}, found ${JSON.stringify(geometry(board, shape))}.`,
    );
}

export function guardControlParts(board: Board, parts: Shape[]): void {
  board.setSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    PARTS_GUARD,
    JSON.stringify(parts.map((part) => part.id).sort()),
  );
  for (const part of parts)
    part.setSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      PART_GUARD,
      JSON.stringify(partPaint(board, part)),
    );
}

export function assertControlParts(board: Board): void {
  const expected = board.getSharedPluginData(
    ZUI_METADATA_NAMESPACE,
    PARTS_GUARD,
  );
  if (!expected) return;
  const parts = board.children.filter((child) =>
    child.getSharedPluginData(ZUI_METADATA_NAMESPACE, PART_GUARD),
  );
  if (JSON.stringify(parts.map((part) => part.id).sort()) !== expected)
    throw new Error(`Control ${board.name} lost a native painter part.`);
  for (const part of parts)
    if (
      part.getSharedPluginData(ZUI_METADATA_NAMESPACE, PART_GUARD) !==
      JSON.stringify(partPaint(board, part))
    )
      throw new Error(
        `Control part ${part.name} changed without a source-property mapping; edit its .zui painter properties.`,
      );
}
