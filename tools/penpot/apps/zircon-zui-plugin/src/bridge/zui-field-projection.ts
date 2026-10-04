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
  const valueProperty = search && typeof props['query'] === 'string' ? 'query' : typeof props['value_text'] === 'string' ? 'value_text' : 'value';
  const value = String(props[valueProperty] ?? '');
  const fallbackProperty = (['text', 'label', 'placeholder'] as const).find((key) =>
    typeof props[key] === 'string' && String(props[key]).trim());
  const property = value.trim() ? valueProperty : fallbackProperty ?? valueProperty;
  const placeholder = property === 'placeholder';
  const characters = String(props[property] ?? '');
  const state = editorButtonState(node);
  const disabled = state.disabled || state.loading;
  const error = ['error', 'danger'].includes(String(props['validation_level'] ?? ''));
  const normal = state.interaction === 'normal' && !state.selected && !state.open;
  const color = disabled ? palette.color(['text_disabled'], '#737373') :
    placeholder ? palette.color(['text_secondary'], '#b3b3b3') : palette.color(['text_primary'], '#e8e8e8');
  const background = disabled ? editorButtonPalette(document)('surface', 2) : palette.color(['surface_recessed'], '#0f0f0f');
  const border = disabled ? palette.color(['border_disabled'], '#363636') : error ? palette.color(['error'], '#f44336') :
    state.focus && !state.pressed && !state.dragging ? palette.color(['focus_ring'], '#66b2ff') :
      state.pressed || state.open || state.hovered || state.dragging ? palette.color(['surface_hover'], '#383838') : palette.color(['separator_soft'], '#363636');
  const fontSize = metric(['editor.typography.body.size'], 14, true);
  const lineHeight = metric(['editor.typography.line_height'], 1.4, true);
  const gap = metric(['editor.density.gap.small'], 4);
  const pad = metric(['editor.density.gap.medium'], 8);
  const gapLarge = metric(['editor.density.gap.large'], 12);
  const borderWidth = metric(['editor.control.border_width'], 1);
  const rowHeight = metric(['editor.density.row_height'], 28, true);
  const denseHeight = metric(['editor.control.height.dense'], 28, true);
  const iconSize = Math.round(Math.max(rowHeight - gapLarge, fontSize));
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
      fontSize, lineHeight, fontFamily: resolveZuiTextStyle(document, host).family, fontWeight: '400', align: 'left',
    },
    paint,
    field: {
      paint,
      search, clear: search && props['has_clear_action'] === true && Boolean(value.trim()),
      stepper: props['layout_stepper'] === true,
      pad, gap, lineHeight: fontSize * lineHeight, iconSize, maxHeight: rowHeight + borderWidth * 4,
      stepperWidth: Math.max(denseHeight - gapLarge + borderWidth * 2, iconSize),
      stepperGlyphWidth: Math.max(borderWidth, Math.round(iconSize - pad + borderWidth * 2)),
      borderWidth, iconColor: color,
      divider: disabled ? palette.color(['border_disabled'], '#363636') : palette.color(['separator_soft'], '#363636'),
      stepperColor: disabled ? palette.color(['text_disabled'], '#737373') : palette.color(['text_secondary'], '#b3b3b3'),
      icons: (document['penpot_field_icons'] ?? {}) as FieldProjection['icons'],
    },
  };
}
