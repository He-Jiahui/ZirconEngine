import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { resolveDesignNumber } from './bridge/zui-prefab-system';
import type { LinearDesiredSize } from './penpot-weighted-flex';

// Keep this contract aligned with runtime ui/layout/pass/material.rs.
const MATERIAL_COMPONENTS = new Set([
  'Button',
  'IconButton',
  'ToggleButton',
  'Checkbox',
  'InputField',
  'TextField',
  'ListRow',
  'ComboBox',
  'RangeField',
  'NumberField',
  'Progress',
  'ProgressBar',
  'LinearProgress',
  'CircularProgress',
  'Spinner',
  'Skeleton',
  'Backdrop',
  'Paper',
  'Modal',
  'Dialog',
  'AlertDialog',
  'Popover',
  'Popper',
  'Tooltip',
  'Snackbar',
  'Menu',
  'Drawer',
  'Switch',
  'ContextActionMenu',
  'MenuItem',
  'Tab',
  'TableRow',
  'VirtualList',
  'ColorField',
  'Vector2Field',
  'Vector3Field',
  'Vector4Field',
  'Label',
]);
const MATERIAL_METRICS = [
  'layout_padding_left',
  'layout_padding_right',
  'layout_padding_top',
  'layout_padding_bottom',
  'layout_spacing',
  'layout_min_width',
  'layout_min_height',
  'layout_icon_size',
  'layout_leading_slot_width',
  'layout_trailing_slot_width',
] as const;
const BUTTON_CONTENT_PADDING = { width: 18, height: 8 };

/** Skin metrics contribute to native desired size before authored box constraints. */
export function measureNativeMaterialContent(
  document: ZuiDocument,
  node: ZuiNode,
  content: LinearDesiredSize,
): LinearDesiredSize | undefined {
  const props = node.props ?? {};
  if (
    !MATERIAL_COMPONENTS.has(node.component) ||
    !MATERIAL_METRICS.some((key) => Object.hasOwn(props, key))
  )
    return undefined;
  const metric = (key: (typeof MATERIAL_METRICS)[number]) =>
    Math.max(0, resolveDesignNumber(document, props[key]) ?? 0);
  const hasIcon =
    ['icon', 'image', 'media', 'source'].some(
      (key) => typeof props[key] === 'string' && props[key] !== '',
    ) ||
    (node.component === 'IconButton' && metric('layout_icon_size') > 0);
  const icon = hasIcon ? metric('layout_icon_size') : 0;
  return {
    width: Math.max(
      metric('layout_min_width'),
      content.width +
        icon +
        (hasIcon && content.width > 0 ? metric('layout_spacing') : 0) +
        metric('layout_leading_slot_width') +
        metric('layout_trailing_slot_width') +
        metric('layout_padding_left') +
        metric('layout_padding_right'),
    ),
    height: Math.max(
      metric('layout_min_height'),
      Math.max(content.height, icon) +
        metric('layout_padding_top') +
        metric('layout_padding_bottom'),
    ),
  };
}

export function measureNativeLeafContent(
  document: ZuiDocument,
  node: ZuiNode,
  text: LinearDesiredSize,
): LinearDesiredSize {
  const material = measureNativeMaterialContent(document, node, text);
  if (material) return material;
  if (
    (node.component === 'Button' || node.component === 'IconButton') &&
    (text.width > 0 || text.height > 0)
  )
    return {
      width: text.width + BUTTON_CONTENT_PADDING.width,
      height: text.height + BUTTON_CONTENT_PADDING.height,
    };
  return text;
}
