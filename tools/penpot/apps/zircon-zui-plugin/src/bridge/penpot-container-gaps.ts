import type { ProjectionContainer } from './penpot-projection-model';
import { ZuiDocumentError, type ZuiTable } from './zui-document';

/** Axis names must match both native ZUI container parsers. */
export function containerAxisGapKeys(
  container: Pick<ProjectionContainer, 'kind' | 'wrap'>,
  sourceKind?: unknown,
): { row: string; column: string } | null {
  if (isMasonryContainer(sourceKind)) return null;
  if (container.kind === 'grid')
    return { row: 'row_gap', column: 'column_gap' };
  if (container.kind === 'flex' && container.wrap)
    return { row: 'vertical_gap', column: 'horizontal_gap' };
  return null;
}

const equal = (left: number, right: number): boolean =>
  Math.abs(left - right) < 0.01;

export function reconcileContainerGapEdits(
  nodeId: string,
  baseline: ProjectionContainer,
  current: ProjectionContainer,
  authored: ZuiTable | undefined,
  sourceKind: unknown = authored?.['kind'],
): Record<string, number> {
  const edits: Record<string, number> = {};
  if (current.kind === 'flex' && current.wrap && current.direction !== 'row')
    throw new ZuiDocumentError(
      `Node ${nodeId} uses vertical wrapping that a native ZUI container cannot represent.`,
    );
  const gapChanged = !equal(current.gap, baseline.gap);
  if (isMasonryContainer(sourceKind)) {
    if (!equal(current.rowGap, current.columnGap))
      throw new ZuiDocumentError(
        `Node ${nodeId} changed independent axis gaps that a native Masonry container's scalar gap cannot represent.`,
      );
    const axisChanged = !equal(current.rowGap, baseline.rowGap);
    if (gapChanged && axisChanged && !equal(current.gap, current.rowGap))
      throw new ZuiDocumentError(
        `Node ${nodeId} changed conflicting values for the native Masonry scalar gap.`,
      );
    const gap = axisChanged ? current.rowGap : current.gap;
    if (!equal(gap, baseline.gap)) edits['gap'] = gap;
    return edits;
  }
  const keys = containerAxisGapKeys(current);
  const baselineKeys = containerAxisGapKeys(baseline);
  const gapContractChanged =
    keys?.row !== baselineKeys?.row || keys?.column !== baselineKeys?.column;
  if (keys) {
    if (gapChanged) edits['gap'] = current.gap;
    for (const [axis, key] of [
      ['rowGap', keys.row],
      ['columnGap', keys.column],
    ] as const) {
      if (
        (gapContractChanged || !equal(current[axis], baseline[axis])) &&
        (gapContractChanged ||
          !gapChanged ||
          !equal(current[axis], current.gap) ||
          authored?.[key] !== undefined)
      )
        edits[key] = current[axis];
    }
    return edits;
  }

  if (current.kind === 'flex') {
    const active = current.direction === 'row' ? 'columnGap' : 'rowGap';
    const inactive = current.direction === 'row' ? 'rowGap' : 'columnGap';
    const axisChanged = !equal(current[active], baseline[active]);
    const directionChanged = current.direction !== baseline.direction;
    // A normal nowrap inspector edit changes only its enabled axis. Linear
    // engine containers read gap; row_gap and column_gap have no effect there.
    const gap =
      axisChanged || directionChanged || gapContractChanged
        ? current[active]
        : current.gap;
    if (
      !equal(current[inactive], baseline[inactive]) &&
      !equal(current[inactive], gap)
    )
      throw new ZuiDocumentError(
        `Node ${nodeId} changed an inactive layout gap that a linear ZUI container cannot represent.`,
      );
    if (!equal(gap, baseline.gap)) edits['gap'] = gap;
    return edits;
  }

  if (gapChanged) edits['gap'] = current.gap;
  return edits;
}

function isMasonryContainer(kind: unknown): boolean {
  return typeof kind === 'string' && /^masonry(?:box)?$/i.test(kind);
}
