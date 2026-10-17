import type { Shape } from '@penpot/plugin-types';
import {
  ZUI_METADATA_NAMESPACE,
  ZUI_METADATA_ROLE,
  ZUI_ROLE_ASSET,
  ZUI_ROLE_AUXILIARY,
} from './metadata';

export const PATH_NUMBER = /[-+]?(?:\d+\.?\d*|\.\d+)(?:[eE][-+]?\d+)?/g;

export function isGeneratedPath(shape: Shape): boolean {
  if (shape.type !== 'path') return false;
  let ancestor: Shape | null = shape;
  while (ancestor) {
    const role = ancestor.getSharedPluginData(
      ZUI_METADATA_NAMESPACE,
      ZUI_METADATA_ROLE,
    );
    if (role === ZUI_ROLE_ASSET) break;
    if (role === ZUI_ROLE_AUXILIARY) return true;
    ancestor = ancestor.parent;
  }
  return false;
}

export function parseIconPathContent(content: string):
  | { skeleton: string; coordinates: number[] }
  | undefined {
  const skeleton = content.replace(PATH_NUMBER, '#');
  if (!skeleton.trim().startsWith('M') || /[^MLCZ#,\s]/.test(skeleton))
    return undefined;
  for (const [, command, parameters] of content.matchAll(
    /([MLCZ])([^MLCZ]*)/g,
  )) {
    if (/^\s*,|,\s*$|,\s*,/.test(parameters!)) return undefined;
    const count = [...parameters!.matchAll(PATH_NUMBER)].length;
    if (count !== (command === 'C' ? 6 : command === 'Z' ? 0 : 2))
      return undefined;
  }
  const coordinates = [...content.matchAll(PATH_NUMBER)].map(([value]) =>
    Number(value),
  );
  if (!coordinates.every(Number.isFinite)) return undefined;
  return { skeleton, coordinates };
}

export function normalizedIconPathContent(
  shape: Shape & { content: string },
): string | undefined {
  const content = shape.content;
  if (
    !isGeneratedPath(shape) ||
    !Number.isFinite(shape.x) ||
    !Number.isFinite(shape.y)
  )
    return undefined;
  const parsed = parseIconPathContent(content);
  if (!parsed) return undefined;
  // World path serialization and shape bounds can drift independently.
  // Align only the first point's offset to the geometry guard's 0.01 precision;
  // keep every internal vector unrounded. Unsupported syntax remains exact.
  const firstX = parsed.coordinates[0]!;
  const firstY = parsed.coordinates[1]!;
  const anchorX = Math.round((firstX - shape.x) * 100) / 100;
  const anchorY = Math.round((firstY - shape.y) * 100) / 100;
  const coordinates = parsed.coordinates.map((value, index) =>
    index % 2 === 0 ? value - firstX + anchorX : value - firstY + anchorY,
  );
  if (
    !Number.isFinite(anchorX) ||
    !Number.isFinite(anchorY) ||
    !coordinates.every(Number.isFinite)
  )
    return undefined;
  let coordinate = 0;
  return content.replace(PATH_NUMBER, () => String(coordinates[coordinate++]!));
}

export function guardedPathContent(shape: Shape & { content: string }): string {
  return normalizedIconPathContent(shape) ?? shape.content;
}
