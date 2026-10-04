import type { Board } from '@penpot/plugin-types';
import type { ZuiDocument } from './bridge/zui-document';
import { reviewBoardNeedsResponsiveRebuild } from './bridge/zui-review-case';
import { ZUI_METADATA_NAMESPACE } from './metadata';
import type { LayoutReviewCaseMessage } from './model';

function priorReviewViewportWidth(
  board: Pick<Board, 'getSharedPluginData'>,
): number | undefined {
  const raw = board.getSharedPluginData(ZUI_METADATA_NAMESPACE, 'review-case');
  if (!raw) return undefined;
  try {
    const previous: unknown = JSON.parse(raw);
    if (typeof previous !== 'object' || previous === null) return undefined;
    const viewport = (previous as { viewport?: unknown }).viewport;
    if (typeof viewport !== 'object' || viewport === null) return undefined;
    const width = (viewport as { width?: unknown }).width;
    return typeof width === 'number' && Number.isFinite(width) && width > 0
      ? width
      : undefined;
  } catch {
    return undefined;
  }
}

/** Reusing a board is safe only when its responsive container geometry still applies. */
export function canReuseReviewBoard(
  previous: Pick<Board, 'getSharedPluginData'>,
  document: ZuiDocument,
  reviewCase: LayoutReviewCaseMessage,
): boolean {
  if (reviewCase.locale !== 'en-US' || Object.keys(reviewCase.data).length > 0)
    return false;
  if (
    reviewBoardNeedsResponsiveRebuild(
      document,
      priorReviewViewportWidth(previous),
      reviewCase.viewport.width,
    )
  )
    return false;

  const isLargeWorkbenchHost = [
    'editor_main_frame.zui',
    'workbench_window.zui',
    'workbench_skeleton.zui',
  ].some((suffix) => document.asset.id.endsWith(suffix));
  // Stable default and scroll cases can clone their native instances. A
  // breakpoint change above must rebuild the semantic layout first.
  if (reviewCase.state === 'default') return true;
  if (
    isLargeWorkbenchHost &&
    document.asset.id.endsWith('workbench_window.zui') &&
    reviewCase.state === 'open'
  )
    return true;
  return (
    reviewCase.state === 'scroll-before' || reviewCase.state === 'scroll-after'
  );
}
