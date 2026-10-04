import {
  type ZuiDocument,
  type ZuiNode,
  ZuiDocumentError,
} from './zui-document';
import { usesWorkbenchLanguage } from './zui-editor-button-identity';
import {
  editorButtonPalette,
  editorButtonState,
} from './zui-editor-button-style';
import { resolveDesignColor } from './zui-prefab-system';

export function isEditorContainedButton(node: ZuiNode): boolean {
  return (
    node.component === 'Button' &&
    node.props?.['button_variant'] === 'contained' &&
    !usesWorkbenchLanguage(node)
  );
}

/** Generic retained buttons consume template_style/colors and typed ButtonColor. */
export function editorContainedButton(
  document: ZuiDocument,
  node: ZuiNode,
): ZuiNode {
  const props = { ...node.props };
  const color = editorButtonPalette(document);
  const state = editorButtonState(node);
  const authored = (key: string) => resolveDesignColor(document, props[key]);
  const variant = String(props['button_color'] ?? 'default').toLowerCase();
  const validation = String(props['validation_level'] ?? '');
  const surface = String(props['surface_variant'] ?? '');
  const severity =
    ['error', 'danger'].includes(validation) ||
    ['error', 'danger'].includes(surface)
      ? 'error'
      : validation === 'warning'
        ? 'warning'
        : validation === 'success' || surface === 'success'
          ? 'success'
          : validation === 'info' || surface === 'info'
            ? 'info'
            : null;
  const primary =
    ['default', 'primary'].includes(variant) ||
    ['primary', 'accent'].includes(surface);
  const semantic = ['warning', 'error', 'success', 'info'].includes(variant);
  const custom =
    !['default', 'primary', 'secondary', 'inherit'].includes(variant) &&
    !semantic
      ? resolveDesignColor(document, props['button_color'])
      : null;
  if (
    !['default', 'primary', 'secondary', 'inherit'].includes(variant) &&
    !semantic &&
    !custom
  )
    throw new ZuiDocumentError(`Unsupported retained ButtonColor ${variant}`);
  const typedFill = semantic
    ? color(`${variant}_container`)
    : (custom ??
      (['default', 'primary'].includes(variant)
        ? color('accent')
        : color('surface_selected')));
  const typedBorder = semantic
    ? color(variant)
    : (custom ??
      (['default', 'primary'].includes(variant) ? color('accent') : null));
  const tone = String(props['text_tone'] ?? '');
  const toneColor = ['muted', 'subtle'].includes(tone)
    ? color('text_secondary')
    : ['accent', 'primary', 'default'].includes(tone)
      ? color('accent')
      : ['error', 'danger'].includes(tone)
        ? color('error')
        : ['warning', 'success', 'info'].includes(tone)
          ? color(tone)
          : color('text_primary');
  const fill = state.disabled
    ? color('surface_disabled')
    : severity
      ? color(`${severity}_container`)
      : ['pressed', 'focused'].includes(state.interaction)
        ? color('surface', 3)
        : state.interaction === 'hover'
          ? color(primary ? 'accent_soft' : 'surface_hover')
          : (authored('background_color') ?? typedFill);
  const border = state.disabled
    ? color('border_disabled')
    : severity
      ? color(severity)
      : (authored('border_color') ??
        (state.interaction === 'focused'
          ? color('focus_ring')
          : state.selected
            ? color('border')
            : (typedBorder ?? color('border'))));
  const text = state.disabled
    ? color('text_disabled')
    : (authored('foreground_color') ??
      (primary && ['normal', 'hover'].includes(state.interaction)
        ? color('surface', 0)
        : toneColor));
  return {
    ...node,
    props: {
      ...props,
      background_color: fill,
      border_color: border,
      foreground_color: text,
    },
  };
}
