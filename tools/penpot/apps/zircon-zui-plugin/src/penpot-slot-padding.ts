import type { Board } from '@penpot/plugin-types';
import type { ProjectionPadding } from './bridge/penpot-projection-model';

export function applySlotPadding(
  board: Board,
  padding: ProjectionPadding | undefined,
): void {
  if (!padding) return;
  const child = board.layoutChild;
  if (!child)
    throw new Error(
      `Missing Penpot layout child for slot padding: ${board.name}`,
    );
  const matchingSides =
    (child.topMargin ?? 0) === padding.top &&
    (child.rightMargin ?? 0) === padding.right &&
    (child.bottomMargin ?? 0) === padding.bottom &&
    (child.leftMargin ?? 0) === padding.left;
  const asymmetric =
    padding.top !== padding.bottom || padding.right !== padding.left;
  if (
    matchingSides &&
    (!asymmetric ||
      child.marginType === undefined ||
      child.marginType === 'multiple')
  )
    return;
  // Some official frontend proxies omit marginType but retain the side setters.
  // Every setter opens host reflow work, so leave equivalent values untouched.
  if (child.marginType !== undefined && child.marginType !== 'multiple')
    child.marginType = 'multiple';
  if ((child.topMargin ?? 0) !== padding.top) child.topMargin = padding.top;
  if ((child.rightMargin ?? 0) !== padding.right)
    child.rightMargin = padding.right;
  if ((child.bottomMargin ?? 0) !== padding.bottom)
    child.bottomMargin = padding.bottom;
  if ((child.leftMargin ?? 0) !== padding.left) child.leftMargin = padding.left;
}

export function captureSlotPadding(
  board: Board,
  baseline: ProjectionPadding | undefined,
  initializingMappedChild = false,
): ProjectionPadding | undefined {
  const child = board.layoutChild;
  if (!baseline && !initializingMappedChild) {
    if (
      child &&
      [
        child.topMargin,
        child.rightMargin,
        child.bottomMargin,
        child.leftMargin,
      ].some((value) => (value ?? 0) !== 0)
    )
      throw new Error(`Unmapped slot margin edit: ${board.name}`);
    return undefined;
  }
  if (!child)
    throw new Error(
      `Missing Penpot layout child for slot padding: ${board.name}`,
    );
  return {
    top: child.topMargin ?? 0,
    right: child.rightMargin ?? 0,
    bottom: child.bottomMargin ?? 0,
    left: child.leftMargin ?? 0,
  };
}
