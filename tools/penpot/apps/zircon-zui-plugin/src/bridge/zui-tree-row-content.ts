import type { ZuiDocument, ZuiNode } from './zui-document';
import { treeRowVisual } from './zui-tree-row-visual';
import type {
  NativePainterContent,
  NativePainterPlacement,
} from '../penpot-native-painter-content';
export interface TreeRowPaintContext {
  document: ZuiDocument;
  width: number;
  height: number;
}

/** Mirrors native template_row_metrics and template_tree_rows geometry gates. */
export function treeRowContent(
  node: ZuiNode,
  context?: TreeRowPaintContext,
): NativePainterContent | null {
  if (!context || !context.document['penpot_host_theme_source']) return null;
  const { document, width, height } = context;
  const content: NativePainterContent = { panels: [], texts: [], icons: [] };
  if (!(
    Number.isFinite(width) &&
    Number.isFinite(height) &&
    width > 0 &&
    height > 0
  ))
    return content;
  const {
    props,
    host,
    palette,
    authored,
    marked,
    iconColor,
    textColor,
    fontSize,
    weight,
    lineRatio,
  } = treeRowVisual(document, node);
  const s = host.metric(['editor.density.gap.small'], 4),
    m = host.metric(['editor.density.gap.medium'], 8),
    l = host.metric(['editor.density.gap.large'], 12);
  const border = host.metric(['editor.control.border_width'], 1),
    row = host.metric(['editor.density.row_height'], 28, true),
    dense = host.metric(['editor.control.height.dense'], 28, true);
  const step = Math.max(0, dense - l + border * 2),
    gap = s + border * 2;
  const depth = Math.max(
    0,
    Math.floor(authored.number(['tree_depth', 'depth'], 0)),
  );
  const override = authored.number(['tree_indent_px'], 0),
    indent = override > 0 ? override : depth * step;
  const dx = l + indent,
    dy = Math.max(0, height - l) / 2,
    iconSize = Math.max(1, row - l),
    ix = dx + l + gap,
    iy = dy + Math.max(0, l - iconSize) / 2;
  const labelX = ix + iconSize + gap,
    contentOnly = props['component_variant'] === 'content';
  const reserve = l + (contentOnly ? 0 : m * 4 + l + s),
    lineHeight =
      host.metric(['editor.typography.body.size'], 14, true) * lineRatio;
  const placement = (
    x: number,
    y: number,
    w: number,
    h: number,
  ): NativePainterPlacement => ({
    x: x / width,
    y: y / height,
    width: w / width,
    height: h / height,
    exactBounds: true,
  });
  const contains = (x: number, y: number, w: number, h: number) =>
    w > 0 && h > 0 && x >= 0 && y >= 0 && x + w <= width && y + h <= height;
  for (let level = 0; level < depth; level++)
    if (border > 0)
      content.panels.push({
        kind: 'tree-indent-guide',
        placement: placement(
          l + s + border + level * step,
          -border,
          border,
          height + border * 2,
        ),
        tone: 'muted',
        color: palette.color(['track'], '#484848'),
        opacity: 0.78,
        radius: 0,
      });
  if (!contains(dx, dy, l, l)) return content;
  const icon = (name: string, x: number, y: number, size: number) =>
    content.icons!.push({
      name,
      placement: placement(x, y, size, size),
      tone: 'primary',
      size: 's',
      color: iconColor,
      exactBounds: true,
    });
  if (!contentOnly)
    icon(
      props['expanded'] === true ? 'chevron-down' : 'chevron-right',
      dx,
      dy,
      l,
    );
  if (!contains(ix, iy, iconSize, iconSize)) return content;
  const name = typeof props['icon'] === 'string' ? props['icon'] : 'box';
  icon(name, ix, iy, iconSize);
  if (typeof node['penpot_icon_svg'] === 'string')
    content.icons!.at(-1)!.svg = node['penpot_icon_svg'];
  const text = typeof props['text'] === 'string' ? props['text'] : '';
  const labelWidth = Math.max(0, width - labelX - reserve),
    labelY = Math.max(0, height - lineHeight) / 2;
  if (text && contains(labelX, labelY, labelWidth, lineHeight))
    content.texts.push({
      value: text,
      placement: placement(labelX, labelY, labelWidth, lineHeight),
      tone: 'primary',
      size: fontSize,
      weight,
      color: textColor,
      lineHeight: lineRatio,
      exactBounds: true,
    });
  const actionSize = m * 2,
    actionGap = l + s,
    buttonSize = actionSize + s;
  const actionX = (index: number) =>
    width - l - actionSize - index * (actionSize + actionGap);
  const actionY = Math.max(0, height - actionSize) / 2;
  const buttonX = (index: number) =>
    actionX(index) + (actionSize - buttonSize) / 2;
  const buttonY = actionY + (actionSize - buttonSize) / 2;
  if (!contentOnly && contains(buttonX(1), buttonY, buttonSize, buttonSize)) {
    const action = (name: string, index: number) => {
      content.panels.push({
        kind: 'tree-action-slot',
        placement: placement(buttonX(index), buttonY, buttonSize, buttonSize),
        tone: 'surface',
        color: palette.color(['surface_hover'], '#333333'),
        borderColor: palette.color(['border'], '#484848'),
        borderWidth: border,
        radius: host.metric(['editor.control.radius.control'], 4),
        opacity: 1,
      });
      icon(
        name,
        buttonX(index) + Math.max(0, buttonSize - actionSize) / 2,
        buttonY + Math.max(0, buttonSize - actionSize) / 2,
        actionSize,
      );
    };
    action('eye', 1);
    if (marked) action('more-vertical', 0);
    else if (
      depth <= 1 ||
      ['Audio', 'Root', 'Environment'].some((key) =>
        node.control_id?.includes(key),
      )
    )
      action('lock', 0);
  }
  return content;
}
