import type { ZuiDocument, ZuiNode, ZuiTable } from './zui-document';
import type { ProjectionText } from './penpot-projection-model';
import { controlProperties } from './zui-control-properties';
import { nativePainterComponent } from './zui-native-painter-role';
import { styledPreviewNode } from './zui-style-projection';
import { authoredPaintPreviewNode } from './zui-paint-properties';

/** Resolve the same authored style layers used by the standard projection. */
export function treeRowVisual(document: ZuiDocument, source: ZuiNode) {
  const ancestors: ZuiNode[] = [];
  const nodes = document.nodes ?? {};
  let nodeId = Object.keys(nodes).find((id) => nodes[id] === source);
  const seen = new Set<string>();
  while (nodeId && !seen.has(nodeId)) {
    seen.add(nodeId);
    const parentId = Object.keys(nodes).find((id) =>
      nodes[id].children?.some((x) => x.node === nodeId),
    );
    if (!parentId) break;
    ancestors.push(nodes[parentId]);
    nodeId = parentId;
  }
  const node = authoredPaintPreviewNode(
    document,
    styledPreviewNode(document, source, ancestors),
    source.control_id ?? source.component,
  );
  const props: ZuiTable = { ...node.props, ...node.state };
  const host = controlProperties(
    document,
    { component: 'TreeRow', props: document.tokens },
    'tree row host',
  );
  const palette = controlProperties(
    document,
    {
      component: 'TreeRow',
      props: document['penpot_host_palette'] as ZuiTable,
    },
    'tree row palette',
  );
  const authored = controlProperties(document, { ...node, props }, 'tree row');
  const marked = props['selected'] === true || props['checked'] === true;
  const unavailable =
    props['disabled'] === true ||
    props['enabled'] === false ||
    props['loading'] === true;
  const colorProperty = unavailable
    ? 'disabled_foreground_color'
    : marked
      ? 'selected_foreground_color'
      : 'foreground_color';
  const textColor = authored.color(
    [colorProperty],
    palette.color(
      [unavailable ? 'text_disabled' : 'text_primary'],
      unavailable ? '#737373' : '#e8e8e8',
    ),
  );
  const iconColor = unavailable
    ? textColor
    : authored.color(
        [marked ? 'selected_icon_color' : 'icon_color'],
        marked ? textColor : palette.color(['text_secondary'], '#b3b3b3'),
      );
  const fontSize = authored.metric(
    ['font_size'],
    host.metric(['editor.typography.body.size'], 14, true),
    true,
  );
  const weight: '400' | '600' =
    authored.number(['font_weight'], 400) >= 600 ? '600' : '400';
  const lineRatio = host.metric(['editor.typography.line_height'], 1.4, true);
  return {
    props,
    host,
    palette,
    authored,
    marked,
    unavailable,
    textColor,
    iconColor,
    fontSize,
    weight,
    lineRatio,
    colorProperty,
  };
}

export function projectTreeRowText(
  document: ZuiDocument,
  node: ZuiNode,
  text: ProjectionText | null,
): ProjectionText | null {
  if (
    !text ||
    !document['penpot_host_theme_source'] ||
    nativePainterComponent(node) !== 'TreeRow'
  )
    return text;
  const visual = treeRowVisual(document, node);
  return {
    ...text,
    color: visual.textColor.slice(0, 7),
    colorOpacity:
      visual.textColor.length === 9
        ? parseInt(visual.textColor.slice(7), 16) / 255
        : 1,
    fontSize: visual.fontSize,
    fontWeight: visual.weight,
    lineHeight: visual.lineRatio,
  };
}

export function treeRowTextColorProperty(
  node: ZuiNode,
):
  | 'foreground_color'
  | 'selected_foreground_color'
  | 'disabled_foreground_color' {
  const props = { ...node.props, ...node.state };
  if (
    props['disabled'] === true ||
    props['loading'] === true ||
    props['enabled'] === false
  )
    return 'disabled_foreground_color';
  return props['selected'] === true || props['checked'] === true
    ? 'selected_foreground_color'
    : 'foreground_color';
}
