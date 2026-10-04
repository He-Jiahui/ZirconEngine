import type { Board, Text } from '@penpot/plugin-types';
import type { ZuiDocument, ZuiNode } from './bridge/zui-document';
import { nativePainterComponent } from './bridge/zui-native-painter-role';
import { treeRowContent } from './bridge/zui-tree-row-content';
import { refreshNativePainterContent } from './penpot-control-content';
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
  refreshNativePainterContent(board, node, document);
  guardControlParts(
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
  const label = treeRowContent(node, {
    document,
    width: board.width,
    height: board.height,
  })?.texts[0];
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
