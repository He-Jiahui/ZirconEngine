import type { ProjectionPaint, ProjectionText } from './penpot-projection-model';
import { controlProperties } from './zui-control-properties';
import { editorButtonState, editorButtonPalette } from './zui-editor-button-style';
import { resolveZuiTextStyle } from './zui-text-style';
import { type ZuiDocument, type ZuiNode, type ZuiTable } from './zui-document';

export const FIELD_ICON_PATHS = {
  search: 'zircon_editor/assets/icons/zircon_editor_shell/controls/search.svg',
  clear: 'zircon_editor/assets/icons/ionicons/close-outline.svg',
  stepper: 'zircon_editor/assets/icons/zircon_editor_shell/controls/field-stepper.svg',
} as const;

export interface FieldProjection {
  paint: ProjectionPaint;
  search: boolean;
  clear: boolean;
  stepper: boolean;
  pad: number;
  padLeft?: number;
  padRight?: number;
  padTop?: number;
  padBottom?: number;
  labelWidth?: number;
  gap: number;
  lineHeight: number;
  iconSize: number;
  maxHeight: number;
  stepperWidth: number;
  stepperGlyphWidth: number;
  borderWidth: number;
  divider: string;
  iconColor: string;
  stepperColor: string;
  icons: Partial<Record<keyof typeof FIELD_ICON_PATHS, string>>;
}

export function isEditorField(node: ZuiNode): boolean {
  return ['InputField', 'NumberField', 'SearchField', 'TextField'].includes(node.component);
}

