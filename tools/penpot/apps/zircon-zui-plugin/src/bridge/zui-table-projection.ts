import type { ZuiDocument, ZuiNode } from './zui-document';
import type { ProjectionText } from './penpot-projection-model';
import { controlProperties } from './zui-control-properties';
import { resolveZuiTextStyle } from './zui-text-style';
import {
  isWorkbenchTable,
  tableCells,
  type TableCell,
} from './zui-table-cells';

export interface TableProjection {
  cells: TableCell[];
  narrow: boolean;
  fontSize: number;
  fontFamily: string;
  lineHeight: number;
  insetX: number;
  insetY: number;
  rowHeight: number;
  gapLarge: number;
  textClipGuard: number;
  actionWidth: number;
  offsetX: number;
  offsetY: number;
  cellOffsets: number[];
}

/** Preview authored row colors from the same transient state as the review host. */
export function workbenchTableStatePreviewNode(
  document: ZuiDocument,
  node: ZuiNode,
): ZuiNode {
  if (!isWorkbenchTable(node)) return node;
  const props = { ...node.props };
  const state = { ...props, ...node.state };
  const unavailable =
    state['disabled'] === true ||
    state['loading'] === true ||
    state['enabled'] === false;
  const marked = state['selected'] === true;
  const background = unavailable
    ? props['disabled_background_color']
    : state['pressed'] === true
      ? props['pressed_background_color']
      : marked
        ? props['selected_background_color']
        : state['hovered'] === true || state['open'] === true
          ? props['hover_background_color']
          : undefined;
  if (background !== undefined) props['background_color'] = background;
  if (unavailable) {
    props['border_color'] = 'transparent';
  } else if (marked && document.tokens?.['editor.accent'] !== undefined) {
    props['border_color'] = '$editor.accent';
  } else if (
    (marked || state['focused'] === true || state['focus_visible'] === true) &&
    props['focus_border_color'] !== undefined
  ) {
    props['border_color'] = props['focus_border_color'];
  }
  return { ...node, props };
}

export function projectWorkbenchTable(
  document: ZuiDocument,
  node: ZuiNode,
):
  | {
      table: TableProjection;
      text: null;
      textFragments: Record<string, ProjectionText>;
    }
  | undefined {
  if (!isWorkbenchTable(node)) return undefined;
  const props = node.props ?? {};
  const tokenNode: ZuiNode = {
    component: 'Table',
    props: Object.fromEntries(
      Object.entries(document.tokens ?? {}).map(([key, value]) => [key, value]),
    ),
  };
  const { metric, color } = controlProperties(
    document,
    tokenNode,
    'table host',
  );
  const { number } = controlProperties(document, node, 'table cell');
  const fontSize = metric(['editor.typography.body.size'], 14, true);
  const lineHeight = metric(['editor.typography.line_height'], 1.4, true);
  const insetX = metric(['editor.density.gap.medium'], 8);
  const insetY = metric(['editor.density.gap.small'], 4);
  const borderWidth = metric(['editor.control.border_width'], 1);
  const cells = tableCells(props);
  const family = resolveZuiTextStyle(document, node, {
    size: fontSize,
    weight: '400',
  }).family;
  const unavailable =
    props['disabled'] === true ||
    props['loading'] === true ||
    props['enabled'] === false;
  const header = node.control_id?.endsWith('TableHeader') === true;
  const tail = node.control_id === 'WorkbenchTableTail';
  return {
    text: null,
    textFragments: Object.fromEntries(
      cells.map((cell) => {
        const textColor = unavailable
          ? color(['editor.text.disabled'], '#737373')
          : header || cell.column >= 2
            ? color(['editor.text.secondary'], '#b3b3b3')
            : color(['editor.text.primary'], '#e8e8e8');
        return [
          cell.key,
          {
            characters: cell.value,
            property: cell.sourceIndex === null ? 'text' : 'options',
            color: textColor.slice(0, 7),
            colorOpacity:
              textColor.length === 9
                ? parseInt(textColor.slice(7), 16) / 255
                : 1,
            fontSize,
            fontWeight: '400',
            fontFamily: family,
            lineHeight,
            align: cell.column >= 2 ? 'right' : 'left',
          } satisfies ProjectionText,
        ];
      }),
    ),
    table: {
      cells,
      narrow: String(props['component_variant'] ?? '')
        .split(/\s+/)
        .includes('layoutNarrow'),
      fontSize,
      fontFamily: family,
      lineHeight,
      insetX,
      insetY,
      rowHeight: metric(['editor.density.row_height'], 28, true),
      gapLarge: metric(['editor.density.gap.large'], 12),
      textClipGuard: Math.max(0, insetX - borderWidth * 2),
      actionWidth: insetX * 2 + insetY * 2,
      offsetX: header || tail ? number(['layout_content_offset_x'], 0) : 0,
      offsetY: header || tail ? number(['layout_content_offset_y'], 0) : 0,
      cellOffsets: ['first', 'second', 'third', 'fourth'].map((name) =>
        number([`layout_${name}_cell_offset_x`], 0),
      ),
    },
  };
}
