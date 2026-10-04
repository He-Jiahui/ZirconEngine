import { type ZuiDocument, type ZuiNode, type ZuiTable } from './zui-document';
import { resolveDesignColor } from './zui-prefab-system';
import { buttonStatePreviewNode } from './zui-button-state-projection';
import {
  editorButtonPalette,
  editorButtonState,
} from './zui-editor-button-style';
import {
  editorContainedButton,
  isEditorContainedButton,
} from './zui-editor-contained-button';
import {
  editorButtonCommand,
  editorButtonKind,
  editorButtonTabKind,
  isEditorWorkbenchButton,
  usesWorkbenchLanguage,
} from './zui-editor-button-identity';

export function hasEditorButtonPainter(
  document: ZuiDocument,
  node: ZuiNode,
): boolean {
  return (
    typeof document['penpot_host_theme_source'] === 'string' &&
    (isEditorWorkbenchButton(node) || isEditorContainedButton(node))
  );
}

/** Editor uses style_selector/workbench_button; Runtime uses surface/render/buttons. */
export function hostButtonPreviewNode(
  document: ZuiDocument,
  node: ZuiNode,
): ZuiNode {
  if (!hasEditorButtonPainter(document, node))
    return buttonStatePreviewNode(node);
  if (!isEditorWorkbenchButton(node))
    return editorContainedButton(document, node);
  const props: ZuiTable = { ...node.props };
  const { selected, open, hovered, dragging, pressed, focus, interaction } =
    editorButtonState(node);
  const color = editorButtonPalette(document);
  const authored = (key: string) => resolveDesignColor(document, props[key]);
  const kind = editorButtonKind(node);
  const visualState =
    interaction === 'normal' && selected ? 'hover' : interaction;
  const hot =
    visualState === 'hover' ||
    (visualState === 'focused' && (hovered || dragging || open || selected));
  let fill =
    kind === 'primary'
      ? color('accent')
      : kind === 'tertiary'
        ? 'transparent'
        : color('surface', 3);
  let text =
    kind === 'primary'
      ? color('surface', 0)
      : kind === 'tertiary'
        ? color('text_secondary')
        : kind === 'danger'
          ? color('error')
          : color('text_primary');
  let border = kind === 'tertiary' ? 'transparent' : color('border');
  let width = 1;
  if (hot) {
    fill = color(kind === 'primary' ? 'focus_ring' : 'surface_hover');
    if (kind === 'tertiary') text = color('text_primary');
  }
  if (visualState === 'pressed') {
    fill =
      kind === 'primary'
        ? color('surface_selected')
        : kind === 'tertiary'
          ? color('popup')
          : color('surface', 2);
    if (kind === 'primary' || kind === 'tertiary') text = color('text_primary');
  }
  if (visualState === 'focused') border = color('focus_ring');
  if (interaction === 'disabled' || interaction === 'loading') {
    fill = color('surface_disabled');
    border = color('border_disabled');
    text = color('text_disabled');
    return {
      ...node,
      props: {
        ...props,
        background_color: fill,
        foreground_color: text,
        icon_color: text,
        border_color: border,
        border_width: width,
      },
    };
  }
  if (usesWorkbenchLanguage(node)) {
    if (
      !selected &&
      interaction === 'normal' &&
      kind !== 'primary' &&
      kind !== 'danger'
    ) {
      fill = authored('background_color') ?? fill;
      border = authored('border_color') ?? border;
    }
    if (kind !== 'primary') text = authored('foreground_color') ?? text;
  }
  let glyph = text;
  if (node.control_id === 'WorkbenchAddComponent') {
    text = color('text_secondary');
    glyph = color('text_primary');
  }
  const command = editorButtonCommand(node);
  if (command) {
    fill =
      command === 'primary'
        ? color(
            pressed
              ? 'surface_selected'
              : selected || open || hovered
                ? 'focus_ring'
                : 'accent',
          )
        : pressed
          ? color('surface', 2)
          : selected || open || hovered
            ? color('surface_hover')
            : color('surface', 3);
    if (!focus || pressed)
      border = command === 'primary' ? fill : color('border');
    text =
      command === 'primary'
        ? pressed
          ? color('text_primary')
          : color('surface', 0)
        : color('accent');
    glyph = text;
  }
  const tab = editorButtonTabKind(node);
  if (tab) {
    fill =
      tab === 'chip' && selected
        ? color('surface', 2)
        : tab === 'asset' && selected
          ? color('surface', 3)
          : tab === 'utility'
            ? !selected && (hovered || open)
              ? color('surface_hover')
              : 'transparent'
            : selected || hovered || open
              ? color('surface_hover')
              : tab === 'tab'
                ? color('surface', 3)
                : 'transparent';
    border = color('border');
    width = tab === 'chip' && selected ? 1 : 0;
    text =
      selected || (tab !== 'tab' && (hovered || open))
        ? color('text_primary')
        : color('text_secondary');
    glyph = text;
  }
  const brightness = props['label_brightness'] ?? props['visual_brightness'];
  const brighten = (value: string): string => {
    if (
      typeof brightness !== 'number' ||
      !Number.isFinite(brightness) ||
      brightness <= 0 ||
      Math.abs(brightness - 1) <= 0.001 ||
      value === 'transparent'
    )
      return value;
    const factor = Math.min(4, brightness);
    return (
      '#' +
      [1, 3, 5]
        .map((offset) =>
          Math.min(
            255,
            Math.round(
              Number.parseInt(value.slice(offset, offset + 2), 16) * factor,
            ),
          )
            .toString(16)
            .padStart(2, '0'),
        )
        .join('') +
      value.slice(7)
    );
  };
  return {
    ...node,
    props: {
      ...props,
      background_color: brighten(fill),
      foreground_color: brighten(text),
      icon_color: brighten(glyph),
      border_color: brighten(border),
      border_width: width,
    },
  };
}
