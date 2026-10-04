import type { ZuiDocument, ZuiNode } from './zui-document';
import { controlProperties } from './zui-control-properties';

export interface DividerProjection {
  vertical: boolean;
  variant: string;
  thickness: number;
  minimumExtent: number;
  inset: number;
  color: string;
}

export function projectDivider(
  document: ZuiDocument,
  node: ZuiNode,
): DividerProjection | undefined {
  if (!['Divider', 'Separator'].includes(node.component)) return;
  const { resolve, metric, color } = controlProperties(
    document,
    node,
    'divider',
  );
  const thickness = metric(['thickness', 'border_width'], 1, true);
  const disabled =
    node.props?.['disabled'] === true || node.state?.['disabled'] === true;
  return {
    vertical:
      String(resolve('orientation') ?? resolve('direction')).toLowerCase() ===
      'vertical',
    variant: String(resolve('variant') ?? 'fullWidth').toLowerCase(),
    thickness,
    minimumExtent: Math.min(thickness, 1),
    inset: metric(['inset'], 8),
    color: disabled
      ? color(['disabled_separator_color'], '#363636')
      : color(['separator_color', 'color'], '#333333'),
  };
}

/** Geometry and empty-frame rules mirror Runtime surface/render/divider.rs. */
export function dividerGeometry(
  divider: DividerProjection,
  width: number,
  height: number,
) {
  if (width <= divider.minimumExtent || height <= divider.minimumExtent)
    return null;
  const extent = divider.vertical ? height : width;
  const thickness = Math.min(
    divider.thickness,
    divider.vertical ? width : height,
  );
  const inset = Math.min(divider.inset, extent * 0.5);
  const leading = ['inset', 'middle'].includes(divider.variant) ? inset : 0;
  const trailing = divider.variant === 'middle' ? inset : 0;
  const length = Math.max(0, extent - leading - trailing);
  if (length <= 0) return null;
  return divider.vertical
    ? {
        x: (width - thickness) / 2,
        y: leading,
        width: thickness,
        height: length,
      }
    : {
        x: leading,
        y: (height - thickness) / 2,
        width: length,
        height: thickness,
      };
}
