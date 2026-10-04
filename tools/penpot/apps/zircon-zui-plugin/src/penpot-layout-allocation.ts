import type { Board } from '@penpot/plugin-types';
import type { CapturedGeometry } from './penpot-capture-validation';
import { ZUI_METADATA_NAMESPACE } from './metadata';
const KEY = 'native-layout-allocation';
const EPSILON = 0.001;
interface Allocation {
  kind?: 'flow' | 'anchored';
  parent: string | null;
  width: number;
  height: number;
  authoredWidth?: number;
  authoredHeight?: number;
}
function allocation(board: Board): Allocation | null {
  const encoded = board.getSharedPluginData(ZUI_METADATA_NAMESPACE, KEY);
  if (!encoded) return null;
  const value = JSON.parse(encoded) as Allocation;
  if (
    !value ||
    typeof value !== 'object' ||
    Array.isArray(value) ||
    !Number.isFinite(value.width) ||
    !Number.isFinite(value.height) ||
    value.width < 0 ||
    value.height < 0 ||
    (value.kind !== undefined &&
      value.kind !== 'flow' &&
      value.kind !== 'anchored') ||
    [value.authoredWidth, value.authoredHeight].some(
      (v) => v !== undefined && (!Number.isFinite(v) || v < 0),
    )
  )
    throw new Error('Invalid native layout allocation metadata');
  return value;
}
/** Keep engine allocation separate from a designer's fixed-size edits. */
export function allocateNativeLayoutSize(
  board: Board,
  width: number,
  height: number,
  kind: 'flow' | 'anchored' = 'flow',
): void {
  if (
    !Number.isFinite(width) ||
    !Number.isFinite(height) ||
    width < 0 ||
    height < 0
  )
    throw new Error('Native layout allocation must be finite and nonnegative');
  if (kind !== 'flow' && kind !== 'anchored')
    throw new Error('Unknown native layout allocation owner');
  const previous = allocation(board);
  const parent = board.parent?.id ?? null;
  // Manual dimensions belong to the parent and layout that captured them.
  const authored =
    previous && previous.parent === parent && (previous.kind ?? 'flow') === kind
      ? previous
      : null;
  const next: Allocation = {
    ...(kind === 'anchored' ? { kind } : {}),
    width,
    height,
    parent,
    ...(authored?.authoredWidth !== undefined
      ? { authoredWidth: authored.authoredWidth }
      : {}),
    ...(authored?.authoredHeight !== undefined
      ? { authoredHeight: authored.authoredHeight }
      : {}),
  };
  const encoded = JSON.stringify(next);
  if (encoded !== board.getSharedPluginData(ZUI_METADATA_NAMESPACE, KEY))
    board.setSharedPluginData(ZUI_METADATA_NAMESPACE, KEY, encoded);
  if (
    Math.abs(board.width - width) > EPSILON ||
    Math.abs(board.height - height) > EPSILON
  )
    board.resize(width, height);
}
/** Preserve authored dimensions for recorded flow or Free/Overlay allocation. */
export function allocatedAuthoredGeometry(
  board: Board,
  current: CapturedGeometry,
  baseline?: CapturedGeometry,
  autoLayoutChild = true,
): CapturedGeometry {
  const allocated = allocation(board);
  if (
    !baseline ||
    !allocated ||
    (allocated.kind === 'anchored' ? autoLayoutChild : !autoLayoutChild) ||
    allocated.parent !== (board.parent?.id ?? null)
  )
    return current;
  const widthChanged = Math.abs(board.width - allocated.width) > EPSILON;
  const heightChanged = Math.abs(board.height - allocated.height) > EPSILON;
  // This function runs only during guarded authored capture. Native refreshes
  // carry these values without inferring edits from intermediate geometry.
  const next: Allocation = {
    ...allocated,
    ...(widthChanged ? { authoredWidth: current.width } : {}),
    ...(heightChanged ? { authoredHeight: current.height } : {}),
  };
  const encoded = JSON.stringify(next);
  if (encoded !== board.getSharedPluginData(ZUI_METADATA_NAMESPACE, KEY))
    board.setSharedPluginData(ZUI_METADATA_NAMESPACE, KEY, encoded);
  return {
    ...current,
    width: widthChanged
      ? current.width
      : (next.authoredWidth ?? baseline.width),
    height: heightChanged
      ? current.height
      : (next.authoredHeight ?? baseline.height),
  };
}