/** Matches retained-host template_fields and its WorkbenchTextFieldStyle selector. */
export function projectEditorField(document: ZuiDocument, node: ZuiNode): {
  field: FieldProjection; text: ProjectionText; paint: ProjectionPaint;
  textFragments?: Record<string, ProjectionText>;
} | undefined {
  if (!document['penpot_host_theme_source'] || !isEditorField(node)) return undefined;
  const host: ZuiNode = { component: 'InputField', props: document.tokens };
  const { metric } = controlProperties(document, host, 'field host');
  const palette = controlProperties(document, {
    component: 'InputField', props: document['penpot_host_palette'] as ZuiTable,
  }, 'field palette');
  const authored = controlProperties(document, node, 'field');
  const props = node.props ?? {};
  const search = node.component === 'SearchField';
  const labelWidth = search ? 0 : authored.metric(['layout_label_width'], 0);
  const valueProperty = search && typeof props['query'] === 'string' ? 'query' : typeof props['value_text'] === 'string' ? 'value_text' : 'value';
  const value = String(props[valueProperty] ?? '');
  const fallbackProperty = (['text', 'label', 'placeholder'] as const).find((key) =>
    typeof props[key] === 'string' && String(props[key]).trim());
  const property = labelWidth > 0 || value.trim() ? valueProperty : fallbackProperty ?? valueProperty;
  const placeholder = property === 'placeholder';
  const characters = String(props[property] ?? '');
  const state = editorButtonState(node);
  const disabled = state.disabled || state.loading;
  const error = ['error', 'danger'].includes(String(props['validation_level'] ?? ''));
  const normal = state.interaction === 'normal' && !state.selected && !state.open;
  const color = disabled ? authored.color(['disabled_foreground_color'], palette.color(['text_disabled'], '#737373')) :
    placeholder ? palette.color(['text_secondary'], '#b3b3b3') : palette.color(['text_primary'], '#e8e8e8');
  const background = disabled ? authored.color(['disabled_background_color'], editorButtonPalette(document)('surface', 2)) :
    palette.color(['surface_recessed'], '#0f0f0f');
  const border = disabled ? palette.color(['border_disabled'], '#363636') : error ? palette.color(['error'], '#f44336') :
    state.focus && !state.pressed && !state.dragging ? palette.color(['focus_ring'], '#66b2ff') :
      state.pressed || state.open || state.hovered || state.dragging ? palette.color(['surface_hover'], '#383838') : palette.color(['separator_soft'], '#363636');
  const fontSize = authored.metric(['font_size'], metric(['editor.typography.body.size'], 14, true), true);
  const lineHeightPixels = props['line_height'] !== undefined
    ? authored.metric(['line_height'], fontSize * metric(['editor.typography.line_height'], 1.4, true), true)
    : fontSize * authored.metric(['line_height_ratio'], metric(['editor.typography.line_height'], 1.4, true), true);
  const lineHeight = lineHeightPixels / fontSize;
  const textStyle = resolveZuiTextStyle(document, node, { size: fontSize, weight: '400' });
  const gap = metric(['editor.density.gap.small'], 4);
  const pad = metric(['editor.density.gap.medium'], 8);
  const gapLarge = metric(['editor.density.gap.large'], 12);
  const borderWidth = metric(['editor.control.border_width'], 1);
  const rowHeight = metric(['editor.density.row_height'], 28, true);
  const denseHeight = metric(['editor.control.height.dense'], 28, true);
  // Native attachment geometry follows host density; an authored font changes
  // only the text line, leaving the clear hit target and stepper unchanged.
  const hostFontSize = metric(['editor.typography.body.size'], 14, true);
  const iconSize = Math.round(Math.max(rowHeight - gapLarge, hostFontSize));
  const paint: ProjectionPaint = {
    fillColor: normal && !search && !error ? authored.color(['background_color'], background) : background,
    strokeColor: normal && !search && !error ? authored.color(['border_color'], border) : border,
    fillOpacity: 1, strokeOpacity: 1, strokeWidth: borderWidth,
    borderRadius: metric(['editor.control.radius.control'], 4), opacity: Math.min(1, authored.metric(['opacity'], 1)),
  };
  for (const kind of ['fill', 'stroke'] as const) {
    const value = paint[`${kind}Color`];
    paint[`${kind}Opacity`] = value?.length === 9 ? parseInt(value.slice(7), 16) / 255 : 1;
    paint[`${kind}Color`] = value?.slice(0, 7) ?? null;
  }
  return {
    text: {
      characters, property: property === 'value' && typeof props['value'] !== 'string' ? null : property,
      color: color.slice(0, 7), colorOpacity: color.length === 9 ? parseInt(color.slice(7), 16) / 255 : 1,
      fontSize, lineHeight, fontFamily: textStyle.family, fontWeight: textStyle.weight, align: 'left',
    },
    paint: labelWidth > 0 ? { ...paint, fillColor: null, strokeColor: null, strokeWidth: 0 } : paint,
    ...(labelWidth > 0 && typeof props['text'] === 'string' && props['text'].trim()
      ? { textFragments: { 'field-label': {
          characters: props['text'], property: 'text',
          color: color.slice(0, 7), colorOpacity: color.length === 9 ? parseInt(color.slice(7), 16) / 255 : 1,
          fontSize, lineHeight, fontFamily: textStyle.family, fontWeight: textStyle.weight, align: 'left' as const,
        } } } : {}),
    field: {
      paint,
      labelWidth,
      search, clear: search && props['has_clear_action'] === true && Boolean(value.trim()),
      stepper: props['layout_stepper'] === true,
      pad,
      padLeft: authored.metric(['layout_padding_left'], pad),
      padRight: authored.metric(['layout_padding_right'], pad),
      padTop: authored.metric(['layout_padding_top'], 0),
      padBottom: authored.metric(['layout_padding_bottom'], 0),
      gap, lineHeight: lineHeightPixels, iconSize, maxHeight: rowHeight + borderWidth * 4,
      stepperWidth: Math.max(denseHeight - gapLarge + borderWidth * 2, iconSize),
      stepperGlyphWidth: Math.max(borderWidth, Math.round(iconSize - pad + borderWidth * 2)),
      borderWidth, iconColor: color,
      divider: disabled ? palette.color(['border_disabled'], '#363636') : palette.color(['separator_soft'], '#363636'),
      stepperColor: disabled ? palette.color(['text_disabled'], '#737373') : palette.color(['text_secondary'], '#b3b3b3'),
      icons: (document['penpot_field_icons'] ?? {}) as FieldProjection['icons'],
    },
  };
}
