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

export function guardedPathContent(shape: Shape & { content: string }): string {
  const content = shape.content;
  if (
    !isGeneratedPath(shape) ||
    !Number.isFinite(shape.x) ||
    !Number.isFinite(shape.y)
  )
    return content;
  // Penpot PathData.toString emits absolute M/L/C/Z world coordinates.
  // Compare bridge-owned icons in their shape space; geometry still guards
  // movement relative to the parent. Unsupported syntax remains exact.
  const skeleton = content.replace(PATH_NUMBER, '#');
  if (!skeleton.trim().startsWith('M') || /[^MLCZ#,\s]/.test(skeleton))
    return content;
  for (const [, command, parameters] of content.matchAll(
    /([MLCZ])([^MLCZ]*)/g,
  )) {
    const count = [...parameters!.matchAll(PATH_NUMBER)].length;
    if (count !== (command === 'C' ? 6 : command === 'Z' ? 0 : 2))
      return content;
  }
  let coordinate = 0;
  return content.replace(PATH_NUMBER, (value) =>
    String(Number(value) - (coordinate++ % 2 === 0 ? shape.x : shape.y)),
  );
}
