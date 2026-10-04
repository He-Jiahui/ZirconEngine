import type { ZuiDocument } from '../src/bridge/zui-document';
import type { LayoutReviewCase } from './zui-layout-review-contract';

const POPUP_ROOTS = new Set([
  'Dialog',
  'ConfirmDialog',
  'CommandPalette',
  'NotificationCenter',
  'Popup',
  'Popover',
  'Modal',
  'ContextMenu',
  'ContextActionMenu',
  'Dropdown',
  'DropdownPopup',
  'Select',
]);

function authoredPopupNodes(
  source?: ZuiDocument,
): Array<[string, NonNullable<ZuiDocument['nodes']>[string]]> {
  if (!source) return [];
  return Object.entries(source.nodes ?? {}).filter(([, node]) => {
    if (!POPUP_ROOTS.has(node.component)) return false;
    const props = node.props ?? {};
    return ['open', 'popup_open'].some((key) => Object.hasOwn(props, key));
  });
}

/** Source-declared popup states, independent of filenames or design-only layers. */
export function popupReviewStates(source?: ZuiDocument): string[] {
  const nodes = authoredPopupNodes(source);
  return nodes.length &&
    nodes.every(([, node]) =>
      ['open', 'popup_open']
        .filter((key) => Object.hasOwn(node.props ?? {}, key))
        .every((key) => typeof node.props?.[key] === 'boolean'),
    )
    ? ['open', 'closed', 'focus-return']
    : [];
}

/** A closed default frame is not evidence of opening, then closing a popup. */
export function sourceScenarioCoverageErrors(
  source: ZuiDocument,
  cases: readonly Pick<LayoutReviewCase, 'state'>[],
): string[] {
  const nodes = authoredPopupNodes(source);
  if (!nodes.length) return [];
  const errors = nodes.flatMap(([nodeId, node]) => {
    const switches = ['open', 'popup_open'].filter((key) =>
      Object.hasOwn(node.props ?? {}, key),
    );
    return switches.some((key) => typeof node.props?.[key] !== 'boolean')
      ? [`${nodeId}: unsupported popup switch value`]
      : [];
  });
  if (errors.length) return errors;
  const states = new Set(cases.map((item) => item.state));
  const missing = popupReviewStates(source).filter(
    (state) => !states.has(state),
  );
  return missing.length
    ? nodes.map(
        ([nodeId]) =>
          `${nodeId}: missing popup review states: ${missing.join(', ')}`,
      )
    : [];
}
