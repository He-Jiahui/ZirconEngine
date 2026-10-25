import type { Board, Text } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { nativePainterComponent } from './bridge/zui-native-painter-role';
import { treeRowContent } from './bridge/zui-tree-row-content';
import { controlProperties } from './bridge/zui-control-properties';
import { refreshTreeRowPainterContent } from './penpot-control-content';
import { setSingleLineTextOverflow } from './penpot-text-overflow';
import {
  guardControlParts,
  updateControlTextGeometryGuard,
} from './penpot-control-guard';
import { ZUI_METADATA_NAMESPACE } from './metadata';

export function isTreeRowPainter(node?: ZuiNode): boolean {
  return nativePainterComponent(node) === 'TreeRow';
}

/** Production reflow recomputes fixed pixel lanes, preserving semantic text edits. */
export function refreshTreeRowContent(
  board: Board,
  node: ZuiNode | undefined,
  document: ZuiDocument,
): boolean {
  if (!node || !isTreeRowPainter(node) || !document['penpot_host_theme_source'])
    return false;
  const updated = refreshTreeRowPainterContent(board, node, document);
  if (updated) guardControlParts(
    board,
    board.children.filter(
      (child) =>
        child.getSharedPluginData(
          ZUI_METADATA_NAMESPACE,
          'content-panel-placement',
        ) ||
        child.getSharedPluginData(
          ZUI_METADATA_NAMESPACE,
          'content-icon-placement',
        ),
    ),
  );
  return true;
}

export function layoutTreeRowText(
  board: Board,
  text: Text,
  node: ZuiNode | undefined,
  document: ZuiDocument,
): boolean {
  if (!node || !isTreeRowPainter(node) || !document['penpot_host_theme_source'])
    return false;
  const overflow = controlProperties(document,
    { ...node, props: { ...node.props, ...node.state } },
    'TreeRow text overflow').resolve('text_overflow');
  const label = treeRowContent(node, {
    document,
    width: board.width,
    height: board.height,
  })?.texts[0];
  // Use the existing selected-font fitter and semantic full-value capture.
  // A fixed native text box alone still wraps long captions into a second line.
  setSingleLineTextOverflow(text, overflow === 'ellipsis' ? 'ellipsis' : undefined);
  text.hidden = !label;
  if (label) {
    text.resize(
      board.width * label.placement.width,
      board.height * label.placement.height,
    );
    text.x = board.x + board.width * label.placement.x;
    text.y = board.y + board.height * label.placement.y;
  }
  updateControlTextGeometryGuard(board, text);
  return true;
}
