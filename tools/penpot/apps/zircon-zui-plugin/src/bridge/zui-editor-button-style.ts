import {
  ZuiDocumentError,
  type ZuiDocument,
  type ZuiNode,
  type ZuiTable,
} from './zui-document';
import { resolveDesignColor } from './zui-prefab-system';

export function editorButtonState(node: ZuiNode) {
  const state = { ...node.props, ...node.state };
  const flag = (key: string) => state[key] === true;
  const declared = String(
    state['button_interaction_state'] ?? '',
  ).toLowerCase();
  const disabled =
    flag('disabled') || state['enabled'] === false || declared === 'disabled';
  const loading = flag('loading') || declared === 'loading';
  const selected = flag('selected') || flag('checked');
  const open = flag('popup_open') || flag('open');
  const hovered = flag('hovered') || declared === 'hover';
  const dragging =
    flag('dragging') || flag('drop_hovered') || flag('active_drag_target');
  const pressed =
    flag('pressed') || flag('enter_pressed') || declared === 'pressed';
  const focus =
    typeof state['focus_visible'] === 'boolean'
      ? flag('focus_visible')
      : flag('focused') || declared === 'focused';
  // UiPainterStyleSelector's legacy scalar precedence retains drag over pressed/focus.
  const interaction = disabled
    ? 'disabled'
    : loading
      ? 'loading'
      : dragging
        ? 'hover'
        : pressed
          ? 'pressed'
          : focus
            ? 'focused'
            : open || (!selected && hovered)
              ? 'hover'
              : 'normal';
  return {
    disabled,
    loading,
    selected,
    open,
    hovered,
    dragging,
    pressed,
    focus,
    interaction,
  };
}

export function editorButtonPalette(document: ZuiDocument) {
  const palette = document['penpot_host_palette'] as ZuiTable | undefined;
  return (key: string, index?: number): string => {
    const value =
      index === undefined
        ? palette?.[key]
        : (palette?.[key] as unknown[])?.[index];
    const resolved = resolveDesignColor(document, value);
    if (!resolved || !/^#[\da-f]{6}(?:[\da-f]{2})?$/i.test(resolved))
      throw new ZuiDocumentError(
        `Editor button host palette is missing ${key}${index ?? ''}`,
      );
    return resolved;
  };
}
