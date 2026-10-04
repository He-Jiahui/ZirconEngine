import type { ProjectionText } from './penpot-projection-model';
import { type ZuiDocument, type ZuiNode, type ZuiTable } from './zui-document';
import { controlProperties } from './zui-control-properties';
import { resolveZuiTextStyle } from './zui-text-style';
import { editorButtonState } from './zui-editor-button-style';
import { propertyRowValues } from './zui-property-row-values';

export interface PropertyRowProjection {
  labelWidth: number;
  labelMinWidth: number;
  hasLabel: boolean;
  hasValue: boolean;
  textInsetX: number;
  textInsetY: number;
  fieldInsetY: number;
  fieldRadius: number;
  fieldBorderWidth: number;
  fieldFill: string;
  fieldBorder: string;
  groupGap: number;
  axisWidth: number;
  axisGap: number;
  axisKeys: string[];
}

export function isComponentPropertyRow(node: ZuiNode): boolean {
  const id = node.control_id ?? '';
  return (
    [
      'WorkbenchMeshRow',
      'WorkbenchMaterialRow',
      'WorkbenchComponentPropertyRowRoot',
      'WorkbenchComponentPropertySlot03Row',
      'WorkbenchComponentPropertySlot04Row',
    ].includes(id) || id.startsWith('WorkbenchComponentPropertyVirtualRow')
  );
}

export function isPropertyRow(node: ZuiNode): boolean {
  return node.component === 'PropertyRow' || isComponentPropertyRow(node);
}

/** Matches retained-host template_property_rows and template_row_metrics. */
export function projectPropertyRow(
  document: ZuiDocument,
  node: ZuiNode,
):
  | {
      propertyRow: PropertyRowProjection;
      text: null;
      textFragments: Record<string, ProjectionText>;
    }
  | undefined {
  if (!document['penpot_host_theme_source'] || !isPropertyRow(node))
    return undefined;
  const values = propertyRowValues(node);
  if (!values.label.characters && !values.value.characters) return undefined;
  const host: ZuiNode = { component: 'PropertyRow', props: document.tokens };
  const { metric } = controlProperties(document, host, 'property row host');
  const palette = controlProperties(
    document,
    {
      component: 'PropertyRow',
      props: document['penpot_host_palette'] as ZuiTable,
    },
    'property row palette',
  );
  const fontSize = metric(['editor.typography.body.size'], 14, true);
  const lineHeight = metric(['editor.typography.line_height'], 1.4, true);
  const family = resolveZuiTextStyle(document, host).family;
  const gap = metric(['editor.density.gap.small'], 4);
  const border = metric(['editor.control.border_width'], 1);
  const rowHeight = metric(['editor.density.row_height'], 28, true);
  const { color } = controlProperties(document, node, 'property row label');
  const state = editorButtonState(node);
  const tone = String(node.props?.['text_tone'] ?? '');
  const toneKey = ['muted', 'subtle'].includes(tone)
    ? 'text_secondary'
    : ['accent', 'primary', 'default'].includes(tone)
      ? 'accent'
      : ['error', 'danger'].includes(tone)
        ? 'error'
        : ['warning', 'success', 'info'].includes(tone)
          ? tone
          : 'text_primary';
  const labelColor = state.disabled
    ? palette.color(['text_disabled'], '#737373')
    : color(['foreground_color'], palette.color([toneKey], '#e8e8e8'));
  const baseText = (
    characters: string,
    property: ProjectionText['property'],
    rawColor: string,
  ): ProjectionText => ({
    characters,
    property,
    color: rawColor.slice(0, 7),
    colorOpacity:
      rawColor.length === 9 ? parseInt(rawColor.slice(7), 16) / 255 : 1,
    fontFamily: family,
    fontSize,
    fontWeight: '400',
    lineHeight,
    align: 'left',
  });
  const textFragments: Record<string, ProjectionText> = {};
  if (values.label.characters)
    textFragments['property-label'] = baseText(
      values.label.characters,
      values.label.property,
      labelColor,
    );
  const valueColor = palette.color(['text_primary'], '#e8e8e8');
  const axes = values.axes.length >= 2 ? values.axes.slice(0, 4) : [];
  if (axes.length) {
    for (const axis of axes) {
      textFragments[axis.key] = baseText(
        axis.value,
        values.value.property,
        valueColor,
      );
      textFragments[`${axis.key}-label`] = baseText(
        axis.axis,
        null,
        palette.color(['text_secondary'], '#b3b3b3'),
      );
    }
  } else if (values.value.characters) {
    textFragments['property-value'] = baseText(
      values.value.characters,
      values.value.property,
      valueColor,
    );
  }
  return {
    propertyRow: {
      labelWidth: isComponentPropertyRow(node)
        ? rowHeight * 4 - gap
        : rowHeight * 3.5,
      labelMinWidth: metric(['editor.typography.title.size'], 20, true) * 4,
      hasLabel: Boolean(values.label.characters),
      hasValue: Boolean(values.value.characters),
      textInsetX: gap + border,
      textInsetY: gap,
      fieldInsetY: Math.max(0, gap - border),
      fieldRadius: metric(['editor.control.radius.control'], 4),
      fieldBorderWidth: border,
      fieldFill: palette.color(['surface_recessed'], '#0f0f0f'),
      fieldBorder:
        !axes.length && (state.focus || state.pressed)
          ? palette.color(['focus_ring'], '#66b2ff')
          : palette.color(['border'], '#484848'),
      groupGap: gap + border * 2,
      axisWidth: metric(['editor.density.gap.large'], 12),
      axisGap: gap,
      axisKeys: axes.map(({ key }) => key),
    },
    text: null,
    textFragments,
  };
}
