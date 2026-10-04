export interface ReviewOrderEntry {
  sourcePath: string;
  sourceKind: string;
}

export function layoutReviewPhase(entry: ReviewOrderEntry): number {
  const path = entry.sourcePath;
  if (
    /\/tests\/fixtures\/|\/ui\/runtime\/fixtures\/|(?:^|[/_-])fixture(?:[/_-]|\.zui$)/.test(
      path,
    )
  )
    return 6;
  if (path.startsWith('examples/') || path.startsWith('zircon_runtime/'))
    return 5;
  if (path.startsWith('zircon_plugins/')) return 4;
  if (
    /theme|style|tokens/.test(entry.sourceKind) ||
    path.includes('/workbench/primitives/')
  )
    return 0;
  if (
    path.includes('/workbench/shell/') ||
    /\/host\/(?:editor_main_frame|workbench_shell)\.zui$/.test(path) ||
    path.endsWith('/windows/workbench_window.zui')
  )
    return 2;
  if (path.includes('/workbench/modules/')) return 3;
  if (/^(component|widget)$/.test(entry.sourceKind)) return 1;
  return 3;
}

export function compareLayoutReviewOrder(
  left: ReviewOrderEntry,
  right: ReviewOrderEntry,
): number {
  return (
    layoutReviewPhase(left) - layoutReviewPhase(right) ||
    left.sourcePath.localeCompare(right.sourcePath)
  );
}
