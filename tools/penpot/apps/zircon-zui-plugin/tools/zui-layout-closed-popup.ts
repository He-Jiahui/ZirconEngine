import type { ZuiDocument } from '../src/bridge/zui-document';
import type { LayoutReviewCase } from './zui-layout-review-contract';
import type { LayoutAuditSnapshot } from './zui-layout-penpot-cases';

/** A closed popup is deliberately absent from the authored review host. */
export function isIntentionallyClosedPopupCase(
  document: ZuiDocument,
  reviewCase: LayoutReviewCase,
  layout: LayoutAuditSnapshot,
): boolean {
  if (reviewCase.host !== 'component') return false;
  const rootId =
    document.root?.node ?? Object.values(document.components ?? {})[0]?.root;
  const root = rootId ? document.nodes?.[rootId] : undefined;
  const closedPopupComponents = new Set([
    'Dialog',
    'ConfirmDialog',
    'CommandPalette',
    'NotificationCenter',
    'DragOverlay',
    'DropdownPopup',
    'ContextMenu',
    'ContextActionMenu',
    'Popup',
    'Popover',
    'Modal',
  ]);
  if (!root || !closedPopupComponents.has(root.component)) return false;
  if (
    reviewCase.state !== 'closed' &&
    reviewCase.state !== 'focus-return' &&
    !(
      reviewCase.state === 'default' &&
      root.props?.['open'] !== true &&
      root.props?.['popup_open'] !== true &&
      (root.props?.['open'] === false || root.props?.['popup_open'] === false)
    )
  )
    return false;
  return (
    layout.semanticNodes.some(
      (node) =>
        node.nodeId === rootId &&
        node.component === root.component &&
        !node.visible,
    ) && layout.semanticNodes.every((node) => !node.visible)
  );
}
